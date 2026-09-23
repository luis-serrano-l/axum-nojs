//! Demo server: one route per component. Handlers only parse input and call `webonsive`.
//! The binary in `main.rs` serves [`router`]; tests and `webonsive-test` call it directly.

use axum::{
    Form, Router,
    extract::Query,
    http::HeaderMap,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use maud::{Markup, html};
use serde::Deserialize;
use webonsive::{
    Cap, Caps, Field, FieldKind, Streamed, Theme, UiState, accordion, caps, color, combobox,
    counter, dialog, flash, form, layout, paged_table, pager, popover_menu, prg, range, select, slot,
    table::{Column, sort_from_query}, tabs, theme_toggle,
};
use std::time::Duration;

/// Every demo path the no-script test and the screenshot test visit.
pub const PATHS: [&str; 14] = [
    "/", "/caps", "/stream", "/settings", "/dialog?dialog=confirm", "/popover", "/tabs?tab.demo=1",
    "/accordion?open.faq=1", "/combobox?q=r", "/list?page=2", "/form", "/counter", "/inputs",
    "/table?sort=size&dir=desc&q=a&per=5&page=2",
];

/// The whole demo app.
pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/caps", get(caps_page))
        .route("/dialog", get(dialog_page))
        .route("/popover", get(popover_page))
        .route("/tabs", get(tabs_page))
        .route("/accordion", get(accordion_page))
        .route("/combobox", get(combobox_page))
        .route("/list", get(list_page))
        .route("/table", get(table_page))
        .route("/form", get(form_page).post(form_submit))
        .route("/counter", get(counter_page).post(counter_submit))
        .route("/stream", get(stream_page))
        .route("/settings", get(settings_page).post(settings_submit))
        .route("/inputs", get(inputs_page).post(inputs_submit))
        .route("/theme", post(theme_submit))
        .merge(caps::router())
        .merge(webonsive::enhance::router())
}

// ---------- helpers ----------

fn theme_of(jar: &CookieJar) -> Theme {
    jar.get("theme").map(|c| Theme::parse(c.value())).unwrap_or_default()
}

/// Every component in the index: path, title (what each route passes to `page`), group, and
/// the platform features it is built on.
const COMPONENTS: [(&str, &str, &str, &str); 13] = [
    ("/dialog", "Dialog", "Overlays", "<dialog>, invoker commands"),
    ("/popover", "Popover menu", "Overlays", "popover, anchor positioning"),
    ("/tabs", "Tabs", "Disclosure", "<details name>, ::details-content"),
    ("/accordion", "Accordion", "Disclosure", "<details name>"),
    ("/combobox", "Combobox", "Input", "<datalist>, <search>"),
    ("/form", "Validated form", "Input", ":user-invalid, PRG"),
    ("/inputs", "Select, range, colour", "Input", "<selectedcontent>, type=range, type=color"),
    ("/counter", "Counter", "Server state", "form POST + cookie"),
    ("/settings", "Settings", "Server state", "UiState, PRG + flash"),
    ("/list", "Load-more list", "Server state", "links + view transitions"),
    ("/table", "Table", "Server state", "sort links, <search> filter, sticky header, ?page=n"),
    ("/caps", "Capabilities", "Server state", "@supports beacons + cookie"),
    ("/stream", "Streaming", "Server state", "declarative shadow DOM slots"),
];
const GROUPS: [&str; 4] = ["Overlays", "Disclosure", "Input", "Server state"];

/// The row above every title: the way back to the index (not on the index) and the theme switch.
fn toolbar(caps: &Caps, theme: Theme, back: bool) -> Markup {
    html! { nav class="wo-toolbar" {
        @if back { a class="wo-back" href="/" { "All components" } } @else { span {} }
        (theme_toggle(caps, "/theme", theme))
    } }
}

