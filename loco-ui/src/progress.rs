//! # Progress
//!
//! How far along a task is: a bar filled to a value out of a maximum, or an indeterminate
//! bar when the server does not know yet.
//!
//! **Platform features:** `<progress>` (baseline 2013) with its label tied by `<label for>`,
//! so a screen reader says the value; `appearance: none` and the `::-webkit-progress-*` and
//! `::-moz-progress-bar` pseudo-elements draw it in the shadcn shape. An indeterminate bar is
//! a `<progress>` with no value.
//!
//! **What it does not do without script:** move on its own; the value is what the server knew
//! when it rendered the page (a streamed page or the enhancement script can send a newer one).
//!
//! **Fallback:** without the pseudo-elements a browser draws its own bar.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.progress(33, 100).label("Upload").render().into_string();
//! assert!(m.contains(r#"<progress id="lui-progress-upload" class="lui-progress" value="33" max="100">33%</progress>"#));
//! let m = ui.progress(0, 0).label("Waiting").render().into_string();
//! assert!(m.contains(r#"class="lui-progress">"#) && !m.contains("value="));
//! // The same in `lui!`:
//! let same = lui! { Progress(0, 0) label="Waiting"; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::props::{Prop, PropKind};
use crate::{Ui, slug};

/// A progress bar, made by [`Ui::progress`].
///
/// **Setters.** Values and items: `.label(..)`.
#[derive(Clone, Debug)]
pub struct Progress<'a> {
    value: u64,
    max: u64,
    label: Option<&'a str>,
}

impl Progress<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[Prop::new("label", PropKind::Value, "text: &'a str")
        .doc("A label above the bar, with the percentage beside it.")];
}

impl Ui {
    /// A bar filled to `value` of `max`; `max` of 0 draws an indeterminate bar.
    pub fn progress<'a>(&self, value: u64, max: u64) -> Progress<'a> {
        Progress {
            value,
            max,
            label: None,
        }
    }
}

impl<'a> Progress<'a> {
    /// A label above the bar, with the percentage beside it.
    pub fn label(mut self, text: &'a str) -> Self {
        self.label = Some(text);
        self
    }
}

impl Render for Progress<'_> {
    fn render(&self) -> Markup {
        let known = self.max > 0;
        let pct = if known {
            (self.value.min(self.max) * 100) / self.max
        } else {
            0
        };
        let id = format!("lui-progress-{}", slug(self.label.unwrap_or("bar")));
        html! {
            div class="lui-progress-field" {
                @if let Some(l) = self.label {
                    label for=(id) { span { (l) } @if known { span class="lui-progress-value" { (pct) "%" } } }
                }
                @if known {
                    progress id=(id) class="lui-progress" value=(self.value.min(self.max)) max=(self.max) { (pct) "%" }
                } @else {
                    progress id=(id) class="lui-progress" {}
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn Progress: an 8px
/// rounded track in the primary colour at 20%, the fill in the primary colour.
pub const CSS: &str = r#"
.lui-progress-field { display: grid; gap: 0.5rem; }
.lui-progress-field label { display: flex; justify-content: space-between; font-size: 0.875rem; font-weight: 500; }
.lui-progress-value { color: var(--lui-muted); font-variant-numeric: tabular-nums; font-weight: 400; }
.lui-progress {
  appearance: none; display: block; width: 100%; height: 0.5rem; border: 0; border-radius: 9999px; overflow: hidden;
  background: color-mix(in srgb, var(--lui-primary) 20%, transparent); accent-color: var(--lui-primary);
}
.lui-progress::-webkit-progress-bar { background: transparent; }
.lui-progress::-webkit-progress-value { background: var(--lui-primary); border-radius: 9999px; transition: width 0.3s; }
.lui-progress::-moz-progress-bar { background: var(--lui-primary); border-radius: 9999px; }
"#;
