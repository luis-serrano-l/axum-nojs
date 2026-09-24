//! # Paged table
//!
//! [`crate::table()`] for data too long for one page: page links, a page-size `<select>`, and a
//! line saying which rows of how many are shown. Everything is a URL, so a page can be
//! bookmarked, and the sort and filter survive paging.
//!
//! **Platform features:**
//! - Ordinary links to `?page=n` that keep `sort`, `dir`, `q` and `per`; the current page is
//!   `aria-current="page"` and the previous/next links are `rel="prev"` / `rel="next"`.
//! - `<form method="get">` with a `<select name="per">` for rows per page; a "Show" button
//!   submits it, so it works with no script and reaches by keyboard (Tab, Space, Enter).
//! - `<output>` for the "1–10 of 36" range, so assistive tech announces it as a result.
//!
//! **Fallback:** none needed. Every control is a link or a form.
//!
//! **Finding:** page links are the whole state; there is nothing to persist. A page size
//! stored in a cookie would make the same URL show different rows for different people.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, paged_table, paged_table::PagedTableOptions, table::{Column, Row}};
//! let cols = [Column::sortable("name", "Name"), Column::plain("note", "Note")];
//! let rows = vec![Row::new(vec![html!{"a"}, html!{"b"}])];
//! let m = paged_table(&Caps::all(), "files", "/table", &cols, &rows, 36, Default::default());
//! let m = paged_table(&Caps::all(), "files", "/table", &cols, &rows, 36,
//!                     PagedTableOptions::default().sort(Some(("name", true))).filter("a").page(2).per_page(10));
//! assert!(m.into_string().contains("11–20 of 36"));
//! ```

use maud::{Markup, html};

use crate::table::{Column, Row, TableOptions, encode, table};
use crate::Caps;

/// Page sizes offered in the select.
pub const PAGE_SIZES: [usize; 4] = [5, 10, 25, 50];

/// Options for [`paged_table`]; `Default::default()` is page 1 of 10, unsorted, unfiltered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PagedTableOptions<'a> {
    /// The current sort as `(key, descending)`, usually from [`crate::table::sort_from_query`].
    pub sort: Option<(&'a str, bool)>,
    /// The current search text.
    pub filter: &'a str,
    /// The page being shown, 1-based; `rows` holds that page only.
    pub page: usize,
    /// Rows per page; one of [`PAGE_SIZES`] is selected in the size control.
    pub per_page: usize,
    /// Everything else the inner [`table`] takes (columns, bulk form, CSV link, empty and
    /// loading states); its `sort`, `filter` and `keep` are overwritten by the pager's.
    pub table: TableOptions<'a>,
}

impl Default for PagedTableOptions<'_> {
    fn default() -> Self {
        PagedTableOptions { sort: None, filter: "", page: 1, per_page: PAGE_SIZES[1], table: TableOptions::default() }
    }
}

