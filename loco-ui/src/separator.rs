//! # Separator
//!
//! A hairline between groups of content, across or down, optionally with a word in the middle
//! ("or" between two sign-in methods).
//!
//! **Platform features:** `<hr>` for a plain horizontal rule (its semantics are a thematic
//! break); a labelled or vertical one is a `role="separator"` element with
//! `aria-orientation`.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! assert_eq!(ui.separator().render().into_string(), r#"<hr class="lui-separator">"#);
//! let or = ui.separator().label("or").render().into_string();
//! assert!(or.contains(r#"role="separator""#) && or.contains(">or<"));
//! assert!(ui.separator().vertical().render().into_string().contains(r#"aria-orientation="vertical""#));
//! // The same in `lui!`:
//! let same = lui! { Separator label="or"; };
//! assert_eq!(same.into_string(), or);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// A separator, made by [`Ui::separator`].
///
/// **Setters.** Values and items: `.label(..)`; switches: `.vertical()`.
#[derive(Clone, Debug, Default)]
pub struct Separator<'a> {
    label: Option<&'a str>,
    vertical: bool,
}

impl Separator<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("label", PropKind::Value, "text: &'a str")
            .doc("A word in the middle of the line."),
        Prop::new("vertical", PropKind::Switch, "")
            .doc("Down instead of across, between items in a row (a `ui.cluster`)."),
    ];
}

impl Ui {
    /// A horizontal rule.
    pub fn separator<'a>(&self) -> Separator<'a> {
        Separator::default()
    }
}

impl<'a> Separator<'a> {
    /// A word in the middle of the line.
    pub fn label(mut self, text: &'a str) -> Self {
        self.label = Some(text);
        self
    }

    /// Down instead of across, between items in a row (a `ui.cluster`).
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
}

impl Render for Separator<'_> {
    fn render(&self) -> Markup {
        html! {
            @if self.vertical {
                span class="lui-separator lui-separator-vertical" role="separator" aria-orientation="vertical" {}
            } @else if let Some(l) = self.label {
                div class="lui-separator lui-separator-label" role="separator" { span { (l) } }
            } @else {
                hr class="lui-separator";
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn Separator: a 1px
/// line in `--lui-line`.
pub const CSS: &str = r#"
.lui-separator { border: 0; margin: 1rem 0; }
hr.lui-separator { height: 1px; background: var(--lui-line); }
.lui-separator-vertical { display: inline-block; align-self: stretch; width: 1px; min-height: 1rem; margin: 0; background: var(--lui-line); }
.lui-separator-label { display: flex; align-items: center; gap: 0.75rem; color: var(--lui-muted); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; }
.lui-separator-label::before, .lui-separator-label::after { content: ""; flex: 1; height: 1px; background: var(--lui-line); }
"#;
