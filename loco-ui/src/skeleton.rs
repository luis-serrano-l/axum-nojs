//! # Skeleton
//!
//! Grey bars in the shape of content that is still coming: the placeholder for a streamed
//! [`crate::Ui::slot`], a lazy panel, anything the server fills in later.
//!
//! **Platform features:** `aria-busy="true"` and `role="status"` with a visually hidden
//! label, so assistive technology hears "Loading" once instead of reading empty boxes; a
//! shimmer drawn with `@keyframes` over a `linear-gradient` that
//! `prefers-reduced-motion: reduce` stops.
//!
//! **Fallback:** without CSS animations the bars are still; nothing else differs.
//!
//! **Accessibility:** `role="status"` with `aria-busy` and a "Loading" name; the bars are
//! `aria-hidden`. Checked by axe-core in headless Firefox on every demo route, both capability
//! variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** it cannot be removed by the client; it is replaced
//! when the real content arrives (a streamed slot, a swap, the next page).
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from(Caps::all());
//! assert_eq!(ui.skeleton(3).render().into_string().matches("lui-skeleton-line").count(), 3);
//! let m = ui.skeleton(2).label("Loading orders").heading().render().into_string();
//! assert!(m.contains("Loading orders") && m.contains("lui-skeleton-heading"));
//! // The same in `lui!`:
//! let same = lui! { Skeleton(2) label="Loading orders" heading; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::i18n::Text;
use crate::props::{Prop, PropKind};

/// Placeholder bars, made by [`Ui::skeleton`]; the last one is shorter, like the end of a
/// paragraph.
///
/// **Setters.** Values and items: `.label(..)`; switches: `.heading()`.
#[derive(Clone, Debug)]
pub struct Skeleton<'a> {
    lines: usize,
    label: &'a str,
    heading: bool,
}

impl Skeleton<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("label", PropKind::Value, "label: &'a str")
            .default("Loading")
            .doc("What screen readers hear instead of \"Loading\"."),
        Prop::new("heading", PropKind::Switch, "")
            .doc("Start with a wider, taller bar standing in for a heading."),
    ];
}

impl Ui {
    /// `lines` bars, announced as "Loading".
    pub fn skeleton<'a>(&self, lines: usize) -> Skeleton<'a> {
        Skeleton {
            lines,
            label: self.text(Text::Loading),
            heading: false,
        }
    }
}

impl<'a> Skeleton<'a> {
    /// What screen readers hear instead of "Loading".
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    /// Start with a wider, taller bar standing in for a heading.
    pub fn heading(mut self) -> Self {
        self.heading = true;
        self
    }
}

impl Render for Skeleton<'_> {
    fn render(&self) -> Markup {
        let lines = self.lines;
        html! {
            div class="lui-skeleton" role="status" aria-busy="true" {
                span class="lui-sr" { (self.label) }
                @if self.heading { span class="lui-skeleton-heading" aria-hidden="true" {} }
                @for i in 0..lines {
                    span class={ "lui-skeleton-line" @if i + 1 == lines && lines > 1 { " lui-skeleton-last" } } aria-hidden="true" {}
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
/* shadcn Skeleton: accent-coloured blocks, rounded-md, a slow pulse. */
.lui-skeleton { display: grid; gap: calc(var(--lui-space) * 1.25); padding-block: var(--lui-space); }
.lui-skeleton-line, .lui-skeleton-heading {
  display: block; height: 1rem; border-radius: var(--lui-radius-sm); background: var(--lui-accent);
  animation: lui-skeleton-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
.lui-skeleton-heading { height: 1.5rem; width: 45%; margin-bottom: calc(var(--lui-space) * 0.5); }
.lui-skeleton-last { width: 60%; }
@keyframes lui-skeleton-pulse { 50% { opacity: 0.5; } }
@media (prefers-reduced-motion: reduce) { .lui-skeleton-line, .lui-skeleton-heading { animation: none; } }
"#;
