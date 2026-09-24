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
//!   is a [`crate::Ui::menu`].
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
//! use axum_nojs::{prelude::*, table::Row};
//! // The URL says: sorted by name, filtered to "a", only two columns shown.
//! let ui = Ui::from_request("/table", "sort=name&dir=asc&q=a&cols=name,size", "");
//! // `sortable`, `numeric` and `width` apply to the column added last.
//! let files = ui.table("files", "/table")
//!     .column("name", "Name").sortable()
//!     .column("size", "Size").sortable().numeric().width("6rem")
//!     .column("note", "Note")
//!     .choose_columns()
//!     .bulk("/files/bulk", [("archive", "Archive"), ("delete", "Delete")])
//!     .csv("/table.csv")
//!     .empty("No files yet.");
//! // The route fetches its data with what the table read from the URL.
//! assert_eq!((files.sort(), files.filter()), (Some(("name", false)), "a"));
//! let row = Row::new([html! { "a.txt" }, html! { "1 KB" }, html! { "—" }])
//!     .key("a.txt")
//!     .detail(html! { p { "Modified today." } })
//!     .menu([MenuItem::link("Open", "/files/a.txt"), MenuItem::action("Delete", "/files/a.txt/delete").danger()]);
//! let html = files.rows([row]).render().into_string();
//! assert!(html.contains("aria-sort=\"ascending\""));
//! assert!(html.contains("<input type=\"checkbox\" class=\"nojs-table-check\" name=\"row\" value=\"a.txt\" form=\"nojs-table-files-bulk\""));
//! assert!(html.contains("href=\"/table.csv?sort=name&amp;dir=asc&amp;q=a&amp;cols=name%2Csize\""));
//! assert!(!html.contains("<td>—</td>"), "a hidden column's cells are not rendered (its name stays in the chooser)");
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::input::Input;
use crate::paged_table::{PagedTableOptions, paged_table_with};
use crate::popover::{MenuItem, Placement, menu};
use crate::{Cap, Caps, Icon, Ui, enhance, slug};

/// One column: the query key it sorts by, its header text, whether it can be sorted, how
/// its cells align and how wide it is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Column<'a> {
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
    #[cfg(test)]
    pub const fn sortable(key: &'a str, label: &'a str) -> Column<'a> {
        Column {
            key,
            label,
            sortable: true,
            numeric: false,
            width: None,
        }
    }

    /// A column with a plain header.
    pub const fn plain(key: &'a str, label: &'a str) -> Column<'a> {
        Column {
            key,
            label,
            sortable: false,
            numeric: false,
            width: None,
        }
    }

    /// A sortable column of numbers: right-aligned, tabular figures.
    #[cfg(test)]
    pub const fn numeric(key: &'a str, label: &'a str) -> Column<'a> {
        Column {
            key,
            label,
            sortable: true,
            numeric: true,
            width: None,
        }
    }
}

/// One row: its cells, and optionally a key (for selection and its menu id), a detail
/// block opened from the first cell, and an action menu in a last column.
#[derive(Clone, Debug)]
pub struct Row<'a> {
    cells: Vec<Markup>,
    key: Option<&'a str>,
    detail: Option<Markup>,
    menu: Vec<MenuItem<'a>>,
}

impl<'a> Row<'a> {
    /// A row of cells, one per column (a hidden column's cell is skipped).
    pub fn new(cells: impl IntoIterator<Item = Markup>) -> Self {
        Row {
            cells: cells.into_iter().collect(),
            key: None,
            detail: None,
            menu: Vec::new(),
        }
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
    pub fn menu(mut self, items: impl IntoIterator<Item = MenuItem<'a>>) -> Self {
        self.menu = items.into_iter().collect();
        self
    }
}

/// A row from its cells: `vec![html! { "a.txt" }, html! { "1 KB" }].into()`.
impl From<Vec<Markup>> for Row<'_> {
    fn from(cells: Vec<Markup>) -> Self {
        Row::new(cells)
    }
}

