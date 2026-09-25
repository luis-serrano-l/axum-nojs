//! Widgets: calendar, upload, kanban and marquee.

use crate::site::page;
use axum::{
    Form, Router,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use loco_ui::calendar::Date;
use loco_ui::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/calendar", get(calendar_page))
        .route("/upload", get(upload_page).post(upload_submit))
        .route("/upload/remove", post(upload_remove))
        .route("/upload/file/{n}", get(upload_file))
        .route("/kanban", get(kanban_page).post(kanban_move))
        .route("/marquee", get(marquee_page))
}

/// Two looping rows: names, then quotes going the other way. Hover or focus stops them.
async fn marquee_page(ui: Ui) -> Page {
    let logos = [
        "Acme", "Globex", "Initech", "Umbrella", "Hooli", "Stark", "Wayne", "Tyrell",
    ];
    let quotes = [
        ("Ada, CTO", "Every page works with script off."),
        ("Grace, SRE", "One stylesheet, no bundler."),
        ("Alan, founder", "Forms that post. Imagine that."),
    ];
    page(
        &ui,
        "Marquee",
        lui! {
            // code: /marquee
            Stack(lui! {
                Marquee("Customers") { @for name in logos { text (name); } }
                Marquee("What people say") reverse duration=30 {
                    @for (who, quote) in quotes {
                        item() { Card description=(who) { p { (quote) } } }
                    }
                }
            });
            // end code
            p class="lui-note" { "Point at a row, or tab into it, and it stops. With reduced motion asked for, or in a browser without " code { "translate" } ", the items sit still and wrap." }
        },
    )
}

async fn calendar_page(ui: Ui) -> Page {
    let soon = |days| Date::today().add_days(days).to_string();
    let (invoice, release) = (soon(6), soon(21));
    page(
        &ui,
        "Calendar",
        lui! {
            // code: /calendar
            Calendar("day") disabled=(|d| d.weekday() >= 5)
                event=(&invoice, "Invoice due") event=(&release, "Release");
            // end code
            p class="lui-note" { @match ui.param("day") {
                Some(d) => { "You picked " (d) ". Weekends cannot be picked; a dot marks an event." },
                None => { "Pick a weekday. The month links and the days are ordinary links: the page comes back with " code { "?day=" } " set." },
            } }
            h2 { "In a form" }
            form class="lui-stack" method="get" action="/calendar" {
                // code: /calendar
                DatePicker("due", "Due date") required disabled=(|d| d.weekday() >= 5);
                DatePicker("born", "Born") native max="2026-12-31";
                Button("Save") primary;
                // end code
            }
        },
    )
}

/// Files sent on `/upload`, per visitor, in memory: 3 files of up to 200 KB each for at most
/// 100 visitors, the oldest dropped first. A demo store, not a pattern for a real one.
type Held = (String, Vec<u8>);
/// A visitor's id and their files.
type Shelf = (String, Vec<Held>);
static UPLOADS: LazyLock<Mutex<Vec<Shelf>>> = LazyLock::new(Mutex::default);
const UPLOAD_MAX: usize = 200 * 1024;

/// Who sent the files: a random id in the saved cookie.
#[derive(Default, Deserialize, Serialize)]
struct Uploader {
    id: String,
}

fn uploads(who: &str) -> Vec<Held> {
    let all = UPLOADS.lock().unwrap_or_else(|e| e.into_inner());
    all.iter()
        .find(|(w, _)| w == who)
        .map(|(_, f)| f.clone())
        .unwrap_or_default()
}

fn keep_uploads(who: &str, files: Vec<Held>) {
    let mut all = UPLOADS.lock().unwrap_or_else(|e| e.into_inner());
    all.retain(|(w, _)| w != who);
    all.push((who.to_string(), files));
    let over = all.len().saturating_sub(100);
    all.drain(..over);
}

/// Raster images show inline; anything else downloads, so an uploaded page or SVG never runs here.
fn upload_type(name: &str) -> Option<&'static str> {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "png" => Some("image/png"),
        "jpg" | "jpeg" => Some("image/jpeg"),
        "gif" => Some("image/gif"),
        "webp" => Some("image/webp"),
        _ => None,
    }
}

async fn upload_page(ui: Ui, Saved(who): Saved<Uploader>) -> Page {
    let files = uploads(&who.id);
    let links: Vec<String> = (0..files.len())
        .map(|n| format!("/upload/file/{n}"))
        .collect();
    // code: /upload
    let up = lui! {
        Upload("/upload", "file") accept="image/*,.txt,.pdf" multiple
            hint="Images, text or PDF, up to 200 KB each. The last three are kept." {
            @for ((name, bytes), href) in files.iter().zip(&links) {
                // An image gets a thumbnail; anything else just its link.
                file (name) (bytes.len() as u64) href=(href) preview=[upload_type(name).map(|_| href)];
            }
            remove "/upload/remove";
        }
    };
    // end code
    page(&ui, "Upload", html! { (ui.flash()) (up) })
}

