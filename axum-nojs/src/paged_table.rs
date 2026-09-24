//! # Paged table
//!
//! A [`crate::Ui::table`] with `.paged(total)`, for data too long for one page: first, previous, numbered, next and last
//! links with an ellipsis over long ranges, a jump-to-page form, a page-size `<select>`, and a
//! line saying which rows of how many are shown (`1–10 of 1,234`). Everything is a URL, so a
//! page can be bookmarked, and the sort, filter and columns survive paging.
//!
//! **Platform features:**
//! - Ordinary links to `?page=n` that keep `sort`, `dir`, `q`, `cols` and the page size; the
//!   current page is `aria-current="page"` and the previous/next links are `rel="prev"` /
//!   `rel="next"`. Past seven pages only the first, the last and the current one's neighbours
//!   are numbered; the gaps are an `aria-hidden` ellipsis.
//! - `<form method="get">` with `<input type="number">` (`min`, `max`) to jump to a page, and another
//!   with a `<select>` for rows per page. Their buttons submit them, so both work with no script
//!   and by keyboard.
//! - The whole block (table and pager) is one swap root, so with the enhancement script a
//!   sort, filter, page or size change replaces both and the page links never go stale.
//! - `<output>` for the range, so assistive tech announces it as a result. Counts use a comma
//!   every three digits.
//!
//! **What it does not do without script:** load the next page on scroll; paging, sorting and
//! filtering are each a navigation.
//!
//! **Fallback:** none needed. Every control is a link or a form.
//!
//! **Server state:** the page size is `per.<id>`, a state key, so the size a visitor picked
//! is remembered in the `nojs-ui` cookie and read back through `state.per_page(id)`. Every
//! page link still names it, so a shared URL shows the same rows for everyone.
//!
//! ```rust
//! use axum_nojs::{prelude::*, table::Row};
//! // The URL's sort, filter and page, and the page size the visitor picked before.
//! let ui = Ui::from_request("/table", "sort=name&dir=desc&q=a&page=20", "nojs-ui=per.files=25");
//! let files = ui.table("files", "/table").column("name", "Name").sortable().column("note", "Note");
//! assert_eq!((files.page(), files.per_page()), (20, 25), "what a database query needs");
//! // Only this page's rows, and the total after filtering.
//! let html = files.rows([Row::new([html! { "a" }, html! { "b" }])]).paged(1234).render().into_string();
//! assert!(html.contains("476–500 of 1,234"));
//! assert!(html.contains("per.files=25&amp;page=50\">Last"));
//! assert!(html.contains("<select name=\"per.files\""));
//! ```

use maud::{Markup, html};

use std::fmt::{self, Write};

use crate::table::{Column, Encoded, Row, TableOptions, TableQuery, table_in};
use crate::enhance;
use crate::{Caps, UiState};

/// Page sizes offered in the select.
pub const PAGE_SIZES: [usize; 4] = [5, 10, 25, 50];

/// How the paged table renders; `Default::default()` is page 1 of 10, unsorted, unfiltered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct PagedTableOptions<'a> {
    /// The current sort as `(key, descending)`.
    pub sort: Option<(&'a str, bool)>,
    /// The current search text.
    pub filter: &'a str,
    /// The page being shown, 1-based.
    pub page: usize,
    /// Rows per page; one of [`PAGE_SIZES`] is selected in the size control. With a `state`
    /// the size the visitor picked (`per.<id>`) wins, capped at the largest of [`PAGE_SIZES`].
    pub per_page: usize,
    /// Everything else the inner table takes (columns, bulk form, CSV link, empty and
    /// loading states); its `sort`, `filter` and `keep` are overwritten by the pager's.
    pub table: TableOptions<'a>,
    /// Remember the page size per table as the state key `per.<id>`.
    pub state: Option<&'a UiState>,
    /// The URL's sort, filter, page and columns, used wherever the setters above left the
    /// default.
    pub query: Option<&'a TableQuery>,
}