/// Every page: the theme from its cookie, the toolbar, the title, what it is built on, the body.
fn page(caps: &Caps, jar: &CookieJar, title: &str, body: Markup) -> Markup {
    let theme = theme_of(jar);
    let built = COMPONENTS.iter().find(|c| c.1 == title).map(|c| c.3);
    layout(caps, title, theme, html! {
        (toolbar(caps, theme, built.is_some()))
        h1 { (title) }
        @if let Some(feats) = built { p class="wo-built" { "Built on " @for f in feats.split(", ") { code { (f) } " " } } }
        (body)
    })
}

// ---------- routes ----------

async fn index(caps: Caps, jar: CookieJar) -> Markup {
    page(&caps, &jar, "Components", html! {
        p class="wo-lede" { "Twelve interactive components for Axum and Maud. Every page here ships zero " code { "<script>" } " tags: the HTML platform does the work, and one optional script only makes the same markup swap in place." }
        @if !caps.has(Cap::Probed) { p class="wo-note" { "First visit: this page is the fallback variant. Reload and the server will know your browser." } }
        div class="wo-index" { @for group in GROUPS {
            h2 { (group) }
            ul { @for (href, title, _, feats) in COMPONENTS.iter().filter(|c| c.2 == group) {
                li { a href=(href) { (title) } span { @for f in feats.split(", ") { code { (f) } " " } } }
            } }
        } }
    })
}

async fn dialog_page(caps: Caps, jar: CookieJar, state: UiState) -> Markup {
    page(&caps, &jar, "Dialog", html! {
        (dialog(&caps, "confirm", "Delete account", html! {
            h2 { "Delete account?" }
            p { "This cannot be undone. The dialog is opened by an invoker button and closed by a " code { "method=dialog" } " form." }
        }, state.dialog() == Some("confirm")))
        p class="wo-note" { "Server-opened: " a href="/dialog?dialog=confirm" { "?dialog=confirm" } }
    })
}

async fn popover_page(caps: Caps, jar: CookieJar) -> Markup {
    page(&caps, &jar, "Popover menu", html! {
        (popover_menu(&caps, "account", "Account", &[("Profile", "/"), ("Settings", "/"), ("Sign out", "/")]))
        p class="wo-note" { "Click outside or press Escape to close." }
    })
}

async fn tabs_page(caps: Caps, jar: CookieJar, state: UiState) -> (UiState, Markup) {
    let body = page(&caps, &jar, "Tabs", html! {
        (tabs(&caps, "demo", &[
            ("Install", html! { p { code { "cargo add webonsive maud axum" } } }),
            ("Use", html! { p { "Call a function, get " code { "Markup" } ", send it." } }),
            ("Why", html! { p { "Because the platform can do this without script now." } }),
        ], Some(&state)))
        p class="wo-note" { "Deep link: " a href="/tabs?tab.demo=2" { "?tab.demo=2" } ". Leave and come back: the tab is remembered." }
    });
    (state, body)
}

async fn accordion_page(caps: Caps, jar: CookieJar, state: UiState) -> (UiState, Markup) {
    let body = page(&caps, &jar, "Accordion", html! {
        (accordion(&caps, "faq", &[
            ("Is this really no JavaScript?", html! { p { "Yes. View source." } }),
            ("Does it animate?", html! { p { "Yes, via ::details-content transitions where supported." } }),
            ("Can several be open?", html! { p { "Pass an empty group name." } }),
        ], Some(&state)))
    });
    (state, body)
}

#[derive(Deserialize)]
struct SearchQuery { q: Option<String> }

const LANGS: [&str; 8] = ["Rust", "Ruby", "Racket", "Python", "Prolog", "Zig", "Swift", "Scala"];

async fn combobox_page(caps: Caps, jar: CookieJar, Query(s): Query<SearchQuery>) -> Markup {
    let q = s.q.unwrap_or_default();
    let hits: Vec<&str> = LANGS.iter().copied()
        .filter(|l| l.to_lowercase().contains(&q.to_lowercase())).collect();
    page(&caps, &jar, "Combobox", html! {
        // One swap root around the form and its results: the script searches as you type.
        div id="langs" data-wo="swap" {
            (combobox(&caps, "/combobox", "q", &LANGS, &q))
            ul { @for h in &hits { li { (h) } } }
            @if hits.is_empty() { p class="wo-note" { "No matches." } }
        }
    })
}

