//! Demo server: one route per component. Handlers only parse input and call `webonsive`.

use axum::{
    Form, Router,
    extract::Query,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use maud::{Markup, html};
use serde::Deserialize;
use webonsive::{
    Cap, Caps, Field, FieldKind, Streamed, Theme, UiState, accordion, caps, combobox, counter,
    dialog, flash, form, layout, pager, popover_menu, prg, slot, tabs, theme_toggle,
};
use std::time::Duration;

#[tokio::main]
async fn main() {
    let app = router();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("http://127.0.0.1:3000");
    axum::serve(listener, app).await.unwrap();
}

fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/caps", get(caps_page))
        .route("/dialog", get(dialog_page))
        .route("/popover", get(popover_page))
        .route("/tabs", get(tabs_page))
        .route("/accordion", get(accordion_page))
        .route("/combobox", get(combobox_page))
        .route("/list", get(list_page))
        .route("/form", get(form_page).post(form_submit))
        .route("/counter", get(counter_page).post(counter_submit))
        .route("/stream", get(stream_page))
        .route("/settings", get(settings_page).post(settings_submit))
        .route("/theme", post(theme_submit))
        .merge(caps::router())
}

// ---------- helpers ----------

fn theme_of(jar: &CookieJar) -> Theme {
    jar.get("theme").map(|c| Theme::parse(c.value())).unwrap_or_default()
}

/// Every page: the theme from its cookie, the body, the theme toggle, and the caps beacons.
fn page(caps: &Caps, jar: &CookieJar, title: &str, body: Markup) -> Markup {
    let theme = theme_of(jar);
    layout(caps, title, theme, html! {
        h1 { (title) }
        (body)
        h2 { "Theme" }
        (theme_toggle(caps, "/theme", theme))
    })
}

// ---------- routes ----------

async fn index(caps: Caps, jar: CookieJar) -> Markup {
    let rows = [
        ("/dialog", "Dialog", "<dialog>, invoker commands"),
        ("/popover", "Popover menu", "popover, anchor positioning"),
        ("/tabs", "Tabs", "<details name>, ::details-content"),
        ("/accordion", "Accordion", "<details name>"),
        ("/combobox", "Combobox", "<datalist>, <search>"),
        ("/list", "Load-more list", "links + view transitions"),
        ("/form", "Validated form", ":user-invalid, PRG"),
        ("/counter", "Counter", "form POST + cookie"),
        ("/caps", "Capabilities", "@supports beacons + cookie"),
        ("/stream", "Streaming", "declarative shadow DOM slots"),
        ("/settings", "Settings", "UiState, PRG + flash"),
    ];
    page(&caps, &jar, "Components", html! {
        p { "Every page here ships zero " code { "<script>" } " tags." }
        @if !caps.has(Cap::Probed) { p class="wo-note" { "First visit: this page is the fallback variant. Reload and the server will know your browser." } }
        table {
            thead { tr { th { "Component" } th { "Platform features" } } }
            tbody { @for (href, name, feats) in rows {
                tr { td { a href=(href) { (name) } } td { (feats) } }
            } }
        }
    })
}

async fn dialog_page(caps: Caps, jar: CookieJar, state: UiState) -> Markup {
    page(&caps, &jar, "Dialog", html! {
        (dialog(&caps, "confirm", "Delete account", html! {
            h2 style="margin-top:0" { "Delete account?" }
            p { "This cannot be undone. The dialog is opened by an invoker button and closed by a " code { "method=dialog" } " form." }
        }, state.dialog() == Some("confirm")))
        p class="wo-note" { "Server-opened: " a href="/dialog?dialog=confirm" { "?dialog=confirm" } }
    })
}

async fn popover_page(caps: Caps, jar: CookieJar) -> Markup {
    page(&caps, &jar, "Popover menu", html! {
        (popover_menu(&caps, "account", "Account", &[("Profile", "/"), ("Settings", "/"), ("Sign out", "/")]))
        p class="wo-note" style="margin-top:1rem" { "Click outside or press Escape to close." }
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
        (combobox(&caps, "/combobox", "q", &LANGS, &q))
        ul { @for h in &hits { li { (h) } } }
        @if hits.is_empty() { p class="wo-note" { "No matches." } }
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

/// Three sections declared slowest first, so out-of-order arrival is visible.
async fn stream_page(caps: Caps, jar: CookieJar) -> Streamed {
    let sections = [("slow", 2000), ("medium", 800), ("fast", 100)];
    let page = Streamed::page(&caps, "Streaming", theme_of(&jar), html! {
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
    })
}

#[derive(Deserialize)]
struct ThemeForm { theme: String }

async fn theme_submit(jar: CookieJar, Form(f): Form<ThemeForm>) -> (CookieJar, Redirect) {
    let theme = Theme::parse(&f.theme);
    (jar.add(Cookie::new("theme", theme.as_str().to_string())), Redirect::to("/"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[tokio::test]
    async fn no_page_ships_script() {
        for path in ["/", "/caps", "/stream", "/settings", "/dialog?dialog=confirm", "/popover", "/tabs?tab=1", "/accordion", "/combobox?q=r", "/list?page=2", "/form", "/counter"] {
            let modern = Cap::ALL.map(|c| format!("wo-cap-{}=1", c.name())).join("; ");
            for cookie in ["", modern.as_str()] {
                let req = Request::get(path).header("cookie", cookie).body(Body::empty()).unwrap();
                let res = router().oneshot(req).await.unwrap();
                assert_eq!(res.status(), 200, "{path}");
                let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
                let html = String::from_utf8(body.to_vec()).unwrap();
                assert!(!html.contains("<script"), "{path} contains a script tag");
            }
        }
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
        assert_eq!(chunks.last().unwrap(), "</body></html>");
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
