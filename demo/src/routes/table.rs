//! The table page: sort, filter, paging, columns, CSV, in-place edit and bulk actions.

use crate::site::page;
use axum::{
    Form, Router,
    response::IntoResponse,
    routing::{get, post},
};
use loco_ui::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/table", get(table_page))
        .route("/table.csv", get(table_csv))
        .route("/table/bulk", post(table_bulk))
        .route("/table/edit", post(table_edit))
}

/// Each page's live component, which the index shows too (`site::preview`), without the
/// visitor's renamed kinds there.
pub(crate) const PREVIEWS: &[super::Preview] =
    &[("/table", |ui| files_page(ui, &Kinds::default()))];

const FILES: [(&str, u32, &str); 12] = [
    ("archive.tar", 40960, "backup"),
    ("build.rs", 1200, "script"),
    ("cargo.lock", 8800, "generated"),
    ("index.html", 2100, "page"),
    ("logo.svg", 3400, "image"),
    ("main.rs", 5600, "source"),
    ("notes.md", 900, "text"),
    ("photo.jpg", 250000, "image"),
    ("readme.md", 4100, "text"),
    ("style.css", 1500, "stylesheet"),
    ("tests.rs", 7700, "source"),
    ("video.mp4", 9800000, "video"),
];

/// A file: its path, its size in bytes and its kind.
type File = (String, u32, &'static str);

/// The table's columns, every one sortable; the CSV writes the visible ones.
const COLUMNS: [&str; 3] = ["name", "size", "kind"];

/// Thirty-six files filtered and sorted on the server as the table's query says, in one place
/// for the page and the CSV.
fn files(q: &TableQuery) -> Vec<File> {
    let mut files: Vec<File> = ["src", "docs", "old"]
        .iter()
        .flat_map(|dir| {
            FILES
                .iter()
                .map(move |f| (format!("{dir}/{}", f.0), f.1 * (dir.len() as u32), f.2))
        })
        .filter(|f| q.matches(&f.0) || q.matches(f.2))
        .collect();
    q.sort_by(&mut files, |a, b, key| match key {
        "size" => a.1.cmp(&b.1),
        "kind" => a.2.cmp(b.2),
        _ => a.0.cmp(&b.0),
    });
    files
}

/// Kinds renamed in place on `/table`, remembered per visitor.
#[derive(Default, Deserialize, Serialize)]
struct Kinds(Vec<(String, String)>);

impl Kinds {
    fn of<'a>(&'a self, file: &str, kind: &'a str) -> &'a str {
        self.0
            .iter()
            .find(|(f, _)| f == file)
            .map_or(kind, |(_, k)| k.as_str())
    }
}

/// A file as a row that expands, has its own menu, can be selected and edited in place.
fn file_row<'a>(f: &'a File, kinds: &'a Kinds) -> Row<'a> {
    let size = format!(
        "{} KB",
        loco_ui::paged_table::thousands(f.1 as usize / 1024)
    );
    Row::from((html! { code { (f.0) } }, size, kinds.of(&f.0, f.2)))
        .key(&f.0)
        .values(["", "", kinds.of(&f.0, f.2)])
        .detail(html! { p { "A " (f.2) " of " (f.1) " bytes, in " code { (f.0.split('/').next().unwrap_or("")) } "." } })
        .menu([MenuItem::link("Open", "/table"), MenuItem::action("Delete", "/table/bulk").danger()])
}

/// The table only renders and links: the route reads its query, fetches, sorts and slices.
fn files_page(ui: &Ui, kinds: &Kinds) -> Markup {
    // code: /table
    let q = ui.table_query("files", &COLUMNS);
    let files = files(&q);
    let (page, total) = q.page_of(&files);
    lui! { Table("files", "/table") paged=(total) choose_columns csv="/table.csv" edit="/table/edit"
            empty="No files match this filter." loading=(ui.param("loading") == Some("1")) {
        column "name" "Name" sortable;
        column "size" "Size" sortable numeric width="7rem";
        column "kind" "Kind" sortable editable width="9rem";
        bulk "/table/bulk" ([("archive", "Archive"), ("delete", "Delete")]);
        rows (page.iter().map(|f| file_row(f, kinds)));
    } }
    // end code
}

async fn table_page(ui: Ui, Saved(kinds): Saved<Kinds>) -> Page {
    let t = files_page(&ui, &kinds);
    page(
        &ui,
        "Table",
        html! {
            p { "Click a header to sort, again to flip. Type to filter. Hide columns, tick rows for the bulk form, open a row's menu or its detail. The page size you pick is remembered for your next visit. Every state is a URL, including " a href="/table?loading=1" { "the loading one" } "." }
            (t)
        },
    )
}

/// The same rows as text/csv, for the sort, filter and columns in the URL.
async fn table_csv(ui: Ui) -> impl IntoResponse {
    let q = ui.table_query("files", &COLUMNS);
    let cols = q.visible(&COLUMNS);
    let mut csv = cols.join(",") + "\n";
    for f in files(&q) {
        let cells = [
            ("name", f.0.clone()),
            ("size", f.1.to_string()),
            ("kind", f.2.to_string()),
        ];
        csv += &cells
            .iter()
            .filter(|(k, _)| cols.contains(k))
            .map(|(_, v)| v.as_str())
            .collect::<Vec<_>>()
            .join(",");
        csv.push('\n');
    }
    (
        [
            ("content-type", "text/csv; charset=utf-8"),
            ("content-disposition", "attachment; filename=\"files.csv\""),
        ],
        csv,
    )
}

/// A row edited in place: the file's new kind, saved, then back to the page it came from.
#[derive(Deserialize)]
struct EditedRow {
    key: String,
    kind: String,
    returns_to: String,
}

async fn table_edit(
    ui: Ui,
    Saved(mut kinds): Saved<Kinds>,
    Form(row): Form<EditedRow>,
) -> Redirect {
    let kind: String = row.kind.trim().chars().take(20).collect();
    kinds.0.retain(|(f, _)| *f != row.key);
    if !kind.is_empty() {
        kinds.0.push((row.key.clone(), kind));
    }
    // Only back to this page: `returns_to` is posted, so it is not trusted as a URL.
    let back = if row.returns_to.starts_with("/table") {
        &row.returns_to
    } else {
        "/table"
    };
    ui.redirect(back)
        .flash(&format!("Saved {}.", row.key))
        .save(&kinds)
}

/// `row=<key>` per ticked box and `action=<value>` from the button: acknowledged with a flash.
async fn table_bulk(ui: Ui, posted: Posted) -> Redirect {
    let rows = posted.all("row").count();
    let action = match posted.get("action") {
        "" => "delete",
        a => a,
    };
    let msg = if rows == 0 {
        "Nothing selected: tick a row first.".to_string()
    } else {
        format!("{action}: {rows} file(s) (not really).")
    };
    ui.redirect("/table").flash(&msg)
}