#[derive(Deserialize)]
struct PageQuery { page: Option<usize> }

async fn list_page(caps: Caps, jar: CookieJar, Query(p): Query<PageQuery>) -> Markup {
    const PER: usize = 8;
    const TOTAL: usize = 50;
    let current = p.page.unwrap_or(1).max(1);
    let rows: Vec<Markup> = (1..=(current * PER).min(TOTAL))
        .map(|n| html! { "Row " (n) })
        .collect();
    page(&caps, &jar, "Load-more list", html! { (pager(&caps, "/list", &rows, current, PER, TOTAL)) })
}

#[derive(Deserialize, Default)]
struct TableQuery { sort: Option<String>, dir: Option<String>, q: Option<String>, page: Option<usize>, per: Option<usize> }

/// Thirty-six files sorted, filtered and paged on the server; the table only renders and links.
async fn table_page(caps: Caps, jar: CookieJar, Query(t): Query<TableQuery>) -> Markup {
    const FILES: [(&str, u32, &str); 12] = [
        ("archive.tar", 40960, "backup"), ("build.rs", 1200, "script"), ("cargo.lock", 8800, "generated"),
        ("index.html", 2100, "page"), ("logo.svg", 3400, "image"), ("main.rs", 5600, "source"),
        ("notes.md", 900, "text"), ("photo.jpg", 250000, "image"), ("readme.md", 4100, "text"),
        ("style.css", 1500, "stylesheet"), ("tests.rs", 7700, "source"), ("video.mp4", 9800000, "video"),
    ];
    let cols = [Column::sortable("name", "Name"), Column::sortable("size", "Size"), Column::sortable("kind", "Kind")];
    let sort = sort_from_query(&cols, t.sort.as_deref(), t.dir.as_deref());
    let q = t.q.unwrap_or_default().to_lowercase();
    let (per, pg) = (t.per.unwrap_or(10).clamp(1, 50), t.page.unwrap_or(1).max(1));
    let mut files: Vec<(String, u32, &str)> = ["src", "docs", "old"].iter()
        .flat_map(|dir| FILES.iter().map(move |f| (format!("{dir}/{}", f.0), f.1 * (dir.len() as u32), f.2)))
        .filter(|f| q.is_empty() || f.0.contains(&q) || f.2.contains(&q)).collect();
    if let Some((key, desc)) = sort {
        files.sort_by(|a, b| match key { "size" => a.1.cmp(&b.1), "kind" => a.2.cmp(b.2), _ => a.0.cmp(&b.0) });
        if desc { files.reverse(); }
    }
    let total = files.len();
    let rows: Vec<Vec<Markup>> = files.iter().skip((pg - 1) * per).take(per)
        .map(|f| vec![html! { code { (f.0) } }, html! { (f.1 / 1024) " KB" }, html! { (f.2) }]).collect();
    page(&caps, &jar, "Table", html! {
        p { "Click a header to sort, again to flip. Type to filter. Page through. Every state is a URL." }
        (paged_table(&caps, "files", "/table", &cols, &rows, sort, &q, pg, per, total))
    })
}

#[derive(Deserialize, Default)]
struct SignUp { name: String, email: String, age: String, handle: String }

fn signup_fields<'a>(v: &'a SignUp, errors: &'a [(&'a str, &'a str)]) -> Vec<Field<'a>> {
    let err = |n: &str| errors.iter().find(|(f, _)| *f == n).map(|(_, m)| *m);
    vec![
        Field { name: "name", label: "Name", kind: FieldKind::Text, value: &v.name, error: err("name"), required: true },
        Field { name: "email", label: "Email", kind: FieldKind::Email, value: &v.email, error: err("email"), required: true },
        Field { name: "age", label: "Age", kind: FieldKind::Number { min: 13, max: 120 }, value: &v.age, error: err("age"), required: true },
        Field { name: "handle", label: "Handle", kind: FieldKind::Pattern { pattern: "[a-z0-9_]{3,16}", hint: "3–16 lowercase letters, digits or _" }, value: &v.handle, error: err("handle"), required: true },
    ]
}

