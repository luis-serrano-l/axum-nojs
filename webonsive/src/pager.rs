//! # Pager
//!
//! A long list delivered a page at a time with a "Load more" link, no script.
//!
//! **Platform features:**
//! - Ordinary links to `?page=n`.
//! - `view-transition-name` on the list, together with the layout's
//!   `@view-transition { navigation: auto }` (Chrome 126+, Safari 18.2+) or the enhancement
//!   script's `startViewTransition`, so the new rows fade in under the old ones. The button
//!   has no name on purpose: a named button morphs from its old spot to its new one, which
//!   reads as a green block sliding down over the fresh rows.
//! - `scroll-margin` + fragment `#more` keeps the viewport on the new rows after navigation.
//!
//! **Fallback:** without `Caps::ViewTransitions` the transition names are omitted and the page
//! navigates normally; the `#more` fragment still scrolls to the new rows.
//!
//! **Finding:** true infinite scroll (loading on scroll) is impossible without script.
//! Cumulative pages (`?page=3` shows rows 1..3×N) give the same *feel* with one click per page.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, pager, pager::PagerOptions};
//! let rows = vec![html!{ li{"a"} }, html!{ li{"b"} }];
//! let m = pager(&Caps::all(), "/list", &rows, 2, Default::default());
//! let m = pager(&Caps::all(), "/list", &rows, 30, PagerOptions::default().page(2).per_page(1));
//! assert!(m.into_string().contains("?page=3#more"));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps, enhance};

/// Options for [`pager`]; `Default::default()` is page 1 of 10 rows per page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagerOptions {
    /// The page being shown, 1-based; `items` holds the rows of pages 1..=page.
    pub page: usize,
    /// Rows per page.
    pub per_page: usize,
}

impl Default for PagerOptions {
    fn default() -> Self {
        PagerOptions { page: 1, per_page: 10 }
    }
}

impl PagerOptions {
    /// The page being shown, 1-based.
    pub fn page(mut self, page: usize) -> Self {
        self.page = page.max(1);
        self
    }

    /// Rows per page.
    pub fn per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page.max(1);
        self
    }
}

/// `items` are the rows for pages 1..=page. `total` is the full row count.
pub fn pager(caps: &Caps, href: &str, items: &[Markup], total: usize, options: PagerOptions) -> Markup {
    let PagerOptions { page, per_page } = options;
    let vt = caps.has(Cap::ViewTransitions);
    let shown = items.len();
    let has_more = page * per_page < total;
    html! {
        div id=(enhance::swap_id("wo-pager", href)) data-wo="swap" class="wo-pager" {
            ol class="wo-pager-list" style=[vt.then_some("view-transition-name: wo-pager-list")] {
                @for (i, item) in items.iter().enumerate() {
                    @if i + 1 == (page - 1) * per_page + 1 && page > 1 {
                        li id="more" class="wo-pager-anchor" { (item) }
                    } @else {
                        li { (item) }
                    }
                }
            }
            p class="wo-note" { "Showing " (shown) " of " (total) }
            @if has_more {
                a class="wo-pager-more" href={ (href) "?page=" (page + 1) "#more" } { "Load more" }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-pager-list { margin: 0; padding-left: 1.5rem; }
.wo-pager-list li { padding: 0.4rem 0; border-bottom: 1px solid var(--wo-line); }
.wo-pager-anchor { scroll-margin-top: 4rem; }
.wo-pager-more { display: inline-block; margin-top: 1rem; padding: 0.5rem 1rem;
  background: var(--wo-accent); color: var(--wo-on-accent); border-radius: var(--wo-radius); text-decoration: none; }
"#;
