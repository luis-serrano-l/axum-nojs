//! # Table
//!
//! A data table an admin panel can sort, filter, select from and act on, no script: every
//! column header is a link that re-requests the page sorted by that column, a search box
//! filters rows on the server, checkboxes pick rows for a bulk form, a "Columns" chooser
//! hides columns through `?cols=`, a row can expand a detail block and carry its own action
//! menu, numbers line up, and a CSV link downloads the current filter.
//!
//! **Platform features:**
//! - Ordinary links to `?sort=<col>&dir=asc|desc` in each `<th>`; clicking the sorted column
//!   again flips the direction. The current one carries `aria-sort` for screen readers.
//! - `<form method="get">` inside a `<search>` element (baseline 2023) for the filter; the
//!   sort, page size and hidden columns are kept in hidden inputs so filtering never loses
//!   them and the URL stays shareable.
//! - Row selection through the form attribute (`form="id"`, Chrome 10, Firefox 4, Safari 5.1):
//!   each checkbox belongs to a
//!   bulk `<form method="post">` that sits after the table, so nothing nests and the row's
//!   own menu can still post.
//! - `<details>` (baseline 2020) in the first cell for a row's detail block; the row's menu
//!   is a [`crate::popover_menu`].
//! - `<colgroup>` widths and `font-variant-numeric: tabular-nums` (Chrome 52, Firefox 34,
//!   Safari 9.1) for numeric columns.
//! - `position: sticky` (Chrome 56, Firefox 32, Safari 13) on the header row.
//! - `view-transition-name` on the body so re-sorted rows fade rather than jump.
//! - `aria-busy` on a loading body, drawn as skeleton bars, for a table filled by a later
//!   stream chunk.
//!
//! **What it does not do without script:** resize or reorder columns, or keep row selection
//! across sorts.
//!
//! **Fallback:** none needed. Without `Caps::ViewTransitions` the transition name is
//! omitted; sorting, filtering, choosing columns and the bulk form are plain navigations and
//! posts either way.
//!
//! **Without script:** "select all" does not exist: there is no way to tick every box without
//! script, so the bulk bar says how many are needed and the server answers "nothing selected"
//! with a flash. Everything else works the same.
//!
//! **Finding:** the server sorts and filters; the component only renders what it is given and
//! the links to ask for something else. That is what keeps it usable with `curl`.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, MenuItem, table, table::{Column, Row, TableOptions}};
//! let cols = [Column::sortable("name", "Name"), Column::numeric("size", "Size").width("6rem"), Column::plain("note", "Note")];
//! let rows = vec![Row::new(vec![html!{"a.txt"}, html!{"1 KB"}, html!{"—"}])];
//! let m = table(&Caps::all(), "files", "/table", &cols, &rows, Default::default());
//!
//! let menu = [MenuItem::link("Open", "/files/a.txt"), MenuItem::action("Delete", "/files/a.txt/delete").danger(true)];
//! let rows = vec![Row::new(vec![html!{"a.txt"}, html!{"1 KB"}, html!{"—"}]).key("a.txt").detail(html!{ p { "Modified today." } }).menu(&menu)];
//! let m = table(&Caps::all(), "files", "/table", &cols, &rows, TableOptions::default()
//!     .sort(Some(("name", false))).filter("a").keep(&[("per", "5")])
//!     .cols(Some(&["name", "size"])).choose_columns(true)
//!     .bulk("/files/bulk", &[("archive", "Archive"), ("delete", "Delete")])
//!     .csv("/table.csv")
//!     .empty("No files yet."));
//! let html = m.into_string();
//! assert!(html.contains("aria-sort=\"ascending\""));
//! assert!(html.contains("<input type=\"checkbox\" name=\"row\" value=\"a.txt\" form=\"wo-table-files-bulk\""));
//! assert!(html.contains("href=\"/table.csv?sort=name&amp;dir=asc&amp;q=a&amp;per=5&amp;cols=name%2Csize\""));
//! assert!(!html.contains("<td>—</td>"), "a hidden column's cells are not rendered (its name stays in the chooser)");
//! ```

use maud::{Markup, html};