async fn form_page(caps: Caps, jar: CookieJar) -> Markup {
    let v = SignUp::default();
    page(&caps, &jar, "Validated form", html! { (form(&caps, "/form", &signup_fields(&v, &[]), "Sign up")) })
}

async fn form_submit(caps: Caps, jar: CookieJar, Form(v): Form<SignUp>) -> axum::response::Response {
    let mut errors = Vec::new();
    if v.handle == "admin" { errors.push(("handle", "That handle is reserved.")); }
    if v.email.ends_with("@example.com") { errors.push(("email", "example.com addresses are not accepted.")); }
    if errors.is_empty() {
        return Redirect::to("/form?ok").into_response();
    }
    page(&caps, &jar, "Validated form", html! {
        p class="wo-error" { "Server-side checks failed. Browser validation passed, these rules only live on the server." }
        (form(&caps, "/form", &signup_fields(&v, &errors), "Sign up"))
    }).into_response()
}

async fn counter_page(caps: Caps, jar: CookieJar) -> Markup {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    page(&caps, &jar, "Counter", html! { (counter(&caps, "/counter", n)) })
}

#[derive(Deserialize)]
struct CounterOp { op: String }

async fn counter_submit(jar: CookieJar, Form(f): Form<CounterOp>) -> (CookieJar, Redirect) {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    let n = match f.op.as_str() { "inc" => n + 1, "dec" => n - 1, _ => 0 };
    (jar.add(Cookie::new("count", n.to_string())), Redirect::to("/counter"))
}

