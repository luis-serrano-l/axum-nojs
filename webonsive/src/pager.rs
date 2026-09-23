//! # Pager
//!
//! A long list delivered a page at a time with a "Load more" link, no script.
//!
//! **Platform features:**
//! - Ordinary links to `?page=n`.
//! - `view-transition-name` on the list and on the button, together with the layout's
//!   `@view-transition { navigation: auto }`, so the new page cross-fades in place
//!   (Chrome 126+, Safari 18.2+).
//! - `scroll-margin` + fragment `#more` keeps the viewport on the new rows after navigation.
//!
//! **Finding:** true infinite scroll (loading on scroll) is impossible without script.
//! Cumulative pages (`?page=3` shows rows 1..3×N) give the same *feel* with one click per page.
//!
//! ```rust
//! use maud::html;
//! use webonsive::pager;
//! let rows = vec![html!{ li{"a"} }, html!{ li{"b"} }];
//! let m = pager(&rows, 1, 10, 2, "/list");
//! ```

use maud::{Markup, html};

/// `items` are the rows for pages 1..=page. `total` is the full row count.
pub fn pager(items: &[Markup], page: usize, per_page: usize, total: usize, href: &str) -> Markup {
    let shown = items.len();
    let has_more = page * per_page < total;
    html! {
        div class="wo-pager" {
            ol class="wo-pager-list" style="view-transition-name: wo-pager-list" {
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
                a class="wo-pager-more" style="view-transition-name: wo-pager-more"
                    href={ (href) "?page=" (page + 1) "#more" } { "Load more" }
            }
        }
    }
}

pub const CSS: &str = r#"
.wo-pager-list { margin: 0; padding-left: 1.5rem; }
.wo-pager-list li { padding: 0.4rem 0; border-bottom: 1px solid var(--wo-line); }
.wo-pager-anchor { scroll-margin-top: 4rem; }
.wo-pager-more { display: inline-block; margin-top: 1rem; padding: 0.5rem 1rem;
  background: var(--wo-accent); color: var(--wo-on-accent); border-radius: var(--wo-radius); text-decoration: none; }
"#;
