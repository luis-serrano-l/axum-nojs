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
    Cap, Caps, Field, FieldGroup, FieldKind, FormLayout, FormOptions, Streamed, Theme, UiState, accordion, accordion::{AccordionItem, AccordionOptions}, caps, color, combobox, combobox::{ComboboxOptions, OptionGroup},
    counter, counter::CounterOptions, color::ColorOptions, select::{Group as SelectGroup, SelectOption, SelectOptions}, dialog, dialog::{DialogOptions, DialogSize}, flash, form, layout, layout::{Palette, Tokens, layout_with}, paged_table, paged_table::PagedTableOptions,
    pager, pager::PagerOptions, popover::{MenuItem, Placement, PopoverOptions}, popover_menu, prg, range, range::RangeOptions, select, slot, tabs::{Tab, TabsOptions},
    table::{Column, Row, TableOptions, cols_from_query, sort_from_query}, tabs, theme_toggle, wizard, wizard::{Step, WizardOptions},
};
use std::time::Duration;

/// Every demo path the no-script test and the screenshot test visit.
pub const PATHS: [&str; 17] = [
    "/", "/caps", "/stream", "/settings", "/dialog?dialog=confirm", "/popover", "/tabs?tab.demo=1",
    "/accordion?open.faq=0,2&open.faq-more=0", "/combobox?q=r&sel=Zig", "/list?page=2", "/form", "/form?layout=inline", "/counter", "/inputs",
    "/table?sort=size&dir=desc&q=a&per.files=5&page=2&cols=name,size", "/wizard?step.signup=1", "/swap?n=3",
];

/// The whole demo app.
pub fn router() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/caps", get(caps_page))
        .route("/dialog", get(dialog_page))
        .route("/dialog/delete", post(dialog_delete))
        .route("/popover", get(popover_page))
        .route("/popover/signout", post(popover_signout))
        .route("/tabs", get(tabs_page))
        .route("/accordion", get(accordion_page))
        .route("/combobox", get(combobox_page))
        .route("/combobox/new", post(combobox_new))
        .route("/list", get(list_page))
        .route("/table", get(table_page))
        .route("/table.csv", get(table_csv))
        .route("/table/bulk", post(table_bulk))
        .route("/wizard", get(wizard_page).post(wizard_submit))
        .route("/form", get(form_page).post(form_submit))
        .route("/counter", get(counter_page).post(counter_submit))
        .route("/stream", get(stream_page))
        .route("/settings", get(settings_page).post(settings_submit))
        .route("/inputs", get(inputs_page).post(inputs_submit))
        .route("/theme", post(theme_submit))
        .route("/swap", get(swap_page).post(swap_submit))
        .merge(caps::router())
        .merge(webonsive::enhance::router())
}

// ---------- helpers ----------

fn theme_of(jar: &CookieJar) -> Theme {
    jar.get("theme").map(|c| Theme::parse(c.value())).unwrap_or_default()
}

/// Every component in the index: path, title (what each route passes to `page`), group, and
/// the platform features it is built on.
const COMPONENTS: [(&str, &str, &str, &str); 15] = [
    ("/dialog", "Dialog", "Overlays", "<dialog>, closedby, invoker commands, form footer"),
    ("/popover", "Popover menu", "Overlays", "popover, anchor positioning, nested popover, form actions"),
    ("/tabs", "Tabs", "Disclosure", "<details name>, ::details-content, view-transition-name, grid"),
    ("/accordion", "Accordion", "Disclosure", "<details name>, ::details-content, interpolate-size"),
    ("/combobox", "Combobox", "Input", "<datalist>, <optgroup>, <search>, aria-live"),
    ("/form", "Validated form", "Input", ":user-invalid, <fieldset>, <output> counters, field-sizing, multipart, PRG"),
    ("/wizard", "Wizard", "Input", "one form per step, PRG, formnovalidate, <progress>, UiState"),
    ("/inputs", "Select, range, colour", "Input", "<selectedcontent>, <optgroup>, formmethod, two-thumb range, color-mix()"),
    ("/counter", "Counter", "Server state", "form POST + cookie, type=number, disabled"),
    ("/settings", "Settings", "Server state", "UiState, PRG + flash"),
    ("/list", "Load-more list", "Server state", "links + view transitions"),
    ("/table", "Table", "Server state", "sort links, <search> filter, form= checkboxes, ?cols=, <details> rows, sticky header, ?page=n"),
    ("/caps", "Capabilities", "Server state", "@supports beacons + cookie"),
    ("/stream", "Streaming", "Server state", "declarative shadow DOM slots"),
    ("/swap", "Swap targets", "Server state", "data-wo-target, data-wo-swap, data-wo-oob, data-wo-indicator, data-wo-push, Wo-Enhance header"),
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
    page_with(caps, jar, title, None, body)
}