#[derive(Deserialize, Default)]
struct Settings { name: String, #[serde(default)] notify: bool }

fn settings_of(jar: &CookieJar) -> Settings {
    jar.get("settings").and_then(|c| c.value().split_once('|'))
        .map(|(name, notify)| Settings { name: name.to_string(), notify: notify == "1" })
        .unwrap_or_default()
}

/// Tabs + form + flash. Everything survives a full navigation: tab in the `wo-ui` cookie,
/// values in a `settings` cookie, flash in a one-shot cookie set by `prg`.
async fn settings_page(caps: Caps, jar: CookieJar, state: UiState) -> (UiState, Markup) {
    let current = settings_of(&jar);
    let hidden = html! { input type="hidden" name="tab" value=(state.tab("settings")); };
    let body = page(&caps, &jar, "Settings", html! {
        (flash(&caps, state.flash()))
        (tabs(&caps, "settings", &[
            ("Profile", html! { form class="wo-form" method="post" action="/settings" { (hidden)
                div class="wo-field" { label for="name" { "Display name" } input id="name" name="name" value=(current.name) required; }
                button type="submit" class="wo-primary" { "Save" } } }),
            ("Notifications", html! { form class="wo-form" method="post" action="/settings" { (hidden)
                input type="hidden" name="name" value=(current.name);
                label { input type="checkbox" name="notify" value="true" checked[current.notify]; " Email me about releases" }
                button type="submit" class="wo-primary" { "Save" } } }),
        ], Some(&state)))
        p class="wo-note" { "Go to " a href="/" { "the index" } " and come back: the open tab and the values are remembered." }
    });
    (state, body)
}

#[derive(Deserialize)]
struct SettingsForm { name: String, #[serde(default)] notify: bool, tab: usize }

async fn settings_submit(jar: CookieJar, Form(f): Form<SettingsForm>) -> (CookieJar, axum::response::Response) {
    let value = format!("{}|{}", f.name.replace('|', ""), u8::from(f.notify));
    let to = format!("/settings?tab.settings={}", f.tab);
    (jar.add(Cookie::new("settings", value)), prg(&to, Some("Settings saved.")))
}

#[derive(Deserialize)]
struct Inputs { size: String, volume: i64, accent: String }

const SIZES: [(&str, &str); 3] = [("s", "Small"), ("m", "Medium"), ("l", "Large")];

/// Select, range and colour in one form; the chosen values live in an `inputs` cookie.
async fn inputs_page(caps: Caps, jar: CookieJar, state: UiState) -> (UiState, Markup) {
    let saved = jar.get("inputs").map(|c| c.value().to_string()).unwrap_or_default();
    let mut parts = saved.split('|');
    let (size, volume, accent) = (parts.next().unwrap_or("m"), parts.next().unwrap_or("40"), parts.next().unwrap_or("#1f6f5f"));
    let options: Vec<(&str, Markup)> = SIZES.iter()
        .map(|(v, l)| (*v, html! { span class="wo-swatch" style={ "background: " (accent) } {} (l) }))
        .collect();
    let body = page(&caps, &jar, "Select, range, colour", html! {
        (flash(&caps, state.flash()))
        form id="inputs" data-wo="swap" class="wo-form" method="post" action="/inputs" {
            div class="wo-field" { label for="size" { "Size" } (select(&caps, "size", &options, size)) }
            div class="wo-field" { label for="f-volume" { "Volume" } (range(&caps, "volume", 0, 100, 5, volume.parse().unwrap_or(40))) }
            div class="wo-field" { label for="f-accent" { "Accent" } (color(&caps, "accent", accent)) }
            button type="submit" class="wo-primary" { "Save" }
        }
        p class="wo-note" { "Without the enhancement script the output and the swatch show the last saved values and update on submit." }
    });
    (state, body)
}

async fn inputs_submit(jar: CookieJar, Form(f): Form<Inputs>) -> (CookieJar, axum::response::Response) {
    let size = SIZES.iter().find(|(v, _)| *v == f.size).map(|(v, _)| *v).unwrap_or("m");
    let accent = if f.accent.len() == 7 && f.accent.starts_with('#') { f.accent.as_str() } else { "#1f6f5f" };
    let value = format!("{size}|{}|{accent}", f.volume.clamp(0, 100));
    (jar.add(Cookie::new("inputs", value)), prg("/inputs", Some("Inputs saved.")))
}

/// Three sections declared slowest first, so out-of-order arrival is visible.
async fn stream_page(caps: Caps, jar: CookieJar) -> Streamed {
    let sections = [("slow", 2000), ("medium", 800), ("fast", 100)];
    let theme = theme_of(&jar);
    let page = Streamed::page(&caps, "Streaming", theme, html! {
        (toolbar(&caps, theme, true))
        h1 { "Streaming" }
        p { @if caps.has(Cap::StreamingDsd) { "Sections arrive out of order into named slots." }
            @else { "This browser has no declarative shadow DOM: sections stream in document order." } }
        @for (id, ms) in sections {
            (slot(&caps, id, html! { section class="wo-stream-section wo-stream-pending" { "Loading " (id) " (" (ms) " ms)…" } }))
        }
    });
    sections.into_iter().fold(page, |page, (id, ms)| page.fill(id, section(id, ms)))
}

async fn section(id: &'static str, ms: u64) -> Markup {
    tokio::time::sleep(Duration::from_millis(ms)).await;
    html! { section class="wo-stream-section" { strong { (id) } " arrived after " (ms) " ms." } }
}

/// What the server believes about this browser, one row per capability.
async fn caps_page(caps: Caps, jar: CookieJar) -> Markup {
    let probed = caps.has(Cap::Probed);
    page(&caps, &jar, "Capabilities", html! {
        @if probed { p { "Beacons have fired. Rows below drive which markup every component emits." } }
        @else { p class="wo-error" { "Not probed yet: the beacons fire while this page loads. Reload to see the result." } }
        table class="wo-caps-table" {
            thead { tr { th { "Capability" } th { "Supported" } th { "Effect" } th { "@supports test" } } }
            tbody { @for cap in Cap::ALL {
                tr {
                    td { code { (cap.name()) } }
                    td { @if caps.has(cap) { span class="wo-yes" { "yes" } } @else if probed { span class="wo-no" { "no" } } @else { span class="wo-note" { "unknown" } } }
                    td { (cap.description()) }
                    td { @match cap.supports() { Some(t) => code { (t) }, None => span class="wo-note" { "always" } } }
                }
            } }
        }
        p class="wo-note" { "Cookies: " @for n in caps.names() { code { "wo-cap-" (n) } " " } }
        p class="wo-note" { "To view any page as another browser, add " code { "?caps=popover,anchor" } " to its URL: the query wins over the cookies." }
    })
}

#[derive(Deserialize)]
struct ThemeForm { theme: String }

async fn theme_submit(jar: CookieJar, headers: HeaderMap, Form(f): Form<ThemeForm>) -> (CookieJar, Redirect) {
    let theme = Theme::parse(&f.theme);
    (jar.add(Cookie::new("theme", theme.as_str().to_string())), Redirect::to(&back_to(&headers)))
}

/// The path of the page a form was posted from (same-origin `Referer`), or `/`.
fn back_to(headers: &HeaderMap) -> String {
    headers
        .get("referer")
        .and_then(|v| v.to_str().ok())
        .and_then(|url| url.splitn(4, '/').nth(3))
        .map(|path| format!("/{path}"))
        .filter(|path| !path.starts_with("//"))
        .unwrap_or_else(|| "/".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    /// The only script on any page is the one optional enhancement tag: no inline script,
    /// no handlers, no `javascript:` URLs. Blitz (no script engine) proves the pages work
    /// without it.
    #[tokio::test]
    async fn pages_ship_only_the_enhancement_script() {
        let tag = webonsive::enhance::script_tag().into_string();
        for path in PATHS {
            let modern = Cap::ALL.map(|c| format!("wo-cap-{}=1", c.name())).join("; ");
            for cookie in ["", modern.as_str()] {
                let req = Request::get(path).header("cookie", cookie).body(Body::empty()).unwrap();
                let res = router().oneshot(req).await.unwrap();
                assert_eq!(res.status(), 200, "{path}");
                let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
                let html = String::from_utf8(body.to_vec()).unwrap();
                assert_eq!(html.matches("<script").count(), 1, "{path}: exactly one script tag");
                assert!(html.contains(&tag), "{path}: the tag is the enhancement script");
                assert!(!html.contains("javascript:"), "{path}");
                let inline_handler = html.split('<').any(|tag| tag.split_whitespace().any(|a| a.starts_with("on") && a.contains('=')));
                assert!(!inline_handler, "{path}: inline event handler");
            }
        }
    }

    #[tokio::test]
    async fn enhancement_script_is_served_immutable() {
        let req = Request::get(webonsive::enhance::script_url()).body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-type"], "text/javascript; charset=utf-8");
        assert!(res.headers()["cache-control"].to_str().unwrap().contains("immutable"));
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, webonsive::enhance::JS.as_bytes());
    }

    #[tokio::test]
    async fn caps_beacon_sets_one_cookie_per_flag() {
        let req = Request::get("/wo/caps?flag=popover").body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 204);
        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(cookie.starts_with("wo-cap-popover=1;"), "{cookie}");
        let req = Request::get("/wo/caps?flag=nope").body(Body::empty()).unwrap();
        assert_eq!(router().oneshot(req).await.unwrap().status(), 404);
    }