impl Default for PagedTableOptions<'_> {
    fn default() -> Self {
        PagedTableOptions { sort: None, filter: "", page: 1, per_page: PAGE_SIZES[1], table: TableOptions::default(), state: None, query: None }
    }
}

// Setters for the tests; the builder fills the struct directly.
#[cfg(test)]
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


    /// Name the page-size parameter `per.<id>` so the `nojs-ui` cookie remembers it, and read
    /// the remembered size back.
    pub fn state(mut self, state: &'a UiState) -> Self {
        self.state = Some(state);
        self
    }

    /// Take the sort, filter, page and columns from the parsed URL.
    pub fn query(mut self, query: &'a TableQuery) -> Self {
        self.query = Some(query);
        self
    }
}

/// `rows` are either every row (`rows.len() == total`), and the component shows the current
/// page of them, or the rows of the current page only, for data too large to build in full;
/// `total` is the full row count after filtering, which sizes the page links.
pub(crate) fn paged_table_with(caps: &Caps, id: &str, href: &str, columns: &[Column], rows: &[Row], total: usize, options: PagedTableOptions) -> Markup {
    let PagedTableOptions { sort, filter, page, per_page, table: inner, state, query } = options;
    let sort = sort.or_else(|| query.and_then(|q| q.sort(columns)));
    let filter = if filter.is_empty() { query.map_or("", |q| q.filter.as_str()) } else { filter };
    let page = query.and_then(|q| q.page).filter(|_| page == 1).unwrap_or(page);
    let cols = inner.cols.is_none().then(|| query.and_then(|q| q.cols(columns))).flatten();
    let inner = TableOptions { cols: inner.cols.or(cols.as_deref()), ..inner };
    let per_key = if state.is_some() { format!("per.{id}") } else { "per".to_string() };
    let remembered = state.and_then(|s| s.per_page(id)).map(|n| n.min(PAGE_SIZES[PAGE_SIZES.len() - 1]));
    let per_page = remembered.unwrap_or(per_page).max(1);
    let pages = total.div_ceil(per_page).max(1);
    let page = page.clamp(1, pages);
    let rows = if rows.len() == total && total > per_page {
        &rows[(page - 1) * per_page..(page * per_page).min(total)]
    } else {
        rows
    };
    let per = per_page.to_string();
    let first = if total == 0 { 0 } else { (page - 1) * per_page + 1 };
    let last = (page * per_page).min(total);
    let dir = |d: bool| if d { "desc" } else { "asc" };
    let cols_value = inner.cols.map(|c| c.join(","));
    // Hidden fields shared by both forms: everything in the URL except what the form sets.
    let carried = |skip: &str| {
        let mut pairs: Vec<(&str, &str)> = Vec::with_capacity(5);
        if let Some((k, d)) = sort {
            pairs.extend([("sort", k), ("dir", dir(d))]);
        }
        if !filter.is_empty() { pairs.push(("q", filter)); }
        if let Some(c) = &cols_value { pairs.push(("cols", c)); }
        if skip != per_key { pairs.push((per_key.as_str(), &per)); }
        pairs
    };
    let mut base = String::new();
    for (k, v) in carried("") {
        let _ = write!(base, "{}={}&", Encoded(k), Encoded(v));
    }
    let link = |n: usize| PageLink { href, base: &base, n };
    let keep = [(per_key.as_str(), per.as_str())];
    html! {
        div id=(enhance::swap_id("nojs-paged-table", id)) data-nojs="swap" class="nojs-paged-table" {
            (table_in(caps, id, href, columns, rows, TableOptions { sort, filter, keep: &keep, ..inner }, false))
            nav class="nojs-paged-table-nav" aria-label="Pages" {
                output class="nojs-paged-table-range" { (Thousands(first)) "–" (Thousands(last)) " of " (Thousands(total)) }
                ul class="nojs-paged-table-pages" {
                    @if page > 1 {
                        li { a class="nojs-paged-table-end" href=(link(1)) { "First" } }
                        li { a rel="prev" href=(link(page - 1)) { "Previous" } }
                    }
                    @for slot in window(page, pages) {
                        @match slot {
                            Some(n) if n == page => li { a aria-current="page" href=(link(n)) { (Thousands(n)) } },
                            Some(n) => li { a href=(link(n)) { (Thousands(n)) } },
                            None => li class="nojs-paged-table-gap" aria-hidden="true" { "…" },
                        }
                    }
                    @if page < pages {
                        li { a rel="next" href=(link(page + 1)) { "Next" } }
                        li { a class="nojs-paged-table-end" href=(link(pages)) { "Last" } }
                    }
                }
                @if pages > 1 {
                    form method="get" action=(href) class="nojs-paged-table-jump" {
                        @for (k, v) in carried("") { input type="hidden" name=(k) value=(v); }
                        label { "Page "
                            input type="number" name="page" min="1" max=(pages) value=(page) inputmode="numeric";
                            " of " (Thousands(pages))
                        }
                        button type="submit" { "Go" }
                    }
                }
                form method="get" action=(href) class="nojs-paged-table-per" {
                    @for (k, v) in carried(&per_key) { input type="hidden" name=(k) value=(v); }
                    label { "Rows per page "
                        select name=(per_key) {
                            @for size in PAGE_SIZES { option value=(size) selected[size == per_page] { (size) } }
                        }
                    }
                    button type="submit" { "Show" }
                }
            }
        }
    }
}