/// `page` under other [`Tokens`] (`/?palette=linen`), to show a theme is a value.
fn page_with(caps: &Caps, jar: &CookieJar, title: &str, tokens: Option<&Tokens>, body: Markup) -> Markup {
    let theme = theme_of(jar);
    let built = COMPONENTS.iter().find(|c| c.1 == title).map(|c| c.3);
    let body = html! {
        (toolbar(caps, theme, built.is_some()))
        h1 { (title) }
        @if let Some(feats) = built { p class="wo-built" { "Built on " @for f in feats.split(", ") { code { (f) } " " } } }
        (body)
    };
    match tokens {
        Some(t) => layout_with(caps, title, theme, t, body),
        None => layout(caps, title, theme, body),
    }
}

/// The second palette from `docs/theming.md`: warm paper, copper accent, amber in the dark.
const LINEN: Tokens = Tokens {
    light: Palette {
        bg: "#f4efe6", fg: "#1d1a17", muted: "#5d574f", line: "#d6cdbf", surface: "#fffdf9",
        accent: "#8a3b12", on_accent: "#ffffff", danger: "#a0261c", ok: "#2f6b3a",
    },
    dark: Palette {
        bg: "#161311", fg: "#ece6dc", muted: "#a59c90", line: "#3a332c", surface: "#1f1b18",
        accent: "#e8965a", on_accent: "#1a0f06", danger: "#ff8f85", ok: "#8fd39a",
    },
    radius: "3px",
    space: "8px",
};

// ---------- routes ----------

#[derive(Deserialize)]
struct IndexQuery { palette: Option<String> }

async fn index(caps: Caps, jar: CookieJar, Query(q): Query<IndexQuery>) -> Markup {
    let tokens = (q.palette.as_deref() == Some("linen")).then_some(&LINEN);
    page_with(&caps, &jar, "Components", tokens, html! {
        p class="wo-lede" { (COMPONENTS.len()) " interactive components for Axum and Maud that work with JavaScript turned off. The HTML platform and plain form posts do the work. Each page loads one optional script, " code { "/wo/enhance.js" } ", which updates the same markup in place instead of reloading. Block it and every page still works." }
        @if !caps.has(Cap::Probed) { p class="wo-note" { "First visit: this page is the fallback variant. Reload and the server will know your browser." } }
        p class="wo-note" { "Theme: " @if tokens.is_some() { a href="/" { "ink and moss" } " · linen and copper" } @else { "ink and moss · " a href="/?palette=linen" { "linen and copper" } } ", see " code { "docs/theming.md" } }
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
        (flash(&caps, jar.get("wo-flash").map(|c| c.value().to_string()).as_deref()))
        (dialog(&caps, "confirm", "Delete account", html! {
            p { "This cannot be undone. Everything you wrote goes with it." }
            label { "Tell us why (optional)" input name="reason" placeholder="Moving on"; }
        }, DialogOptions::default().title("Delete account?").size(DialogSize::Sm).danger(true)
            .confirm("Delete account", "/dialog/delete").returns_to("/dialog").cancel_label("Keep it")
            .open(state.dialog() == Some("confirm"))))
        p class="wo-note" { "Opened by an invoker button; the footer is a real form posting to " code { "/dialog/delete" } " with a hidden " code { "returns_to" } " so the server comes back here. Server-opened: " a href="/dialog?dialog=confirm" { "?dialog=confirm" } }
    })
}