    /// Collect the body frames of `path` as they arrive.
    async fn frames(path: &str, cookie: &str) -> Vec<String> {
        use http_body_util::BodyExt;
        let req = Request::get(path).header("cookie", cookie).body(Body::empty()).unwrap();
        let mut body = router().oneshot(req).await.unwrap().into_body();
        let mut out = Vec::new();
        while let Some(frame) = body.frame().await {
            let frame = frame.unwrap();
            if let Some(data) = frame.data_ref() {
                out.push(String::from_utf8(data.to_vec()).unwrap());
            }
        }
        out
    }

    #[tokio::test]
    async fn stream_is_chunked_in_completion_order() {
        let chunks = frames("/stream", "wo-cap-probed=1; wo-cap-streaming_dsd=1").await;
        assert!(chunks.len() >= 5, "expected prefix + 3 fills + suffix, got {}", chunks.len());
        assert!(chunks[0].contains("<template shadowrootmode=\"open\">"));
        assert!(chunks[0].contains("<slot name=\"slow\">"));
        let order: Vec<&str> = chunks[1..4].iter().map(|c| c.split("slot=\"").nth(1).unwrap().split('"').next().unwrap()).collect();
        assert_eq!(order, ["fast", "medium", "slow"]);
        assert!(chunks.last().unwrap().ends_with("</script></body></html>"), "suffix carries the enhancement tag");
    }