use crate::popover::{MenuItem, Placement, PopoverOptions, popover_menu};
use crate::{Cap, Caps, enhance};

/// One column: the query key it sorts by, its header text, whether it can be sorted, how
/// its cells align and how wide it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Column<'a> {
    /// Value of `?sort=` for this column, and its name in `?cols=`.
    pub key: &'a str,
    /// Header text.
    pub label: &'a str,
    /// Whether the header is a sort link.
    pub sortable: bool,
    /// Right-aligned tabular figures.
    pub numeric: bool,
    /// A CSS width for the `<col>`, such as `6rem` or `30%`.
    pub width: Option<&'a str>,
}

impl<'a> Column<'a> {
    /// A column whose header sorts the table.
    pub const fn sortable(key: &'a str, label: &'a str) -> Column<'a> {
        Column { key, label, sortable: true, numeric: false, width: None }
    }

    /// A column with a plain header.
    pub const fn plain(key: &'a str, label: &'a str) -> Column<'a> {
        Column { key, label, sortable: false, numeric: false, width: None }
    }

    /// A sortable column of numbers: right-aligned, tabular figures.
    pub const fn numeric(key: &'a str, label: &'a str) -> Column<'a> {
        Column { key, label, sortable: true, numeric: true, width: None }
    }

    /// A fixed width for the column.
    pub const fn width(mut self, width: &'a str) -> Column<'a> {
        self.width = Some(width);
        self
    }
}

/// One row: its cells, and optionally a key (for selection and its menu id), a detail
/// block opened from the first cell, and an action menu in a last column.
#[derive(Clone, Debug)]
pub struct Row<'a> {
    cells: Vec<Markup>,
    key: Option<&'a str>,
    detail: Option<Markup>,
    menu: &'a [MenuItem<'a>],
}

impl<'a> Row<'a> {
    /// A row of cells, one per visible column.
    pub const fn new(cells: Vec<Markup>) -> Self {
        Row { cells, key: None, detail: None, menu: &[] }
    }
    /// The value posted for this row when its checkbox is ticked; also names its menu.
    pub const fn key(mut self, key: &'a str) -> Self {
        self.key = Some(key);
        self
    }
    /// A block shown under the first cell when its `<details>` is opened.
    pub fn detail(mut self, detail: Markup) -> Self {
        self.detail = Some(detail);
        self
    }
    /// Items of the row's action menu (needs a `key`).
    pub const fn menu(mut self, items: &'a [MenuItem<'a>]) -> Self {
        self.menu = items;
        self
    }
}

/// Parse `?sort=<key>&dir=<asc|desc>` into what [`table`] takes: `(key, descending)`.
/// Unknown keys give `None`, so a hand-edited URL cannot ask for a column that is not there.
pub fn sort_from_query<'a>(columns: &[Column<'a>], sort: Option<&str>, dir: Option<&str>) -> Option<(&'a str, bool)> {
    let key = sort?;
    let col = columns.iter().find(|c| c.sortable && c.key == key)?;
    Some((col.key, dir == Some("desc")))
}

/// Parse `?cols=a,b` into the visible keys, keeping only keys the table has and only when at
/// least one is left; `None` means every column.
pub fn cols_from_query<'a>(columns: &[Column<'a>], cols: Option<&str>) -> Option<Vec<&'a str>> {
    let wanted = cols?;
    let keys: Vec<&str> = columns.iter().filter(|c| wanted.split(',').any(|w| w == c.key)).map(|c| c.key).collect();
    (!keys.is_empty()).then_some(keys)
}