#[derive(Deserialize)]
struct DeleteForm { #[serde(default)] reason: String, #[serde(default)] returns_to: String }

/// The confirm form's target: only ever redirects to a local path from `returns_to`.
async fn dialog_delete(Form(f): Form<DeleteForm>) -> axum::response::Response {
    let back = if f.returns_to.starts_with('/') && !f.returns_to.starts_with("//") { f.returns_to.as_str() } else { "/dialog" };
    let msg = if f.reason.is_empty() { "Account deleted (not really)".to_string() } else { format!("Account deleted (not really). Reason: {}", f.reason) };
    prg::<axum::body::Body>(back, Some(&msg))
}

async fn popover_page(caps: Caps, jar: CookieJar) -> Markup {
    const THEME: &[MenuItem] = &[MenuItem::link("Light", "/popover?theme=light"), MenuItem::link("Dark", "/popover?theme=dark")];
    page(&caps, &jar, "Popover menu", html! {
        (flash(&caps, jar.get("wo-flash").map(|c| c.value().to_string()).as_deref()))
        div class="wo-popover-row" {
            (popover_menu(&caps, "account", "Account", &[
                MenuItem::heading("Signed in as Ada"),
                MenuItem::link("Profile", "/popover").icon("@").shortcut("g p"),
                MenuItem::link("Settings", "/settings").icon("\u{2699}").shortcut("g s"),
                MenuItem::link("Billing", "/popover").icon("$").disabled(true),
                MenuItem::separator(),
                MenuItem::submenu("Theme", "account-theme", THEME).icon("\u{25d0}"),
                MenuItem::separator(),
                MenuItem::action("Sign out", "/popover/signout").icon("\u{2192}").danger(true),
            ], Default::default()))
            (popover_menu(&caps, "more", "More", &[
                MenuItem::link("Documentation", "/").icon("?"),
                MenuItem::action("Clear cache", "/popover/signout"),
            ], PopoverOptions::default().placement(Placement::BottomEnd)))
        }
        p class="wo-note" { "Links, a heading, a disabled item, a submenu that is another popover, and a " code { "<form method=\"post\">" } " action. Click outside or press Escape to close; the second menu opens end-aligned." }
    })
}

/// A menu action: Post/Redirect/Get back to the menu page with a flash.
async fn popover_signout() -> axum::response::Response {
    prg::<axum::body::Body>("/popover", Some("Signed out (not really)"))
}

async fn tabs_page(caps: Caps, jar: CookieJar, state: UiState) -> (UiState, Markup) {
    let open = state.tab("demo");
    let body = page(&caps, &jar, "Tabs", html! {
        (tabs(&caps, "demo", &[
            Tab::new("Install", html! { p { code { "cargo add webonsive maud axum" } } }),
            Tab::new("Use", html! { p { "Call a function, get " code { "Markup" } ", send it." } }).badge(3),
            // Lazy: the body is rendered only by the request that opens the tab.
            if open == 2 { Tab::new("Why", html! { p { "Because the platform can do this without script now. (Rendered on demand.)" } }) } else { Tab::lazy("Why") },
        ], TabsOptions::default().state(&state).select_below(true)))
        p class="wo-note" { "Deep link: " a href="/tabs?tab.demo=2" { "?tab.demo=2" } ". Leave and come back: the tab is remembered. The third tab is lazy; under 40rem the strip becomes a select." }
        h2 { "Vertical" }
        (tabs(&caps, "side", &[
            Tab::new("General", html! { p { "Titles stack on the left; the open panel sits beside them." } }),
            Tab::new("Members", html! { p { "Twelve members." } }).badge(12),
            Tab::new("Danger zone", html! { p { "Nothing here is destructive." } }),
        ], TabsOptions::default().state(&state).vertical(true)))
    });
    (state, body)
}

async fn accordion_page(caps: Caps, jar: CookieJar, state: UiState) -> (UiState, Markup) {
    let body = page(&caps, &jar, "Accordion", html! {
        (accordion(&caps, "faq", &[
            AccordionItem::new("Does this need JavaScript?", html! { p { "No. Turn it off and reload: every control still works through links and form posts. The one script on the page only swaps the answer in place instead of reloading." } })
                .icon("\u{1F50D}").summary("Every open and close is a link the server answers."),
            AccordionItem::new("Does it animate?", html! { p { "Yes, via ::details-content transitions where supported." } })
                .icon("\u{1F3AC}").summary("Height animates to auto in Chrome; elsewhere it snaps."),
            AccordionItem::new("Can several be open?", html! {
                p { "Yes: this group is " code { "multi" } ", so " code { "?open.faq=0,2" } " keeps two open. A body can hold another group:" }
                (accordion(&caps, "faq-more", &[
                    AccordionItem::new("Nested", html! { p { "Its own key, " code { "open.faq-more" } "." } }),
                    AccordionItem::new("Exclusive", html! { p { "This inner group opens one at a time." } }),
                ], AccordionOptions::default().state(&state)))
            }).icon("\u{1F4DA}").summary("Lists, links and a nested accordion."),
        ], AccordionOptions::default().state(&state).multi(true).controls(true)))
        p class="wo-note" { "Deep link: " a href="/accordion?open.faq=0,2" { "?open.faq=0,2" } ". Leave and come back: the open sections are remembered." }
    });
    (state, body)
}

const LANGS: [OptionGroup; 3] = [
    OptionGroup::new("Systems", &["Rust", "Zig", "Swift"]),
    OptionGroup::new("Scripting", &["Ruby", "Python", "Racket"]),
    OptionGroup::flat(&["Prolog", "Scala"]),
];

/// `?q=text&sel=a&sel=b`: the text and the repeated selection, in order.
fn combobox_query(pairs: &[(String, String)]) -> (String, Vec<String>) {
    let q = pairs.iter().find(|(k, _)| k == "q").map(|(_, v)| v.clone()).unwrap_or_default();
    let sel = pairs.iter().filter(|(k, _)| k == "sel").map(|(_, v)| v.clone()).collect();
    (q, sel)
}

async fn combobox_page(caps: Caps, jar: CookieJar, state: UiState, Query(pairs): Query<Vec<(String, String)>>) -> (UiState, Markup) {
    let (q, sel) = combobox_query(&pairs);
    let sel: Vec<&str> = sel.iter().map(String::as_str).collect();
    let hits: Vec<&str> = LANGS.iter().flat_map(|g| g.values().iter().copied())
        .filter(|l| !q.is_empty() && l.to_lowercase().contains(&q.to_lowercase())).collect();
    let body = page(&caps, &jar, "Combobox", html! {
        (flash(&caps, state.flash()))
        // One swap root around the form and its results: the script searches as you type.
        div id="langs" data-wo="swap" {
            (combobox(&caps, "q", "/combobox", ComboboxOptions::default()
                .query(&q).suggestions(&LANGS).results(&hits).selected(&sel).multi(true)
                .create("/combobox/new").label("Language").placeholder("Type a language")))
        }
        p class="wo-note" { "Pick several: each result adds a chip, each chip's \u{d7} removes it, and the chips ride along with the next search. Type a language that is not here to get a Create row." }
    });
    (state, body)
}

#[derive(Deserialize)]
struct NewLang { name: String, #[serde(default)] sel: Vec<String> }

async fn combobox_new(Form(f): Form<NewLang>) -> axum::response::Response {
    let name = f.name.trim();
    let mut to = String::from("/combobox?q=");
    for s in f.sel.iter().map(String::as_str).chain([name]) { to.push_str(&format!("&sel={}", s)); }
    prg::<axum::body::Body>(&to, Some(&format!("Added {name} (not really: the demo has no database).")))
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
    page(&caps, &jar, "Load-more list", html! { (pager(&caps, "/list", &rows, TOTAL, PagerOptions::default().page(current).per_page(PER))) })
}

#[derive(Deserialize, Default)]
struct TableQuery { sort: Option<String>, dir: Option<String>, q: Option<String>, page: Option<usize>, cols: Option<String>, loading: Option<u8> }

const FILES: [(&str, u32, &str); 12] = [
    ("archive.tar", 40960, "backup"), ("build.rs", 1200, "script"), ("cargo.lock", 8800, "generated"),
    ("index.html", 2100, "page"), ("logo.svg", 3400, "image"), ("main.rs", 5600, "source"),
    ("notes.md", 900, "text"), ("photo.jpg", 250000, "image"), ("readme.md", 4100, "text"),
    ("style.css", 1500, "stylesheet"), ("tests.rs", 7700, "source"), ("video.mp4", 9800000, "video"),
];
const FILE_COLS: [Column; 3] = [Column::sortable("name", "Name"), Column::numeric("size", "Size").width("7rem"), Column::sortable("kind", "Kind").width("9rem")];

/// Thirty-six files sorted and filtered on the server, in one place for the page and the CSV.
fn files(sort: Option<(&str, bool)>, q: &str) -> Vec<(String, u32, &'static str)> {
    let mut files: Vec<(String, u32, &str)> = ["src", "docs", "old"].iter()
        .flat_map(|dir| FILES.iter().map(move |f| (format!("{dir}/{}", f.0), f.1 * (dir.len() as u32), f.2)))
        .filter(|f| q.is_empty() || f.0.contains(q) || f.2.contains(q)).collect();
    if let Some((key, desc)) = sort {
        files.sort_by(|a, b| match key { "size" => a.1.cmp(&b.1), "kind" => a.2.cmp(b.2), _ => a.0.cmp(&b.0) });
        if desc { files.reverse(); }
    }
    files
}

/// The table only renders and links; a row can expand, has its own menu and can be selected.
async fn table_page(caps: Caps, jar: CookieJar, state: UiState, Query(t): Query<TableQuery>) -> (UiState, Markup) {
    let sort = sort_from_query(&FILE_COLS, t.sort.as_deref(), t.dir.as_deref());
    let cols = cols_from_query(&FILE_COLS, t.cols.as_deref());
    let q = t.q.unwrap_or_default().to_lowercase();
    let (per, pg) = (state.per_page("files").unwrap_or(10).clamp(1, 50), t.page.unwrap_or(1).max(1));
    let files = files(sort, &q);
    const MENU: [MenuItem; 2] = [MenuItem::link("Open", "/table"), MenuItem::action("Delete", "/table/bulk").danger(true)];
    let rows: Vec<Row> = files.iter().skip((pg - 1) * per).take(per)
        .map(|f| Row::new(vec![html! { code { (f.0) } }, html! { (paged_table::thousands(f.1 as usize / 1024)) " KB" }, html! { (f.2) }])
            .key(&f.0).detail(html! { p { "A " (f.2) " of " (f.1) " bytes, in " code { (f.0.split('/').next().unwrap_or("")) } "." } }).menu(&MENU)).collect();
    let options = TableOptions::default().cols(cols.as_deref()).choose_columns(true).bulk("/table/bulk", &[("archive", "Archive"), ("delete", "Delete")])
        .csv("/table.csv").empty("No files match this filter.").loading(t.loading == Some(1));
    let body = page(&caps, &jar, "Table", html! {
        (flash(&caps, state.flash()))
        p { "Click a header to sort, again to flip. Type to filter. Hide columns, tick rows for the bulk form, open a row's menu or its detail. The page size you pick is remembered for your next visit. Every state is a URL, including " a href="/table?loading=1" { "the loading one" } "." }
        (paged_table(&caps, "files", "/table", &FILE_COLS, &rows, files.len(), PagedTableOptions::default().sort(sort).filter(&q).page(pg).per_page(per).state(&state).table(options)))
    });
    (state, body)
}

/// The same rows as text/csv, for the sort and filter in the URL.
async fn table_csv(Query(t): Query<TableQuery>) -> impl IntoResponse {
    let sort = sort_from_query(&FILE_COLS, t.sort.as_deref(), t.dir.as_deref());
    let cols = cols_from_query(&FILE_COLS, t.cols.as_deref()).unwrap_or_else(|| FILE_COLS.iter().map(|c| c.key).collect());
    let q = t.q.unwrap_or_default().to_lowercase();
    let mut csv = cols.join(",") + "\n";
    for f in files(sort, &q) {
        let cells = [("name", f.0.clone()), ("size", f.1.to_string()), ("kind", f.2.to_string())];
        csv += &cells.iter().filter(|(k, _)| cols.contains(k)).map(|(_, v)| v.as_str()).collect::<Vec<_>>().join(",");
        csv.push('\n');
    }
    ([("content-type", "text/csv; charset=utf-8"), ("content-disposition", "attachment; filename=\"files.csv\"")], csv)
}

/// `row=<key>` per ticked box and `action=<value>` from the button: acknowledged with a flash.
async fn table_bulk(Form(pairs): Form<Vec<(String, String)>>) -> axum::response::Response {
    let rows = pairs.iter().filter(|(k, _)| k == "row").count();
    let action = pairs.iter().find(|(k, _)| k == "action").map(|(_, v)| v.as_str()).unwrap_or("delete");
    let msg = if rows == 0 { "Nothing selected: tick a row first.".to_string() } else { format!("{action}: {rows} file(s) (not really).") };
    prg::<axum::body::Body>("/table", Some(&msg))
}

/// What the wizard has collected so far, kept in one `wizard` cookie as `k=v&k=v`.
fn wizard_data(jar: &CookieJar) -> Vec<(String, String)> {
    let raw = jar.get("wizard").map(|c| c.value().to_string()).unwrap_or_default();
    raw.split('&').filter_map(|p| p.split_once('=')).map(|(k, v)| (k.to_string(), v.replace('+', " "))).collect()
}

/// Server rules for a wizard step: `(field, message)` per problem.
fn wizard_check(step: usize, data: &[(String, String)]) -> Vec<(&'static str, &'static str)> {
    let get = |k: &str| data.iter().find(|(n, _)| n == k).map(|(_, v)| v.trim()).unwrap_or("");
    let mut errors = Vec::new();
    if step == 0 {
        if get("name").is_empty() { errors.push(("name", "Enter your name.")); }
        if !get("email").contains('@') { errors.push(("email", "Enter an email address with an @.")); }
        else if get("email").ends_with("@example.com") { errors.push(("email", "example.com addresses are not accepted.")); }
    }
    errors
}

fn wizard_steps(state: &UiState, data: &[(String, String)], errors: &[(&str, &str)]) -> [Step; 3] {
    let get = |k: &str| data.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()).unwrap_or("");
    let err = |k: &str| errors.iter().find(|(n, _)| *n == k).map(|(_, m)| *m);
    let field = |name: &'static str, label: &str, kind: &str| html! {
        label for={ "w-" (name) } { (label) }
        input id={ "w-" (name) } type=(kind) name=(name) value=(get(name)) required
            aria-invalid=[err(name).map(|_| "true")] aria-describedby=[err(name).map(|_| format!("w-{name}-error"))];
        @if let Some(m) = err(name) { p id={ "w-" (name) "-error" } class="wo-error" { (m) } }
    };
    [
        Step::new("Account", html! { (field("name", "Name", "text")) (field("email", "Email", "email")) }).error(!errors.is_empty()),
        Step::new("Newsletter", html! {
            label { "Digest" select name="digest" { @for d in ["daily", "weekly", "never"] { option value=(d) selected[get("digest") == d] { (d) } } } }
            label { "Topics" input type="text" name="topics" value=(get("topics")) placeholder="rust, html"; }
        }).optional(true),
        Step::new("Review", wizard::summary(state, "signup", &[
            ("Name", get("name"), 0), ("Email", get("email"), 0), ("Digest", get("digest"), 1), ("Topics", get("topics"), 1),
        ])),
    ]
}

