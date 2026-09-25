//! # Layout
//!
//! The HTML shell every demo page uses: doctype, head, inline stylesheet, `<main>`, the
//! capability beacons that teach the server what this browser supports, and the optional
//! [`crate::enhance`] script tag (the page works the same without it).
//!
//! **Platform features:** `@view-transition { navigation: auto }` (Chrome 126+, Safari 18.2+,
//! not Firefox) lets elements with a `view-transition-name` morph across full-page navigations.
//! The root itself swaps instantly: the default 0.25 s cross-fade made every click feel slow.
//! `<link rel="expect" blocking="render">` (Chrome 124+) holds the transition until `<main>`
//! is parsed. `prefers-color-scheme` + custom properties give light/dark with no script.
//!
//! **Fallback:** browsers without view transitions navigate normally. Theme colours are plain
//! custom properties switched by a media query and `data-theme`, so no `light-dark()` needed.
//!
//! **Theming:** every colour, radius and spacing the components use is a `--lui-*` custom
//! property. [`Tokens`] holds them for light and dark; `ui.page(..).tokens(&tokens)` emits them once per page
//! as a `<style>` after the stylesheet, so a different palette is a struct, not a CSS file.
//! `docs/theming.md` lists each token and what it affects.
//!
//! ```rust
//! use loco_ui::{prelude::*, layout::{Palette, Tokens}};
//! let ui = Ui::from(Caps::all());
//! let page = ui.page("Title", html! { p { "body" } });
//! let tokens = Tokens { light: Palette { primary: "#7a3b1e", ..Tokens::default().light }, ..Default::default() };
//! assert!(page.tokens(&tokens).into_string().contains("--lui-primary: #7a3b1e"));
//! ```

use maud::{DOCTYPE, Markup, PreEscaped, Render, html};

use crate::{Caps, Theme, caps, enhance, stylesheet};

/// One colour scheme's worth of tokens, as CSS colour values. The roles follow shadcn/ui:
/// `primary` is the brand colour, `accent` is the quiet surface under a hovered item. By
/// default each role is an alias onto a step of [`Tokens::gray`] or [`Tokens::brand`]
/// (`var(--lui-gray-1)`), so changing a scale changes every role built on it; any CSS colour
/// works in its place.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Page background (`--lui-bg`).
    pub bg: &'static str,
    /// Text on the page background (`--lui-fg`).
    pub fg: &'static str,
    /// Secondary text: notes, labels, table headers (`--lui-muted`).
    pub muted: &'static str,
    /// Borders and rules (`--lui-line`).
    pub line: &'static str,
    /// Raised surfaces: `<code>`, the demo stage, open panels (`--lui-surface`).
    pub surface: &'static str,
    /// Cards and stat tiles (`--lui-card`).
    pub card: &'static str,
    /// Dialogs, drawers, popovers, menus and toasts (`--lui-popover`).
    pub popover: &'static str,
    /// Secondary buttons, the tab list, chips and badges (`--lui-secondary`).
    pub secondary: &'static str,
    /// Hover and highlighted surface: menu items, ghost buttons, rows (`--lui-accent`).
    pub accent: &'static str,
    /// Text on the accent surface (`--lui-on-accent`).
    pub on_accent: &'static str,
    /// Primary buttons, checked controls, the current page and step (`--lui-primary`).
    pub primary: &'static str,
    /// Text on the primary colour (`--lui-on-primary`).
    pub on_primary: &'static str,
    /// Borders of inputs, selects and textareas (`--lui-input`).
    pub input: &'static str,
    /// The focus ring, drawn at 50% opacity (`--lui-ring`).
    pub ring: &'static str,
    /// Link text (`--lui-link`): the brand's text step, which clears 4.5:1 on the background
    /// where the solid `primary` may not (in the dark scheme it does not).
    pub link: &'static str,
    /// Errors and "no" (`--lui-danger`).
    pub danger: &'static str,
    /// Text on a danger fill: danger buttons and badges (`--lui-on-danger`).
    pub on_danger: &'static str,
    /// Success and "yes" (`--lui-ok`).
    pub ok: &'static str,
    /// Warnings: worked, but look (`--lui-warn`).
    pub warn: &'static str,
}

impl Palette {
    /// The custom property declarations for this palette (`--lui-bg: …;`), one line per
    /// group. The theme builder re-declares them on its previews, since a role such as
    /// `var(--lui-gray-1)` resolves where it is declared.
    pub fn declarations(&self) -> String {
        format!(
            "  --lui-bg: {}; --lui-fg: {}; --lui-muted: {}; --lui-line: {};\n  --lui-surface: {}; --lui-card: {}; --lui-popover: {}; --lui-secondary: {};\n  --lui-accent: {}; --lui-on-accent: {}; --lui-primary: {}; --lui-on-primary: {};\n  --lui-input: {}; --lui-ring: {}; --lui-link: {};\n  --lui-danger: {}; --lui-on-danger: {}; --lui-ok: {}; --lui-warn: {};\n",
            self.bg,
            self.fg,
            self.muted,
            self.line,
            self.surface,
            self.card,
            self.popover,
            self.secondary,
            self.accent,
            self.on_accent,
            self.primary,
            self.on_primary,
            self.input,
            self.ring,
            self.link,
            self.danger,
            self.on_danger,
            self.ok,
            self.warn
        )
    }
}