/// Options for [`table`]; `Default::default()` is unsorted, unfiltered, every column, no
/// selection, no CSV link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableOptions<'a> {
    /// The current sort as `(key, descending)`, usually from [`sort_from_query`].
    pub sort: Option<(&'a str, bool)>,
    /// The current search text, echoed into the box and kept in the sort links.
    pub filter: &'a str,
    /// Extra query pairs (a page size, say) carried by every link and the filter form.
    pub keep: &'a [(&'a str, &'a str)],
    /// Visible column keys, usually from [`cols_from_query`]; `None` shows all.
    pub cols: Option<&'a [&'a str]>,
    /// Show the "Columns" chooser (links that toggle `?cols=`); also shown whenever `cols`
    /// is set, so a hidden column can always be brought back.
    pub choose_columns: bool,
    /// A bulk post `action` and its buttons `(value, label)`: adds a checkbox column and a
    /// bar after the table. The form posts `row=<key>` per ticked row and `action=<value>`.
    pub bulk: Option<(&'a str, &'a [(&'a str, &'a str)])>,
    /// Base URL of a CSV download; the current sort, filter and columns are appended.
    pub csv: Option<&'a str>,
    /// Message of the empty body.
    pub empty: &'a str,
    /// Draw skeleton rows with `aria-busy` instead of `rows`: the data is still coming.
    pub loading: bool,
}

impl Default for TableOptions<'_> {
    fn default() -> Self {
        TableOptions { sort: None, filter: "", keep: &[], cols: None, choose_columns: false, bulk: None, csv: None, empty: "No rows match.", loading: false }
    }
}

impl<'a> TableOptions<'a> {
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
    /// Extra query pairs carried by every link and the filter form.
    pub fn keep(mut self, keep: &'a [(&'a str, &'a str)]) -> Self {
        self.keep = keep;
        self
    }
    /// Visible column keys.
    pub fn cols(mut self, cols: Option<&'a [&'a str]>) -> Self {
        self.cols = cols;
        self
    }
    /// Show the "Columns" chooser.
    pub fn choose_columns(mut self, choose: bool) -> Self {
        self.choose_columns = choose;
        self
    }
    /// A bulk post action with its buttons.
    pub fn bulk(mut self, action: &'a str, buttons: &'a [(&'a str, &'a str)]) -> Self {
        self.bulk = Some((action, buttons));
        self
    }
    /// Base URL of the CSV download.
    pub fn csv(mut self, href: &'a str) -> Self {
        self.csv = Some(href);
        self
    }
    /// Message of the empty body.
    pub fn empty(mut self, message: &'a str) -> Self {
        self.empty = message;
        self
    }
    /// Show skeleton rows instead of `rows`.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }
}

/// `rows` are already sorted and filtered by the caller; `options` says how, so the links
/// and the filter box reflect it.
pub fn table(caps: &Caps, id: &str, href: &str, columns: &[Column], rows: &[Row], options: TableOptions) -> Markup {
    table_in(caps, id, href, columns, rows, options, true)
}

