//! # Skeleton
//!
//! Grey bars in the shape of content that is still coming: the placeholder for a streamed
//! [`crate::slot`], a lazy panel, anything the server fills in later.
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
//! use axum_nojs::{Caps, skeleton, skeleton_with, skeleton::SkeletonOptions};
//! let m = skeleton(&Caps::all(), 3).into_string();
//! assert_eq!(m.matches("nojs-skeleton-line").count(), 3);
//! let m = skeleton_with(&Caps::all(), 2, SkeletonOptions::default().label("Loading orders").heading()).into_string();
//! assert!(m.contains("Loading orders") && m.contains("nojs-skeleton-heading"));
//! ```

use maud::{Markup, html};

use crate::Caps;

/// Options for [`skeleton`].
#[derive(Clone, Debug)]
pub struct SkeletonOptions<'a> {
    /// What screen readers hear (default "Loading").
    pub label: &'a str,
    /// A wider, taller first bar standing in for a heading.
    pub heading: bool,
}

impl Default for SkeletonOptions<'_> {
    fn default() -> Self {
        SkeletonOptions { label: "Loading", heading: false }
    }
}

impl<'a> SkeletonOptions<'a> {
    /// What screen readers hear.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }
    /// Start with a heading bar.
    pub fn heading(mut self) -> Self {
        self.heading = true;
        self
    }
}

/// A skeleton of `lines` lines.
/// [`skeleton_with`] takes the options.
pub fn skeleton(caps: &Caps, lines: usize) -> Markup {
    skeleton_with(caps, lines, Default::default())
}

/// `lines` placeholder bars; the last one is shorter, like the end of a paragraph.
pub fn skeleton_with(_caps: &Caps, lines: usize, options: SkeletonOptions) -> Markup {
    html! {
        div class="nojs-skeleton" role="status" aria-busy="true" {
            span class="nojs-sr" { (options.label) }
            @if options.heading { span class="nojs-skeleton-heading" aria-hidden="true" {} }
            @for i in 0..lines {
                span class={ "nojs-skeleton-line" @if i + 1 == lines && lines > 1 { " nojs-skeleton-last" } } aria-hidden="true" {}
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
