//! Server state: counter, settings, load-more list, capabilities, streaming and swap targets.

use crate::site::{page, shell};
use axum::{
    Form, Router,
    http::HeaderMap,
    response::{IntoResponse, Response},
    routing::get,
};
use axum_nojs::Streamed;
use axum_nojs::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/counter", get(counter_page).post(counter_submit))
        .route("/settings", get(settings_page).post(settings_submit))
        .route("/list", get(list_page))
        .route("/caps", get(caps_page))
        .route("/stream", get(stream_page))
        .route("/swap", get(swap_page).post(swap_submit))
}

async fn list_page(ui: Ui) -> Page {
    page(
        &ui,
        "Load-more list",
        html! {
            // code: /list
            (ui.pager("/list", 50).per_page(8).rows(|i| html! { "Row " (i + 1) }))
            // end code
        },
    )
}

#[derive(Default, Deserialize, Serialize)]
struct Count {
    n: i64,
}

/// The counter's rules, shared by the page (to render them) and the post (to apply them).
fn counter(ui: &Ui, n: i64) -> axum_nojs::counter::Counter<'static> {
    // code: /counter
    ui.counter("/counter", n).min(0).max(20).step(2).typed()
    // end code
}

async fn counter_page(ui: Ui, Saved(c): Saved<Count>) -> Page {
    page(
        &ui,
        "Counter",
        html! {
            p { "Steps of two between 0 and 20. The buttons switch off at the ends; a typed value off the step or the bounds is refused by the browser and clamped by the server." }
            (counter(&ui, c.n))
        },
    )
}

#[derive(Deserialize)]
struct CounterOp {
    op: String,
    value: Option<i64>,
}

async fn counter_submit(ui: Ui, Saved(c): Saved<Count>, Form(f): Form<CounterOp>) -> Redirect {
    // code: /counter
    let n = counter(&ui, c.n).apply(&f.op, f.value);
    ui.redirect("/counter").save(&Count { n })
    // end code
}

/// The notes added on the swap page, one `note=` pair each.
#[derive(Default, Deserialize, Serialize)]
struct Notes(Vec<(String, String)>);

/// Two controls outside any swap root that name their target: the link swaps one `<span>`,
/// the form appends to a list. The same requests are plain navigations without the script.
async fn swap_page(ui: Ui, Saved(notes): Saved<Notes>) -> Page {
    let n: u32 = ui.param("n").and_then(|n| n.parse().ok()).unwrap_or(1);
    page(
        &ui,
        "Swap targets",
        html! {
            (ui.flash())
            p class="nojs-note" { "Neither control sits inside a swap root. " code { "data-nojs-target" } " names the root to update and " code { "data-nojs-swap" } " how; without the script both are ordinary navigations to the same URL." }
            // code: /swap
            p { "Count: " span id="count" data-nojs="swap" { (n) } " " a href={ "/swap?n=" (n + 1) } data-nojs-target="#count" { "Add one" }
                " · " a href={ "/swap?n=" (n + 10) } data-nojs-target="#count" data-nojs-push="false" { "Add ten, keep the URL" } }
            // end code
            p { "Notes so far: " span id="note-count" { (notes.0.len()) } }
            // code: /swap
            form method="post" action="/swap" data-nojs-target="#log" data-nojs-swap="append" data-nojs-indicator="#saving" {
                (ui.input("note", "Note").hide_label().required().placeholder("A note").autocomplete("off"))
                (ui.button("Add note").primary())
                " " span id="saving" class="nojs-note" hidden { "Saving…" }
            }
            ol id="log" data-nojs="swap" { @for (_, note) in &notes.0 { li { (note) } } }
            // end code
        },
    )
}

#[derive(Deserialize)]
struct SwapForm {
    note: String,
}

/// An enhanced request (`Nojs-Enhance: 1`) gets only the new `<li>` inside an `#log` to append,
/// plus the note count marked `data-nojs-oob` so it updates wherever it is on the page; a plain
/// one gets Post/Redirect/Get to the full page, which shows both anyway.
async fn swap_submit(
    ui: Ui,
    headers: HeaderMap,
    Saved(mut notes): Saved<Notes>,
    Form(f): Form<SwapForm>,
) -> Response {
    notes.0.push(("note".into(), f.note.clone()));
    if headers.contains_key("nojs-enhance") {
        let cookies = ui.redirect("/swap").save(&notes).set_cookies();
        let set = cookies
            .into_iter()
            .map(|c| (axum::http::header::SET_COOKIE, c));
        return (axum::response::AppendHeaders(set), html! { ol id="log" { li { (f.note) } } span id="note-count" data-nojs-oob { (notes.0.len()) } }).into_response();
    }
    ui.redirect("/swap")
        .flash("Note added")
        .save(&notes)
        .into_response()
}

