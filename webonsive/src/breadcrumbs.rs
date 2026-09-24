//! # Breadcrumbs
//!
//! Where this page sits: a trail of links ending in the current page. A long trail folds its
//! middle into a disclosure so the ends stay readable on a phone.
//!
//! **Platform features:** `<nav aria-label="Breadcrumb">` around an ordered list, the last
//! item marked `aria-current="page"`; separators drawn by CSS `::before` so screen readers do
//! not read them; the folded middle is a `<details>` element.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use webonsive::{Caps, breadcrumbs};
//! let m = breadcrumbs(&Caps::all(), &[("Home", "/"), ("Projects", "/projects"), ("Webonsive", "")]).into_string();
//! assert!(m.contains(r#"<a href="/projects">Projects</a>"#));
//! assert!(m.contains(r#"aria-current="page">Webonsive"#));
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
        nav class="wo-breadcrumbs" aria-label="Breadcrumb" {
            ol {
                @for (label, href) in head { li { a href=(href) { (label) } } }
                @if fold {
                    li class="wo-breadcrumbs-fold" {
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
.wo-breadcrumbs { font-size: 0.875rem; color: var(--wo-muted); margin-bottom: calc(var(--wo-space) * 2); }
.wo-breadcrumbs > ol { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 0; list-style: none; margin: 0; padding: 0; }
.wo-breadcrumbs > ol > li + li::before { content: "/"; margin-inline: 0.5rem; color: var(--wo-line); }
.wo-breadcrumbs a { color: var(--wo-muted); }
.wo-breadcrumbs a:hover { color: var(--wo-accent); }
.wo-breadcrumbs [aria-current] { color: var(--wo-fg); font-weight: 600; }
.wo-breadcrumbs-fold { position: relative; }
.wo-breadcrumbs-fold details { display: inline-block; }
.wo-breadcrumbs-fold summary { display: inline; list-style: none; cursor: pointer; padding-inline: 0.25rem; border-radius: var(--wo-radius); }
.wo-breadcrumbs-fold summary::-webkit-details-marker { display: none; }
.wo-breadcrumbs-fold summary:hover { background: var(--wo-surface); }
.wo-breadcrumbs-fold ol {
  position: absolute; z-index: 5; top: 100%; left: 0; margin: 0.25rem 0 0; padding: var(--wo-space) 0;
  list-style: none; min-width: 10rem; background: var(--wo-surface);
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius);
}
.wo-breadcrumbs-fold ol a { display: block; padding: 0.25rem 1rem; }
"#;
