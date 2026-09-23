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
    Field, FieldKind, Theme, accordion, combobox, counter, dialog, form, layout, pager,
    popover_menu, tabs, theme_toggle,
};

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
        .route("/dialog", get(dialog_page))
        .route("/popover", get(popover_page))
        .route("/tabs", get(tabs_page))
        .route("/accordion", get(accordion_page))
        .route("/combobox", get(combobox_page))
        .route("/list", get(list_page))
        .route("/form", get(form_page).post(form_submit))
        .route("/counter", get(counter_page).post(counter_submit))
        .route("/theme", post(theme_submit))
}

// ---------- helpers ----------

fn theme_of(jar: &CookieJar) -> Theme {
    jar.get("theme").map(|c| Theme::parse(c.value())).unwrap_or_default()
}

fn page(jar: &CookieJar, title: &str, body: Markup) -> Markup {
    let theme = theme_of(jar);
    layout(title, theme, html! {
        h1 { (title) }
        (body)
        h2 { "Theme" }
        (theme_toggle("/theme", theme))
    })
}

// ---------- routes ----------

async fn index(jar: CookieJar) -> Markup {
    let rows = [
        ("/dialog", "Dialog", "<dialog>, invoker commands"),
        ("/popover", "Popover menu", "popover, anchor positioning"),
        ("/tabs", "Tabs", "<details name>, ::details-content"),
        ("/accordion", "Accordion", "<details name>"),
        ("/combobox", "Combobox", "<datalist>, <search>"),
        ("/list", "Load-more list", "links + view transitions"),
        ("/form", "Validated form", ":user-invalid, PRG"),
        ("/counter", "Counter", "form POST + cookie"),
    ];
    page(&jar, "Components", html! {
        p { "Every page here ships zero " code { "<script>" } " tags." }
        table {
            thead { tr { th { "Component" } th { "Platform features" } } }
            tbody { @for (href, name, feats) in rows {
                tr { td { a href=(href) { (name) } } td { (feats) } }
            } }
        }
    })
}

async fn dialog_page(jar: CookieJar) -> Markup {
    page(&jar, "Dialog", html! {
        (dialog("confirm", "Delete account", html! {
            h2 style="margin-top:0" { "Delete account?" }
            p { "This cannot be undone. The dialog is opened by an invoker button and closed by a " code { "method=dialog" } " form." }
        }))
    })
}

async fn popover_page(jar: CookieJar) -> Markup {
    page(&jar, "Popover menu", html! {
        (popover_menu("account", "Account", &[("Profile", "/"), ("Settings", "/"), ("Sign out", "/")]))
        p class="wo-note" style="margin-top:1rem" { "Click outside or press Escape to close." }
    })
}

#[derive(Deserialize)]
struct TabQuery { tab: Option<usize> }

async fn tabs_page(jar: CookieJar, Query(q): Query<TabQuery>) -> Markup {
    let active = q.tab.unwrap_or(0);
    page(&jar, "Tabs", html! {
        (tabs("demo", active, &[
            ("Install", html! { p { code { "cargo add webonsive maud axum" } } }),
            ("Use", html! { p { "Call a function, get " code { "Markup" } ", send it." } }),
            ("Why", html! { p { "Because the platform can do this without script now." } }),
        ]))
        p class="wo-note" { "Deep link: " a href="/tabs?tab=2" { "?tab=2" } }
    })
}

async fn accordion_page(jar: CookieJar) -> Markup {
    page(&jar, "Accordion", html! {
        (accordion("faq", &[
            ("Is this really no JavaScript?", html! { p { "Yes. View source." } }),
            ("Does it animate?", html! { p { "Yes, via ::details-content transitions where supported." } }),
            ("Can several be open?", html! { p { "Pass an empty group name." } }),
        ]))
    })
}

#[derive(Deserialize)]
struct SearchQuery { q: Option<String> }

const LANGS: [&str; 8] = ["Rust", "Ruby", "Racket", "Python", "Prolog", "Zig", "Swift", "Scala"];

async fn combobox_page(jar: CookieJar, Query(s): Query<SearchQuery>) -> Markup {
    let q = s.q.unwrap_or_default();
    let hits: Vec<&str> = LANGS.iter().copied()
        .filter(|l| l.to_lowercase().contains(&q.to_lowercase())).collect();
    page(&jar, "Combobox", html! {
        (combobox("/combobox", "q", &LANGS, &q))
        ul { @for h in &hits { li { (h) } } }
        @if hits.is_empty() { p class="wo-note" { "No matches." } }
    })
}

#[derive(Deserialize)]
struct PageQuery { page: Option<usize> }

async fn list_page(jar: CookieJar, Query(p): Query<PageQuery>) -> Markup {
    const PER: usize = 8;
    const TOTAL: usize = 50;
    let current = p.page.unwrap_or(1).max(1);
    let rows: Vec<Markup> = (1..=(current * PER).min(TOTAL))
        .map(|n| html! { "Row " (n) })
        .collect();
    page(&jar, "Load-more list", html! { (pager(&rows, current, PER, TOTAL, "/list")) })
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

async fn form_page(jar: CookieJar) -> Markup {
    let v = SignUp::default();
    page(&jar, "Validated form", html! { (form("/form", &signup_fields(&v, &[]), "Sign up")) })
}

async fn form_submit(jar: CookieJar, Form(v): Form<SignUp>) -> axum::response::Response {
    let mut errors = Vec::new();
    if v.handle == "admin" { errors.push(("handle", "That handle is reserved.")); }
    if v.email.ends_with("@example.com") { errors.push(("email", "example.com addresses are not accepted.")); }
    if errors.is_empty() {
        return Redirect::to("/form?ok").into_response();
    }
    page(&jar, "Validated form", html! {
        p class="wo-error" { "Server-side checks failed. Browser validation passed, these rules only live on the server." }
        (form("/form", &signup_fields(&v, &errors), "Sign up"))
    }).into_response()
}

async fn counter_page(jar: CookieJar) -> Markup {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    page(&jar, "Counter", html! { (counter("/counter", n)) })
}

#[derive(Deserialize)]
struct CounterOp { op: String }

async fn counter_submit(jar: CookieJar, Form(f): Form<CounterOp>) -> (CookieJar, Redirect) {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    let n = match f.op.as_str() { "inc" => n + 1, "dec" => n - 1, _ => 0 };
    (jar.add(Cookie::new("count", n.to_string())), Redirect::to("/counter"))
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
        for path in ["/", "/dialog", "/popover", "/tabs?tab=1", "/accordion", "/combobox?q=r", "/list?page=2", "/form", "/counter"] {
            let res = router().oneshot(Request::get(path).body(Body::empty()).unwrap()).await.unwrap();
            assert_eq!(res.status(), 200, "{path}");
            let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
            let html = String::from_utf8(body.to_vec()).unwrap();
            assert!(!html.contains("<script"), "{path} contains a script tag");
        }
    }
}