/// [`table`], as a swap root or not: inside a [`crate::paged_table()`] the pager's root is the
/// swap root, so a sort also refreshes the page links.
pub(crate) fn table_in(caps: &Caps, id: &str, href: &str, columns: &[Column], rows: &[Row], options: TableOptions, swap: bool) -> Markup {
    let TableOptions { sort, filter, keep, cols, choose_columns, bulk, csv, empty, loading } = options;
    let root = enhance::swap_id("wo-table", id);
    let bulk_id = format!("{root}-bulk");
    let vt = caps.has(Cap::ViewTransitions).then(|| format!("view-transition-name: wo-table-{id}"));
    let shown = |c: &Column| cols.is_none_or(|v| v.contains(&c.key));
    let visible: Vec<(usize, &Column)> = columns.iter().enumerate().filter(|(_, c)| shown(c)).collect();
    let cols_value = cols.map(|v| v.join(","));
    // Query pairs every link and form carries besides the sort: filter, extras, columns.
    let mut carried: Vec<(&str, &str)> = Vec::new();
    if !filter.is_empty() {
        carried.push(("q", filter));
    }
    carried.extend(keep.iter().copied());
    if let Some(v) = &cols_value {
        carried.push(("cols", v));
    }
    let sort_pairs = |s: Option<(&str, bool)>| s.map(|(k, d)| format!("sort={k}&dir={}", if d { "desc" } else { "asc" }));
    let query = |s: Option<(&str, bool)>, pairs: &[(&str, &str)]| {
        let all: Vec<String> = sort_pairs(s).into_iter().chain(pairs.iter().map(|(k, v)| format!("{}={}", encode(k), encode(v)))).collect();
        if all.is_empty() { String::new() } else { format!("?{}", all.join("&")) }
    };
    let cols_link = |key: &str| -> String {
        let current: Vec<&str> = cols.map(|v| v.to_vec()).unwrap_or_else(|| columns.iter().map(|c| c.key).collect());
        let next: Vec<&str> = if current.contains(&key) {
            current.iter().copied().filter(|k| *k != key).collect()
        } else {
            columns.iter().map(|c| c.key).filter(|k| current.contains(k) || *k == key).collect()
        };
        let joined = next.join(",");
        let mut pairs: Vec<(&str, &str)> = carried.iter().copied().filter(|(k, _)| *k != "cols").collect();
        pairs.push(("cols", &joined));
        format!("{href}{}", query(sort, &pairs))
    };
    let has_menu = rows.iter().any(|r| !r.menu.is_empty());
    let span = visible.len() + usize::from(bulk.is_some()) + usize::from(has_menu);
    html! {
        div id=(root) data-wo=[swap.then_some("swap")] class="wo-table" {
            div class="wo-table-toolbar" {
                search class="wo-table-filter" {
                    form method="get" action=(href) {
                        @if let Some((key, desc)) = sort {
                            input type="hidden" name="sort" value=(key);
                            input type="hidden" name="dir" value=(if desc { "desc" } else { "asc" });
                        }
                        @for (k, v) in &carried { @if *k != "q" { input type="hidden" name=(k) value=(v); } }
                        input type="search" name="q" value=(filter) placeholder="Filter rows…" aria-label="Filter rows" autocomplete="off";
                        button type="submit" { "Filter" }
                        @if !filter.is_empty() {
                            a class="wo-table-clear" href={ (href) (query(sort, &carried.iter().copied().filter(|(k, _)| *k != "q").collect::<Vec<_>>())) } { "Clear" }
                        }
                    }
                }
                @if choose_columns || cols.is_some() {
                    details class="wo-table-cols" {
                        summary { "Columns" }
                        ul {
                            @for c in columns {
                                @let on = shown(c);
                                li { a href=(cols_link(c.key)) aria-pressed=(on) {
                                    span class="wo-table-cols-mark" aria-hidden="true" { @if on { "\u{2611}" } @else { "\u{2610}" } } " " (c.label)
                                } }
                            }
                        }
                    }
                }
                @if let Some(base) = csv {
                    a class="wo-table-csv" href={ (base) (query(sort, &carried)) } download { "Download CSV" }
                }
            }
            table {
                colgroup {
                    @if bulk.is_some() { col class="wo-table-select-col"; }
                    @for (_, c) in &visible { col style=[c.width.map(|w| format!("width: {w}"))]; }
                    @if has_menu { col class="wo-table-menu-col"; }
                }
                thead { tr {
                    @if bulk.is_some() { th scope="col" class="wo-table-select" { span class="wo-sr" { "Select" } } }
                    @for (_, col) in &visible {
                        @let sorted = sort.filter(|(k, _)| *k == col.key);
                        @let aria = sorted.map(|(_, d)| if d { "descending" } else { "ascending" });
                        th scope="col" aria-sort=[aria] class={ @if sorted.is_some() { "wo-table-sorted" } @if col.numeric { " wo-table-num" } } {
                            @if col.sortable {
                                @let next_desc = matches!(sorted, Some((_, false)));
                                @let pairs: Vec<(&str, &str)> = carried.clone();
                                a href={ (href) (query(Some((col.key, next_desc)), &pairs)) } {
                                    (col.label)
                                    @match sorted { Some((_, true)) => span class="wo-table-arrow" { "▼" }, Some((_, false)) => span class="wo-table-arrow" { "▲" }, None => {} }
                                }
                            } @else { (col.label) }
                        }
                    }
                    @if has_menu { th scope="col" class="wo-table-menu" { span class="wo-sr" { "Actions" } } }
                } }
                tbody style=[vt] aria-busy=[loading.then_some("true")] {
                    @if loading {
                        @for _ in 0..3 { tr class="wo-table-skeleton" { @for _ in 0..span { td { span {} } } } }
                    } @else if rows.is_empty() {
                        tr { td colspan=(span) class="wo-table-empty" { (empty) } }
                    }
                    @for row in rows.iter().filter(|_| !loading) { tr {
                        @if bulk.is_some() {
                            td class="wo-table-select" {
                                @if let Some(k) = row.key { input type="checkbox" name="row" value=(k) form=(bulk_id) aria-label={ "Select " (k) }; }
                            }
                        }
                        @for (n, (i, col)) in visible.iter().enumerate() {
                            @let cell = row.cells.get(*i);
                            td class=[col.numeric.then_some("wo-table-num")] {
                                @match (n, &row.detail) {
                                    (0, Some(detail)) => details class="wo-table-detail" {
                                        summary { @if let Some(c) = cell { (c) } }
                                        div class="wo-table-detail-body" { (detail) }
                                    },
                                    _ => @if let Some(c) = cell { (c) },
                                }
                            }
                        }
                        @if has_menu {
                            td class="wo-table-menu" {
                                @if let (Some(k), false) = (row.key, row.menu.is_empty()) {
                                    (popover_menu(caps, &format!("{root}-{}", slug(k)), "\u{22ef}", row.menu, PopoverOptions::default().placement(Placement::BottomEnd)))
                                }
                            }
                        }
                    } }
                }
            }
            @if let Some((action, buttons)) = bulk {
                form method="post" action=(action) id=(bulk_id) class="wo-table-bulk" {
                    span { "With the selected rows:" }
                    @for (value, label) in buttons { button type="submit" name="action" value=(value) { (label) } }
                }
            }
        }
    }
}