    #[tokio::test]
    async fn stream_fallback_is_in_document_order() {
        let chunks = frames("/stream", "wo-cap-probed=1").await;
        let html = chunks.concat();
        assert!(!html.contains("<template") && !html.contains("<slot"));
        let pos = |s: &str| html.find(s).unwrap();
        assert!(pos("slow</strong>") < pos("medium</strong>") && pos("medium</strong>") < pos("fast</strong>"));
        assert!(chunks.len() >= 4, "streamed in pieces, got {}", chunks.len());
    }

    #[tokio::test]
    async fn state_round_trip_through_prg_and_cookies() {
        // POST → 303 with a flash cookie and the tab in the redirect.
        let req = Request::post("/settings").header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("name=Ada&notify=true&tab=1")).unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 303);
        assert_eq!(res.headers().get("location").unwrap(), "/settings?tab.settings=1");
        let cookies: Vec<String> = res.headers().get_all("set-cookie").iter().map(|v| v.to_str().unwrap().to_string()).collect();
        assert!(cookies.iter().any(|c| c.starts_with("wo-flash=Settings%20saved.")), "{cookies:?}");
        assert!(cookies.iter().any(|c| c.starts_with("settings=Ada")), "{cookies:?}");
        // GET the redirect target: flash shown and cleared, tab persisted to wo-ui, values filled in.
        let req = Request::get("/settings?tab.settings=1").header("cookie", "wo-flash=Settings%20saved.; settings=Ada|1").body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        let cookies: Vec<String> = res.headers().get_all("set-cookie").iter().map(|v| v.to_str().unwrap().to_string()).collect();
        assert!(cookies.iter().any(|c| c.starts_with("wo-ui=tab.settings=1;")), "{cookies:?}");
        assert!(cookies.iter().any(|c| c.starts_with("wo-flash=; Path=/; Max-Age=0")), "{cookies:?}");
        let html = String::from_utf8(axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
        assert!(html.contains("Settings saved.") && html.contains("value=\"Ada\"") && html.contains("checked"));
        assert!(html.contains("<details name=\"settings\" open>") && html.contains("href=\"/settings?tab.settings=0\""));
        // Coming back with only the cookie: the tab is still open, nothing is rewritten.
        let req = Request::get("/settings").header("cookie", "wo-ui=tab.settings=1").body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert!(res.headers().get("set-cookie").is_none());
        let html = String::from_utf8(axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
        assert!(html.contains("<details name=\"settings\" open><summary><a href=\"/settings?tab.settings=1\">Notifications"));
    }

    #[tokio::test]
    async fn markup_follows_caps() {
        async fn body(path: &str, cookie: &str) -> String {
            let req = Request::get(path).header("cookie", cookie).body(Body::empty()).unwrap();
            let res = router().oneshot(req).await.unwrap();
            let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
            String::from_utf8(body.to_vec()).unwrap()
        }
        let old = body("/dialog", "").await;
        assert!(old.contains("href=\"#confirm\"") && !old.contains("commandfor"));
        assert!(old.contains("wo-caps"), "unknown browser gets beacons");
        let new = body("/dialog", "wo-cap-probed=1; wo-cap-invokers=1").await;
        assert!(new.contains("commandfor=\"confirm\"") && !new.contains("href=\"#confirm\""));
        assert!(!new.contains("class=\"wo-caps\""), "probed browser gets no beacons");
        assert!(body("/popover", "").await.contains("wo-popover-details"));
        assert!(body("/tabs", "wo-cap-details_content=1").await.contains("wo-tabs-panel"));
        assert!(body("/tabs", "").await.contains("wo-accordion-body"));
    }
}
