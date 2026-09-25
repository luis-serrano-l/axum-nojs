//! # One-time code
//!
//! The box for a code sent by text or email: one field, drawn as a row of cells, that the
//! phone's keyboard offers to fill from the message. One field (not six) so paste, autofill
//! and the back key all just work.
//!
//! **Platform features:** `autocomplete="one-time-code"` (Safari 12, and Chrome on Android
//! through the WebOTP flow), `inputmode="numeric"` (Chrome 66, Firefox 95, Safari 12.1),
//! `pattern` and `maxlength` for the length; a monospace face with `letter-spacing` and a
//! repeating background draws the cells.
//!
//! **Accessibility:** a labelled text field (not a row of fields), so it is announced once and
//! typed or pasted in one go; the pattern's hint says how many digits. Checked by axe-core in
//! headless Firefox on every demo route, both capability variants, light and dark (no serious
//! or critical violation).
//!
//! **What it does not do without script:** move the caret to the next cell as it would with
//! separate boxes (there is one box), or send the form when the last digit is typed.
//!
//! **Fallback:** without the cell background it is a plain spaced-out field.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.input_otp("code", "Code from the text message").render().into_string();
//! assert!(m.contains(r#"autocomplete="one-time-code""#) && m.contains(r#"inputmode="numeric""#));
//! assert!(m.contains(r#"pattern="[0-9]{6}""#) && m.contains(r#"maxlength="6""#));
//!
//! let m = ui.input_otp("code", "Code").length(4).render().into_string();
//! assert!(m.contains(r#"pattern="[0-9]{4}""#) && m.contains("--lui-otp-cells: 4"));
//! // The same in `lui!`:
//! let same = lui! { InputOtp("code", "Code") length=4; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::i18n::Text;
use crate::props::{Prop, PropKind};

/// A one-time code field, made by [`Ui::input_otp`].
///
/// **Setters.** Values and items: `.length(..)`.
#[derive(Clone, Debug)]
pub struct InputOtp<'a> {
    ui: &'a Ui,
    name: &'a str,
    label: &'a str,
    length: u8,
}

impl InputOtp<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[Prop::new("length", PropKind::Number, "digits: u8")
        .default("6")
        .attr("maxlength")
        .doc("How many digits the code has.")];
}

impl Ui {
    /// A field posting `name`, labelled `label`, for a six-digit code.
    pub fn input_otp<'a>(&'a self, name: &'a str, label: &'a str) -> InputOtp<'a> {
        InputOtp {
            ui: self,
            name,
            label,
            length: 6,
        }
    }
}

impl InputOtp<'_> {
    /// How many digits the code has.
    pub fn length(mut self, digits: u8) -> Self {
        self.length = digits.max(1);
        self
    }
}

impl Render for InputOtp<'_> {
    fn render(&self) -> Markup {
        let n = self.length;
        let pattern = format!("[0-9]{{{n}}}");
        let hint = self.ui.fill(Text::Digits, &[&n]);
        let value = self.ui.param(self.name).unwrap_or("");
        html! {
            div class="lui-otp" style={ "--lui-otp-cells: " (n) } {
                (self.ui.input(self.name, self.label)
                    .value(value)
                    .pattern(&pattern, &hint)
                    .maxlength(usize::from(n))
                    .inputmode("numeric")
                    .autocomplete("one-time-code")
                    .class("lui-otp-input"))
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn InputOTP: square
/// cells side by side; here one field whose background draws them and whose letter spacing
/// puts one digit in each.
pub const CSS: &str = r#"
.lui-otp { --lui-otp-cell: 2.5rem; }
.lui-otp .lui-otp-input {
  box-sizing: content-box; width: calc(var(--lui-otp-cell) * var(--lui-otp-cells)); height: var(--lui-otp-cell); padding: 0;
  font-family: var(--lui-font-mono); font-size: 1.25rem; font-variant-numeric: tabular-nums;
  letter-spacing: calc(var(--lui-otp-cell) - 1ch); text-indent: calc((var(--lui-otp-cell) - 1ch) / 2);
  background: linear-gradient(to right, transparent calc(var(--lui-otp-cell) - 1px), var(--lui-input) 0) 0 0 / var(--lui-otp-cell) 100% repeat-x, var(--lui-bg);
}
"#;
