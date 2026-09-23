//! # Table
//!
//! A data table an admin panel can sort and filter, no script: every column header is a link
//! that re-requests the page sorted by that column, and a search box above it filters rows on
//! the server. The header stays put while the rows scroll.
//!
//! **Platform features:**
//! - Ordinary links to `?sort=<col>&dir=asc|desc` in each `<th>`; clicking the sorted column
//!   again flips the direction. The current one carries `aria-sort` for screen readers.
//! - `<form method="get">` inside a `<search>` element (baseline 2023) for the filter; the
//!   sort is kept in hidden inputs so filtering never loses it and the URL stays shareable.
//! - `position: sticky` (Chrome 56, Firefox 32, Safari 13) on the header row.
//! - `view-transition-name` on the body so re-sorted rows fade rather than jump.
//!
//! **Fallback:** none needed. Without `Caps::ViewTransitions` the transition name is
//! omitted; sorting and filtering are plain navigations either way.
//!
//! **Finding:** the server sorts and filters; the component only renders what it is given and
//! the links to ask for something else. That is what keeps it usable with `curl`.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, table, table::Column};
//! let cols = [Column::sortable("name", "Name"), Column::sortable("size", "Size"), Column::plain("note", "Note")];
//! let rows = vec![vec![html!{"a.txt"}, html!{"1 KB"}, html!{"—"}]];
//! let m = table(&Caps::all(), "files", "/table", &cols, &rows, Some(("name", false)), "");
//! assert!(m.into_string().contains("aria-sort=\"ascending\""));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps, enhance};

/// One column: the query key it sorts by, its header text, and whether it can be sorted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Column<'a> {
    /// Value of `?sort=` for this column.
    pub key: &'a str,
    /// Header text.
    pub label: &'a str,
    /// Whether the header is a sort link.
    pub sortable: bool,
}

impl<'a> Column<'a> {
    /// A column whose header sorts the table.
    pub fn sortable(key: &'a str, label: &'a str) -> Column<'a> {
        Column { key, label, sortable: true }
    }

    /// A column with a plain header.
    pub fn plain(key: &'a str, label: &'a str) -> Column<'a> {
        Column { key, label, sortable: false }
    }
}

/// Parse `?sort=<key>&dir=<asc|desc>` into what [`table`] takes: `(key, descending)`.
/// Unknown keys give `None`, so a hand-edited URL cannot ask for a column that is not there.
pub fn sort_from_query<'a>(columns: &[Column<'a>], sort: Option<&str>, dir: Option<&str>) -> Option<(&'a str, bool)> {
    let key = sort?;
    let col = columns.iter().find(|c| c.sortable && c.key == key)?;
    Some((col.key, dir == Some("desc")))
}

/// `rows` are already sorted and filtered by the caller. `sort` is `(key, descending)`;
/// `filter` is the current search text, echoed into the box and kept in the sort links.
pub fn table(
    caps: &Caps,
    id: &str,
    href: &str,
    columns: &[Column],
    rows: &[Vec<Markup>],
    sort: Option<(&str, bool)>,
    filter: &str,
) -> Markup {
    let vt = caps.has(Cap::ViewTransitions).then(|| format!("view-transition-name: wo-table-{id}"));
    let q = if filter.is_empty() { String::new() } else { format!("&q={}", encode(filter)) };
    html! {
        div id=(enhance::swap_id("wo-table", id)) data-wo="swap" class="wo-table" {
            search class="wo-table-filter" {
                form method="get" action=(href) {
                    @if let Some((key, desc)) = sort {
                        input type="hidden" name="sort" value=(key);
                        input type="hidden" name="dir" value=(if desc { "desc" } else { "asc" });
                    }
                    input type="search" name="q" value=(filter) placeholder="Filter rows…" aria-label="Filter rows" autocomplete="off";
                    button type="submit" { "Filter" }
                    @if !filter.is_empty() { a class="wo-table-clear" href=(clear_href(href, sort)) { "Clear" } }
                }
            }
            table {
                thead { tr { @for col in columns {
                    @let sorted = sort.filter(|(k, _)| *k == col.key);
                    @let aria = sorted.map(|(_, d)| if d { "descending" } else { "ascending" });
                    th scope="col" aria-sort=[aria] class=[sorted.map(|_| "wo-table-sorted")] {
                        @if col.sortable {
                            @let next_desc = matches!(sorted, Some((_, false)));
                            a href={ (href) "?sort=" (col.key) "&dir=" (if next_desc { "desc" } else { "asc" }) (q) } {
                                (col.label)
                                @match sorted { Some((_, true)) => span class="wo-table-arrow" { "▼" }, Some((_, false)) => span class="wo-table-arrow" { "▲" }, None => {} }
                            }
                        } @else { (col.label) }
                    }
                } } }
                tbody style=[vt] {
                    @if rows.is_empty() { tr { td colspan=(columns.len()) class="wo-table-empty" { "No rows match." } } }
                    @for row in rows { tr { @for cell in row { td { (cell) } } } }
                }
            }
        }
    }
}

fn clear_href(href: &str, sort: Option<(&str, bool)>) -> String {
    match sort {
        Some((k, d)) => format!("{href}?sort={k}&dir={}", if d { "desc" } else { "asc" }),
        None => href.to_string(),
    }
}

/// Percent-encode a query value: everything but unreserved characters.
fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-table-filter form { display: flex; gap: var(--wo-space); align-items: center; margin-bottom: var(--wo-space); }
.wo-table-filter input { flex: 1; min-width: 8rem; }
.wo-table-clear { color: var(--wo-muted); }
.wo-table thead th { position: sticky; top: 0; background: var(--wo-bg); z-index: 1; }
.wo-table th a { color: inherit; text-decoration: none; }
.wo-table th a:hover { color: var(--wo-accent); text-decoration: underline; }
.wo-table th.wo-table-sorted { color: var(--wo-fg); }
.wo-table-arrow { font-size: 0.7em; margin-left: 0.3em; }
.wo-table-empty { color: var(--wo-muted); text-align: center; padding: 1.5rem; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_flip_direction_and_keep_the_filter() {
        let cols = [Column::sortable("name", "Name"), Column::plain("note", "Note")];
        let m = table(&Caps::NONE, "t", "/t", &cols, &[], Some(("name", false)), "a b").into_string();
        assert!(m.contains("href=\"/t?sort=name&amp;dir=desc&amp;q=a+b\""), "{m}");
        assert!(m.contains("aria-sort=\"ascending\""));
        assert!(m.contains("No rows match."));
        assert!(!m.contains("view-transition-name"));
        assert_eq!(sort_from_query(&cols, Some("note"), None), None);
        assert_eq!(sort_from_query(&cols, Some("name"), Some("desc")), Some(("name", true)));
    }
}