/// A key made safe for an `id`: anything but letters, digits, `-` and `_` becomes `-`.
fn slug(key: &str) -> String {
    key.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect()
}

/// Percent-encode a query value: everything but unreserved characters.
pub(crate) fn encode(s: &str) -> String {
    Encoded(s).to_string()
}

/// [`encode`] written straight into a formatter, so a link can be built without a `String`
/// per pair.
pub(crate) struct Encoded<'a>(pub(crate) &'a str);

impl std::fmt::Display for Encoded<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::fmt::Write;
        for b in self.0.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => f.write_char(b as char)?,
                b' ' => f.write_char('+')?,
                _ => write!(f, "%{b:02X}")?,
            }
        }
        Ok(())
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-table-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: var(--wo-space) calc(var(--wo-space) * 2); margin-bottom: var(--wo-space); }
.wo-table-filter { flex: 1; min-width: 14rem; }
.wo-table-filter form { display: flex; gap: var(--wo-space); align-items: center; }
.wo-table-filter input { flex: 1; min-width: 8rem; }
.wo-table-clear, .wo-table-csv { color: var(--wo-muted); font-size: 0.875rem; }
.wo-table-cols { position: relative; font-size: 0.875rem; }
.wo-table-cols summary { cursor: pointer; list-style: none; padding: 0.35rem 0.75rem; border: 1px solid var(--wo-line); border-radius: var(--wo-radius); background: var(--wo-surface); }
.wo-table-cols summary::-webkit-details-marker { display: none; }
.wo-table-cols ul { position: absolute; right: 0; z-index: 2; margin: 0.25rem 0 0; padding: 0.25rem 0; list-style: none; min-width: 10rem; background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius); box-shadow: 0 8px 24px color-mix(in srgb, var(--wo-fg) 12%, transparent); }
.wo-table-cols li { max-width: none; }
.wo-table-cols a { display: block; padding: 0.35rem 0.75rem; color: inherit; text-decoration: none; }
.wo-table-cols a:hover { background: var(--wo-bg); }
.wo-table-cols-mark { color: var(--wo-accent); }
.wo-table table { table-layout: auto; }
.wo-table thead th { position: sticky; top: 0; background: var(--wo-bg); z-index: 1; }
.wo-table th a { color: inherit; text-decoration: none; }
.wo-table th a:hover { color: var(--wo-accent); text-decoration: underline; }
.wo-table th.wo-table-sorted { color: var(--wo-fg); }
.wo-table-arrow { font-size: 0.7em; margin-left: 0.3em; }
.wo-table-num { text-align: right; font-variant-numeric: tabular-nums; }
.wo-table-select { width: 2.5rem; text-align: center; }
.wo-table-menu { width: 3.5rem; text-align: center; text-wrap: nowrap; }
.wo-table-select input { margin: 0; }
.wo-table-menu .wo-popover > button, .wo-table-menu .wo-popover summary { padding: 0 0.5rem; line-height: 1.6; }
.wo-table-empty { color: var(--wo-muted); text-align: center; padding: 1.5rem; }
.wo-table-detail summary { cursor: pointer; list-style: none; }
.wo-table-detail summary::-webkit-details-marker { display: none; }
.wo-table-detail summary::before { content: "\25B8"; color: var(--wo-muted); margin-right: 0.4em; }
.wo-table-detail[open] summary::before { content: "\25BE"; }
.wo-table-detail-body { margin: 0.5rem 0 0.25rem 1.2em; font-size: 0.875rem; color: var(--wo-muted); }
.wo-table-detail-body > :last-child { margin-bottom: 0; }
.wo-table-bulk { display: flex; flex-wrap: wrap; align-items: center; gap: var(--wo-space); margin-top: var(--wo-space); font-size: 0.875rem; color: var(--wo-muted); }
.wo-table-skeleton td span { display: block; height: 0.9em; margin: 0.2em 0; border-radius: 0.45em; background: var(--wo-line); animation: wo-table-pulse 1.2s ease-in-out infinite; }
@keyframes wo-table-pulse { 50% { opacity: 0.45; } }
@media (prefers-reduced-motion: reduce) { .wo-table-skeleton td span { animation: none; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_flip_direction_and_keep_the_filter() {
        let cols = [Column::sortable("name", "Name"), Column::plain("note", "Note")];
        let opts = TableOptions::default().sort(Some(("name", false))).filter("a b").keep(&[("per", "5")]);
        let m = table(&Caps::NONE, "t", "/t", &cols, &[], opts).into_string();
        assert!(m.contains("href=\"/t?sort=name&amp;dir=desc&amp;q=a+b&amp;per=5\""), "{m}");
        assert!(m.contains("name=\"per\" value=\"5\""));
        assert!(m.contains("href=\"/t?sort=name&amp;dir=asc&amp;per=5\""), "clear link");
        assert!(m.contains("aria-sort=\"ascending\""));
        assert!(m.contains("No rows match."));
        assert!(!m.contains("view-transition-name"));
        assert_eq!(sort_from_query(&cols, Some("note"), None), None);
        assert_eq!(sort_from_query(&cols, Some("name"), Some("desc")), Some(("name", true)));
    }

    #[test]
    fn columns_links_toggle_one_key_in_table_order() {
        let cols = [Column::sortable("a", "A"), Column::numeric("b", "B"), Column::plain("c", "C")];
        assert_eq!(cols_from_query(&cols, Some("c,zzz,a")), Some(vec!["a", "c"]));
        assert_eq!(cols_from_query(&cols, Some("zzz")), None);
        let m = table(&Caps::NONE, "t", "/t", &cols, &[], TableOptions::default().cols(Some(&["a", "c"])).filter("x")).into_string();
        assert!(m.contains("href=\"/t?q=x&amp;cols=c\" aria-pressed=\"true\""), "a shown column links to hiding it: {m}");
        assert!(m.contains("href=\"/t?q=x&amp;cols=a%2Cb%2Cc\" aria-pressed=\"false\""), "a hidden one links to showing it in place");
        assert!(m.contains("name=\"cols\" value=\"a,c\""), "the filter form keeps the columns");
        assert!(!m.contains(">B<"));
    }

    #[test]
    fn loading_and_bulk_states() {
        let cols = [Column::sortable("a", "A")];
        let m = table(&Caps::NONE, "t", "/t", &cols, &[], TableOptions::default().loading(true).bulk("/b", &[("x", "X")])).into_string();
        assert!(m.contains("aria-busy=\"true\"") && m.contains("wo-table-skeleton"));
        assert!(m.contains("<form method=\"post\" action=\"/b\" id=\"wo-table-t-bulk\""));
        assert!(m.contains("<button type=\"submit\" name=\"action\" value=\"x\">X</button>"));
        assert_eq!(slug("src/a.txt"), "src-a-txt");
    }
}