#[derive(Default, Deserialize, Serialize)]
struct Settings {
    name: String,
    #[serde(default)]
    notify: bool,
}

/// Tabs + form + flash. Everything survives a full navigation: the tab in the `nojs-ui`
/// cookie, the values in `nojs-settings`, the flash in a one-shot cookie.
async fn settings_page(ui: Ui, Saved(s): Saved<Settings>) -> Page {
    let form = |id| ui.form("/settings").id(id).submit("Save");
    page(
        &ui,
        "Settings",
        html! {
            // code: /settings
            (ui.flash().dismiss().auto_hide())
            (ui.tabs("settings")
                .tab("Profile", form("profile").text("name", "Display name").required().value(&s.name)
                    .hidden("notify", if s.notify { "true" } else { "false" }).render())
                .tab("Notifications", form("notify").hidden("name", &s.name)
                    .checkbox("notify", "Email me about releases").checked(s.notify).render()))
            // end code
            p class="nojs-note" { "Go to " a href="/" { "the index" } " and come back: the open tab and the values are remembered. Saving with notifications off stacks a warning under the confirmation; the name " code { "admin" } " is refused with an alert. The confirmation fades after six seconds unless reduced motion is on." }
        },
    )
}

/// Saves and says so; a warning stacks when notifications go off; `admin` is refused.
async fn settings_submit(ui: Ui, Form(s): Form<Settings>) -> Redirect {
    let back = ui.redirect("/settings");
    if s.name.trim().eq_ignore_ascii_case("admin") {
        return back.danger("The name admin is reserved; nothing was saved.");
    }
    // code: /settings
    let back = back.ok("Settings saved.");
    let back = if s.notify {
        back
    } else {
        back.warn("You will not hear about releases.")
    };
    back.save(&s)
    // end code
}

/// Three sections declared slowest first, so out-of-order arrival is visible.
async fn stream_page(ui: Ui) -> Streamed {
    let sections = [("slow", 2000), ("medium", 800), ("fast", 100)];
    let body = html! {
        p { @if ui.has(Cap::StreamingDsd) { "Sections arrive out of order into named slots." }
            @else { "This browser has no declarative shadow DOM: sections stream in document order." } }
        @for (id, ms) in sections {
            // code: /stream
            (ui.slot(id, html! { section class="nojs-stream-section nojs-stream-pending" {
                (ui.skeleton(2).label(&format!("Loading {id} ({ms} ms)")).heading())
            } }))
            // end code
        }
    };
    // code: /stream
    let page = ui.stream("Streaming", shell(&ui, "Streaming", body));
    sections
        .into_iter()
        .fold(page, |page, (id, ms)| page.fill(id, section(id, ms)))
    // end code
}

async fn section(id: &'static str, ms: u64) -> Markup {
    tokio::time::sleep(Duration::from_millis(ms)).await;
    html! { section class="nojs-stream-section" { strong { (id) } " arrived after " (ms) " ms." } }
}

/// What the server believes about this browser, one row per capability.
async fn caps_page(ui: Ui) -> Page {
    let probed = ui.has(Cap::Probed);
    page(
        &ui,
        "Capabilities",
        html! {
            @if probed { p { "Beacons have fired. Rows below drive which markup every component emits." } }
            @else { p class="nojs-error" { "Not probed yet: the beacons fire while this page loads. Reload to see the result." } }
            table class="nojs-caps-table" {
                thead { tr { th { "Capability" } th { "Supported" } th { "Effect" } th { "@supports test" } } }
                // code: /caps
                tbody { @for cap in Cap::ALL {
                    tr {
                        td { code { (cap.name()) } }
                        td { @if ui.has(cap) { span class="nojs-yes" { "yes" } } @else if probed { span class="nojs-no" { "no" } } @else { span class="nojs-note" { "unknown" } } }
                        td { (cap.description()) }
                        td { @match cap.supports() { Some(t) => code { (t) }, None => span class="nojs-note" { "always" } } }
                    }
                } }
                // end code
            }
            p class="nojs-note" { "Cookies: " @for n in ui.names() { code { "nojs-cap-" (n) } " " } }
            p class="nojs-note" { "To view any page as another browser, add " code { "?caps=popover,anchor" } " to its URL: the query wins over the cookies." }
        },
    )
}