/// The numbered slots: every page up to seven, else the first, the last and the current
/// page's neighbours, with `None` for each gap.
fn window(page: usize, pages: usize) -> Vec<Option<usize>> {
    if pages <= 7 {
        return (1..=pages).map(Some).collect();
    }
    // Near an end, show five in a row so the list keeps its length.
    let (lo, hi) = match page {
        p if p <= 4 => (2, 5),
        p if p + 3 >= pages => (pages - 4, pages - 1),
        p => (p - 1, p + 1),
    };
    let mut out = vec![Some(1)];
    if lo > 2 { out.push(None); }
    out.extend((lo..=hi).map(Some));
    if hi < pages - 1 { out.push(None); }
    out.push(Some(pages));
    out
}

/// `1234567` as `1,234,567`.
pub fn thousands(n: usize) -> String {
    Thousands(n).to_string()
}

/// [`thousands`] written straight into the page: no `String` per number.
struct Thousands(usize);

impl fmt::Display for Thousands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (mut digits, mut len, mut n) = ([0u8; 20], 0, self.0);
        loop {
            digits[len] = b'0' + (n % 10) as u8;
            len += 1;
            n /= 10;
            if n == 0 { break; }
        }
        for i in (0..len).rev() {
            f.write_char(digits[i] as char)?;
            if i > 0 && i % 3 == 0 { f.write_char(',')?; }
        }
        Ok(())
    }
}

/// `href?<carried pairs>page=n`, written into the attribute as it renders.
struct PageLink<'a> {
    href: &'a str,
    base: &'a str,
    n: usize,
}

