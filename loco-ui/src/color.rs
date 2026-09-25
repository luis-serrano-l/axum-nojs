//! # Color
//!
//! A colour picker whose value the server remembers, no script. It can offer a row of preset
//! swatches and an opacity slider.
//!
//! **Platform features:** `<input type="color">` (Chrome 20, Firefox 29, Safari 12.1). The
//! swatch beside it is a plain `<span>` painted with the server's current value through the
//! custom properties `--lui-color-value` and `--lui-color-alpha`, mixed with `color-mix()`
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
//! use loco_ui::{prelude::*, color::hex_alpha};
//! let ui = Ui::from(Caps::all());
//! assert!(ui.color("accent", "#2f5bea").render().into_string().contains("type=\"color\""));
//! let m = ui.color("accent", "#2f5bea").presets(&["#1f6f5f", "#b3261e"]).alpha(80).label("Accent");
//! let html = m.render().into_string();
//! assert!(html.contains("name=\"accent-preset\" value=\"#b3261e\""));
//! assert!(html.contains("name=\"accent-alpha\"") && html.contains(r#"<label for="f-accent">"#));
//! assert_eq!(hex_alpha("#2f5bea", 80), "#2f5beacc");
//!
//! // The same in `lui!`:
//! let same = lui! {
//!     Color("accent", "#2f5bea") presets=(&["#1f6f5f", "#b3261e"]) alpha=80 label="Accent";
//! };
//! assert_eq!(same.into_string(), m.render().into_string());
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::i18n::{Strings, Text};
use crate::props::{Prop, PropKind};
use crate::{Caps, Ui};

/// A colour input with a swatch of its current value, made by [`Ui::color`].
///
/// **Setters.** Values and items: `.presets(..)`, `.alpha(..)`, `.label(..)`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Color<'a> {
    name: &'a str,
    value: &'a str,
    presets: &'a [&'a str],
    alpha: Option<u8>,
    label: Option<&'a str>,
    strings: &'static Strings,
}

impl Color<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("presets", PropKind::Value, "presets: &'a [&'a str]")
            .doc("`#rrggbb` swatches that post `<name>-preset` when clicked."),
        Prop::new("alpha", PropKind::Number, "percent: u8")
            .doc("An opacity slider (`<name>-alpha`) at `percent`, clamped to 100."),
        Prop::new("label", PropKind::Value, "label: &'a str")
            .doc("A `<label>` above the picker, in a `div.lui-field` like a form field."),
    ];
}

impl Ui {
    /// A colour input named `name` holding the `#rrggbb` `value`.
    pub fn color<'a>(&self, name: &'a str, value: &'a str) -> Color<'a> {
        Color {
            name,
            value,
            strings: self.strings,
            ..Color::default()
        }
    }
}

impl<'a> Color<'a> {
    /// `#rrggbb` swatches that post `<name>-preset` when clicked.
    pub fn presets(mut self, presets: &'a [&'a str]) -> Self {
        self.presets = presets;
        self
    }

    /// An opacity slider (`<name>-alpha`) at `percent`, clamped to 100.
    pub fn alpha(mut self, percent: u8) -> Self {
        self.alpha = Some(percent.min(100));
        self
    }

    /// A `<label>` above the picker, in a `div.lui-field` like a form field.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

/// `#rrggbb` plus an opacity percent as `#rrggbbaa`.
pub fn hex_alpha(hex: &str, percent: u8) -> String {
    format!(
        "{hex}{:02x}",
        (u32::from(percent.min(100)) * 255 + 50) / 100
    )
}

impl Render for Color<'_> {
    fn render(&self) -> Markup {
        let Color {
            strings: _,
            name,
            value,
            presets,
            alpha,
            label,
        } = *self;
        let id = format!("f-{name}");
        let pct = alpha.unwrap_or(100);
        let preset_name = format!("{name}-preset");
        crate::labelled(
            label,
            &id,
            html! {
                div class="lui-color" {
                    input type="color" class="lui-color-input" id=(id) name=(name) value=(value);
                    span class="lui-color-swatch" style={ "--lui-color-value: " (value) "; --lui-color-alpha: " (pct) "%" } aria-hidden="true" {}
                    code { @if pct < 100 { (hex_alpha(value, pct)) } @else { (value) } }
                    @if alpha.is_some() {
                        label class="lui-color-alpha" {
                            (self.strings.get(Text::Opacity).replace("{}", "").trim_end()) " "
                            input type="range" class="lui-color-alpha-range" id={ (id) "-alpha" } name={ (name) "-alpha" } min="0" max="100" value=(pct);
                            span { output for={ (id) "-alpha" } { (pct) } "%" }
                        }
                    }
                    @if !presets.is_empty() {
                        span class="lui-color-presets" role="group" aria-label=(self.strings.get(Text::Presets)) {
                            @for p in presets {
                                @let use_p = self.strings.fill(Text::UseValue, &[p]);
                                (Button::new(Caps::NONE, "").name(&preset_name).value(p).label(&use_p).pressed(p.eq_ignore_ascii_case(value)).style(format!("--lui-color-value: {p}")))
                            }
                        }
                    }
                }
            },
        )
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-color { display: inline-flex; flex-wrap: wrap; align-items: center; gap: var(--lui-space); }
.lui-color-input { width: 3rem; height: 2.25rem; padding: 2px; }
.lui-color-swatch {
  width: 2.25rem; height: 2.25rem; border-radius: var(--lui-radius-sm); border: 1px solid var(--lui-input); box-shadow: var(--lui-shadow-xs);
  background: linear-gradient(color-mix(in srgb, var(--lui-color-value) var(--lui-color-alpha, 100%), transparent) 0 0),
    repeating-conic-gradient(var(--lui-line) 0 25%, var(--lui-surface) 0 50%) 0 0 / 0.75rem 0.75rem;
}
.lui-color-alpha { display: inline-flex; align-items: center; gap: 0.5rem; font-weight: 400; }
.lui-color-alpha-range { width: 8rem; accent-color: var(--lui-primary); }
.lui-color-alpha output { min-width: 3ch; text-align: right; font-variant-numeric: tabular-nums; }
.lui-color-presets { display: flex; flex-basis: 100%; gap: 0.5rem; }
/* A preset is a button drawn as a round swatch of its colour. */
.lui-color-presets .lui-button {
  width: 1.75rem; height: 1.75rem; min-height: 0; padding: 0; border-radius: 50%;
  background: var(--lui-color-value); border: 2px solid var(--lui-bg); box-shadow: 0 0 0 1px var(--lui-input);
}
.lui-color-presets .lui-button:hover { background: var(--lui-color-value); box-shadow: 0 0 0 1px var(--lui-ring); }
.lui-color-presets .lui-button[aria-pressed=true] { box-shadow: 0 0 0 2px var(--lui-fg); }
"#;