/// A table's URL parameters: `sort`, `dir`, `q`, `page` and `cols`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct TableQuery {
    /// `?sort=<key>`, checked against the columns by [`TableQuery::sort`].
    pub sort: Option<String>,
    /// `?dir=desc`.
    pub desc: bool,
    /// `?q=`, the search text as typed.
    pub filter: String,
    /// `?page=`, 1-based.
    pub page: Option<usize>,
    /// `?cols=a,b`, checked against the columns by [`TableQuery::cols`].
    pub cols: Option<String>,
}

impl TableQuery {
    /// Read the table's parameters from the request.
    pub fn from_ui(ui: &Ui) -> TableQuery {
        TableQuery {
            sort: ui.param("sort").map(str::to_string),
            desc: ui.param("dir") == Some("desc"),
            filter: ui.param("q").unwrap_or("").trim().to_string(),
            page: ui
                .param("page")
                .and_then(|v| v.parse().ok())
                .filter(|&n| n > 0),
            cols: ui.param("cols").map(str::to_string),
        }
    }
    /// The sort as `(key, descending)`, only for a sortable column of `columns`.
    pub fn sort<'c>(&self, columns: &[Column<'c>]) -> Option<(&'c str, bool)> {
        sort_from_query(
            columns,
            self.sort.as_deref(),
            Some(if self.desc { "desc" } else { "asc" }),
        )
    }
    /// The visible column keys, only those `columns` has; `None` means every column.
    pub fn cols<'c>(&self, columns: &[Column<'c>]) -> Option<Vec<&'c str>> {
        cols_from_query(columns, self.cols.as_deref())
    }
}

/// Parse `?sort=<key>&dir=<asc|desc>` into `(key, descending)`. Unknown keys give `None`,
/// so a hand-edited URL cannot ask for a column that is not there.
pub(crate) fn sort_from_query<'a>(
    columns: &[Column<'a>],
    sort: Option<&str>,
    dir: Option<&str>,
) -> Option<(&'a str, bool)> {
    let key = sort?;
    let col = columns.iter().find(|c| c.sortable && c.key == key)?;
    Some((col.key, dir == Some("desc")))
}

/// Parse `?cols=a,b` into the visible keys, keeping only keys the table has and only when at
/// least one is left; `None` means every column.
pub(crate) fn cols_from_query<'a>(
    columns: &[Column<'a>],
    cols: Option<&str>,
) -> Option<Vec<&'a str>> {
    let wanted = cols?;
    let keys: Vec<&str> = columns
        .iter()
        .filter(|c| wanted.split(',').any(|w| w == c.key))
        .map(|c| c.key)
        .collect();
    (!keys.is_empty()).then_some(keys)
}

/// How the table renders; `Default::default()` is unsorted, unfiltered, every column, no
/// selection, no CSV link.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct TableOptions<'a> {
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
        TableOptions {
            sort: None,
            filter: "",
            keep: &[],
            cols: None,
            choose_columns: false,
            bulk: None,
            csv: None,
            empty: "No rows match.",
            loading: false,
        }
    }
}