fn wizard_view(caps: &Caps, jar: &CookieJar, state: &UiState, data: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    page(caps, jar, "Wizard", html! {
        (flash(caps, state.flash()))
        p { "Three steps, one form each. The server checks every step; the second can be skipped. Close the tab and come back to " a href="/wizard" { "/wizard" } ": you resume where you left off." }
        (wizard(caps, "signup", "/wizard", &wizard_steps(state, data, errors), state, WizardOptions::default().finish("Create account")))
    })
}

async fn wizard_page(caps: Caps, jar: CookieJar, state: UiState) -> (UiState, Markup) {
    let body = wizard_view(&caps, &jar, &state, &wizard_data(&jar), &[]);
    (state, body)
}

/// Check this step: answer 422 with the same step and its messages, or merge the fields into
/// the cookie (kept a week, so closing the browser loses nothing) and redirect to the next step.
async fn wizard_submit(caps: Caps, jar: CookieJar, headers: HeaderMap, Form(fields): Form<Vec<(String, String)>>) -> axum::response::Response {
    let field = |k: &str| fields.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    let step: usize = field("step").and_then(|v| v.parse().ok()).unwrap_or(0);
    let skip = field("skip") == Some("1");
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");
    let state = UiState::from_request("/wizard", &format!("step.signup={step}"), cookie);
    let mut data = wizard_data(&jar);
    for (k, v) in fields.iter().filter(|(k, _)| !skip && k != "step" && k != "skip") {
        data.retain(|(n, _)| n != k);
        data.push((k.clone(), v.clone()));
    }
    let errors = if skip { Vec::new() } else { wizard_check(step, &data) };
    if !errors.is_empty() {
        return (axum::http::StatusCode::UNPROCESSABLE_ENTITY, wizard_view(&caps, &jar, &state, &data, &errors)).into_response();
    }
    if step + 1 >= 3 {
        return (jar.remove(Cookie::from("wizard")), prg::<axum::body::Body>(&state.link("step.signup", "0"), Some("Account created (well, the cookie was cleared)."))).into_response();
    }
    let value: String = data.iter().map(|(k, v)| format!("{k}={}", v.replace(' ', "+"))).collect::<Vec<_>>().join("&");
    let kept = Cookie::parse(format!("wizard={value}; Path=/; Max-Age=604800; SameSite=Lax")).expect("cookie");
    (jar.add(kept), prg::<axum::body::Body>(&state.link("step.signup", &(step + 1).to_string()), None)).into_response()
}

