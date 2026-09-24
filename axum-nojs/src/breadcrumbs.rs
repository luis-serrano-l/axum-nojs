//! # Breadcrumbs
//!
//! Where this page sits: a trail of links ending in the current page. A long trail folds its
//! middle into a disclosure so the ends stay readable on a phone.
//!
//! **Platform features:** `<nav aria-label="Breadcrumb">` around an ordered list, the last
//! item marked `aria-current="page"`; separators drawn by CSS `::before` so screen readers do
//! not read them; the folded middle is a `<details>` element.
//!
//! **What it does not do without script:** collapse to fit the width as it changes; the fold is
//! decided by item count on the server.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::{Caps, breadcrumbs};
//! let m = breadcrumbs(&Caps::all(), &[("Home", "/"), ("Projects", "/projects"), ("axum-nojs", "")]).into_string();
//! assert!(m.contains(r#"<a href="/projects">Projects</a>"#));
//! assert!(m.contains(r#"aria-current="page">axum-nojs"#));
//! // More than four: the middle folds into <details>.
//! let long = [("Home", "/"), ("A", "/a"), ("B", "/a/b"), ("C", "/a/b/c"), ("Here", "")];
//! assert!(breadcrumbs(&Caps::all(), &long).into_string().contains("<details"));
//! ```

use maud::{Markup, html};

use crate::Caps;

/// `trail` is `(label, href)` from the root to the current page; the last `href` is ignored.
pub fn breadcrumbs(_caps: &Caps, trail: &[(&str, &str)]) -> Markup {
    let Some(((current, _), before)) = trail.split_last() else { return html! {} };
    let fold = before.len() > 3;
    let (head, middle, tail) = if fold { (&before[..1], &before[1..before.len() - 1], &before[before.len() - 1..]) } else { (before, &before[..0], &before[..0]) };
    html! {
        nav class="nojs-breadcrumbs" aria-label="Breadcrumb" {
            ol {
                @for (label, href) in head { li { a href=(href) { (label) } } }
                @if fold {
                    li class="nojs-breadcrumbs-fold" {
                        details {
                            summary aria-label={ "Show " (middle.len()) " more" } { "\u{2026}" }
                            ol { @for (label, href) in middle { li { a href=(href) { (label) } } } }
                        }
                    }
                }
                @for (label, href) in tail { li { a href=(href) { (label) } } }
                li { span aria-current="page" { (current) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-breadcrumbs { font-size: 0.875rem; color: var(--nojs-muted); margin-bottom: calc(var(--nojs-space) * 2); }
.nojs-breadcrumbs > ol { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 0; list-style: none; margin: 0; padding: 0; }
.nojs-breadcrumbs > ol > li + li::before { content: "/"; margin-inline: 0.5rem; color: var(--nojs-line); }
.nojs-breadcrumbs a { color: var(--nojs-muted); }
.nojs-breadcrumbs a:hover { color: var(--nojs-accent); }
.nojs-breadcrumbs [aria-current] { color: var(--nojs-fg); font-weight: 600; }
.nojs-breadcrumbs-fold { position: relative; }
.nojs-breadcrumbs-fold details { display: inline-block; }
.nojs-breadcrumbs-fold summary { display: inline; list-style: none; cursor: pointer; padding-inline: 0.25rem; border-radius: var(--nojs-radius); }
.nojs-breadcrumbs-fold summary::-webkit-details-marker { display: none; }
.nojs-breadcrumbs-fold summary:hover { background: var(--nojs-surface); }
.nojs-breadcrumbs-fold ol {
  position: absolute; z-index: 5; top: 100%; left: 0; margin: 0.25rem 0 0; padding: var(--nojs-space) 0;
  list-style: none; min-width: 10rem; background: var(--nojs-surface);
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
}
.nojs-breadcrumbs-fold ol a { display: block; padding: 0.25rem 1rem; }
"#;