/// A 12-step colour scale after Radix Colors, one array per scheme, each step written as a
/// CSS colour (`oklch(L C H)` or `#rrggbb`). Each step has one job:
///
/// | Steps | Job |
/// |-------|-----|
/// | 1–2 | App and subtle backgrounds (page, cards, code) |
/// | 3–5 | Component surfaces: normal, hover, pressed |
/// | 6–8 | Borders: rules, inputs, the focus ring |
/// | 9–10 | Solid fills and their hover |
/// | 11–12 | Muted and full-contrast text |
///
/// Emitted as `--lui-<name>-1` … `--lui-<name>-12`. oklch steps are converted to hex on the
/// way out, since Chrome 109 has no `oklch()`; anything else is emitted as written.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scale {
    /// Steps 1–12 for the light scheme.
    pub light: [&'static str; 12],
    /// Steps 1–12 for the dark scheme.
    pub dark: [&'static str; 12],
}

impl Scale {
    /// Radix slate: a gray with a little blue in it, as Linear's.
    pub const SLATE: Scale = Scale {
        light: [
            "oklch(0.991 0.001 286.4)",
            "oklch(0.983 0.003 286.4)",
            "oklch(0.956 0.004 286.3)",
            "oklch(0.932 0.005 286.3)",
            "oklch(0.910 0.007 277.2)",
            "oklch(0.887 0.010 286.2)",
            "oklch(0.853 0.011 280.4)",
            "oklch(0.794 0.016 277.8)",
            "oklch(0.645 0.016 277.7)",
            "oklch(0.611 0.015 272.6)",
            "oklch(0.502 0.014 264.4)",
            "oklch(0.241 0.010 248.2)",
        ],
        dark: [
            "oklch(0.179 0.004 286.0)",
            "oklch(0.213 0.004 264.5)",
            "oklch(0.252 0.006 271.2)",
            "oklch(0.283 0.007 248.1)",
            "oklch(0.312 0.008 255.6)",
            "oklch(0.347 0.010 254.0)",
            "oklch(0.399 0.012 252.9)",
            "oklch(0.489 0.016 251.7)",
            "oklch(0.537 0.015 262.3)",
            "oklch(0.583 0.015 266.6)",
            "oklch(0.769 0.010 258.3)",
            "oklch(0.949 0.003 264.5)",
        ],
    };

    /// Radix indigo: the brand scale by default, close to Linear's blue-violet.
    pub const INDIGO: Scale = Scale {
        light: [
            "oklch(0.994 0.001 286.4)",
            "oklch(0.982 0.008 271.3)",
            "oklch(0.961 0.017 267.8)",
            "oklch(0.935 0.031 269.8)",
            "oklch(0.902 0.047 269.6)",
            "oklch(0.862 0.068 271.1)",
            "oklch(0.806 0.088 271.4)",
            "oklch(0.731 0.112 270.4)",
            "oklch(0.544 0.191 267.0)",
            "oklch(0.511 0.195 266.6)",
            "oklch(0.509 0.172 267.2)",
            "oklch(0.313 0.086 268.6)",
        ],
        dark: [
            "oklch(0.191 0.025 276.5)",
            "oklch(0.209 0.030 274.8)",
            "oklch(0.272 0.071 268.0)",
            "oklch(0.318 0.095 267.2)",
            "oklch(0.362 0.104 267.0)",
            "oklch(0.403 0.111 268.8)",
            "oklch(0.449 0.120 268.9)",
            "oklch(0.502 0.137 268.3)",
            "oklch(0.544 0.191 267.0)",
            "oklch(0.589 0.176 269.3)",
            "oklch(0.776 0.114 273.0)",
            "oklch(0.911 0.043 269.6)",
        ],
    };

    /// A whole scale from one colour, shaped like `like` (`Scale::INDIGO` for a brand,
    /// `Scale::SLATE` for a gray): `seed` becomes step 9 in both schemes, step 10 keeps the
    /// profile's distance from step 9, and every other step takes the profile's lightness
    /// with the seed's hue and its chroma scaled by the seed's. So steps 11 and 12 stay as
    /// readable as the profile's whatever the seed. `seed` is `#rrggbb` or `oklch(..)`;
    /// `None` for anything else. The theme builder uses it.
    ///
    /// ```rust
    /// use loco_ui::layout::Scale;
    /// let teal = Scale::derive("#12a594", &Scale::INDIGO).unwrap();
    /// assert_eq!(teal.light[8], "#12a594");
    /// assert!(teal.css("brand", true).starts_with("  --lui-brand-1: #"));
    /// ```
    pub fn derive(seed: &str, like: &Scale) -> Option<DerivedScale> {
        let (l0, c0, h0) = crate::oklch::parse(seed)?;
        let seed_hex = crate::oklch::to_hex((l0, c0, h0));
        let steps = |profile: &[&str; 12]| -> Option<[String; 12]> {
            let p: Vec<_> = profile
                .iter()
                .map(|v| crate::oklch::parse(v))
                .collect::<Option<_>>()?;
            let (l9, c9, _) = p[8];
            let ratio = if c9 > 0.001 { c0 / c9 } else { 1.0 };
            Some(std::array::from_fn(|i| match i {
                8 => seed_hex.clone(),
                9 => crate::oklch::to_hex(((l0 + p[9].0 - l9).clamp(0.0, 1.0), p[9].1 * ratio, h0)),
                _ => crate::oklch::to_hex((p[i].0, p[i].1 * ratio, h0)),
            }))
        };
        Some(DerivedScale {
            light: steps(&like.light)?,
            dark: steps(&like.dark)?,
        })
    }

    /// `--lui-<name>-n: <colour>;` for one scheme's steps.
    fn declarations(steps: &[&str; 12], name: &str) -> String {
        let mut out = String::from("  ");
        for (i, v) in steps.iter().enumerate() {
            let v = crate::oklch::hex(v).unwrap_or_else(|| (*v).to_string());
            out.push_str(&format!("--lui-{name}-{}: {v}; ", i + 1));
        }
        out.push('\n');
        out
    }
}

/// Depth and gradients for the light scheme, emitted after its colours. Each shadow stacks a
/// tight contact shadow over softer ambient ones, as Linear and Radix Themes do:
/// `--lui-shadow-xs` under controls, `-sm` under cards, `-md` under popovers and menus, `-lg`
/// under dialogs, sheets and toasts. `--lui-highlight` is an inset top edge for raised
/// surfaces (add it to their `box-shadow`). The gradients run between brand steps, in oklch:
/// `--lui-gradient-primary` fills primary buttons (put it in `background-image` over a
/// `background-color: var(--lui-primary)`, so a browser without `in oklch`, Chrome before 111,
/// keeps the flat fill), `--lui-gradient-ring` draws focus rings and gradient borders.
pub const DEPTH_LIGHT: &str = "  --lui-shadow-xs: 0 1px 1px rgb(0 0 0 / 0.04), 0 1px 2px rgb(0 0 0 / 0.06);
  --lui-shadow-sm: 0 1px 2px rgb(0 0 0 / 0.06), 0 2px 6px -1px rgb(0 0 0 / 0.06);
  --lui-shadow-md: 0 1px 2px rgb(0 0 0 / 0.05), 0 4px 8px -2px rgb(0 0 0 / 0.08), 0 12px 20px -6px rgb(0 0 0 / 0.08);
  --lui-shadow-lg: 0 1px 3px rgb(0 0 0 / 0.06), 0 8px 16px -4px rgb(0 0 0 / 0.1), 0 24px 48px -12px rgb(0 0 0 / 0.18);
  --lui-highlight: inset 0 1px 0 rgb(255 255 255 / 0.6);
  --lui-gradient-primary: linear-gradient(in oklch to bottom, var(--lui-brand-9), var(--lui-brand-11));
  --lui-gradient-ring: linear-gradient(in oklch 135deg, var(--lui-brand-11), var(--lui-brand-8));
";

/// [`DEPTH_LIGHT`] for the dark scheme: deeper ambient shadows (a soft black shadow barely
/// shows on a near-black page), a fainter highlight that reads as a lit top edge, and the
/// primary gradient running from step 9 down to the darker step 8, so white text holds 4.5:1
/// over all of it.
pub const DEPTH_DARK: &str = "  --lui-shadow-xs: 0 1px 1px rgb(0 0 0 / 0.3), 0 1px 2px rgb(0 0 0 / 0.4);
  --lui-shadow-sm: 0 1px 2px rgb(0 0 0 / 0.4), 0 2px 8px -1px rgb(0 0 0 / 0.4);
  --lui-shadow-md: 0 1px 2px rgb(0 0 0 / 0.4), 0 6px 12px -3px rgb(0 0 0 / 0.5), 0 16px 28px -8px rgb(0 0 0 / 0.5);
  --lui-shadow-lg: 0 2px 4px rgb(0 0 0 / 0.4), 0 12px 24px -6px rgb(0 0 0 / 0.6), 0 32px 64px -16px rgb(0 0 0 / 0.7);
  --lui-highlight: inset 0 1px 0 rgb(255 255 255 / 0.07);
  --lui-gradient-primary: linear-gradient(in oklch to bottom, var(--lui-brand-9), var(--lui-brand-8));
  --lui-gradient-ring: linear-gradient(in oklch 135deg, var(--lui-brand-11), var(--lui-brand-8));
";

/// A scale made at run time by [`Scale::derive`], as `#rrggbb` steps.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DerivedScale {
    /// Steps 1–12 for the light scheme.
    pub light: [String; 12],
    /// Steps 1–12 for the dark scheme.
    pub dark: [String; 12],
}