/// The sign-up fields as posted: text values by name, and for each file field its name and size.
#[derive(Default)]
struct SignUp { values: Vec<(String, String)>, files: Vec<(String, usize)> }

impl SignUp {
    fn get(&self, k: &str) -> &str {
        self.values.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()).unwrap_or("")
    }
}

fn form_view(caps: &Caps, jar: &CookieJar, state: &UiState, inline: bool, v: &SignUp, errors: &[(&str, &str)]) -> Markup {
    let err = |n: &str| errors.iter().find(|(f, _)| *f == n).map(|(_, m)| *m);
    let account = [
        Field::new("name", "Name", FieldKind::Text).value(v.get("name")).required(true).error(err("name")),
        Field::new("email", "Email", FieldKind::Email).value(v.get("email")).required(true).error(err("email")),
        Field::new("age", "Age", FieldKind::Number { min: 13, max: 120 }).value(v.get("age")).required(true),
        Field::new("handle", "Handle", FieldKind::Pattern { pattern: "[a-z0-9_]{3,16}", hint: "3–16 lowercase letters, digits or _" })
            .value(v.get("handle")).required(true).error(err("handle")),
    ];
    let profile = [
        Field::new("bio", "Bio", FieldKind::Textarea { rows: 3 }).value(v.get("bio")).max_len(160).help("Grows as you type where the browser supports it."),
        Field::new("avatar", "Avatar", FieldKind::File { accept: "image/png,image/jpeg", multiple: false }).help("PNG or JPEG."),
        Field::new("start", "Start date", FieldKind::Date { min: "2026-01-01", max: "2027-12-31" }).value(v.get("start")),
        Field::new("call", "Best time to call", FieldKind::Time { min: "09:00", max: "17:00" }).value(v.get("call")).help("Office hours, 09:00 to 17:00."),
    ];
    let layout = if inline { FormLayout::Inline } else { FormLayout::Stacked };
    page(caps, jar, "Validated form", html! {
        (flash(caps, state.flash()))
        p { "Labels " @if inline { "beside the fields. " a href="/form" { "Put them above" } } @else { "above the fields. " a href="/form?layout=inline" { "Put them beside" } } "." }
        @if !errors.is_empty() { p class="wo-error" { "Server-side checks failed. Browser validation passed, these rules only live on the server." } }
        (form(caps, "/form", &[FieldGroup::new("Account", &account), FieldGroup::new("Profile", &profile)], FormOptions::default().submit("Sign up").layout(layout)))
    })
}

