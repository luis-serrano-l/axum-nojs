//! # Color
//!
//! A colour picker whose value the server remembers, no script. It can offer a row of preset
//! swatches and an opacity slider.
//!
//! **Platform features:** `<input type="color">` (Chrome 20, Firefox 29, Safari 12.1). The
//! swatch beside it is a plain `<span>` painted with the server's current value through the
//! custom properties `--wo-color-value` and `--wo-color-alpha`, mixed with `color-mix()`
//! (Chrome 111, Firefox 113, Safari 16.2) over a checkerboard so transparency shows. Presets
//! are `<button name="<name>-preset" value="#rrggbb">`: one click posts the form with that
//! colour. Opacity is an `<input type="range">` named `<name>-alpha` (0 to 100).
//!
//! **What it does not do without script:** a live preview of the chosen colour before the form
//! is sent, and an eyedropper.
//!
//! **Fallback:** none needed; a browser without a colour picker shows a text field that
//! accepts `#rrggbb`.
//!
//! **Enhanced:** the enhancement script repaints the swatch and the code while the picker moves
//! and mirrors the opacity into its `<output>`.
//!
//! ```rust
//! use webonsive::{Caps, color, color::{ColorOptions, hex_alpha}};
//! let m = color(&Caps::all(), "accent", "#2f5bea", Default::default());
//! assert!(m.into_string().contains("type=\"color\""));
//! let m = color(&Caps::all(), "accent", "#2f5bea", ColorOptions::default().presets(&["#1f6f5f", "#b3261e"]).alpha(80));
//! let html = m.into_string();
//! assert!(html.contains("name=\"accent-preset\" value=\"#b3261e\""));
//! assert!(html.contains("name=\"accent-alpha\""));
//! assert_eq!(hex_alpha("#2f5bea", 80), "#2f5beacc");
//! ```

use maud::{Markup, html};

use crate::Caps;

/// Options for [`color`]; `Default::default()` is the picker alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ColorOptions<'a> {
    /// `#rrggbb` swatches that post `<name>-preset` when clicked.
    pub presets: &'a [&'a str],
    /// Opacity in percent; `Some` shows the slider.
    pub alpha: Option<u8>,
}

impl<'a> ColorOptions<'a> {
    /// Preset swatches.
    pub fn presets(mut self, presets: &'a [&'a str]) -> Self {
        self.presets = presets;
        self
    }
    /// Show the opacity slider at `percent` (clamped to 100).
    pub fn alpha(mut self, percent: u8) -> Self {
        self.alpha = Some(percent.min(100));
        self
    }
}

/// `#rrggbb` plus an opacity percent as `#rrggbbaa`.
pub fn hex_alpha(hex: &str, percent: u8) -> String {
    format!("{hex}{:02x}", (u32::from(percent.min(100)) * 255 + 50) / 100)
}

/// A colour input named `name` with the current `#rrggbb` value and a swatch of it.
pub fn color(_caps: &Caps, name: &str, value: &str, options: ColorOptions) -> Markup {
    let ColorOptions { presets, alpha } = options;
    let id = format!("f-{name}");
    let pct = alpha.unwrap_or(100);
    html! {
        div class="wo-color" {
            input type="color" id=(id) name=(name) value=(value);
            span class="wo-color-swatch" style={ "--wo-color-value: " (value) "; --wo-color-alpha: " (pct) "%" } aria-hidden="true" {}
            code { @if pct < 100 { (hex_alpha(value, pct)) } @else { (value) } }
            @if alpha.is_some() {
                label class="wo-color-alpha" {
                    "Opacity "
                    input type="range" id={ (id) "-alpha" } name={ (name) "-alpha" } min="0" max="100" value=(pct);
                    span { output for={ (id) "-alpha" } { (pct) } "%" }
                }
            }
            @if !presets.is_empty() {
                span class="wo-color-presets" role="group" aria-label="Presets" {
                    @for p in presets {
                        button type="submit" name={ (name) "-preset" } value=(p) aria-label={ "Use " (p) }
                            aria-pressed=(if p.eq_ignore_ascii_case(value) { "true" } else { "false" }) style={ "--wo-color-value: " (p) } {}
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-color { display: inline-flex; flex-wrap: wrap; align-items: center; gap: var(--wo-space); }
.wo-color > input { width: 3rem; height: 2.25rem; padding: 2px; }
.wo-color-swatch {
  width: 2rem; height: 2rem; border-radius: var(--wo-radius); border: 1px solid var(--wo-line);
  background: linear-gradient(color-mix(in srgb, var(--wo-color-value) var(--wo-color-alpha, 100%), transparent) 0 0),
    repeating-conic-gradient(var(--wo-line) 0 25%, var(--wo-surface) 0 50%) 0 0 / 0.75rem 0.75rem;
}
.wo-color-alpha { display: inline-flex; align-items: center; gap: 0.5rem; font-weight: 400; }
.wo-color-alpha input { width: 8rem; accent-color: var(--wo-accent); }
.wo-color-alpha output { min-width: 3ch; text-align: right; font-variant-numeric: tabular-nums; }
.wo-color-presets { display: flex; flex-basis: 100%; gap: 0.4rem; }
.wo-color-presets button {
  width: 1.75rem; height: 1.75rem; padding: 0; border-radius: 50%; cursor: pointer;
  background: var(--wo-color-value); border: 2px solid var(--wo-surface); box-shadow: 0 0 0 1px var(--wo-line);
}
.wo-color-presets button[aria-pressed=true] { box-shadow: 0 0 0 2px var(--wo-fg); }
"#;