impl DerivedScale {
    /// `--lui-<name>-1: #…; … --lui-<name>-12: #…;` for one scheme.
    pub fn css(&self, name: &str, dark: bool) -> String {
        let steps = if dark { &self.dark } else { &self.light };
        Scale::declarations(&std::array::from_fn(|i| steps[i].as_str()), name)
    }

    /// The text colour for the solid step 9: white, or near-black when white would fall
    /// under 4.5:1 (a light seed such as yellow or lime).
    pub fn on_solid(&self) -> &'static str {
        if crate::oklch::contrast("#ffffff", &self.light[8]) >= 4.5 {
            "#ffffff"
        } else {
            "#111113"
        }
    }

    /// The scale as a `layout::Scale` const to paste into Rust.
    pub fn rust(&self, name: &str) -> String {
        let list = |s: &[String; 12]| {
            s.iter()
                .map(|v| format!("\"{v}\""))
                .collect::<Vec<_>>()
                .join(", ")
        };
        format!(
            "const {name}: Scale = Scale {{\n    light: [{}],\n    dark: [{}],\n}};\n",
            list(&self.light),
            list(&self.dark)
        )
    }
}

/// Every `--lui-*` token: the two scales, a light and a dark palette of roles over them, and
/// the two shape tokens. `Default` is a Linear-like theme: Radix slate for the grays and
/// indigo for the brand, with status colours from Radix Colors step 11 (the step made for
/// text; the light green and amber a shade darker so they clear 4.5:1 on the subtle
/// background too). A test checks every text role on every surface in both schemes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tokens {
    /// The neutral scale (`--lui-gray-1` … `-12`): backgrounds, surfaces, lines and text.
    pub gray: Scale,
    /// The brand scale (`--lui-brand-1` … `-12`): primary fills, links and the focus ring.
    pub brand: Scale,
    /// Colours for the light scheme and for `data-theme="light"`.
    pub light: Palette,
    /// Colours for `prefers-color-scheme: dark` and for `data-theme="dark"`.
    pub dark: Palette,
    /// Corner radius of dialogs and popovers (`--lui-radius`); controls use
    /// `--lui-radius-sm` (2px less) and cards `--lui-radius-lg` (4px more). The depth tokens
    /// are emitted per scheme beside the colours: see [`DEPTH_LIGHT`].
    pub radius: &'static str,
    /// The spacing unit every gap and padding is a multiple of (`--lui-space`). The scale
    /// `--lui-space-{1,2,3,4,6,8}` is derived from it: step n is n/2 units (4px each by
    /// default, as Tailwind's `gap-n`), and the layout primitives' `.gap(n)` uses it.
    pub space: &'static str,
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens {
            gray: Scale::SLATE,
            brand: Scale::INDIGO,
            light: Palette {
                bg: "var(--lui-gray-1)",
                fg: "var(--lui-gray-12)",
                muted: "var(--lui-gray-11)",
                line: "var(--lui-gray-6)",
                surface: "var(--lui-gray-2)",
                card: "var(--lui-gray-1)",
                popover: "var(--lui-gray-1)",
                secondary: "var(--lui-gray-3)",
                accent: "var(--lui-gray-3)",
                on_accent: "var(--lui-gray-12)",
                primary: "var(--lui-brand-9)",
                on_primary: "#ffffff",
                input: "var(--lui-gray-7)",
                ring: "var(--lui-brand-8)",
                link: "var(--lui-brand-11)",
                danger: "#ce2c31",
                on_danger: "var(--lui-gray-1)",
                ok: "#1f7d53",
                warn: "#9c5b00",
            },
            dark: Palette {
                bg: "var(--lui-gray-1)",
                fg: "var(--lui-gray-12)",
                muted: "var(--lui-gray-11)",
                line: "var(--lui-gray-6)",
                surface: "var(--lui-gray-2)",
                card: "var(--lui-gray-2)",
                popover: "var(--lui-gray-2)",
                secondary: "var(--lui-gray-3)",
                accent: "var(--lui-gray-4)",
                on_accent: "var(--lui-gray-12)",
                primary: "var(--lui-brand-9)",
                on_primary: "#ffffff",
                input: "var(--lui-gray-7)",
                ring: "var(--lui-brand-8)",
                link: "var(--lui-brand-11)",
                danger: "#ff9592",
                on_danger: "var(--lui-gray-1)",
                ok: "#3dd68c",
                warn: "#ffca16",
            },
            radius: "0.5rem",
            space: "8px",
        }
    }
}