// Setters for the tests; the builder fills the struct directly.
#[cfg(test)]
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
    /// A bulk post action with its buttons.
    pub fn bulk(mut self, action: &'a str, buttons: &'a [(&'a str, &'a str)]) -> Self {
        self.bulk = Some((action, buttons));
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
pub(crate) fn table_with(
    caps: &Caps,
    id: &str,
    href: &str,
    columns: &[Column],
    rows: &[Row],
    options: TableOptions,
) -> Markup {
    table_in(caps, id, href, columns, rows, options, true)
}

/// [`table_with`], as a swap root or not: inside a paged table the pager's root is the swap
/// root, so a sort also refreshes the page links.
pub(crate) fn table_in(
    caps: &Caps,
    id: &str,
    href: &str,
    columns: &[Column],
    rows: &[Row],
    options: TableOptions,
    swap: bool,
) -> Markup {
    let TableOptions {
        sort,
        filter,
        keep,
        cols,
        choose_columns,
        bulk,
        csv,
        empty,
        loading,
    } = options;
    let root = enhance::swap_id("nojs-table", id);
    let filter_id = format!("{root}-q");
    let bulk_id = format!("{root}-bulk");
    let vt = caps
        .has(Cap::ViewTransitions)
        .then(|| format!("view-transition-name: nojs-table-{id}"));
    let shown = |c: &Column| cols.is_none_or(|v| v.contains(&c.key));
    let visible: Vec<(usize, &Column)> = columns
        .iter()
        .enumerate()
        .filter(|(_, c)| shown(c))
        .collect();
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
    let sort_pairs = |s: Option<(&str, bool)>| {
        s.map(|(k, d)| format!("sort={k}&dir={}", if d { "desc" } else { "asc" }))
    };
    let query = |s: Option<(&str, bool)>, pairs: &[(&str, &str)]| {
        let all: Vec<String> = sort_pairs(s)
            .into_iter()
            .chain(
                pairs
                    .iter()
                    .map(|(k, v)| format!("{}={}", encode(k), encode(v))),
            )
            .collect();
        if all.is_empty() {
            String::new()
        } else {
            format!("?{}", all.join("&"))
        }
    };
    let cols_link = |key: &str| -> String {
        let current: Vec<&str> = cols
            .map(|v| v.to_vec())
            .unwrap_or_else(|| columns.iter().map(|c| c.key).collect());
        let next: Vec<&str> = if current.contains(&key) {
            current.iter().copied().filter(|k| *k != key).collect()
        } else {
            columns
                .iter()
                .map(|c| c.key)
                .filter(|k| current.contains(k) || *k == key)
                .collect()
        };
        let joined = next.join(",");
        let mut pairs: Vec<(&str, &str)> = carried
            .iter()
            .copied()
            .filter(|(k, _)| *k != "cols")
            .collect();
        pairs.push(("cols", &joined));
        format!("{href}{}", query(sort, &pairs))
    };
    let has_menu = rows.iter().any(|r| !r.menu.is_empty());
    let span = visible.len() + usize::from(bulk.is_some()) + usize::from(has_menu);
    html! {
        div id=(root) data-nojs=[swap.then_some("swap")] class="nojs-table" {
            div class="nojs-table-toolbar" {
                search class="nojs-table-filter" {
                    form method="get" action=(href) {
                        @if let Some((key, desc)) = sort {
                            input type="hidden" name="sort" value=(key);
                            input type="hidden" name="dir" value=(if desc { "desc" } else { "asc" });
                        }
                        @for (k, v) in &carried { @if *k != "q" { input type="hidden" name=(k) value=(v); } }
                        (Input::search_box("q", "Filter rows", filter).id(&filter_id).placeholder("Filter rows…").autocomplete("off").class("nojs-table-filter-input"))
                        (Button::new(*caps, "Filter"))
                        @if !filter.is_empty() {
                            a class="nojs-table-clear" href={ (href) (query(sort, &carried.iter().copied().filter(|(k, _)| *k != "q").collect::<Vec<_>>())) } { "Clear" }
                        }
                    }
                }
                @if choose_columns || cols.is_some() {
                    details class="nojs-table-cols" {
                        summary class="nojs-button" { "Columns" (Icon::ChevronDown) }
                        ul {
                            @for c in columns {
                                @let on = shown(c);
                                li { a href=(cols_link(c.key)) aria-pressed=(on) {
                                    span class="nojs-table-cols-mark" aria-hidden="true" { @if on { "\u{2611}" } @else { "\u{2610}" } } " " (c.label)
                                } }
                            }
                        }
                    }
                }
                @if let Some(base) = csv {
                    a class="nojs-table-csv" href={ (base) (query(sort, &carried)) } download { "Download CSV" }
                }
            }
            table {
                colgroup {
                    @if bulk.is_some() { col class="nojs-table-select-col"; }
                    @for (_, c) in &visible { col style=[c.width.map(|w| format!("width: {w}"))]; }
                    @if has_menu { col class="nojs-table-menu-col"; }
                }
                thead { tr {
                    @if bulk.is_some() { th scope="col" class="nojs-table-select" { span class="nojs-sr" { "Select" } } }
                    @for (_, col) in &visible {
                        @let sorted = sort.filter(|(k, _)| *k == col.key);
                        @let aria = sorted.map(|(_, d)| if d { "descending" } else { "ascending" });
                        th scope="col" aria-sort=[aria] class={ @if sorted.is_some() { "nojs-table-sorted" } @if col.numeric { " nojs-table-num" } } {
                            @if col.sortable {
                                @let next_desc = matches!(sorted, Some((_, false)));
                                @let pairs: Vec<(&str, &str)> = carried.clone();
                                a href={ (href) (query(Some((col.key, next_desc)), &pairs)) } {
                                    (col.label)
                                    @match sorted { Some((_, true)) => span class="nojs-table-arrow" { "▼" }, Some((_, false)) => span class="nojs-table-arrow" { "▲" }, None => {} }
                                }
                            } @else { (col.label) }
                        }
                    }
                    @if has_menu { th scope="col" class="nojs-table-menu" { span class="nojs-sr" { "Actions" } } }
                } }
                tbody style=[vt] aria-busy=[loading.then_some("true")] {
                    @if loading {
                        @for _ in 0..3 { tr class="nojs-table-skeleton" { @for _ in 0..span { td { span {} } } } }
                    } @else if rows.is_empty() {
                        tr { td colspan=(span) class="nojs-table-empty" { (empty) } }
                    }
                    @for row in rows.iter().filter(|_| !loading) { tr {
                        @if bulk.is_some() {
                            td class="nojs-table-select" {
                                @if let Some(k) = row.key { input type="checkbox" class="nojs-table-check" name="row" value=(k) form=(bulk_id) aria-label={ "Select " (k) }; }
                            }
                        }
                        @for (n, (i, col)) in visible.iter().enumerate() {
                            @let cell = row.cells.get(*i);
                            td class=[col.numeric.then_some("nojs-table-num")] {
                                @match (n, &row.detail) {
                                    (0, Some(detail)) => details class="nojs-table-detail" {
                                        summary { @if let Some(c) = cell { (c) } }
                                        div class="nojs-table-detail-body" { (detail) }
                                    },
                                    _ => @if let Some(c) = cell { (c) },
                                }
                            }
                        }
                        @if has_menu {
                            td class="nojs-table-menu" {
                                @if let (Some(k), false) = (row.key, row.menu.is_empty()) {
                                    (menu(caps, &format!("{root}-{}", slug(k)), "Row actions", &row.menu, Placement::BottomEnd, true))
                                }
                            }
                        }
                    } }
                }
            }
            @if let Some((action, buttons)) = bulk {
                form method="post" action=(action) id=(bulk_id) class="nojs-table-bulk" {
                    span { "With the selected rows:" }
                    @for (value, label) in buttons { (Button::new(*caps, label).small().name("action").value(value)) }
                }
            }
        }
    }
}

/// A data table, made by [`Ui::table`]. It reads its sort, filter, page and visible columns
/// from the request (`?sort=&dir=&q=&page=&cols=`), so a route asks it how to fetch the
/// rows ([`Table::sort`], [`Table::filter`]) and hands them over with [`Table::rows`].
#[derive(Clone, Debug)]
pub struct Table<'a> {
    ui: &'a Ui,
    id: &'a str,
    href: &'a str,
    query: TableQuery,
    columns: Vec<Column<'a>>,
    rows: Vec<Row<'a>>,
    total: Option<usize>,
    choose_columns: bool,
    bulk: Option<(&'a str, Vec<(&'a str, &'a str)>)>,
    csv: Option<&'a str>,
    empty: &'a str,
    loading: bool,
}

impl Ui {
    /// A table `id` whose links and forms go to `href`; add columns with [`Table::column`].
    pub fn table<'a>(&'a self, id: &'a str, href: &'a str) -> Table<'a> {
        Table {
            ui: self,
            id,
            href,
            query: TableQuery::from_ui(self),
            columns: Vec::new(),
            rows: Vec::new(),
            total: None,
            choose_columns: false,
            bulk: None,
            csv: None,
            empty: "No rows match.",
            loading: false,
        }
    }
}

