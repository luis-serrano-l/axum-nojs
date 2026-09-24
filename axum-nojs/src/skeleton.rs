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
//! ```

use maud::{Markup, Render, html};

use crate::Ui;

/// Placeholder bars, made by [`Ui::skeleton`]; the last one is shorter, like the end of a
/// paragraph.
#[derive(Clone, Debug)]
pub struct Skeleton<'a> {
    lines: usize,
    label: &'a str,
    heading: bool,
}

impl Ui {
    /// `lines` bars, announced as "Loading".
    pub fn skeleton<'a>(&self, lines: usize) -> Skeleton<'a> {
        Skeleton { lines, label: "Loading", heading: false }
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
.nojs-skeleton { display: grid; gap: calc(var(--nojs-space) * 1.25); padding-block: var(--nojs-space); }
.nojs-skeleton-line, .nojs-skeleton-heading {
  display: block; height: 0.8rem; border-radius: var(--nojs-radius);
  background: linear-gradient(90deg, var(--nojs-line) 0%, color-mix(in srgb, var(--nojs-line) 40%, var(--nojs-surface)) 50%, var(--nojs-line) 100%);
  background-size: 200% 100%; animation: nojs-skeleton-shimmer 1.4s linear infinite;
}
.nojs-skeleton-heading { height: 1.3rem; width: 45%; margin-bottom: calc(var(--nojs-space) * 0.5); }
.nojs-skeleton-last { width: 60%; }
@keyframes nojs-skeleton-shimmer { from { background-position: 100% 0; } to { background-position: -100% 0; } }
@media (prefers-reduced-motion: reduce) { .nojs-skeleton-line, .nojs-skeleton-heading { animation: none; } }
"#;