impl Tokens {
    /// The CSS that sets these tokens: `:root` for light, the dark palette under
    /// `prefers-color-scheme: dark` unless `data-theme="light"`, and again under
    /// `data-theme="dark"`. [`crate::stylesheet`] starts with `Tokens::default().css()`.
    pub fn css(&self) -> String {
        let scheme = |p: &Palette, dark: bool| {
            let pick = |s: &Scale| if dark { s.dark } else { s.light };
            Scale::declarations(&pick(&self.gray), "gray")
                + &Scale::declarations(&pick(&self.brand), "brand")
                + &p.declarations()
                + if dark { DEPTH_DARK } else { DEPTH_LIGHT }
        };
        let (light, dark) = (scheme(&self.light, false), scheme(&self.dark, true));
        format!(
            ":root {{\n  color-scheme: light dark;\n{light}  --lui-radius: {}; --lui-space: {};\n  --lui-space-1: calc(var(--lui-space) * 0.5); --lui-space-2: var(--lui-space); --lui-space-3: calc(var(--lui-space) * 1.5);\n  --lui-space-4: calc(var(--lui-space) * 2); --lui-space-6: calc(var(--lui-space) * 3); --lui-space-8: calc(var(--lui-space) * 4);\n  --lui-radius-sm: max(0px, var(--lui-radius) - 2px); --lui-radius-lg: calc(var(--lui-radius) + 4px);\n  --lui-overlay: rgb(0 0 0 / 0.5);\n}}\n\
             @media (prefers-color-scheme: dark) {{\n  :root:not([data-theme=\"light\"]) {{\n{dark}  }}\n}}\n\
             :root[data-theme=\"dark\"] {{\n  color-scheme: dark;\n{dark}}}\n\
             :root[data-theme=\"light\"] {{ color-scheme: light; }}\n",
            self.radius, self.space
        )
    }

    /// `value` as `#rrggbb` in the light or dark scheme: a hex or `oklch()` colour, or a
    /// `var(--lui-gray-n)` / `var(--lui-brand-n)` alias resolved through the scales. `None`
    /// for anything else. The theme builder fills its colour inputs with it.
    pub fn color(&self, dark: bool, value: &str) -> Option<String> {
        let step = |name: &str| {
            let rest = value.trim().strip_prefix("var(--lui-")?.strip_suffix(')')?;
            let n: usize = rest.strip_prefix(name)?.strip_prefix('-')?.parse().ok()?;
            let s = if name == "gray" {
                &self.gray
            } else {
                &self.brand
            };
            let steps = if dark { s.dark } else { s.light };
            steps.get(n.checked_sub(1)?).copied()
        };
        let v = step("gray").or_else(|| step("brand")).unwrap_or(value);
        crate::oklch::hex(v)
    }
}

/// Wrap `body` in a full page with the default [`Tokens`]. Beacons are added while the
/// browser is still unknown.
pub fn layout(caps: &Caps, title: &str, theme: Theme, body: Markup) -> Markup {
    page(caps, "en", title, theme, None, &[], true, body)
}

/// [`layout`] under a different set of [`Tokens`]: the overrides are emitted once, in a
/// `<style>` right after the stylesheet, so every component on the page picks them up.
pub fn layout_with(
    caps: &Caps,
    title: &str,
    theme: Theme,
    tokens: &Tokens,
    body: Markup,
) -> Markup {
    page(caps, "en", title, theme, Some(tokens), &[], true, body)
}