impl<'a> Table<'a> {
    fn last(mut self, change: impl FnOnce(&mut Column<'a>)) -> Self {
        if let Some(c) = self.columns.last_mut() {
            change(c);
        }
        self
    }

    /// A column: `key` names it in `?sort=` and `?cols=`, `label` is its header.
    pub fn column(mut self, key: &'a str, label: &'a str) -> Self {
        self.columns.push(Column::plain(key, label));
        self
    }

    /// The column added last sorts the table: its header links to `?sort=<key>`.
    pub fn sortable(self) -> Self {
        self.last(|c| c.sortable = true)
    }

    /// The column added last holds numbers: right-aligned, tabular figures.
    pub fn numeric(self) -> Self {
        self.last(|c| c.numeric = true)
    }

    /// A CSS width for the column added last, such as `6rem` or `30%`.
    pub fn width(self, width: &'a str) -> Self {
        self.last(|c| c.width = Some(width))
    }

    /// The rows, already sorted and filtered as [`Table::sort`] and [`Table::filter`] say;
    /// cells in column order.
    pub fn rows(mut self, rows: impl IntoIterator<Item = Row<'a>>) -> Self {
        self.rows = rows.into_iter().collect();
        self
    }

    /// Page the rows: `total` is the row count after filtering. Given every row, the table
    /// shows the current page of them; given only the current page's rows ([`Table::page`],
    /// [`Table::per_page`]), it shows those. The page size the visitor picks is remembered
    /// as `per.<id>`.
    pub fn paged(mut self, total: usize) -> Self {
        self.total = Some(total);
        self
    }

    /// A "Columns" chooser: links that toggle `?cols=`.
    pub fn choose_columns(mut self) -> Self {
        self.choose_columns = true;
        self
    }

    /// A checkbox per row (rows need a [`Row::key`]) and a bar of `(value, label)` buttons
    /// posting to `action`: `row=<key>` per ticked row and `action=<value>`.
    pub fn bulk(
        mut self,
        action: &'a str,
        buttons: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Self {
        self.bulk = Some((action, buttons.into_iter().collect()));
        self
    }

    /// A "Download CSV" link to `href` with the current sort, filter and columns appended.
    pub fn csv(mut self, href: &'a str) -> Self {
        self.csv = Some(href);
        self
    }

    /// What the body says when there are no rows.
    pub fn empty(mut self, message: &'a str) -> Self {
        self.empty = message;
        self
    }

    /// Skeleton rows with `aria-busy` instead of the rows: the data is still coming.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// The requested sort as `(column key, descending)`, only for a sortable column.
    pub fn sort(&self) -> Option<(&'a str, bool)> {
        self.query.sort(&self.columns)
    }

    /// The requested search text, trimmed, as typed.
    pub fn filter(&self) -> &str {
        &self.query.filter
    }

    /// The keys of the columns to show, in table order.
    pub fn visible(&self) -> Vec<&'a str> {
        self.query
            .cols(&self.columns)
            .unwrap_or_else(|| self.columns.iter().map(|c| c.key).collect())
    }

    /// The requested page, 1-based.
    pub fn page(&self) -> usize {
        self.query.page.unwrap_or(1)
    }

    /// Rows per page: the visitor's remembered choice, or 10.
    pub fn per_page(&self) -> usize {
        self.ui
            .state
            .per_page(self.id)
            .unwrap_or(crate::paged_table::PAGE_SIZES[1])
    }
}

impl Render for Table<'_> {
    fn render(&self) -> Markup {
        let cols = self.query.cols(&self.columns);
        let options = TableOptions {
            sort: self.sort(),
            filter: self.filter(),
            cols: cols.as_deref(),
            choose_columns: self.choose_columns,
            bulk: self
                .bulk
                .as_ref()
                .map(|(action, buttons)| (*action, buttons.as_slice())),
            csv: self.csv,
            empty: self.empty,
            loading: self.loading,
            ..TableOptions::default()
        };
        match self.total {
            Some(total) => {
                let paged = PagedTableOptions {
                    table: options,
                    query: Some(&self.query),
                    state: Some(&self.ui.state),
                    ..PagedTableOptions::default()
                };
                paged_table_with(
                    &self.ui.caps,
                    self.id,
                    self.href,
                    &self.columns,
                    &self.rows,
                    total,
                    paged,
                )
            }
            None => table_with(
                &self.ui.caps,
                self.id,
                self.href,
                &self.columns,
                &self.rows,
                options,
            ),
        }
    }
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
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    f.write_char(b as char)?
                }
                b' ' => f.write_char('+')?,
                _ => write!(f, "%{b:02X}")?,
            }
        }
        Ok(())
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
/* shadcn Data Table: a toolbar (filter input, outline buttons), a bordered rounded frame
   round the table, text-sm cells, muted/50 row hover, a DropdownMenu for columns. */