#[derive(Deserialize)]
struct FormQuery { layout: Option<String> }

async fn form_page(caps: Caps, jar: CookieJar, state: UiState, Query(q): Query<FormQuery>) -> (UiState, Markup) {
    let body = form_view(&caps, &jar, &state, q.layout.as_deref() == Some("inline"), &SignUp::default(), &[]);
    (state, body)
}

/// A multipart post (the avatar is a file): server rules, then PRG with a flash or the form again.
async fn form_submit(caps: Caps, jar: CookieJar, state: UiState, mut parts: axum::extract::Multipart) -> axum::response::Response {
    let mut v = SignUp::default();
    while let Ok(Some(part)) = parts.next_field().await {
        let (name, file) = (part.name().unwrap_or("").to_string(), part.file_name().map(str::to_string));
        match file {
            Some(f) => { let n = part.bytes().await.map(|b| b.len()).unwrap_or(0); if !f.is_empty() { v.files.push((f, n)); } }
            None => v.values.push((name, part.text().await.unwrap_or_default())),
        }
    }
    let mut errors = Vec::new();
    if v.get("handle") == "admin" { errors.push(("handle", "That handle is reserved.")); }
    if v.get("email").ends_with("@example.com") { errors.push(("email", "example.com addresses are not accepted.")); }
    if errors.is_empty() {
        let got = v.files.iter().map(|(f, n)| format!(" with {f} ({n} bytes)")).collect::<String>();
        return prg::<axum::body::Body>("/form", Some(&format!("Signed up as {}{got}.", v.get("handle")))).into_response();
    }
    (axum::http::StatusCode::UNPROCESSABLE_ENTITY, form_view(&caps, &jar, &state, false, &v, &errors)).into_response()
}

const COUNTER: CounterOptions = CounterOptions { min: Some(0), max: Some(20), step: 2, typed: true };