/// The whole document: `tokens` overrides and then `css` (a user component's styles, see
/// `Page::css`) follow the stylesheet in the head, each once.
#[allow(clippy::too_many_arguments)]
pub(crate) fn page(
    caps: &Caps,
    lang: &str,
    title: &str,
    theme: Theme,
    tokens: Option<&Tokens>,
    css: &[&str],
    script: bool,
    body: Markup,
) -> Markup {
    // The stylesheet is most of the page: size the buffer once instead of doubling into it.
    let size = stylesheet().len() + body.0.len() + 2048;
    html! {
        (Reserve(size))
        (DOCTYPE)
        html lang=(lang) data-theme=(theme.as_str()) {
            head {
                (meta(title))
                // Hold a cross-document view transition until `#main` is parsed. Streamed
                // pages must not: their parse ends only when the last slot has filled.
                link rel="expect" href="#main" blocking="render";
                style { (PreEscaped(stylesheet())) }
                @if let Some(t) = tokens { style class="lui-tokens" { (PreEscaped(t.css())) } }
                @if !css.is_empty() { style class="lui-user" { @for c in css { (PreEscaped(crate::minify_css(c))) } } }
            }
            body {
                (header())
                main id="main" { (body) }
                (caps::beacons(caps))
                @if script { (enhance::script_tag()) }
            }
        }
    }
}

/// `<head>`: charset, viewport, title and the inline stylesheet. `stream` reuses it.
pub fn head(title: &str) -> Markup {
    html! {
        (Reserve(stylesheet().len() + 256))
        head {
            (meta(title))
            style { (PreEscaped(stylesheet())) }
        }
    }
}

fn meta(title: &str) -> Markup {
    html! {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { (title) }
    }
}

/// Grows the buffer `html!` is writing into by `.0` bytes and writes nothing: a size hint
/// for a template whose splices dwarf its literals.
pub(crate) struct Reserve(pub(crate) usize);

impl Render for Reserve {
    fn render_to(&self, buffer: &mut String) {
        buffer.reserve(self.0);
    }
}

