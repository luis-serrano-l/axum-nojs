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
//! **What it does not do without script:** it cannot be removed by the client; it is replaced
//! when the real content arrives (a streamed slot, a swap, the next page).
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! assert_eq!(ui.skeleton(3).render().into_string().matches("nojs-skeleton-line").count(), 3);
//! let m = ui.skeleton(2).label("Loading orders").heading().render().into_string();
//! assert!(m.contains("Loading orders") && m.contains("nojs-skeleton-heading"));
//! // The same in `nojs!`:
//! let same = nojs! { Skeleton(2) label="Loading orders" heading; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
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
            label: "Loading",
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
            div class="nojs-skeleton" role="status" aria-busy="true" {
                span class="nojs-sr" { (self.label) }
                @if self.heading { span class="nojs-skeleton-heading" aria-hidden="true" {} }
                @for i in 0..lines {
                    span class={ "nojs-skeleton-line" @if i + 1 == lines && lines > 1 { " nojs-skeleton-last" } } aria-hidden="true" {}
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
/* shadcn Skeleton: accent-coloured blocks, rounded-md, a slow pulse. */
.nojs-skeleton { display: grid; gap: calc(var(--nojs-space) * 1.25); padding-block: var(--nojs-space); }
.nojs-skeleton-line, .nojs-skeleton-heading {
  display: block; height: 1rem; border-radius: var(--nojs-radius-sm); background: var(--nojs-accent);
  animation: nojs-skeleton-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}
.nojs-skeleton-heading { height: 1.5rem; width: 45%; margin-bottom: calc(var(--nojs-space) * 0.5); }
.nojs-skeleton-last { width: 60%; }
@keyframes nojs-skeleton-pulse { 50% { opacity: 0.5; } }
@media (prefers-reduced-motion: reduce) { .nojs-skeleton-line, .nojs-skeleton-heading { animation: none; } }
"#;