impl fmt::Display for PageLink<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}?{}page={}", self.href, self.base, self.n)
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-paged-table-nav { display: flex; flex-wrap: wrap; align-items: center; gap: var(--nojs-space) calc(var(--nojs-space) * 2); margin-top: var(--nojs-space); color: var(--nojs-muted); }
.nojs-paged-table-pages { display: flex; flex-wrap: wrap; gap: 0.25rem; list-style: none; margin: 0; padding: 0; }
.nojs-paged-table-pages a { display: inline-block; min-width: 2rem; padding: 0.25rem 0.5rem; text-align: center; text-decoration: none; border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); color: var(--nojs-fg); }
.nojs-paged-table-pages a:hover { border-color: var(--nojs-accent); }
.nojs-paged-table-pages a[aria-current="page"] { background: var(--nojs-accent); color: var(--nojs-on-accent); border-color: transparent; }
.nojs-paged-table-gap { align-self: center; padding: 0 0.25rem; }
.nojs-paged-table-jump, .nojs-paged-table-per { display: flex; align-items: center; gap: var(--nojs-space); }
.nojs-paged-table-jump { margin-left: auto; }
.nojs-paged-table-jump input { width: 5em; }
@media (max-width: 40rem) { .nojs-paged-table-jump { margin-left: 0; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_keep_sort_filter_and_size() {
        let cols = [Column::sortable("n", "N")];
        let opts = PagedTableOptions::default().sort(Some(("n", true))).filter("x").page(2).per_page(5);
        let m = paged_table_with(&Caps::NONE, "t", "/t", &cols, &[], 12, opts).into_string();
        assert!(m.contains("href=\"/t?sort=n&amp;dir=desc&amp;q=x&amp;per=5&amp;page=3\""), "{m}");
        assert!(m.contains("rel=\"prev\"") && m.contains("rel=\"next\""));
        assert!(m.contains("6–10 of 12"));
        assert!(m.contains("value=\"5\" selected"));
        let empty = paged_table_with(&Caps::NONE, "t", "/t", &cols, &[], 0, PagedTableOptions::default().page(9)).into_string();
        assert!(empty.contains("0–0 of 0") && !empty.contains("rel="));
        assert!(!empty.contains("nojs-paged-table-jump"), "no jump form for a single page");
    }

    #[test]
    fn window_and_separators() {
        let w = |p, n| window(p, n).iter().map(|s| s.map_or("…".into(), |n| n.to_string())).collect::<Vec<_>>().join(" ");
        assert_eq!(w(3, 7), "1 2 3 4 5 6 7");
        assert_eq!(w(1, 20), "1 2 3 4 5 … 20");
        assert_eq!(w(10, 20), "1 … 9 10 11 … 20");
        assert_eq!(w(19, 20), "1 … 16 17 18 19 20");
        assert_eq!(w(5, 8), "1 … 4 5 6 7 8");
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1000), "1,000");
        assert_eq!(thousands(1234567), "1,234,567");
    }

    #[test]
    fn state_names_the_size_per_table() {
        let state = UiState::parse("/t", "per.t=5", "");
        let opts = PagedTableOptions::default().per_page(state.per_page("t").unwrap()).page(2).state(&state);
        let m = paged_table_with(&Caps::NONE, "t", "/t", &[Column::plain("n", "N")], &[], 40, opts).into_string();
        assert!(m.contains("href=\"/t?per.t=5&amp;page=8\">Last"), "{m}");
        assert!(m.contains("href=\"/t?per.t=5&amp;page=1\">First"));
        assert!(m.contains("<input type=\"number\" name=\"page\" min=\"1\" max=\"8\" value=\"2\""));
        assert!(m.contains("<select name=\"per.t\">"));
    }

    #[test]
    fn the_query_state_and_all_rows_are_enough() {
        let cols = [Column::sortable("n", "N"), Column::plain("x", "X")];
        let rows: Vec<Row> = (1..=12).map(|n| Row::new(vec![html! { "row " (n) }, html! {}])).collect();
        let query = TableQuery::from_ui(&crate::Ui::from_request("/t", "sort=n&dir=desc&q=r%C3%A9&page=3&cols=n&other=1", ""));
        assert_eq!(query.filter, "r\u{e9}");
        let state = UiState::parse("/t", "per.t=5", "");
        let m = paged_table_with(&Caps::NONE, "t", "/t", &cols, &rows, 12, PagedTableOptions::default().query(&query).state(&state)).into_string();
        assert!(m.contains("11–12 of 12"), "{m}");
        assert!(m.contains("row 11") && m.contains("row 12") && !m.contains("row 10<"), "sliced to page 3");
        assert!(m.contains("sort=n&amp;dir=desc&amp;q=r%C3%A9&amp;cols=n&amp;per.t=5&amp;page=2"), "{m}");
        let huge = UiState::parse("/t", "per.t=100000", "");
        let m = paged_table_with(&Caps::NONE, "t", "/t", &cols, &rows, 12, PagedTableOptions::default().state(&huge)).into_string();
        assert!(m.contains("<option value=\"50\" selected"), "a remembered size is capped");
    }
}