/// The site header shown on every page.
pub fn header() -> Markup {
    html! {
        header class="lui-header" {
            a href="/" { strong { "loco-ui" } }
            span { "Interactive HTML for Rust servers, works without JavaScript" }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
@view-transition { navigation: auto; }
/* The root does not cross-fade: the swap is instant and only named parts morph, so a
   navigation never feels slower than the plain reload it replaces. */
::view-transition-old(root), ::view-transition-new(root) { animation: none; }
::view-transition-group(*) { animation-duration: 120ms; animation-timing-function: ease-out; }
::view-transition-old(*), ::view-transition-new(*) { animation-duration: 120ms; }
/* The transition overlay must not eat clicks: a counter tapped twice quickly would lose the
   second tap while the first one is still morphing. */
::view-transition { pointer-events: none; }

/* The --lui-* tokens come first in stylesheet(), from Tokens::default().css(). */

* { box-sizing: border-box; }
/* Type: the system stack only, no web font. Body text is 1rem/1.5; controls, tables and
   menus use shadcn's text-sm (0.875rem/1.25rem); weights are 500 for labels and buttons,
   600 for headings. Numbers in tables and stats are tabular. */
:root {
  --lui-font-sans: ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  --lui-font-mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
html {
  font-family: var(--lui-font-sans); line-height: 1.5;
  -webkit-text-size-adjust: 100%; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;
}
body { margin: 0; background: var(--lui-bg); color: var(--lui-fg); }
main { max-width: 52rem; margin: 0 auto; padding: calc(var(--lui-space) * 4) calc(var(--lui-space) * 2) calc(var(--lui-space) * 8); }
p, li { max-width: 44rem; }
.lui-header {
  display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 1rem;
  max-width: 52rem; margin: 0 auto; padding: calc(var(--lui-space) * 2);
  color: var(--lui-muted);
}
.lui-header a { color: var(--lui-fg); text-decoration: none; font-size: 1rem; letter-spacing: -0.01em; }
.lui-header span { font-size: 0.875rem; }
.lui-header strong { font-weight: 600; }
h1 { font-size: 2.25rem; line-height: 2.5rem; letter-spacing: -0.025em; font-weight: 600; margin: 0 0 0.75rem; }
h2 { font-size: 1.5rem; line-height: 2rem; letter-spacing: -0.0125em; font-weight: 600; margin: 2rem 0 0.5rem; }
h3 { font-size: 1.125rem; line-height: 1.75rem; font-weight: 600; }
p { margin: 0 0 1rem; }
a { color: var(--lui-link); text-underline-offset: 0.15em; text-decoration-thickness: 1px; }
code {
  font-family: var(--lui-font-mono); font-size: 0.875em;
  background: var(--lui-surface); border: 1px solid var(--lui-line); border-radius: var(--lui-radius-sm); padding: 0.05em 0.35em;
}
/* Controls are styled in button.rs and input.rs. Focus everywhere is a 3px ring at 50%;
   aria-invalid turns borders and the ring to --lui-danger. */
details > summary { cursor: pointer; font-weight: 500; }
:focus-visible { outline: 3px solid color-mix(in srgb, var(--lui-ring) 50%, transparent); outline-offset: 0; }
[aria-invalid=true] { border-color: var(--lui-danger); }
[aria-invalid=true]:focus-visible { outline-color: color-mix(in srgb, var(--lui-danger) 20%, transparent); }
/* A swap root or form with a request in flight (set by the enhancement script only). The
   fade waits so a fast answer never flickers; --lui-busy: 1 turns it off. */
/* Gap steps for the layout primitives (stack, cluster, grid, split); their default gaps sit
   in :where() so one of these always wins. */
.lui-gap-0 { gap: 0; }
.lui-gap-1 { gap: var(--lui-space-1); }
.lui-gap-2 { gap: var(--lui-space-2); }
.lui-gap-3 { gap: var(--lui-space-3); }
.lui-gap-4 { gap: var(--lui-space-4); }
.lui-gap-6 { gap: var(--lui-space-6); }
.lui-gap-8 { gap: var(--lui-space-8); }
.lui-sr { position: absolute; width: 1px; height: 1px; margin: -1px; padding: 0; overflow: hidden; clip-path: inset(50%); text-wrap: nowrap; border: 0; }
[data-lui-busy] { opacity: var(--lui-busy, 0.6); transition: opacity 0.15s 0.2s; cursor: progress; }
table { border-collapse: collapse; width: 100%; font-size: 0.875rem; line-height: 1.25rem; font-variant-numeric: tabular-nums; }
/* shadcn Table: h-10 heads in the muted colour, p-2 cells, a rule under each row, muted/50 hover. */
th, td { text-align: left; padding: 0.5rem; border-bottom: 1px solid var(--lui-line); vertical-align: middle; }
th { height: 2.5rem; color: var(--lui-muted); font-weight: 500; white-space: nowrap; }
tbody tr { transition: background-color 0.15s; }
tbody tr:hover { background: color-mix(in srgb, var(--lui-accent) 50%, transparent); }
.lui-note { color: var(--lui-muted); font-size: 0.875rem; }
/* The dashed boxes on the demo's layout page. */
.lui-layout-tile { padding: calc(var(--lui-space) * 1.5); border: 1px dashed var(--lui-input); border-radius: var(--lui-radius-sm); background: var(--lui-surface); font-size: 0.875rem; }
.lui-yes { color: var(--lui-ok); font-weight: 600; }
.lui-no { color: var(--lui-danger); font-weight: 600; }

/* Demo shell, after the shadcn docs: the sidebar of every component beside the page, a
   toolbar with the way back and the theme switch, the lede under a title, "built on" as
   outline badges, the plate (a preview box over a muted code block), and the index as a
   gallery of the components themselves, per group. */
/* The frame: from 60rem the component list is a sticky column beside a readable page (the
   index spreads wider); narrower, it follows a closed <details> that hides it until opened,
   so it is a plain <nav> of links either way and needs no script. */
/* Wider than <main> (52rem): the frame spreads to 84rem, centred, by negative margins (not
   :has(), which Blitz lacks); the header follows where :has() works. */
.lui-site {
  display: grid; gap: calc(var(--lui-space) * 3); width: min(84rem, calc(100vw - 2rem));
  margin-inline: calc((100% - min(84rem, calc(100vw - 2rem))) / 2);
}
body:has(.lui-site) > .lui-header { max-width: 84rem; }
.lui-site-main { min-width: 0; }
.lui-site-menu > summary {
  padding: 0.5rem 0.75rem; font-size: 0.875rem; border: 1px solid var(--lui-line);
  border-radius: var(--lui-radius); background: var(--lui-card); box-shadow: var(--lui-shadow-xs);
}
.lui-site-menu[open] > summary { margin-bottom: var(--lui-space); }
@media (max-width: 59.99rem) {
  .lui-site-menu:not([open]) + .lui-sidebar { display: none; }
}
@media (min-width: 60rem) {
  .lui-site { grid-template-columns: 14rem minmax(0, 1fr); gap: calc(var(--lui-space) * 6); align-items: start; }
  .lui-site-menu { display: none; }
  .lui-site-nav { position: sticky; top: calc(var(--lui-space) * 2); max-height: calc(100vh - var(--lui-space) * 4); overflow-y: auto; }
  .lui-site-main:not(.lui-site-wide) { max-width: 56rem; }
}
.lui-toolbar { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: var(--lui-space); margin: 0 0 calc(var(--lui-space) * 3); min-height: 2.25rem; }
.lui-popover-row { display: flex; justify-content: space-between; gap: var(--lui-space); margin-bottom: calc(var(--lui-space) * 2); }
.lui-back { color: var(--lui-muted); text-decoration: none; font-size: 0.875rem; font-weight: 500; }
.lui-back::before { content: "\2190"; margin-right: 0.35em; }
.lui-back:hover { color: var(--lui-fg); }
.lui-lede { font-size: 1.125rem; line-height: 1.75rem; color: var(--lui-muted); margin-bottom: 1rem; }
.lui-built { color: var(--lui-muted); font-size: 0.875rem; margin: 0 0 1.5rem; }
.lui-built code {
  display: inline-block; margin: 0 0.25rem 0.25rem 0; padding: 0.125rem 0.5rem; white-space: nowrap;
  font-size: 0.75rem; line-height: 1rem; font-weight: 500; color: var(--lui-fg);
  background: transparent; border: 1px solid var(--lui-line); border-radius: var(--lui-radius-sm);
}
/* A component page's plate: the live component on a stage, the code that drew it joined
   underneath. One per page; no transform, overflow or contain on the stage, so dialogs,
   drawers and toasts still escape it. */
.lui-plate { margin: 0 0 2rem; }
.lui-stage {
  padding: calc(var(--lui-space) * 5) calc(var(--lui-space) * 4); background: var(--lui-bg);
  border: 1px solid var(--lui-line); border-bottom: 0; border-radius: var(--lui-radius-lg) var(--lui-radius-lg) 0 0;
}
.lui-stage > :last-child { margin-bottom: 0; }
.lui-stage h2:first-child { margin-top: 0; }
@media (max-width: 40rem) { .lui-stage { padding: calc(var(--lui-space) * 3) calc(var(--lui-space) * 2); } }
.lui-snippet {
  margin: 0; max-width: none; overflow: hidden;
  border: 1px solid var(--lui-line); border-radius: 0 0 var(--lui-radius-lg) var(--lui-radius-lg);
  background: var(--lui-surface);
}
.lui-snippet figcaption {
  display: flex; flex-wrap: wrap; justify-content: space-between; gap: 0.25rem 1rem;
  padding: 0.5rem calc(var(--lui-space) * 2); border-bottom: 1px solid var(--lui-line);
  color: var(--lui-muted); font-size: 0.75rem;
}
.lui-snippet figcaption span:first-child { color: var(--lui-fg); font-weight: 500; font-family: var(--lui-font-mono); }
.lui-snippet pre { margin: 0; padding: calc(var(--lui-space) * 2); overflow-x: auto; scrollbar-color: var(--lui-line) transparent; line-height: 1.7; tab-size: 4; }
.lui-snippet pre code { background: none; border: 0; padding: 0; font-size: 0.8125rem; color: var(--lui-fg); }
/* Highlighted Rust in the status colours, as a GitHub-like theme: keywords in danger, strings
   in ok, numbers and types in warn, comments muted, macros bold. */
.lui-hl-k { color: var(--lui-danger); }
.lui-hl-s { color: var(--lui-ok); }
.lui-hl-n, .lui-hl-t { color: color-mix(in srgb, var(--lui-warn) 70%, var(--lui-fg)); }
.lui-hl-c { color: var(--lui-muted); font-style: italic; }
.lui-hl-m { color: var(--lui-fg); font-weight: 600; }
.lui-hl-f { color: var(--lui-fg); }
/* The demo's props tables under the snippet: one <details> per builder. */
.lui-theme-builder { display: grid; gap: calc(var(--lui-space) * 2); max-width: none; }
.lui-theme-builder fieldset > div { display: grid; grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr)); gap: var(--lui-space); }
.lui-theme-builder fieldset { margin: 0; padding: calc(var(--lui-space) * 2); border: 1px solid var(--lui-line); border-radius: var(--lui-radius); }
.lui-theme-previews { display: grid; grid-template-columns: repeat(auto-fit, minmax(18rem, 1fr)); gap: calc(var(--lui-space) * 2); margin-block: calc(var(--lui-space) * 2); }
.lui-theme-preview { padding: calc(var(--lui-space) * 2); background: var(--lui-bg); color: var(--lui-fg); border: 1px solid var(--lui-line); border-radius: var(--lui-radius); }
/* The builder's depth row: a tile per shadow step, then the primary gradient. */
.lui-theme-depth { display: flex; flex-wrap: wrap; gap: calc(var(--lui-space) * 2); margin-top: calc(var(--lui-space) * 3); }
.lui-theme-depth span { display: grid; place-items: center; width: 3.5rem; height: 3.5rem; border-radius: var(--lui-radius); background: var(--lui-card); color: var(--lui-muted); font-size: 0.75rem; }
.lui-theme-depth .lui-theme-gradient { background: var(--lui-primary); background-image: var(--lui-gradient-primary); box-shadow: var(--lui-shadow-sm), var(--lui-highlight); }
.lui-props { max-width: none; margin: 0 0 2rem; }
.lui-props details { border: 1px solid var(--lui-line); border-radius: var(--lui-radius); margin: 0 0 0.5rem; }
.lui-props summary { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 0.5rem; padding: 0.5rem 0.75rem; cursor: pointer; }
.lui-props summary span { margin-left: auto; color: var(--lui-muted); font-size: 0.8125rem; }
.lui-props-scroll { overflow-x: auto; border-top: 1px solid var(--lui-line); }
.lui-playground > form > p { margin: 0; padding: 0.5rem 0.75rem; border-top: 1px solid var(--lui-line); }
.lui-playground-preview { padding: 1rem 0.75rem; border-top: 1px solid var(--lui-line); }
.lui-playground > pre { margin: 0; padding: 0.5rem 0.75rem; border-top: 1px solid var(--lui-line); overflow-x: auto; font-size: 0.8125rem; }
.lui-props table { width: 100%; border-collapse: collapse; font-size: 0.8125rem; }
.lui-props th, .lui-props td { text-align: left; vertical-align: top; padding: 0.375rem 0.75rem; border-bottom: 1px solid var(--lui-line); }
.lui-props th { color: var(--lui-muted); font-weight: 500; white-space: nowrap; }
.lui-props tbody tr:last-child td { border-bottom: 0; }
.lui-props td:nth-child(-n+2), .lui-props td:nth-child(4), .lui-props td:nth-child(5) { white-space: nowrap; }
.lui-props td:nth-child(3) { min-width: 10rem; }
.lui-props td:nth-child(6) { min-width: 16rem; }
.lui-props td:nth-child(7) { white-space: nowrap; }
.lui-index { max-width: none; }
.lui-index h2 { margin: 3rem 0 0.25rem; font-size: 1.5rem; line-height: 2rem; }
.lui-index-layer { margin: 0 0 1rem; color: var(--lui-muted); font-size: 0.875rem; }
.lui-index h3 { margin: 1.5rem 0 0.75rem; font-size: 0.875rem; font-weight: 500; color: var(--lui-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.lui-index > ul { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(18rem, 1fr)); gap: 1rem; }
/* Each component is a card: its name (the link to its page), what it is for, and the
   component itself, live, on a small stage. Widgets, blocks and flows take the whole row. */
.lui-index > ul > li {
  display: grid; grid-template-rows: auto auto 1fr; min-width: 0; max-width: none; padding: 1rem;
  background: var(--lui-card); border: 1px solid var(--lui-line); border-radius: var(--lui-radius-lg);
  box-shadow: var(--lui-shadow-xs);
}
.lui-index > ul > li.lui-index-wide { grid-column: 1 / -1; }
.lui-index > ul > li > p { margin: 0 0 0.25rem; }
.lui-index > ul > li > p:first-child > a { font-size: 1rem; font-weight: 600; line-height: 1.5rem; color: var(--lui-fg); text-decoration: none; }
.lui-index > ul > li > p:first-child > a:hover { text-decoration: underline; }
.lui-index-stage {
  min-width: 0; margin-top: 0.75rem; padding: calc(var(--lui-space) * 3) calc(var(--lui-space) * 2);
  background: var(--lui-bg); border: 1px solid var(--lui-line); border-radius: var(--lui-radius);
}
.lui-index-stage > :last-child { margin-bottom: 0; }

@media (prefers-reduced-motion: reduce) {
  *, ::before, ::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  ::view-transition-group(*), ::view-transition-old(*), ::view-transition-new(*) { animation: none !important; }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oklch::contrast;

    /// Every text colour clears WCAG AA (4.5:1) on each surface it is drawn on, in both
    /// schemes, for the default tokens: the roles resolve through the scales first.
    #[test]
    fn default_text_clears_4_5_to_1() {
        let (t, mut failed) = (Tokens::default(), Vec::new());
        for (dark, p) in [(false, &t.light), (true, &t.dark)] {
            let c = |v: &str| {
                t.color(dark, v)
                    .unwrap_or_else(|| panic!("{v} is not a colour"))
            };
            let mut pairs = vec![
                ("on_accent", p.on_accent, "accent", p.accent),
                ("on_primary", p.on_primary, "primary", p.primary),
                ("on_danger", p.on_danger, "danger", p.danger),
            ];
            // Status colours are text on the page's surfaces; chips and badges tint their own.
            for (bn, bv) in [
                ("bg", p.bg),
                ("surface", p.surface),
                ("card", p.card),
                ("popover", p.popover),
                ("secondary", p.secondary),
            ] {
                for (fname, fv) in [
                    ("fg", p.fg),
                    ("muted", p.muted),
                    ("link", p.link),
                    ("danger", p.danger),
                    ("ok", p.ok),
                    ("warn", p.warn),
                ] {
                    if bn != "secondary" || matches!(fname, "fg" | "muted" | "link") {
                        pairs.push((fname, fv, bn, bv));
                    }
                }
            }
            for (fname, fv, bn, bv) in pairs {
                let r = contrast(&c(fv), &c(bv));
                if r < 4.5 {
                    failed.push(format!("{fname} on {bn} is {r:.2}:1 (dark: {dark})"));
                }
            }
        }
        assert!(failed.is_empty(), "{failed:#?}");
    }

    /// White text stays readable over every stop of the primary gradient, in both schemes.
    #[test]
    fn on_primary_clears_4_5_to_1_over_the_gradient() {
        let t = Tokens::default();
        for (dark, depth, p) in [(false, DEPTH_LIGHT, &t.light), (true, DEPTH_DARK, &t.dark)] {
            let line = depth
                .lines()
                .find(|l| l.contains("--lui-gradient-primary"))
                .unwrap();
            let stops: Vec<&str> = line
                .split("var(")
                .skip(1)
                .map(|v| v.split(')').next().unwrap())
                .collect();
            assert_eq!(stops.len(), 2, "{line}");
            for stop in stops {
                let bg = t.color(dark, &format!("var({stop})")).unwrap();
                let r = contrast(&t.color(dark, p.on_primary).unwrap(), &bg);
                assert!(
                    r >= 4.5,
                    "on_primary over {stop} is {r:.2}:1 (dark: {dark})"
                );
            }
        }
    }

    /// A derived scale keeps the profile's text steps readable whatever the seed: brand 11
    /// (links) and gray 11 and 12 on the backgrounds, and `on_solid` on step 9.
    #[test]
    fn derived_scales_keep_text_readable() {
        let indigo = Scale::derive("#3e63dd", &Scale::INDIGO).unwrap();
        assert_eq!(indigo.light[8], "#3e63dd");
        assert_eq!(indigo.light[10], "#3a5bc7");
        let mut failed = Vec::new();
        for brand in [
            "#3e63dd", "#12a594", "#e5484d", "#ffe629", "#8e4ec6", "#f76b15", "#000000",
        ] {
            for gray in [
                "#8b8d98", "#8d8d8d", "#8e8c99", "#868e8b", "#8d8d86", "#ffffff",
            ] {
                let (b, g) = (
                    Scale::derive(brand, &Scale::INDIGO).unwrap(),
                    Scale::derive(gray, &Scale::SLATE).unwrap(),
                );
                if contrast(b.on_solid(), &b.light[8]) < 4.5 {
                    failed.push(format!("{brand}: on_solid on step 9"));
                }
                for (dark, bs, gs) in [(false, &b.light, &g.light), (true, &b.dark, &g.dark)] {
                    for bg in 0..3 {
                        for (name, fg) in [
                            ("brand 11", &bs[10]),
                            ("gray 11", &gs[10]),
                            ("gray 12", &gs[11]),
                        ] {
                            let r = contrast(fg, &gs[bg]);
                            if r < 4.5 {
                                failed.push(format!(
                                    "{brand}/{gray}: {name} on gray {} is {r:.2} (dark: {dark})",
                                    bg + 1
                                ));
                            }
                        }
                    }
                }
            }
        }
        assert!(failed.is_empty(), "{failed:#?}");
    }

    #[test]
    fn scales_are_emitted_as_hex_and_roles_alias_them() {
        let css = Tokens::default().css();
        assert!(css.contains("--lui-gray-12: #1c2024;"), "{css}");
        assert!(css.contains("--lui-brand-9: #3e63dd;"));
        assert!(css.contains("--lui-bg: var(--lui-gray-1);"));
        assert!(!css.contains("oklch("));
        assert_eq!(
            Tokens::default()
                .color(true, "var(--lui-gray-1)")
                .as_deref(),
            Some("#111113")
        );
    }
}