async fn counter_page(caps: Caps, jar: CookieJar) -> Markup {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    page(&caps, &jar, "Counter", html! {
        p { "Steps of two between 0 and 20. The buttons switch off at the ends; a typed value off the step or the bounds is refused by the browser and clamped by the server." }
        (counter(&caps, "/counter", n, COUNTER))
    })
}

#[derive(Deserialize)]
struct CounterOp { op: String, value: Option<i64> }

async fn counter_submit(jar: CookieJar, Form(f): Form<CounterOp>) -> (CookieJar, Redirect) {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    let n = COUNTER.apply(n, &f.op, f.value);
    (jar.add(Cookie::new("count", n.to_string())), Redirect::to("/counter"))
}

#[derive(Deserialize)]
struct SwapQuery { n: Option<u32> }

fn notes_of(jar: &CookieJar) -> Vec<String> {
    jar.get("notes").map(|c| c.value().split('|').filter(|s| !s.is_empty()).map(str::to_string).collect()).unwrap_or_default()
}

/// Two controls outside any swap root that name their target: the link swaps one `<span>`,
/// the form appends to a list. The same requests are plain navigations without the script.
async fn swap_page(caps: Caps, jar: CookieJar, Query(q): Query<SwapQuery>) -> Markup {
    let n = q.n.unwrap_or(1);
    page(&caps, &jar, "Swap targets", html! {
        (flash(&caps, jar.get("wo-flash").map(|c| c.value().to_string()).as_deref()))
        p class="wo-note" { "Neither control sits inside a swap root. " code { "data-wo-target" } " names the root to update and " code { "data-wo-swap" } " how; without the script both are ordinary navigations to the same URL." }
        p { "Count: " span id="count" data-wo="swap" { (n) } " " a href={ "/swap?n=" (n + 1) } data-wo-target="#count" { "Add one" }
            " · " a href={ "/swap?n=" (n + 10) } data-wo-target="#count" data-wo-push="false" { "Add ten, keep the URL" } }
        p { "Notes so far: " span id="note-count" { (notes_of(&jar).len()) } }
        form method="post" action="/swap" data-wo-target="#log" data-wo-swap="append" data-wo-indicator="#saving" {
            input name="note" required placeholder="A note" aria-label="Note" autocomplete="off";
            button type="submit" class="wo-primary" { "Add note" }
            " " span id="saving" class="wo-note" hidden { "Saving…" }
        }
        ol id="log" data-wo="swap" { @for note in notes_of(&jar) { li { (note) } } }
    })
}

#[derive(Deserialize)]
struct SwapForm { note: String }

/// An enhanced request (`Wo-Enhance: 1`) gets only the new `<li>` inside an `#log` to append,
/// plus the note count marked `data-wo-oob` so it updates wherever it is on the page; a plain
/// one gets Post/Redirect/Get to the full page, which shows both anyway.
async fn swap_submit(jar: CookieJar, headers: HeaderMap, Form(f): Form<SwapForm>) -> axum::response::Response {
    let note = f.note.replace('|', " ");
    let mut notes = notes_of(&jar);
    notes.push(note.clone());
    let jar = jar.add(Cookie::new("notes", notes.join("|")));
    if headers.contains_key("wo-enhance") {
        return (jar, html! { ol id="log" { li { (note) } } span id="note-count" data-wo-oob { (notes.len()) } }).into_response();
    }
    (jar, prg::<axum::body::Body>("/swap", Some("Note added"))).into_response()
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
            Tab::new("Profile", html! { form class="wo-form" method="post" action="/settings" { (hidden)
                div class="wo-field" { label for="name" { "Display name" } input id="name" name="name" value=(current.name) required; }
                button type="submit" class="wo-primary" { "Save" } } }),
            Tab::new("Notifications", html! { form class="wo-form" method="post" action="/settings" { (hidden)
                input type="hidden" name="name" value=(current.name);
                label { input type="checkbox" name="notify" value="true" checked[current.notify]; " Email me about releases" }
                button type="submit" class="wo-primary" { "Save" } } }),
        ], TabsOptions::default().state(&state)))
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

/// The inputs page's values: from the query while filtering (unsaved), else from the cookie.
#[derive(Deserialize, Default)]
struct Inputs {
    size: Option<String>, volume: Option<i64>, accent: Option<String>,
    #[serde(rename = "accent-alpha")] alpha: Option<u8>,
    #[serde(rename = "accent-preset")] preset: Option<String>,
    price_min: Option<i64>, price_max: Option<i64>, country: Option<String>,
    #[serde(rename = "country-q")] country_q: Option<String>,
}