/// The multipart post: every non-empty `file` part up to the size limit, then PRG.
async fn upload_submit(
    ui: Ui,
    Saved(who): Saved<Uploader>,
    mut parts: axum::extract::Multipart,
) -> Redirect {
    let who = if who.id.is_empty() {
        Uploader {
            id: format!(
                "{:x}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |d| d.as_nanos())
            ),
        }
    } else {
        who
    };
    let (mut files, mut kept, mut refused) = (uploads(&who.id), 0, 0);
    while let Ok(Some(part)) = parts.next_field().await {
        let name: String = part
            .file_name()
            .unwrap_or("")
            .chars()
            .filter(|c| !c.is_control() && *c != '/' && *c != '\\')
            .take(80)
            .collect();
        match part.bytes().await {
            Ok(b) if !name.is_empty() && b.len() <= UPLOAD_MAX => {
                files.push((name, b.to_vec()));
                kept += 1;
            }
            Ok(b) if !name.is_empty() || !b.is_empty() => refused += 1,
            _ => {}
        }
    }
    let over = files.len().saturating_sub(3);
    files.drain(..over);
    keep_uploads(&who.id, files);
    let msg = match (kept, refused) {
        (0, 0) => "Choose a file first.".to_string(),
        (k, 0) => format!("Uploaded {k} file(s)."),
        (k, r) => format!("Uploaded {k}; {r} over 200 KB refused."),
    };
    ui.redirect("/upload").flash(&msg).save(&who)
}

#[derive(Deserialize)]
struct Removed {
    file: String,
}

async fn upload_remove(ui: Ui, Saved(who): Saved<Uploader>, Form(r): Form<Removed>) -> Redirect {
    let mut files = uploads(&who.id);
    files.retain(|(n, _)| *n != r.file);
    keep_uploads(&who.id, files);
    ui.redirect("/upload")
        .flash(&format!("Removed {}.", r.file))
}

async fn upload_file(
    Saved(who): Saved<Uploader>,
    axum::extract::Path(n): axum::extract::Path<usize>,
) -> Response {
    let Some((name, bytes)) = uploads(&who.id).into_iter().nth(n) else {
        return axum::http::StatusCode::NOT_FOUND.into_response();
    };
    match upload_type(&name) {
        Some(t) => ([("content-type", t.to_string())], bytes).into_response(),
        None => (
            [
                ("content-type", "application/octet-stream".to_string()),
                (
                    "content-disposition",
                    format!("attachment; filename=\"{}\"", name.replace('"', "")),
                ),
            ],
            bytes,
        )
            .into_response(),
    }
}

/// Where each card of the demo board is, remembered per visitor: `(card, column)`.
#[derive(Deserialize, Serialize)]
struct Board(Vec<(String, String)>);

const CARDS: [(&str, &str, &str); 6] = [
    ("docs", "Write the component guide", "M24"),
    ("calendar", "Calendar and date picker", "M23"),
    ("upload", "Upload with progress", "M23"),
    ("kanban", "This board", "M23"),
    ("buttons", "Button primitive", "M21"),
    ("tokens", "shadcn tokens", "M20"),
];
const LANES: [(&str, &str); 3] = [("todo", "To do"), ("doing", "Doing"), ("done", "Done")];

impl Default for Board {
    fn default() -> Self {
        let start = ["todo", "doing", "doing", "doing", "done", "done"];
        Board(
            CARDS
                .iter()
                .zip(start)
                .map(|(c, l)| (c.0.to_string(), l.to_string()))
                .collect(),
        )
    }
}

async fn kanban_page(ui: Ui, Saved(board): Saved<Board>) -> Page {
    // code: /kanban
    let k = lui! {
        Kanban("/kanban") {
            @for (lane, title) in LANES {
                column (lane) (title) limit=[(lane == "doing").then_some(2)] {
                    @for (key, _) in board.0.iter().filter(|(_, l)| l == lane) {
                        @if let Some((key, text, note)) = CARDS.iter().find(|c| c.0 == key) {
                            card (key) (text) note=(note);
                        }
                    }
                }
            }
        }
    };
    // end code
    page(
        &ui,
        "Kanban",
        html! { (ui.flash()) (k) p class="lui-note" { "Each arrow posts the card and its new column; the server moves it and redirects back. Doing has a limit of two: past it, its count turns red." } },
    )
}

#[derive(Deserialize)]
struct Move {
    card: String,
    to: String,
}

/// A move: the card goes last in its new column (known cards and columns only), then PRG.
async fn kanban_move(ui: Ui, Saved(mut board): Saved<Board>, Form(m): Form<Move>) -> Redirect {
    let known = LANES.iter().any(|l| l.0 == m.to) && board.0.iter().any(|(c, _)| *c == m.card);
    if known {
        board.0.retain(|(c, _)| *c != m.card);
        board.0.push((m.card, m.to));
    }
    ui.redirect("/kanban").save(&board)
}
