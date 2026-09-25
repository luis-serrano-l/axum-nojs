//! # Stack
//!
//! Children one under the other with an even gap between them and no margins to fight: the
//! vertical rhythm of a form, a card body, a settings page.
//!
//! **Platform features:** a flex column with `gap` (Chrome 84, Firefox 63, Safari 14.1);
//! the gap is a step of the `--nojs-space-*` scale.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.stack(html! { p { "One" } p { "Two" } }).render().into_string();
//! assert_eq!(m, r#"<div class="nojs-stack"><p>One</p><p>Two</p></div>"#);
//! // The same in `nojs!`:
//! let same = nojs! { Stack(html! { p { "One" } p { "Two" } }); };
//! assert_eq!(same.into_string(), m);
//! // `.gap(n)` picks a step: n × 4px with the default `--nojs-space`.
//! let m = ui.stack(html! { p { "Tight" } }).gap(2).render().into_string();
//! assert!(m.contains(r#"class="nojs-stack nojs-gap-2""#));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// A vertical stack, made by [`Ui::stack`].
///
/// **Setters.** Values and items: `.gap(..)`.
#[derive(Clone, Debug)]
pub struct Stack {
    content: Markup,
    gap: Option<u8>,
}

impl Stack {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[Prop::new("gap", PropKind::Number, "n: u8")
        .doc("The gap as a step of the `--nojs-space-*` scale.")];
}

impl Ui {
    /// `content`'s top-level elements one under the other, 16px apart by default.
    pub fn stack(&self, content: Markup) -> Stack {
        Stack { content, gap: None }
    }
}

impl Stack {
    /// The gap as a step of the `--nojs-space-*` scale: 0, 1, 2, 3, 4, 6 or 8 (n × 4px by
    /// default); other values take the step below.
    pub fn gap(mut self, n: u8) -> Self {
        self.gap = Some(n);
        self
    }
}

impl Render for Stack {
    fn render(&self) -> Markup {
        html! {
            div class={ "nojs-stack" @if let Some(n) = self.gap { " " (crate::gap_class(n)) } } { (self.content) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-stack { display: flex; flex-direction: column; }
:where(.nojs-stack) { gap: var(--nojs-space-4); }
.nojs-stack > * { margin-block: 0; }
/* Fields and blocks take the full width; a button or badge keeps its own. */
.nojs-stack > :is(.nojs-button, .nojs-badge) { align-self: flex-start; }
"#;
