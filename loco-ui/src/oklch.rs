//! Colour arithmetic for the tokens: `oklch()` to `#rrggbb`, and the WCAG contrast ratio.
//! The scales in [`crate::layout::Scale`] are written in oklch, where equal steps look equal,
//! and emitted as hex, because Chrome 109 (the oldest browser the pages support) has no
//! `oklch()`.

/// `#rrggbb` for `oklch(L C H)` (L as 0–1 or a percentage, H in degrees), or for a
/// `#rrggbb` passed through. Colours outside sRGB are clipped per channel. `None` for
/// anything else.
pub(crate) fn hex(value: &str) -> Option<String> {
    let v = value.trim();
    if is_hex(v) {
        return Some(v.to_ascii_lowercase());
    }
    parse(v).map(to_hex)
}

fn is_hex(v: &str) -> bool {
    v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
}

/// A colour as oklch lightness (0–1), chroma and hue (degrees), from `oklch(L C H)` or
/// `#rrggbb`.
pub(crate) fn parse(value: &str) -> Option<(f64, f64, f64)> {
    let v = value.trim();
    if is_hex(v) {
        return Some(from_hex(v));
    }
    let inner = v.strip_prefix("oklch(")?.strip_suffix(')')?;
    let mut parts = inner.split_whitespace();
    let l = match parts.next()? {
        p if p.ends_with('%') => p.trim_end_matches('%').parse::<f64>().ok()? / 100.0,
        p => p.parse::<f64>().ok()?,
    };
    let c: f64 = parts.next()?.parse().ok()?;
    let h: f64 = parts.next()?.trim_end_matches("deg").parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((l, c, h))
}

/// `#rrggbb` for an oklch triple, clipped to sRGB per channel.
pub(crate) fn to_hex((l, c, h): (f64, f64, f64)) -> String {
    let (a, b) = (c * h.to_radians().cos(), c * h.to_radians().sin());
    let l_ = (l + 0.396_337_777_4 * a + 0.215_803_757_3 * b).powi(3);
    let m_ = (l - 0.105_561_345_8 * a - 0.063_854_172_8 * b).powi(3);
    let s_ = (l - 0.089_484_177_5 * a - 1.291_485_548 * b).powi(3);
    let rgb = [
        4.076_741_662_1 * l_ - 3.307_711_591_3 * m_ + 0.230_969_929_2 * s_,
        -1.268_438_004_6 * l_ + 2.609_757_401_1 * m_ - 0.341_319_396_5 * s_,
        -0.004_196_086_3 * l_ - 0.703_418_614_7 * m_ + 1.707_614_701 * s_,
    ];
    let byte = |x: f64| {
        let x = x.clamp(0.0, 1.0);
        let g = if x <= 0.003_130_8 {
            12.92 * x
        } else {
            1.055 * x.powf(1.0 / 2.4) - 0.055
        };
        (g * 255.0).round() as u8
    };
    format!(
        "#{:02x}{:02x}{:02x}",
        byte(rgb[0]),
        byte(rgb[1]),
        byte(rgb[2])
    )
}

/// The oklch triple of a `#rrggbb` colour.
fn from_hex(hex: &str) -> (f64, f64, f64) {
    let [r, g, b] = [1, 3, 5].map(|i| linear(hex, i));
    let l = (0.412_221_470_8 * r + 0.536_332_536_3 * g + 0.051_445_992_9 * b).cbrt();
    let m = (0.211_903_498_2 * r + 0.680_699_545_1 * g + 0.107_396_956_6 * b).cbrt();
    let s = (0.088_302_461_9 * r + 0.281_718_837_6 * g + 0.629_978_700_5 * b).cbrt();
    let lightness = 0.210_454_255_3 * l + 0.793_617_785 * m - 0.004_072_046_8 * s;
    let a = 1.977_998_495_1 * l - 2.428_592_205 * m + 0.450_593_709_9 * s;
    let b = 0.025_904_037_1 * l + 0.782_771_766_2 * m - 0.808_675_766 * s;
    (
        lightness,
        a.hypot(b),
        b.atan2(a).to_degrees().rem_euclid(360.0),
    )
}

/// One channel of a `#rrggbb` colour (at byte offset `i`) in linear light.
fn linear(hex: &str, i: usize) -> f64 {
    let c = f64::from(u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(0)) / 255.0;
    if c <= 0.040_45 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

/// Relative luminance of a `#rrggbb` colour (WCAG 2).
fn luminance(hex: &str) -> f64 {
    0.2126 * linear(hex, 1) + 0.7152 * linear(hex, 3) + 0.0722 * linear(hex, 5)
}

/// The WCAG 2 contrast ratio of two `#rrggbb` colours, from 1 to 21.
pub(crate) fn contrast(a: &str, b: &str) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oklch_round_trips_the_radix_steps() {
        // Radix slate 12 and indigo 9, written in oklch to three places.
        assert_eq!(hex("oklch(0.241 0.010 248.2)").as_deref(), Some("#1c2024"));
        assert_eq!(hex("oklch(0.544 0.191 267.0)").as_deref(), Some("#3e63dd"));
        assert_eq!(hex("oklch(100% 0 0)").as_deref(), Some("#ffffff"));
        assert_eq!(hex("#ABCDEF").as_deref(), Some("#abcdef"));
        assert_eq!(hex("var(--x)"), None);
        assert!((contrast("#ffffff", "#000000") - 21.0).abs() < 1e-9);
        assert_eq!(to_hex(parse("#3e63dd").unwrap()), "#3e63dd");
    }
}