impl<'a> PagedTableOptions<'a> {
    /// The current sort as `(key, descending)`.
    pub fn sort(mut self, sort: Option<(&'a str, bool)>) -> Self {
        self.sort = sort;
        self
    }

    /// The current search text.
    pub fn filter(mut self, filter: &'a str) -> Self {
        self.filter = filter;
        self
    }

    /// The page being shown, 1-based.
    pub fn page(mut self, page: usize) -> Self {
        self.page = page;
        self
    }

    /// Rows per page.
    pub fn per_page(mut self, per_page: usize) -> Self {
        self.per_page = per_page;
        self
    }

    /// Options for the inner table (columns, bulk form, CSV, empty and loading states).
    pub fn table(mut self, table: TableOptions<'a>) -> Self {
        self.table = table;
        self
    }
}

/// `rows` are the rows of the current page only; `total` is the full row count after
/// filtering, which sizes the page links.
pub fn paged_table(caps: &Caps, id: &str, href: &str, columns: &[Column], rows: &[Row], total: usize, options: PagedTableOptions) -> Markup {
    let PagedTableOptions { sort, filter, page, per_page, table: inner } = options;
    let per_page = per_page.max(1);
    let pages = total.div_ceil(per_page).max(1);
    let page = page.clamp(1, pages);
    let per = per_page.to_string();
    let first = if total == 0 { 0 } else { (page - 1) * per_page + 1 };
    let last = (page * per_page).min(total);
    let mut base = String::new();
    if let Some((k, d)) = sort {
        base.push_str(&format!("sort={k}&dir={}&", if d { "desc" } else { "asc" }));
    }
    if !filter.is_empty() {
        base.push_str(&format!("q={}&", encode(filter)));
    }
    let cols_value = inner.cols.map(|c| c.join(","));
    if let Some(c) = &cols_value {
        base.push_str(&format!("cols={}&", encode(c)));
    }
    let link = |n: usize| format!("{href}?{base}per={per}&page={n}");
    html! {
        div class="wo-paged-table" {
            (table(caps, id, href, columns, rows, TableOptions { sort, filter, keep: &[("per", &per)], ..inner }))
            nav class="wo-paged-table-nav" aria-label="Pages" {
                output class="wo-paged-table-range" { (first) "–" (last) " of " (total) }
                ul class="wo-paged-table-pages" {
                    @if page > 1 { li { a rel="prev" href=(link(page - 1)) { "Previous" } } }
                    @for n in 1..=pages {
                        li { @if n == page { a aria-current="page" href=(link(n)) { (n) } } @else { a href=(link(n)) { (n) } } }
                    }
                    @if page < pages { li { a rel="next" href=(link(page + 1)) { "Next" } } }
                }
                form method="get" action=(href) class="wo-paged-table-per" {
                    @if let Some((k, d)) = sort {
                        input type="hidden" name="sort" value=(k);
                        input type="hidden" name="dir" value=(if d { "desc" } else { "asc" });
                    }
                    @if !filter.is_empty() { input type="hidden" name="q" value=(filter); }
                    @if let Some(c) = &cols_value { input type="hidden" name="cols" value=(c); }
                    label { "Rows per page "
                        select name="per" {
                            @for size in PAGE_SIZES { option value=(size) selected[size == per_page] { (size) } }
                        }
                    }
                    button type="submit" { "Show" }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-paged-table-nav { display: flex; flex-wrap: wrap; align-items: center; gap: var(--wo-space) calc(var(--wo-space) * 2); margin-top: var(--wo-space); color: var(--wo-muted); }
.wo-paged-table-pages { display: flex; flex-wrap: wrap; gap: 0.25rem; list-style: none; margin: 0; padding: 0; }
.wo-paged-table-pages a { display: inline-block; min-width: 2rem; padding: 0.25rem 0.5rem; text-align: center; text-decoration: none; border: 1px solid var(--wo-line); border-radius: var(--wo-radius); color: var(--wo-fg); }
.wo-paged-table-pages a:hover { border-color: var(--wo-accent); }
.wo-paged-table-pages a[aria-current="page"] { background: var(--wo-accent); color: var(--wo-on-accent); border-color: transparent; }
.wo-paged-table-per { display: flex; align-items: center; gap: var(--wo-space); margin-left: auto; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_keep_sort_filter_and_size() {
        let cols = [Column::sortable("n", "N")];
        let opts = PagedTableOptions::default().sort(Some(("n", true))).filter("x").page(2).per_page(5);
        let m = paged_table(&Caps::NONE, "t", "/t", &cols, &[], 12, opts).into_string();
        assert!(m.contains("href=\"/t?sort=n&amp;dir=desc&amp;q=x&amp;per=5&amp;page=3\""), "{m}");
        assert!(m.contains("rel=\"prev\"") && m.contains("rel=\"next\""));
        assert!(m.contains("6–10 of 12"));
        assert!(m.contains("value=\"5\" selected"));
        let empty = paged_table(&Caps::NONE, "t", "/t", &cols, &[], 0, PagedTableOptions::default().page(9)).into_string();
        assert!(empty.contains("0–0 of 0") && !empty.contains("rel="));
    }
}
