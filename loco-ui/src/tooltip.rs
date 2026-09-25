//! # Tooltip
//!
//! A short label that appears beside a control while the pointer is over it or it has focus:
//! what an icon button does, what a shortcut is.
//!
//! **Platform features:** CSS only: the text is a `role="tooltip"` element shown by
//! `:hover` and `:focus-within` (baseline 2020) on the wrapper, and the trigger inside names
//! it with `aria-describedby` so a screen reader says it too. `@media (hover: none)` (Chrome
//! 41, Firefox 64, Safari 9) keeps it off touch screens, where hover does not exist.
//!
//! **Accessibility:** `role="tooltip"` joined to its trigger by `aria-describedby`; shown on
//! hover and on keyboard focus. Checked by axe-core in headless Firefox on every demo route,
//! both capability variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** open after a delay, close on Escape while the
//! pointer stays over it, or flip to fit the viewport. `popover="hint"` with hover triggers
//! would do some of this, but it is Chromium-only and needs script to open on hover.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.tooltip("Copy the link", html! { (ui.button("").icon().ghost().label("Copy").content(html! { (Icon::Copy) })) });
//! let m = m.render().into_string();
//! assert!(m.contains(r#"role="tooltip""#) && m.contains("Copy the link"));
//! assert!(m.contains(r#"aria-describedby="lui-tooltip-copy-the-link""#));
//! let below = ui.tooltip("Saved", html! { span { "3" } }).below().render().into_string();
//! assert!(below.contains("lui-tooltip-below"));
//! // The same in `lui!`:
//! let same = lui! { Tooltip("Copy the link", lui! {
//!     Button("") icon ghost label="Copy" content=(html! { (Icon::Copy) });
//! }); };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, PreEscaped, Render, html};

use crate::props::{Prop, PropKind};
use crate::{Ui, slug};

/// A trigger with a tooltip, made by [`Ui::tooltip`].
///
/// **Setters.** Values and items: `.id(..)`; switches: `.below()`.
#[derive(Clone, Debug)]
pub struct Tooltip<'a> {
    text: &'a str,
    trigger: Markup,
    below: bool,
    id: Option<&'a str>,
}

impl Tooltip<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("below", PropKind::Switch, "")
            .doc("Show the text under the trigger instead of above it."),
        Prop::new("id", PropKind::Value, "id: &'a str")
            .attr("id")
            .doc("The tooltip's id, `lui-tooltip-<slug of the text>` by default."),
    ];
}

impl Ui {
    /// `trigger` (a button, an icon, a word) with `text` shown above it on hover and focus.
    pub fn tooltip<'a>(&self, text: &'a str, trigger: Markup) -> Tooltip<'a> {
        Tooltip {
            text,
            trigger,
            below: false,
            id: None,
        }
    }
}

impl<'a> Tooltip<'a> {
    /// Show the text under the trigger instead of above it.
    pub fn below(mut self) -> Self {
        self.below = true;
        self
    }

    /// The tooltip's id, `lui-tooltip-<slug of the text>` by default.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }
}

impl Render for Tooltip<'_> {
    fn render(&self) -> Markup {
        let id = self.id.map_or_else(
            || format!("lui-tooltip-{}", slug(self.text)),
            str::to_string,
        );
        // The trigger is caller markup: its first element gets `aria-describedby` by string,
        // since Maud cannot add an attribute to markup it did not build.
        let trigger = self.trigger.0.replacen(
            '>',
            &format!(" aria-describedby=\"{id}\">"),
            usize::from(self.trigger.0.starts_with('<')),
        );
        html! {
            span class={ "lui-tooltip" @if self.below { " lui-tooltip-below" } } {
                (PreEscaped(trigger))
                span id=(id) role="tooltip" class="lui-tooltip-text" { (self.text) }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn Tooltip: the primary
/// colour as the background, text-xs, rounded-md, px-3 py-1.5.
pub const CSS: &str = r#"
.lui-tooltip { position: relative; display: inline-flex; }
.lui-tooltip-text {
  position: absolute; z-index: 30; left: 50%; bottom: calc(100% + 6px); translate: -50% 0;
  width: max-content; max-width: 16rem; padding: 0.375rem 0.75rem; pointer-events: none;
  font-size: 0.75rem; line-height: 1rem; font-weight: 400; text-align: center;
  color: var(--lui-on-primary); background: var(--lui-primary); border-radius: var(--lui-radius-sm);
  opacity: 0; visibility: hidden; transition: opacity 0.15s, visibility 0.15s;
}
.lui-tooltip-below .lui-tooltip-text { bottom: auto; top: calc(100% + 6px); }
.lui-tooltip:hover .lui-tooltip-text, .lui-tooltip:focus-within .lui-tooltip-text { opacity: 1; visibility: visible; }
@media (hover: none) { .lui-tooltip:hover:not(:focus-within) .lui-tooltip-text { opacity: 0; visibility: hidden; } }
@media (prefers-reduced-motion: reduce) { .lui-tooltip-text { transition: none; } }
"#;
