//! Demo server: one route per component. Handlers only parse input and call `axum-nojs`.
//! The binary in `main.rs` serves [`router`]; tests and `axum-nojs-test` call it directly.

use axum::{
    Form, Router,
    extract::{Query, RawQuery},
    http::HeaderMap,
    response::{IntoResponse, Redirect},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use maud::{Markup, html};
use serde::Deserialize;
use axum_nojs::{
    Ui, theme::THEME_COOKIE,
    Cap, Caps, Field, FieldGroup, FieldKind, FormLayout, FormOptions, Streamed, Theme, UiState, accordion_with, accordion::{AccordionItem, AccordionOptions}, caps, color_with, combobox_with, combobox::{ComboboxOptions, OptionGroup}, counter_with, counter::CounterOptions, color::ColorOptions, select::{Group as SelectGroup, SelectOption, SelectOptions}, dialog_with, dialog::{DialogOptions, DialogSize}, flash_with, flash::{FlashOptions, Level, stack}, form_with, form::fields, layout, layout::{Palette, Tokens, layout_with}, paged_table, paged_table_with, paged_table::PagedTableOptions, pager_with, pager::PagerOptions, popover::{MenuItem, Placement, PopoverOptions}, popover_menu, popover_menu_with, prg, range, range_with, range::RangeOptions, select, select_with, slot, tabs::{Tab, TabsOptions},
    table::{Column, Row, TableOptions, TableQuery}, tabs_with, theme_toggle, wizard, wizard_with, wizard::{Step, WizardOptions},
    breadcrumbs, command_palette_with, drawer_with, empty_state_with, palette::{self, Command, PaletteOptions}, skeleton_with, skeleton::SkeletonOptions, stat_with, stat::StatOptions, toasts_with, toast::ToastOptions, drawer::DrawerOptions, empty_state::EmptyOptions,
};
use std::time::Duration;
use tower_http::compression::{CompressionLayer, predicate::{DefaultPredicate, Predicate}};

/// Every demo path the no-script test and the screenshot test visit.
pub const PATHS: [&str; 21] = [
    "/", "/caps", "/stream", "/settings", "/dialog?dialog=confirm", "/popover", "/tabs?tab.demo=1",
    "/accordion?open.faq=0,2&open.faq-more=0", "/combobox?q=r&sel=Zig", "/list?page=2", "/form", "/form?layout=inline", "/counter", "/inputs",
    "/table?sort=size&dir=desc&q=a&per.files=5&page=2&cols=name,size", "/wizard?step.signup=1", "/swap?n=3",
    "/toast", "/nav", "/dashboard?orders=none", "/palette?q=ta",
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
        .route("/toast", get(toast_page).post(toast_submit))
        .route("/nav", get(nav_page))
        .route("/dashboard", get(dashboard_page))
        .route("/palette", get(palette_page))
        .merge(caps::router())
        .merge(axum_nojs::enhance::router())
        .layer(axum::middleware::from_fn(axum_nojs::enhance::slim))
        .layer(CompressionLayer::new().compress_when(DefaultPredicate::new().and(WholeBody)))
}

/// Compress only bodies whose size is known up front. A streamed page (`/stream`) has no
/// exact size, and gzip would hold its chunks until the buffer fills, so it is sent as is.
#[derive(Clone, Copy)]
struct WholeBody;

impl Predicate for WholeBody {
    fn should_compress<B: axum::body::HttpBody>(&self, response: &axum::http::Response<B>) -> bool {
        response.body().size_hint().exact().is_some()
    }
}

// ---------- helpers ----------

/// Every component in the index: path, title (what each route passes to `page`), group, and
/// the platform features it is built on.
const COMPONENTS: [(&str, &str, &str, &str); 19] = [
    ("/palette", "Command palette", "Navigation", "popover, <datalist>, <search>, accesskey, GET + 303"),
    ("/nav", "Drawer and breadcrumbs", "Navigation", "<dialog>, invoker commands, closedby, @starting-style, <details>"),
    ("/toast", "Toasts", "Feedback", "position: fixed, role=alert, CSS fade, PRG"),
    ("/dashboard", "Stats and empty states", "Feedback", "auto-fit grid, form POST"),
    ("/dialog", "Dialog", "Overlays", "<dialog>, closedby, invoker commands, form footer"),
    ("/popover", "Popover menu", "Overlays", "popover, anchor positioning, nested popover, form actions"),
    ("/tabs", "Tabs", "Disclosure", "<details name>, ::details-content, view-transition-name, grid"),
    ("/accordion", "Accordion", "Disclosure", "<details name>, ::details-content, interpolate-size"),
    ("/combobox", "Combobox", "Input", "<datalist>, <optgroup>, <search>, aria-live"),
    ("/form", "Validated form", "Input", ":user-invalid, <fieldset>, <output> counters, field-sizing, multipart, PRG"),
    ("/wizard", "Wizard", "Input", "one form per step, PRG, formnovalidate, <progress>, UiState"),
    ("/inputs", "Select, range, colour", "Input", "<selectedcontent>, <optgroup>, formmethod, two-thumb range, color-mix()"),
    ("/counter", "Counter", "Server state", "form POST + cookie, type=number, disabled"),
    ("/settings", "Settings", "Server state", "UiState, PRG + flash, role=alert, CSS auto-hide"),
    ("/list", "Load-more list", "Server state", "links + view transitions"),
    ("/table", "Table", "Server state", "sort links, <search> filter, form= checkboxes, ?cols=, <details> rows, sticky header, ?page=n"),
    ("/caps", "Capabilities", "Server state", "@supports beacons + cookie"),
    ("/stream", "Streaming", "Server state", "declarative shadow DOM slots, skeleton placeholders, aria-busy"),
    ("/swap", "Swap targets", "Server state", "data-nojs-target, data-nojs-swap, data-nojs-oob, data-nojs-indicator, data-nojs-push, Nojs-Enhance header"),
];
const GROUPS: [&str; 6] = ["Overlays", "Disclosure", "Navigation", "Input", "Feedback", "Server state"];

/// The row above every title: the way back to the index (not on the index) and the theme switch.
fn toolbar(caps: &Caps, theme: Theme, back: bool) -> Markup {
    html! { nav class="nojs-toolbar" {
        @if back { a class="nojs-back" href="/" { "All components" } } @else { span {} }
        (theme_toggle(caps, "/theme", theme))
    } }
}

/// Every page: the theme from its cookie, the toolbar, the title, what it is built on, the body.
fn page(ui: &Ui, title: &str, body: Markup) -> Markup {
    page_with(ui, title, None, body)
}

/// `page` under other [`Tokens`] (`/?palette=linen`), to show a theme is a value.
fn page_with(ui: &Ui, title: &str, tokens: Option<&Tokens>, body: Markup) -> Markup {
    let (caps, theme) = (&ui.caps, ui.theme);
    let built = COMPONENTS.iter().find(|c| c.1 == title).map(|c| c.3);
    let body = html! {
        (toolbar(caps, theme, built.is_some()))
        h1 { (title) }
        @if let Some(feats) = built { p class="nojs-built" { "Built on " @for f in feats.split(", ") { code { (f) } " " } } }
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
        accent: "#8a3b12", on_accent: "#ffffff", danger: "#a0261c", ok: "#2f6b3a", warn: "#7a5500",
    },
    dark: Palette {
        bg: "#161311", fg: "#ece6dc", muted: "#a59c90", line: "#3a332c", surface: "#1f1b18",
        accent: "#e8965a", on_accent: "#1a0f06", danger: "#ff8f85", ok: "#8fd39a", warn: "#f0c060",
    },
    radius: "3px",
    space: "8px",
};

// ---------- routes ----------

#[derive(Deserialize)]
struct IndexQuery { palette: Option<String> }

async fn index(ui: Ui, Query(q): Query<IndexQuery>) -> Markup {
    let tokens = (q.palette.as_deref() == Some("linen")).then_some(&LINEN);
    page_with(&ui, "Components", tokens, html! {
        p class="nojs-lede" { (COMPONENTS.len()) " interactive components for Axum and Maud that work with JavaScript turned off. The HTML platform and plain form posts do the work. Each page loads one optional script, " code { "/nojs/enhance.js" } ", which updates the same markup in place instead of reloading. Block it and every page still works." }
        @if !ui.has(Cap::Probed) { p class="nojs-note" { "First visit: this page is the fallback variant. Reload and the server will know your browser." } }
        p class="nojs-note" { "Theme: " @if tokens.is_some() { a href="/" { "ink and moss" } " · linen and copper" } @else { "ink and moss · " a href="/?palette=linen" { "linen and copper" } } ", see " code { "docs/theming.md" } }
        div class="nojs-index" { @for group in GROUPS {
            h2 { (group) }
            ul { @for (href, title, _, feats) in COMPONENTS.iter().filter(|c| c.2 == group) {
                li { a href=(href) { (title) } span { @for f in feats.split(", ") { code { (f) } " " } } }
            } }
        } }
        // Idle-time fetch of every component page, so the click is served from cache.
        @for (href, ..) in COMPONENTS { link rel="prefetch" href=(href); }
    })
}

async fn dialog_page(ui: Ui) -> Markup {
    page(&ui, "Dialog", html! {
        (ui.flash())
        (dialog_with(&ui, "confirm", "Delete account", html! {
            p { "This cannot be undone. Everything you wrote goes with it." }
            label { "Tell us why (optional)" input name="reason" placeholder="Moving on"; }
        }, DialogOptions::default().title("Delete account?").size(DialogSize::Sm).danger()
            .confirm("Delete account", "/dialog/delete").cancel_label("Keep it").state(&ui.state)))
        p class="nojs-note" { "Opened by an invoker button; the footer is a real form posting to " code { "/dialog/delete" } " with a hidden " code { "returns_to" } " so the server comes back here. Server-opened: " a href="/dialog?dialog=confirm" { "?dialog=confirm" } }
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

async fn popover_page(ui: Ui) -> Markup {
    const THEME: &[MenuItem] = &[MenuItem::link("Light", "/popover?theme=light"), MenuItem::link("Dark", "/popover?theme=dark")];
    page(&ui, "Popover menu", html! {
        (ui.flash())
        div class="nojs-popover-row" {
            (popover_menu(&ui, "Account", &[
                MenuItem::heading("Signed in as Ada"),
                MenuItem::link("Profile", "/popover").icon("@").shortcut("g p"),
                MenuItem::link("Settings", "/settings").icon("\u{2699}").shortcut("g s"),
                MenuItem::link("Billing", "/popover").icon("$").disabled(),
                MenuItem::separator(),
                MenuItem::submenu("Theme", "account-theme", THEME).icon("\u{25d0}"),
                MenuItem::separator(),
                MenuItem::action("Sign out", "/popover/signout").icon("\u{2192}").danger(),
            ]))
            (popover_menu_with(&ui, "more", "More", &[
                MenuItem::link("Documentation", "/").icon("?"),
                MenuItem::action("Clear cache", "/popover/signout"),
            ], PopoverOptions::default().placement(Placement::BottomEnd)))
        }
        p class="nojs-note" { "Links, a heading, a disabled item, a submenu that is another popover, and a " code { "<form method=\"post\">" } " action. Click outside or press Escape to close; the second menu opens end-aligned." }
    })
}

/// A menu action: Post/Redirect/Get back to the menu page with a flash.
async fn popover_signout() -> axum::response::Response {
    prg::<axum::body::Body>("/popover", Some("Signed out (not really)"))
}

async fn tabs_page(ui: Ui) -> (Ui, Markup) {
    let body = page(&ui, "Tabs", html! {
        // Hovering or focusing a tab title fetches it early; the click reuses the answer.
        div data-nojs-prefetch { (tabs_with(&ui, "demo", &[
            Tab::new("Install", html! { p { code { "cargo add axum-nojs maud axum" } } }),
            Tab::new("Use", html! { p { "Call a function, get " code { "Markup" } ", send it." } }).badge(3),
            // Lazy: the body is rendered only by the request that opens the tab.
            Tab::lazy_with("Why", &|| html! { p { "Because the platform can do this without script now. (Rendered on demand.)" } }),
        ], TabsOptions::default().state(&ui.state).select_below())) }
        p class="nojs-note" { "Deep link: " a href="/tabs?tab.demo=2" { "?tab.demo=2" } ". Leave and come back: the tab is remembered. The third tab is lazy; under 40rem the strip becomes a select." }
        h2 { "Vertical" }
        (tabs_with(&ui, "side", &[
            Tab::new("General", html! { p { "Titles stack on the left; the open panel sits beside them." } }),
            Tab::new("Members", html! { p { "Twelve members." } }).badge(12),
            Tab::new("Danger zone", html! { p { "Nothing here is destructive." } }),
        ], TabsOptions::default().state(&ui.state).vertical()))
    });
    (ui, body)
}

async fn accordion_page(ui: Ui) -> (Ui, Markup) {
    let body = page(&ui, "Accordion", html! {
        (accordion_with(&ui, "faq", &[
            AccordionItem::new("Does this need JavaScript?", html! { p { "No. Turn it off and reload: every control still works through links and form posts. The one script on the page only swaps the answer in place instead of reloading." } })
                .icon("\u{1F50D}").summary("Every open and close is a link the server answers."),
            AccordionItem::new("Does it animate?", html! { p { "Yes, via ::details-content transitions where supported." } })
                .icon("\u{1F3AC}").summary("Height animates to auto in Chrome; elsewhere it snaps."),
            AccordionItem::new("Can several be open?", html! {
                p { "Yes: this group is " code { "multi" } ", so " code { "?open.faq=0,2" } " keeps two open. A body can hold another group:" }
                (accordion_with(&ui, "faq-more", &[
                    AccordionItem::new("Nested", html! { p { "Its own key, " code { "open.faq-more" } "." } }),
                    AccordionItem::new("Exclusive", html! { p { "This inner group opens one at a time." } }),
                ], AccordionOptions::default().state(&ui.state)))
            }).icon("\u{1F4DA}").summary("Lists, links and a nested accordion."),
        ], AccordionOptions::default().state(&ui.state).multi().controls()))
        p class="nojs-note" { "Deep link: " a href="/accordion?open.faq=0,2" { "?open.faq=0,2" } ". Leave and come back: the open sections are remembered." }
    });
    (ui, body)
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

async fn combobox_page(ui: Ui, Query(pairs): Query<Vec<(String, String)>>) -> (Ui, Markup) {
    let (q, sel) = combobox_query(&pairs);
    let sel: Vec<&str> = sel.iter().map(String::as_str).collect();
    let hits: Vec<&str> = LANGS.iter().flat_map(|g| g.values().iter().copied())
        .filter(|l| !q.is_empty() && l.to_lowercase().contains(&q.to_lowercase())).collect();
    let body = page(&ui, "Combobox", html! {
        (ui.flash())
        // One swap root around the form and its results: the script searches as you type.
        div id="langs" data-nojs="swap" {
            (combobox_with(&ui, "q", "/combobox", ComboboxOptions::default()
                .query(&q).suggestions(&LANGS).results(&hits).selected(&sel).multi()
                .create("/combobox/new").label("Language").placeholder("Type a language")))
        }
        p class="nojs-note" { "Pick several: each result adds a chip, each chip's \u{d7} removes it, and the chips ride along with the next search. Type a language that is not here to get a Create row." }
    });
    (ui, body)
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

async fn list_page(ui: Ui, Query(p): Query<PageQuery>) -> Markup {
    const PER: usize = 8;
    const TOTAL: usize = 50;
    let current = p.page.unwrap_or(1).max(1);
    let rows: Vec<Markup> = (1..=(current * PER).min(TOTAL))
        .map(|n| html! { "Row " (n) })
        .collect();
    page(&ui, "Load-more list", html! { (pager_with(&ui, "/list", &rows, TOTAL, PagerOptions::default().page(current).per_page(PER))) })
}

#[derive(Deserialize, Default)]
struct Loading { loading: Option<u8> }

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
async fn table_page(ui: Ui, RawQuery(raw): RawQuery, Query(l): Query<Loading>) -> (Ui, Markup) {
    let t = TableQuery::parse(raw.as_deref().unwrap_or(""));
    let files = files(t.sort(&FILE_COLS), &t.filter.to_lowercase());
    const MENU: [MenuItem; 2] = [MenuItem::link("Open", "/table"), MenuItem::action("Delete", "/table/bulk").danger()];
    let rows: Vec<Row> = files.iter()
        .map(|f| Row::new(vec![html! { code { (f.0) } }, html! { (paged_table::thousands(f.1 as usize / 1024)) " KB" }, html! { (f.2) }])
            .key(&f.0).detail(html! { p { "A " (f.2) " of " (f.1) " bytes, in " code { (f.0.split('/').next().unwrap_or("")) } "." } }).menu(&MENU)).collect();
    let options = TableOptions::default().choose_columns().bulk("/table/bulk", &[("archive", "Archive"), ("delete", "Delete")])
        .csv("/table.csv").empty("No files match this filter.").loading(l.loading == Some(1));
    let body = page(&ui, "Table", html! {
        (ui.flash())
        p { "Click a header to sort, again to flip. Type to filter. Hide columns, tick rows for the bulk form, open a row's menu or its detail. The page size you pick is remembered for your next visit. Every state is a URL, including " a href="/table?loading=1" { "the loading one" } "." }
        (paged_table_with(&ui, "files", "/table", &FILE_COLS, &rows, files.len(), PagedTableOptions::default().query(&t).state(&ui.state).table(options)))
    });
    (ui, body)
}

/// The same rows as text/csv, for the sort and filter in the URL.
async fn table_csv(RawQuery(raw): RawQuery) -> impl IntoResponse {
    let t = TableQuery::parse(raw.as_deref().unwrap_or(""));
    let cols = t.cols(&FILE_COLS).unwrap_or_else(|| FILE_COLS.iter().map(|c| c.key).collect());
    let mut csv = cols.join(",") + "\n";
    for f in files(t.sort(&FILE_COLS), &t.filter.to_lowercase()) {
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
    const ACCOUNT: [Field; 2] = [Field::new("name", "Name", FieldKind::Text).required(), Field::new("email", "Email", FieldKind::Email).required()];
    let get = |k: &str| data.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()).unwrap_or("");
    [
        Step::new("Account", fields(&[FieldGroup::plain(&ACCOUNT)], FormOptions::default().values(data).errors(errors))).error(!errors.is_empty()),
        Step::new("Newsletter", html! {
            label { "Digest" select name="digest" { @for d in ["daily", "weekly", "never"] { option value=(d) selected[get("digest") == d] { (d) } } } }
            label { "Topics" input type="text" name="topics" value=(get("topics")) placeholder="rust, html"; }
        }).optional(),
        Step::new("Review", wizard::summary(state, "signup", &[
            ("Name", get("name"), 0), ("Email", get("email"), 0), ("Digest", get("digest"), 1), ("Topics", get("topics"), 1),
        ])),
    ]
}

fn wizard_view(ui: &Ui, data: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    page(ui, "Wizard", html! {
        (ui.flash())
        p { "Three steps, one form each. The server checks every step; the second can be skipped. Close the tab and come back to " a href="/wizard" { "/wizard" } ": you resume where you left off." }
        (wizard_with(ui, "signup", "/wizard", &wizard_steps(&ui.state, data, errors), &ui.state, WizardOptions::default().finish("Create account")))
    })
}

async fn wizard_page(ui: Ui, jar: CookieJar) -> (Ui, Markup) {
    let body = wizard_view(&ui, &wizard_data(&jar), &[]);
    (ui, body)
}

/// Check this step: answer 422 with the same step and its messages, or merge the fields into
/// the cookie (kept a week, so closing the browser loses nothing) and redirect to the next step.
async fn wizard_submit(ui: Ui, jar: CookieJar, headers: HeaderMap, Form(fields): Form<Vec<(String, String)>>) -> axum::response::Response {
    let field = |k: &str| fields.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    let step: usize = field("step").and_then(|v| v.parse().ok()).unwrap_or(0);
    let skip = field("skip") == Some("1");
    let cookie = headers.get("cookie").and_then(|v| v.to_str().ok()).unwrap_or("");
    // The posted step, not the URL's, is the one being checked.
    let ui = Ui { state: UiState::from_request("/wizard", &format!("step.signup={step}"), cookie), ..ui };
    let mut data = wizard_data(&jar);
    for (k, v) in fields.iter().filter(|(k, _)| !skip && k != "step" && k != "skip") {
        data.retain(|(n, _)| n != k);
        data.push((k.clone(), v.clone()));
    }
    let errors = if skip { Vec::new() } else { wizard_check(step, &data) };
    if !errors.is_empty() {
        return (axum::http::StatusCode::UNPROCESSABLE_ENTITY, wizard_view(&ui, &data, &errors)).into_response();
    }
    if step + 1 >= 3 {
        return (jar.remove(Cookie::from("wizard")), prg::<axum::body::Body>(&ui.state.link("step.signup", "0"), Some("Account created (well, the cookie was cleared)."))).into_response();
    }
    let value: String = data.iter().map(|(k, v)| format!("{k}={}", v.replace(' ', "+"))).collect::<Vec<_>>().join("&");
    let kept = Cookie::parse(format!("wizard={value}; Path=/; Max-Age=604800; SameSite=Lax")).expect("cookie");
    (jar.add(kept), prg::<axum::body::Body>(&ui.state.link("step.signup", &(step + 1).to_string()), None)).into_response()
}

/// The sign-up fields as posted: text values by name, and for each file field its name and size.
#[derive(Default)]
struct SignUp { values: Vec<(String, String)>, files: Vec<(String, usize)> }

impl SignUp {
    fn get(&self, k: &str) -> &str {
        self.values.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str()).unwrap_or("")
    }
}

fn form_view(ui: &Ui, inline: bool, v: &SignUp, errors: &[(&str, &str)]) -> Markup {
    let account = [
        Field::new("name", "Name", FieldKind::Text).required(),
        Field::new("email", "Email", FieldKind::Email).required(),
        Field::new("age", "Age", FieldKind::Number { min: 13, max: 120 }).required(),
        Field::new("handle", "Handle", FieldKind::Pattern { pattern: "[a-z0-9_]{3,16}", hint: "3–16 lowercase letters, digits or _" }).required(),
    ];
    let profile = [
        Field::new("bio", "Bio", FieldKind::Textarea { rows: 3 }).maxlength(160).help("Grows as you type where the browser supports it."),
        Field::new("avatar", "Avatar", FieldKind::File { accept: "image/png,image/jpeg", multiple: false }).help("PNG or JPEG."),
        Field::new("start", "Start date", FieldKind::Date { min: "2026-01-01", max: "2027-12-31" }),
        Field::new("call", "Best time to call", FieldKind::Time { min: "09:00", max: "17:00" }).help("Office hours, 09:00 to 17:00."),
    ];
    let layout = if inline { FormLayout::Inline } else { FormLayout::Stacked };
    page(ui, "Validated form", html! {
        (ui.flash())
        p { "Labels " @if inline { "beside the fields. " a href="/form" { "Put them above" } } @else { "above the fields. " a href="/form?layout=inline" { "Put them beside" } } "." }
        @if !errors.is_empty() { p class="nojs-error" { "Server-side checks failed. Browser validation passed, these rules only live on the server." } }
        (form_with(ui, "/form", &[FieldGroup::new("Account", &account), FieldGroup::new("Profile", &profile)],
            FormOptions::default().submit("Sign up").layout(layout).values(&v.values).errors(errors)))
    })
}

#[derive(Deserialize)]
struct FormQuery { layout: Option<String> }

async fn form_page(ui: Ui, Query(q): Query<FormQuery>) -> (Ui, Markup) {
    let body = form_view(&ui, q.layout.as_deref() == Some("inline"), &SignUp::default(), &[]);
    (ui, body)
}

/// A multipart post (the avatar is a file): server rules, then PRG with a flash or the form again.
async fn form_submit(ui: Ui, mut parts: axum::extract::Multipart) -> axum::response::Response {
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
    (axum::http::StatusCode::UNPROCESSABLE_ENTITY, form_view(&ui, false, &v, &errors)).into_response()
}

const COUNTER: CounterOptions = CounterOptions { min: Some(0), max: Some(20), step: 2, typed: true };

async fn counter_page(ui: Ui, jar: CookieJar) -> Markup {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    page(&ui, "Counter", html! {
        p { "Steps of two between 0 and 20. The buttons switch off at the ends; a typed value off the step or the bounds is refused by the browser and clamped by the server." }
        (counter_with(&ui, "/counter", n, COUNTER))
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
async fn swap_page(ui: Ui, jar: CookieJar, Query(q): Query<SwapQuery>) -> Markup {
    let n = q.n.unwrap_or(1);
    page(&ui, "Swap targets", html! {
        (ui.flash())
        p class="nojs-note" { "Neither control sits inside a swap root. " code { "data-nojs-target" } " names the root to update and " code { "data-nojs-swap" } " how; without the script both are ordinary navigations to the same URL." }
        p { "Count: " span id="count" data-nojs="swap" { (n) } " " a href={ "/swap?n=" (n + 1) } data-nojs-target="#count" { "Add one" }
            " · " a href={ "/swap?n=" (n + 10) } data-nojs-target="#count" data-nojs-push="false" { "Add ten, keep the URL" } }
        p { "Notes so far: " span id="note-count" { (notes_of(&jar).len()) } }
        form method="post" action="/swap" data-nojs-target="#log" data-nojs-swap="append" data-nojs-indicator="#saving" {
            input name="note" required placeholder="A note" aria-label="Note" autocomplete="off";
            button type="submit" class="nojs-primary" { "Add note" }
            " " span id="saving" class="nojs-note" hidden { "Saving…" }
        }
        ol id="log" data-nojs="swap" { @for note in notes_of(&jar) { li { (note) } } }
    })
}

#[derive(Deserialize)]
struct SwapForm { note: String }

/// An enhanced request (`Nojs-Enhance: 1`) gets only the new `<li>` inside an `#log` to append,
/// plus the note count marked `data-nojs-oob` so it updates wherever it is on the page; a plain
/// one gets Post/Redirect/Get to the full page, which shows both anyway.
async fn swap_submit(jar: CookieJar, headers: HeaderMap, Form(f): Form<SwapForm>) -> axum::response::Response {
    let note = f.note.replace('|', " ");
    let mut notes = notes_of(&jar);
    notes.push(note.clone());
    let jar = jar.add(Cookie::new("notes", notes.join("|")));
    if headers.contains_key("nojs-enhance") {
        return (jar, html! { ol id="log" { li { (note) } } span id="note-count" data-nojs-oob { (notes.len()) } }).into_response();
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

/// Tabs + form + flash. Everything survives a full navigation: tab in the `nojs-ui` cookie,
/// values in a `settings` cookie, flash in a one-shot cookie set by `prg`.
async fn settings_page(ui: Ui, jar: CookieJar) -> (Ui, Markup) {
    let Settings { name, notify } = settings_of(&jar);
    let (tab, here) = (ui.state.tab("settings").to_string(), format!("/settings?tab.settings={}", ui.state.tab("settings")));
    let values = [("tab".to_string(), tab), ("name".to_string(), name), ("notify".to_string(), notify.to_string())];
    const PROFILE: [Field; 2] = [Field::new("tab", "", FieldKind::Hidden), Field::new("name", "Display name", FieldKind::Text).required()];
    const NOTIFY: [Field; 3] = [Field::new("tab", "", FieldKind::Hidden), Field::new("name", "", FieldKind::Hidden), Field::new("notify", "Email me about releases", FieldKind::Checkbox)];
    let save = |id, fields| form_with(&ui, "/settings", &[FieldGroup::plain(fields)], FormOptions::default().submit("Save").values(&values).id(id));
    let body = page(&ui, "Settings", html! {
        (flash_with(&ui, ui.state.flash(), FlashOptions::default().dismiss(&here).auto_hide()))
        (tabs_with(&ui, "settings", &[Tab::new("Profile", save("profile", &PROFILE)), Tab::new("Notifications", save("notify", &NOTIFY))], TabsOptions::default().state(&ui.state)))
        p class="nojs-note" { "Go to " a href="/" { "the index" } " and come back: the open tab and the values are remembered. Saving with notifications off stacks a warning under the confirmation; the name " code { "admin" } " is refused with an alert. The confirmation fades after six seconds unless reduced motion is on." }
    });
    (ui, body)
}

#[derive(Deserialize)]
struct SettingsForm { name: String, #[serde(default)] notify: bool, tab: usize }

/// Saves and says so; a warning stacks when notifications go off; `admin` is refused.
async fn settings_submit(jar: CookieJar, Form(f): Form<SettingsForm>) -> (CookieJar, axum::response::Response) {
    let to = format!("/settings?tab.settings={}", f.tab);
    if f.name.trim().eq_ignore_ascii_case("admin") {
        return (jar, prg(&to, Some(&stack(&[(Level::Danger, "The name admin is reserved; nothing was saved.")]))));
    }
    let value = format!("{}|{}", f.name.replace('|', ""), u8::from(f.notify));
    let mut said = vec![(Level::Ok, "Settings saved.")];
    if !f.notify { said.push((Level::Warn, "You will not hear about releases.")); }
    (jar.add(Cookie::new("settings", value)), prg(&to, Some(&stack(&said))))
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
async fn inputs_page(ui: Ui, jar: CookieJar, Query(q): Query<Inputs>) -> (Ui, Markup) {
    let v = if q.country_q.is_some() { q } else { saved_inputs(&jar) };
    let accent = v.accent.as_deref().unwrap_or("#1f6f5f");
    let (lo, hi) = range::order(v.price_min.unwrap_or(20), v.price_max.unwrap_or(80));
    // `(value, label, icon)` tuples become options through `From`.
    let sizes = SIZES.map(SelectOption::from);
    let countries = COUNTRIES.map(|(group, cs)| (group, cs.map(SelectOption::from)));
    let groups = countries.each_ref().map(|(group, cs)| SelectGroup::new(group, cs));
    let body = page(&ui, "Select, range, colour", html! {
        (ui.flash())
        form id="inputs" data-nojs="swap" class="nojs-form" method="post" action="/inputs" {
            div class="nojs-field" { label for="size" { "Size" } (select(&ui, "size", &[SelectGroup::flat(&sizes)], v.size.as_deref().unwrap_or("m"))) }
            div class="nojs-field" { label for="country" { "Country" }
                (select_with(&ui, "country", &groups, v.country.as_deref().unwrap_or("es"), SelectOptions::default().search("/inputs", v.country_q.as_deref().unwrap_or("")))) }
            div class="nojs-field" { label for="f-volume" { "Volume" } (range_with(&ui, "volume", v.volume.unwrap_or(40), RangeOptions::default().step(5))) }
            div class="nojs-field" { label for="f-price_min" { "Price" } (range::range_pair_with(&ui, "price", (lo, hi), RangeOptions::default().step(5))) }
            div class="nojs-field" { label for="f-accent" { "Accent" } (color_with(&ui, "accent", accent, ColorOptions::default().presets(&ACCENTS).alpha(v.alpha.unwrap_or(100)))) }
            button type="submit" class="nojs-primary" { "Save" }
        }
        p class="nojs-note" { "Without the enhancement script the outputs and the swatch show the last saved values and update on submit, and the country filter needs its button." }
    });
    (ui, body)
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

/// Toasts come back from a post like a flash: the one-shot cookie, several at once.
async fn toast_page(ui: Ui) -> (Ui, Markup) {
    let body = page(&ui, "Toasts", html! {
        p { "Each button posts, the server redirects back, and the answer shows in the corner. Calm ones fade after five seconds (hover to keep them); errors stay until dismissed." }
        form method="post" action="/toast" {
            button type="submit" name="kind" value="ok" class="nojs-primary" { "Send invite" } " "
            button type="submit" name="kind" value="warn" { "Copy link" } " "
            button type="submit" name="kind" value="danger" { "Sync now" } " "
            button type="submit" name="kind" value="all" { "All three" }
        }
        (toasts_with(&ui, ui.state.flash(), ToastOptions::default().dismiss("/toast")))
    });
    (ui, body)
}

#[derive(Deserialize)]
struct ToastForm { kind: String }

async fn toast_submit(Form(f): Form<ToastForm>) -> axum::response::Response {
    let all = [(Level::Ok, "Invite sent to ada@example.org."), (Level::Warn, "Link copied; it expires in an hour."), (Level::Danger, "Sync failed: the server did not answer.")];
    let picked: Vec<_> = all.into_iter().filter(|(l, _)| f.kind == "all" || l.as_str() == f.kind).collect();
    prg("/toast", Some(&stack(&picked)))
}

/// A sidebar on wide screens, a drawer on narrow ones, and breadcrumbs above the content.
async fn nav_page(ui: Ui) -> Markup {
    let links = html! { ul {
        li { a href="/nav" aria-current="page" { "Overview" } }
        li { a href="/table" { "Files" } } li { a href="/dashboard" { "Reports" } } li { a href="/settings" { "Settings" } }
    } };
    page(&ui, "Drawer and breadcrumbs", drawer_with(&ui, "site", "Menu", links, html! {
        (breadcrumbs(&ui, &[("Home", "/"), ("Projects", "/nav"), ("axum-nojs", "")]))
        p { "Wider than 60rem the navigation is a sidebar; narrower, the menu button opens it as a drawer. Escape or a click outside closes it." }
        p { "A long trail folds its middle so both ends stay readable:" }
        (breadcrumbs(&ui, &[("Home", "/"), ("Projects", "/nav"), ("axum-nojs", "/nav"), ("Components", "/"), ("Navigation", "/nav"), ("Breadcrumbs", "")]))
        p class="nojs-note" { "Server-opened: " a href="/nav?dialog=site" { "?dialog=site" } }
    }, DrawerOptions::default().title("axum-nojs").sidebar().open(ui.state.dialog() == Some("site"))))
}

#[derive(Deserialize)]
struct DashboardQuery { orders: Option<String> }

/// Stat cards over a list that may be empty (`?orders=none`).
async fn dashboard_page(ui: Ui, Query(q): Query<DashboardQuery>) -> Markup {
    let none = q.orders.as_deref() == Some("none");
    page(&ui, "Stats and empty states", html! {
        div class="nojs-stat-grid" {
            (stat_with(&ui, "Visitors", "12,480", StatOptions::default().delta("+8.2%").note("last 7 days")))
            (stat_with(&ui, "Orders", if none { "0" } else { "3" }, StatOptions::default().delta(if none { "-3" } else { "0" })))
            (stat_with(&ui, "Error rate", "0.4%", StatOptions::default().delta("-0.2 pt").down_is_good().href("/table")))
            (stat_with(&ui, "p95 latency", "38 ms", StatOptions::default().delta("+6 ms").down_is_good()))
        }
        h2 { "Recent orders" }
        @if none {
            (empty_state_with(&ui, "No orders yet", EmptyOptions::default().icon("\u{1f4e6}")
                .text(html! { "Orders show up here as soon as a customer checks out." })
                .link("Show sample orders", "/dashboard")))
        } @else {
            ul { li { "#1042, Ada Lovelace, 3 items" } li { "#1041, Grace Hopper, 1 item" } li { "#1040, Alan Turing, 2 items" } }
            p class="nojs-note" { a href="/dashboard?orders=none" { "See the empty state" } }
        }
    })
}

/// Every demo page as a command, plus a few deep links.
fn commands() -> Vec<Command<'static>> {
    COMPONENTS.iter().map(|(href, title, ..)| Command::new(title, href).group("Components"))
        .chain([
            Command::new("Notification settings", "/settings?tab.settings=1").group("Shortcuts").keywords("email releases"),
            Command::new("Largest files", "/table?sort=size&dir=desc").group("Shortcuts").keywords("sort size big"),
            Command::new("Open the delete dialog", "/dialog?dialog=confirm").group("Shortcuts").keywords("account remove"),
        ]).collect()
}

#[derive(Deserialize)]
struct PaletteQuery { q: Option<String> }

/// An exact command name redirects; anything else lists the matches.
async fn palette_page(ui: Ui, Query(p): Query<PaletteQuery>) -> axum::response::Response {
    let cmds = commands();
    let q = p.q.as_deref().filter(|q| !q.trim().is_empty());
    if let Some(c) = q.and_then(|q| palette::exact(&cmds, q)) { return Redirect::to(c.href).into_response(); }
    let options = q.map_or(PaletteOptions::default(), |q| PaletteOptions::default().query(q));
    page(&ui, "Command palette", html! {
        p { "Open it with the button or the access key, type, pick a suggestion and press Enter. An exact name goes straight to the page; anything else lists what matches." }
        (command_palette_with(&ui, "cmd", "/palette", &cmds, options))
    }).into_response()
}

/// Three sections declared slowest first, so out-of-order arrival is visible.
async fn stream_page(ui: Ui) -> Streamed {
    let sections = [("slow", 2000), ("medium", 800), ("fast", 100)];
    let theme = ui.theme;
    let page = Streamed::page(&ui, "Streaming", theme, html! {
        (toolbar(&ui, theme, true))
        h1 { "Streaming" }
        p { @if ui.has(Cap::StreamingDsd) { "Sections arrive out of order into named slots." }
            @else { "This browser has no declarative shadow DOM: sections stream in document order." } }
        @for (id, ms) in sections {
            (slot(&ui, id, html! { section class="nojs-stream-section nojs-stream-pending" {
                (skeleton_with(&ui, 2, SkeletonOptions::default().label(&format!("Loading {id} ({ms} ms)")).heading()))
            } }))
        }
    });
    sections.into_iter().fold(page, |page, (id, ms)| page.fill(id, section(id, ms)))
}

async fn section(id: &'static str, ms: u64) -> Markup {
    tokio::time::sleep(Duration::from_millis(ms)).await;
    html! { section class="nojs-stream-section" { strong { (id) } " arrived after " (ms) " ms." } }
}

/// What the server believes about this browser, one row per capability.
async fn caps_page(ui: Ui) -> Markup {
    let probed = ui.has(Cap::Probed);
    page(&ui, "Capabilities", html! {
        @if probed { p { "Beacons have fired. Rows below drive which markup every component emits." } }
        @else { p class="nojs-error" { "Not probed yet: the beacons fire while this page loads. Reload to see the result." } }
        table class="nojs-caps-table" {
            thead { tr { th { "Capability" } th { "Supported" } th { "Effect" } th { "@supports test" } } }
            tbody { @for cap in Cap::ALL {
                tr {
                    td { code { (cap.name()) } }
                    td { @if ui.has(cap) { span class="nojs-yes" { "yes" } } @else if probed { span class="nojs-no" { "no" } } @else { span class="nojs-note" { "unknown" } } }
                    td { (cap.description()) }
                    td { @match cap.supports() { Some(t) => code { (t) }, None => span class="nojs-note" { "always" } } }
                }
            } }
        }
        p class="nojs-note" { "Cookies: " @for n in ui.names() { code { "nojs-cap-" (n) } " " } }
        p class="nojs-note" { "To view any page as another browser, add " code { "?caps=popover,anchor" } " to its URL: the query wins over the cookies." }
    })
}

#[derive(Deserialize)]
struct ThemeForm { theme: String }

async fn theme_submit(jar: CookieJar, headers: HeaderMap, Form(f): Form<ThemeForm>) -> (CookieJar, Redirect) {
    let theme = Theme::parse(&f.theme);
    (jar.add(Cookie::new(THEME_COOKIE, theme.as_str().to_string())), Redirect::to(&back_to(&headers)))
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
        let tag = axum_nojs::enhance::script_tag().into_string();
        for path in PATHS {
            let modern = Cap::ALL.map(|c| format!("nojs-cap-{}=1", c.name())).join("; ");
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
    async fn whole_pages_are_compressed_and_streams_are_not() {
        let gz = |path: &str| Request::get(path).header("accept-encoding", "gzip").body(Body::empty()).unwrap();
        let res = router().oneshot(gz("/table")).await.unwrap();
        assert_eq!(res.headers()["content-encoding"], "gzip", "a whole page is compressed");
        let res = router().oneshot(gz("/stream")).await.unwrap();
        assert!(res.headers().get("content-encoding").is_none(), "a stream keeps its chunks");
        let res = router().oneshot(Request::get("/table").body(Body::empty()).unwrap()).await.unwrap();
        assert!(res.headers().get("content-encoding").is_none());
        assert!(axum::body::HttpBody::size_hint(res.body()).exact().is_some(), "plain pages carry a Content-Length");
    }

    #[tokio::test]
    async fn enhanced_requests_get_the_page_without_its_stylesheet() {
        let get = |enhanced: bool| {
            let req = Request::get("/tabs?tab.demo=1");
            let req = if enhanced { req.header("nojs-enhance", "1") } else { req };
            router().oneshot(req.body(Body::empty()).unwrap())
        };
        let full = get(false).await.unwrap();
        assert_eq!(full.headers()["vary"], "nojs-enhance, cookie");
        let full = String::from_utf8(axum::body::to_bytes(full.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
        let slim = get(true).await.unwrap();
        assert_eq!(slim.headers()["vary"], "nojs-enhance, cookie");
        let slim = String::from_utf8(axum::body::to_bytes(slim.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
        assert!(full.contains("<style>") && !slim.contains("<style>"), "the stylesheet stays home");
        assert!(slim.contains("id=\"nojs-tabs-demo\"") && slim.contains("<title>"), "the swap root and title are still there");
        assert!(slim.len() * 3 < full.len(), "slim is {} of {} bytes", slim.len(), full.len());
    }

    #[tokio::test]
    async fn enhancement_script_is_served_immutable() {
        let req = Request::get(axum_nojs::enhance::script_url()).body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(res.headers()["content-type"], "text/javascript; charset=utf-8");
        assert!(res.headers()["cache-control"].to_str().unwrap().contains("immutable"));
        let body = axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap();
        assert_eq!(body, axum_nojs::enhance::served().as_bytes());
    }

    #[tokio::test]
    async fn caps_beacon_sets_one_cookie_per_flag() {
        let req = Request::get("/nojs/caps?flag=popover").body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 204);
        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(cookie.starts_with("nojs-cap-popover=1;"), "{cookie}");
        let req = Request::get("/nojs/caps?flag=nope").body(Body::empty()).unwrap();
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
        let chunks = frames("/stream", "nojs-cap-probed=1; nojs-cap-streaming_dsd=1").await;
        assert!(chunks.len() >= 6, "expected head + shell + 3 fills + suffix, got {}", chunks.len());
        assert!(chunks[0].ends_with("</head>") && chunks[0].contains("<style>"), "the head, stylesheet included, goes first");
        assert!(chunks[1].contains("<template shadowrootmode=\"open\">"));
        assert!(chunks[1].contains("<slot name=\"slow\">"));
        let order: Vec<&str> = chunks[2..5].iter().map(|c| c.split("slot=\"").nth(1).unwrap().split('"').next().unwrap()).collect();
        assert_eq!(order, ["fast", "medium", "slow"]);
        assert!(chunks.last().unwrap().ends_with("</script></body></html>"), "suffix carries the enhancement tag");
    }

    #[tokio::test]
    async fn stream_fallback_is_in_document_order() {
        let chunks = frames("/stream", "nojs-cap-probed=1").await;
        let html = chunks.concat();
        assert!(!html.contains("<template") && !html.contains("<slot"));
        let pos = |s: &str| html.find(s).unwrap();
        assert!(pos("slow</strong>") < pos("medium</strong>") && pos("medium</strong>") < pos("fast</strong>"));
        assert!(chunks.len() >= 5, "streamed in pieces, got {}", chunks.len());
        assert!(chunks[0].ends_with("</head>"), "the head goes first");
    }

    #[tokio::test]
    async fn palette_exact_name_redirects_and_toast_posts_stack() {
        let res = router().oneshot(Request::get("/palette?q=largest%20FILES").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), 303);
        assert_eq!(res.headers().get("location").unwrap(), "/table?sort=size&dir=desc");
        let res = router().oneshot(Request::get("/palette?q=zzz").body(Body::empty()).unwrap()).await.unwrap();
        assert_eq!(res.status(), 200);
        let req = Request::post("/toast").header("content-type", "application/x-www-form-urlencoded").body(Body::from("kind=all")).unwrap();
        let res = router().oneshot(req).await.unwrap();
        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(cookie.starts_with("nojs-flash=ok%3AInvite") && cookie.contains("%0Awarn%3A") && cookie.contains("%0Adanger%3A"), "{cookie}");
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
        assert!(cookies.iter().any(|c| c.starts_with("nojs-flash=ok%3ASettings%20saved.")), "{cookies:?}");
        assert!(cookies.iter().any(|c| c.starts_with("settings=Ada")), "{cookies:?}");
        // GET the redirect target: flash shown and cleared, tab persisted to nojs-ui, values filled in.
        let req = Request::get("/settings?tab.settings=1").header("cookie", "nojs-flash=Settings%20saved.; settings=Ada|1").body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        let cookies: Vec<String> = res.headers().get_all("set-cookie").iter().map(|v| v.to_str().unwrap().to_string()).collect();
        assert!(cookies.iter().any(|c| c.starts_with("nojs-ui=tab.settings=1;")), "{cookies:?}");
        assert!(cookies.iter().any(|c| c.starts_with("nojs-flash=; Path=/; Max-Age=0")), "{cookies:?}");
        let html = String::from_utf8(axum::body::to_bytes(res.into_body(), usize::MAX).await.unwrap().to_vec()).unwrap();
        assert!(html.contains("Settings saved.") && html.contains("value=\"Ada\"") && html.contains("checked"));
        assert!(html.contains("<details name=\"settings\" open>") && html.contains("href=\"/settings?tab.settings=0\""));
        // Coming back with only the cookie: the tab is still open, nothing is rewritten.
        let req = Request::get("/settings").header("cookie", "nojs-ui=tab.settings=1").body(Body::empty()).unwrap();
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
        assert!(old.contains("nojs-caps"), "unknown browser gets beacons");
        let new = body("/dialog", "nojs-cap-probed=1; nojs-cap-invokers=1").await;
        assert!(new.contains("commandfor=\"confirm\"") && !new.contains("href=\"#confirm\""));
        assert!(!new.contains("class=\"nojs-caps\""), "probed browser gets no beacons");
        assert!(body("/popover", "").await.contains("nojs-popover-details"));
        assert!(body("/tabs", "nojs-cap-details_content=1").await.contains("nojs-tabs-panel"));
        assert!(body("/tabs", "").await.contains("nojs-accordion-body"));
    }
}