.nojs-table-toolbar { display: flex; flex-wrap: wrap; align-items: center; gap: var(--nojs-space); margin-bottom: calc(var(--nojs-space) * 2); }
.nojs-table-filter { flex: 1; min-width: 14rem; }
.nojs-table-filter form { display: flex; gap: var(--nojs-space); align-items: center; }
.nojs-table-filter-input { flex: 1; min-width: 8rem; max-width: 24rem; }
.nojs-table-clear, .nojs-table-csv { color: var(--nojs-muted); font-size: 0.875rem; }
.nojs-table-clear:hover, .nojs-table-csv:hover { color: var(--nojs-fg); }
.nojs-table-cols { position: relative; font-size: 0.875rem; }
.nojs-table-cols summary { list-style: none; }
.nojs-table-cols summary::-webkit-details-marker { display: none; }
.nojs-table-cols ul {
  position: absolute; right: 0; z-index: 2; margin: 0.25rem 0 0; padding: 0.25rem; list-style: none; min-width: 10rem;
  background: var(--nojs-popover); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); box-shadow: var(--nojs-shadow-lg);
}
.nojs-table-cols li { max-width: none; }
.nojs-table-cols a { display: flex; gap: 0.5rem; padding: 0.375rem 0.5rem; border-radius: var(--nojs-radius-sm); color: inherit; text-decoration: none; }
.nojs-table-cols a:hover { background: var(--nojs-accent); color: var(--nojs-on-accent); }
.nojs-table-cols-mark { color: var(--nojs-fg); }
.nojs-table > table { border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); }
.nojs-table table { table-layout: auto; border-collapse: separate; border-spacing: 0; }
.nojs-table tbody tr:last-child > * { border-bottom: 0; }
.nojs-table thead th { position: sticky; top: 0; background: var(--nojs-bg); z-index: 1; }
.nojs-table thead th:first-child { border-top-left-radius: var(--nojs-radius); }
.nojs-table thead th:last-child { border-top-right-radius: var(--nojs-radius); }
.nojs-table th a { color: inherit; text-decoration: none; display: inline-flex; align-items: center; gap: 0.25rem; }
.nojs-table th a:hover { color: var(--nojs-fg); }
.nojs-table th.nojs-table-sorted { color: var(--nojs-fg); }
.nojs-table-arrow { font-size: 0.75em; margin-left: 0.25em; }
/* Code in a cell is plain monospace text, so a long path wraps without a broken box. */
.nojs-table td code { background: none; border: 0; padding: 0; }
.nojs-table-num { text-align: right; font-variant-numeric: tabular-nums; white-space: nowrap; }
.nojs-table-select { width: 2.5rem; text-align: center; }
.nojs-table-menu { width: 3.5rem; text-align: center; text-wrap: nowrap; }
.nojs-table-check { margin: 0; }
.nojs-table-empty { color: var(--nojs-muted); text-align: center; padding: 1.5rem; height: 6rem; }
.nojs-table-detail summary { cursor: pointer; list-style: none; font-weight: 400; }
.nojs-table-detail summary::-webkit-details-marker { display: none; }
.nojs-table-detail summary::before { content: "\25B8"; color: var(--nojs-muted); margin-right: 0.4em; }
.nojs-table-detail[open] summary::before { content: "\25BE"; }
.nojs-table-detail-body { margin: 0.5rem 0 0.25rem 1.2em; font-size: 0.875rem; color: var(--nojs-muted); }
.nojs-table-detail-body > :last-child { margin-bottom: 0; }
.nojs-table-bulk { display: flex; flex-wrap: wrap; align-items: center; gap: var(--nojs-space); margin-top: calc(var(--nojs-space) * 2); font-size: 0.875rem; color: var(--nojs-muted); }
.nojs-table-skeleton td span { display: block; height: 1rem; margin: 0.125rem 0; border-radius: var(--nojs-radius-sm); background: var(--nojs-accent); animation: nojs-table-pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite; }
@keyframes nojs-table-pulse { 50% { opacity: 0.5; } }
@media (prefers-reduced-motion: reduce) { .nojs-table-skeleton td span { animation: none; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_flip_direction_and_keep_the_filter() {
        let cols = [
            Column::sortable("name", "Name"),
            Column::plain("note", "Note"),
        ];
        let opts = TableOptions::default()
            .sort(Some(("name", false)))
            .filter("a b")
            .keep(&[("per", "5")]);
        let m = table_with(&Caps::NONE, "t", "/t", &cols, &[], opts).into_string();
        assert!(
            m.contains("href=\"/t?sort=name&amp;dir=desc&amp;q=a+b&amp;per=5\""),
            "{m}"
        );
        assert!(m.contains("name=\"per\" value=\"5\""));
        assert!(
            m.contains("href=\"/t?sort=name&amp;dir=asc&amp;per=5\""),
            "clear link"
        );
        assert!(m.contains("aria-sort=\"ascending\""));
        assert!(m.contains("No rows match."));
        assert!(!m.contains("view-transition-name"));
        assert_eq!(sort_from_query(&cols, Some("note"), None), None);
        assert_eq!(
            sort_from_query(&cols, Some("name"), Some("desc")),
            Some(("name", true))
        );
    }

    #[test]
    fn columns_links_toggle_one_key_in_table_order() {
        let cols = [
            Column::sortable("a", "A"),
            Column::numeric("b", "B"),
            Column::plain("c", "C"),
        ];
        assert_eq!(
            cols_from_query(&cols, Some("c,zzz,a")),
            Some(vec!["a", "c"])
        );
        assert_eq!(cols_from_query(&cols, Some("zzz")), None);
        let m = table_with(
            &Caps::NONE,
            "t",
            "/t",
            &cols,
            &[],
            TableOptions::default().cols(Some(&["a", "c"])).filter("x"),
        )
        .into_string();
        assert!(
            m.contains("href=\"/t?q=x&amp;cols=c\" aria-pressed=\"true\""),
            "a shown column links to hiding it: {m}"
        );
        assert!(
            m.contains("href=\"/t?q=x&amp;cols=a%2Cb%2Cc\" aria-pressed=\"false\""),
            "a hidden one links to showing it in place"
        );
        assert!(
            m.contains("name=\"cols\" value=\"a,c\""),
            "the filter form keeps the columns"
        );
        assert!(!m.contains(">B<"));
    }

    #[test]
    fn loading_and_bulk_states() {
        let cols = [Column::sortable("a", "A")];
        let m = table_with(
            &Caps::NONE,
            "t",
            "/t",
            &cols,
            &[],
            TableOptions::default()
                .loading(true)
                .bulk("/b", &[("x", "X")]),
        )
        .into_string();
        assert!(m.contains("aria-busy=\"true\"") && m.contains("nojs-table-skeleton"));
        assert!(m.contains("<form method=\"post\" action=\"/b\" id=\"nojs-table-t-bulk\""));
        assert!(m.contains("<button type=\"submit\" class=\"nojs-button nojs-button-small\" name=\"action\" value=\"x\">X</button>"), "{m}");
        assert_eq!(slug("src/a.txt"), "src-a-txt");
    }
}