const SIZES: [(&str, &str, &str); 3] = [("s", "Small", "🐭"), ("m", "Medium", "🐕"), ("l", "Large", "🐘")];
const ACCENTS: [&str; 5] = ["#1f6f5f", "#2f5bea", "#b3261e", "#8a5a00", "#6b3fa0"];
/// `(value, name, flag)`.
type Country = (&'static str, &'static str, &'static str);
const COUNTRIES: [(&str, [Country; 7]); 3] = [
    ("Europe", [("es", "Spain", "🇪🇸"), ("fr", "France", "🇫🇷"), ("de", "Germany", "🇩🇪"), ("it", "Italy", "🇮🇹"), ("pt", "Portugal", "🇵🇹"), ("nl", "Netherlands", "🇳🇱"), ("se", "Sweden", "🇸🇪")]),
    ("Americas", [("us", "United States", "🇺🇸"), ("ca", "Canada", "🇨🇦"), ("mx", "Mexico", "🇲🇽"), ("br", "Brazil", "🇧🇷"), ("ar", "Argentina", "🇦🇷"), ("cl", "Chile", "🇨🇱"), ("co", "Colombia", "🇨🇴")]),
    ("Asia", [("jp", "Japan", "🇯🇵"), ("kr", "South Korea", "🇰🇷"), ("in", "India", "🇮🇳"), ("id", "Indonesia", "🇮🇩"), ("vn", "Vietnam", "🇻🇳"), ("th", "Thailand", "🇹🇭"), ("ph", "Philippines", "🇵🇭")]),
];

/// The saved values: `size|volume|accent|alpha|price_min|price_max|country`.
fn saved_inputs(jar: &CookieJar) -> Inputs {
    let saved = jar.get("inputs").map(|c| c.value().to_string()).unwrap_or_default();
    let p: Vec<&str> = saved.split('|').collect();
    let at = |i: usize| p.get(i).filter(|v| !v.is_empty()).map(|v| v.to_string());
    Inputs {
        size: at(0), volume: at(1).and_then(|v| v.parse().ok()), accent: at(2), alpha: at(3).and_then(|v| v.parse().ok()),
        price_min: at(4).and_then(|v| v.parse().ok()), price_max: at(5).and_then(|v| v.parse().ok()), country: at(6), ..Inputs::default()
    }
}

/// Select, range and colour in one form; the chosen values live in an `inputs` cookie. The
/// country filter is a GET through the same form, so its values come from the query.
async fn inputs_page(caps: Caps, jar: CookieJar, state: UiState, Query(q): Query<Inputs>) -> (UiState, Markup) {
    let v = if q.country_q.is_some() { q } else { saved_inputs(&jar) };
    let accent = v.accent.as_deref().unwrap_or("#1f6f5f");
    let (lo, hi) = range::order(v.price_min.unwrap_or(20), v.price_max.unwrap_or(80));
    let sizes: Vec<SelectOption> = SIZES.iter().map(|(v, l, i)| SelectOption::new(v, l).icon(i)).collect();
    let countries: Vec<Vec<SelectOption>> = COUNTRIES.iter().map(|(_, cs)| cs.iter().map(|(v, l, i)| SelectOption::new(v, l).icon(i)).collect()).collect();
    let groups: Vec<SelectGroup> = COUNTRIES.iter().zip(&countries).map(|((g, _), cs)| SelectGroup::new(g, cs)).collect();
    let body = page(&caps, &jar, "Select, range, colour", html! {
        (flash(&caps, state.flash()))
        form id="inputs" data-wo="swap" class="wo-form" method="post" action="/inputs" {
            div class="wo-field" { label for="size" { "Size" } (select(&caps, "size", &[SelectGroup::flat(&sizes)], v.size.as_deref().unwrap_or("m"), Default::default())) }
            div class="wo-field" { label for="country" { "Country" }
                (select(&caps, "country", &groups, v.country.as_deref().unwrap_or("es"), SelectOptions::default().search("/inputs", v.country_q.as_deref().unwrap_or("")))) }
            div class="wo-field" { label for="f-volume" { "Volume" } (range(&caps, "volume", v.volume.unwrap_or(40), RangeOptions::default().step(5))) }
            div class="wo-field" { label for="f-price_min" { "Price" } (range::range_pair(&caps, "price", (lo, hi), RangeOptions::default().step(5))) }
            div class="wo-field" { label for="f-accent" { "Accent" } (color(&caps, "accent", accent, ColorOptions::default().presets(&ACCENTS).alpha(v.alpha.unwrap_or(100)))) }
            button type="submit" class="wo-primary" { "Save" }
        }
        p class="wo-note" { "Without the enhancement script the outputs and the swatch show the last saved values and update on submit, and the country filter needs its button." }
    });
    (state, body)
}

async fn inputs_submit(jar: CookieJar, Form(f): Form<Inputs>) -> (CookieJar, axum::response::Response) {
    let size = SIZES.iter().find(|(v, ..)| Some(*v) == f.size.as_deref()).map(|(v, ..)| *v).unwrap_or("m");
    let hex = |c: &Option<String>| c.as_deref().filter(|c| c.len() == 7 && c.starts_with('#')).map(str::to_string);
    let accent = hex(&f.preset).or(hex(&f.accent)).unwrap_or_else(|| "#1f6f5f".into());
    let country = COUNTRIES.iter().flat_map(|(_, cs)| cs).find(|c| Some(c.0) == f.country.as_deref()).map(|c| c.0).unwrap_or("es");
    let (lo, hi) = range::order(f.price_min.unwrap_or(20).clamp(0, 100), f.price_max.unwrap_or(80).clamp(0, 100));
    let value = format!("{size}|{}|{accent}|{}|{lo}|{hi}|{country}", f.volume.unwrap_or(40).clamp(0, 100), f.alpha.unwrap_or(100).min(100));
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
