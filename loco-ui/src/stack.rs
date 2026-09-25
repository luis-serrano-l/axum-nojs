//! # Stack
//!
//! Children one under the other with an even gap between them and no margins to fight: the
//! vertical rhythm of a form, a card body, a settings page.
//!
//! **Platform features:** a flex column with `gap` (Chrome 84, Firefox 63, Safari 14.1);
//! the gap is a step of the `--lui-space-*` scale.
//!
//! **Accessibility:** layout only: no roles, reading order is source order. Checked by axe-core
//! in headless Firefox on every demo route, both capability variants, light and dark (no
//! serious or critical violation).
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.stack(html! { p { "One" } p { "Two" } }).render().into_string();
//! assert_eq!(m, r#"<div class="lui-stack"><p>One</p><p>Two</p></div>"#);
//! // The same in `lui!`:
//! let same = lui! { Stack(html! { p { "One" } p { "Two" } }); };
//! assert_eq!(same.into_string(), m);
//! // `.gap(n)` picks a step: n × 4px with the default `--lui-space`.
//! let m = ui.stack(html! { p { "Tight" } }).gap(2).render().into_string();
//! assert!(m.contains(r#"class="lui-stack lui-gap-2""#));
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
        .doc("The gap as a step of the `--lui-space-*` scale.")];
}

impl Ui {
    /// `content`'s top-level elements one under the other, 16px apart by default.
    pub fn stack(&self, content: Markup) -> Stack {
        Stack { content, gap: None }
    }
}

impl Stack {
    /// The gap as a step of the `--lui-space-*` scale: 0, 1, 2, 3, 4, 6 or 8 (n × 4px by
    /// default); other values take the step below.
    pub fn gap(mut self, n: u8) -> Self {
        self.gap = Some(n);
        self
    }
}

impl Render for Stack {
    fn render(&self) -> Markup {
        html! {
            div class={ "lui-stack" @if let Some(n) = self.gap { " " (crate::gap_class(n)) } } { (self.content) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-stack { display: flex; flex-direction: column; }
:where(.lui-stack) { gap: var(--lui-space-4); }
.lui-stack > * { margin-block: 0; }
/* Fields and blocks take the full width; a button or badge keeps its own. */
.lui-stack > :is(.lui-button, .lui-badge) { align-self: flex-start; }
"#;
