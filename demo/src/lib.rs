//! Demo server: one route per component. Handlers only parse input and call `axum-nojs`.
//! The binary in `main.rs` serves [`router`]; tests and `axum-nojs-test` call it directly.

use axum::{
    Form, Router,
    extract::Query,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_nojs::calendar::Date;
use axum_nojs::layout::{Palette, Tokens};
use axum_nojs::prelude::*;
use axum_nojs::{
    Row, Streamed,
    table::Table,
    wizard::{Posted, Wizard},
};
use serde::{Deserialize, Serialize};
use std::{sync::LazyLock, time::Duration};
use tower_http::compression::{
    CompressionLayer,
    predicate::{DefaultPredicate, Predicate},
};

/// Every demo path the no-script test and the screenshot test visit.
pub const PATHS: [&str; 26] = [
    "/",
    "/caps",
    "/button?loading=1",
    "/field?email=ada",
    "/card",
    "/layout",
    "/calendar?month.day=2026-09&day=2026-09-17",
    "/stream",
    "/settings",
    "/dialog?dialog=confirm",
    "/popover",
    "/tabs?tab.demo=1",
    "/accordion?open.faq=0,2&open.faq-more=0",
    "/combobox?q=r&sel=Zig",
    "/list?page=2",
    "/form",
    "/form?layout=inline",
    "/counter",
    "/inputs",
    "/table?sort=size&dir=desc&q=a&per.files=5&page=2&cols=name,size",
    "/wizard?step.signup=1",
    "/swap?n=3",
    "/toast",
    "/nav",
    "/dashboard?orders=none",
    "/palette?q=ta",
];

/// The whole demo app.
pub fn router() -> Router {
    // Load syntect's grammars and highlight the snippets now, not on the first page view.
    std::thread::spawn(|| LazyLock::force(&CODE));
    Router::new()
        .route("/", get(index))
        .route("/caps", get(caps_page))
        .route("/button", get(button_page))
        .route("/field", get(field_page))
        .route("/card", get(card_page))
        .route("/layout", get(layout_page))
        .route("/calendar", get(calendar_page))
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
        .merge(axum_nojs::caps::router())
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

/// Every component in the index: path, title (what each route passes to `page`), group, the
/// platform features it is built on, and what it is for in plain words.
const COMPONENTS: [(&str, &str, &str, &str, &str); 24] = [
    (
        "/calendar",
        "Calendar",
        "Input",
        "<table>, links or radios, aria-current=date, :has(:checked), ?month=",
        "A month you can page through and pick a day from.",
    ),
    (
        "/button",
        "Buttons and badges",
        "Primitives",
        "<button>, invoker commands, popovertarget, aria-busy, inline <svg>",
        "The button every other component is built from, with badges and icons.",
    ),
    (
        "/field",
        "Fields",
        "Primitives",
        "<label>, aria-describedby, :user-invalid, role=switch, <fieldset>",
        "A labelled input, checkbox, switch and radio group, with help and errors.",
    ),
    (
        "/card",
        "Cards and avatars",
        "Primitives",
        "grid, <img alt=\"\">, loading=lazy",
        "A box with a header, body and footer, and a picture that falls back to initials.",
    ),
    (
        "/layout",
        "Layout",
        "Primitives",
        "flex gap, flex-wrap, repeat(auto-fill), custom properties",
        "Stack, cluster, grid and split: even spacing with no margins, and columns that wrap on their own.",
    ),
    (
        "/palette",
        "Command palette",
        "Navigation",
        "popover, <datalist>, <search>, accesskey, GET + 303",
        "Jump to any page by typing its name.",
    ),
    (
        "/nav",
        "Drawer and breadcrumbs",
        "Navigation",
        "<dialog>, invoker commands, closedby, @starting-style, <details>",
        "A sidebar that turns into a drawer on small screens, with a trail back up.",
    ),
    (
        "/toast",
        "Toasts",
        "Feedback",
        "position: fixed, role=alert, CSS fade, PRG",
        "Short messages in the corner after a form is sent.",
    ),
    (
        "/dashboard",
        "Stats and empty states",
        "Feedback",
        "auto-fit grid, form POST",
        "Numbers with how they changed, and what to show when there is nothing yet.",
    ),
    (
        "/dialog",
        "Dialog",
        "Overlays",
        "<dialog>, closedby, invoker commands, form footer",
        "Ask before doing something that cannot be undone.",
    ),
    (
        "/popover",
        "Popover menu",
        "Overlays",
        "popover, anchor positioning, nested popover, form actions",
        "A menu of links and actions that opens over the page.",
    ),
    (
        "/tabs",
        "Tabs",
        "Disclosure",
        "<details name>, ::details-content, view-transition-name, grid",
        "Several panels in one place, one open at a time.",
    ),
    (
        "/accordion",
        "Accordion",
        "Disclosure",
        "<details name>, ::details-content, interpolate-size",
        "Questions that open to their answers.",
    ),
    (
        "/combobox",
        "Combobox",
        "Input",
        "<datalist>, <optgroup>, <search>, aria-live",
        "Search a list and pick one item or several.",
    ),
    (
        "/form",
        "Validated form",
        "Input",
        ":user-invalid, <fieldset>, <output> counters, field-sizing, multipart, PRG",
        "Fields the browser checks first and the server checks again.",
    ),
    (
        "/wizard",
        "Wizard",
        "Input",
        "one form per step, PRG, formnovalidate, <progress>, UiState",
        "A long form split into steps you can leave and come back to.",
    ),
    (
        "/inputs",
        "Select, range, colour",
        "Input",
        "<selectedcontent>, <optgroup>, formmethod, two-thumb range, color-mix()",
        "Pick a size, a country, a volume, a price range and a colour.",
    ),
    (
        "/counter",
        "Counter",
        "Server state",
        "form POST + cookie, type=number, disabled",
        "A number that goes up and down within limits.",
    ),
    (
        "/settings",
        "Settings",
        "Server state",
        "UiState, PRG + flash, role=alert, CSS auto-hide",
        "Tabs of settings that stay where you left them.",
    ),
    (
        "/list",
        "Load-more list",
        "Server state",
        "links + view transitions",
        "A long list shown a page at a time.",
    ),
    (
        "/table",
        "Table",
        "Server state",
        "sort links, <search> filter, form= checkboxes, ?cols=, <details> rows, sticky header, ?page=n",
        "Sort, filter, page through and select rows of data.",
    ),
    (
        "/caps",
        "Capabilities",
        "Server state",
        "@supports beacons + cookie",
        "What the server knows this browser can do.",
    ),
    (
        "/stream",
        "Streaming",
        "Server state",
        "declarative shadow DOM slots, skeleton placeholders, aria-busy",
        "A page that sends its fast parts first.",
    ),
    (
        "/swap",
        "Swap targets",
        "Server state",
        "data-nojs-target, data-nojs-swap, data-nojs-oob, data-nojs-indicator, data-nojs-push, Nojs-Enhance header",
        "Update one part of the page without reloading it.",
    ),
];
const GROUPS: [&str; 7] = [
    "Primitives",
    "Overlays",
    "Disclosure",
    "Navigation",
    "Input",
    "Feedback",
    "Server state",
];

/// The second palette from `docs/theming.md`: warm paper, copper primary, amber in the dark.
const LINEN: Tokens = Tokens {
    light: Palette {
        bg: "#f4efe6",
        fg: "#1d1a17",
        muted: "#5d574f",
        line: "#d6cdbf",
        surface: "#fffdf9",
        card: "#fffdf9",
        popover: "#fffdf9",
        secondary: "#ebe3d6",
        accent: "#ebe3d6",
        on_accent: "#1d1a17",
        primary: "#8a3b12",
        on_primary: "#ffffff",
        input: "#d6cdbf",
        ring: "#b5764f",
        danger: "#a0261c",
        ok: "#2f6b3a",
        warn: "#7a5500",
    },
    dark: Palette {
        bg: "#161311",
        fg: "#ece6dc",
        muted: "#a59c90",
        line: "#3a332c",
        surface: "#1f1b18",
        card: "#1f1b18",
        popover: "#1f1b18",
        secondary: "#2b2521",
        accent: "#2b2521",
        on_accent: "#ece6dc",
        primary: "#e8965a",
        on_primary: "#1a0f06",
        input: "#4a4038",
        ring: "#a8683a",
        danger: "#ff8f85",
        ok: "#8fd39a",
        warn: "#f0c060",
    },
    radius: "3px",
    space: "8px",
};

/// The row above every title: the way back to the index (not on the index) and the theme switch.
fn toolbar(ui: &Ui, back: bool) -> Markup {
    html! { nav class="nojs-toolbar" {
        @if back { a class="nojs-back" href="/" { "All components" } } @else { span {} }
        (ui.theme_toggle("/theme"))
    } }
}

/// What every page shows around its body: the toolbar and the title, and on a component page
/// what it is for and built on, then the body on a stage with the code that drew it underneath.
fn shell(ui: &Ui, title: &str, body: Markup) -> Markup {
    let component = COMPONENTS.iter().find(|c| c.1 == title);
    let Some(c) = component else {
        return html! { (toolbar(ui, false)) h1 { (title) } (body) };
    };
    html! {
        (toolbar(ui, true))
        h1 { (title) }
        p class="nojs-lede" { (c.4) }
        p class="nojs-built" { "Built on " @for f in c.3.split(", ") { code { (f) } " " } }
        // The live component and the code that drew it, joined as one plate.
        div class="nojs-plate" {
            div class="nojs-stage" { (body) }
            figure class="nojs-snippet" {
                figcaption { span { "demo/src/lib.rs" } span { "The code behind the component above" } }
                pre { code { @if let Some((_, code)) = CODE.iter().find(|h| h.0 == c.0) { (maud::PreEscaped(code)) } } }
            }
        }
    }
}

/// This file, so every component page can show the code that draws it.
const SOURCE: &str = include_str!("lib.rs");

/// The lines of this file between `// code: <href>` and `// end code`, dedented, one block per
/// pair: a page shows the component call, and the handler or helper next to it if it has one.
fn snippet(href: &str) -> String {
    let open = format!("// code: {href}");
    let mut lines = SOURCE.lines();
    let mut blocks = Vec::new();
    while lines.any(|l| l.trim() == open) {
        let block: Vec<&str> = lines
            .by_ref()
            .take_while(|l| l.trim() != "// end code")
            .collect();
        let indent = block
            .iter()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.len() - l.trim_start().len())
            .min()
            .unwrap_or(0);
        blocks.push(
            block
                .iter()
                .map(|l| l.get(indent..).unwrap_or(""))
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }
    blocks.join("\n\n")
}

/// Every component's snippet, highlighted once at startup: `(href, html)`.
static CODE: LazyLock<Vec<(&str, String)>> = LazyLock::new(|| {
    COMPONENTS
        .iter()
        .map(|c| (c.0, highlight(&snippet(c.0)).into_string()))
        .collect()
});

/// Rust source as spans the stylesheet colours, parsed by syntect's Rust grammar on the
/// server (no script). Only seven classes, `nojs-hl-{k,s,n,c,m,f,t}`, coloured with tokens
/// in `layout.rs`, instead of syntect's own HTML with a class per scope and a bundled theme.
fn highlight(code: &str) -> Markup {
    use syntect::{
        easy::ScopeRangeIterator,
        parsing::{ParseState, ScopeStack, SyntaxSet},
        util::LinesWithEndings,
    };
    static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
    /// Scope prefix to class: keyword, string, number, comment, macro, function, type.
    const CLASSES: [(&str, &str); 11] = [
        ("comment", "c"),
        ("string", "s"),
        ("constant.numeric", "n"),
        ("support.macro", "m"),
        ("support.function", "f"),
        ("entity.name.function", "f"),
        ("keyword.control", "k"),
        ("storage", "k"),
        ("support.type", "t"),
        ("entity.name", "t"),
        ("constant.other", "t"),
    ];
    let class = |stack: &ScopeStack| {
        stack
            .as_slice()
            .iter()
            .rev()
            .find_map(|scope| {
                let name = scope.build_string();
                CLASSES
                    .iter()
                    .find(|(prefix, _)| name.starts_with(prefix))
                    .map(|c| c.1)
            })
            .unwrap_or("")
    };
    let rust = SYNTAXES
        .find_syntax_by_extension("rs")
        .expect("syntect ships Rust");
    let (mut state, mut stack) = (ParseState::new(rust), ScopeStack::new());
    let mut parts: Vec<(&str, String)> = Vec::new();
    for line in LinesWithEndings::from(code) {
        let ops = state.parse_line(line, &SYNTAXES).unwrap_or_default();
        for (range, op) in ScopeRangeIterator::new(&ops, line) {
            stack.apply(op).ok();
            let kind = class(&stack);
            match parts.last_mut() {
                Some((last, text)) if *last == kind => text.push_str(&line[range]),
                _ if range.is_empty() => {}
                _ => parts.push((kind, line[range].to_string())),
            }
        }
    }
    html! { @for (kind, text) in parts { @if kind.is_empty() { (text) } @else { span class={ "nojs-hl-" (kind) } { (text) } } } }
}

fn page(ui: &Ui, title: &str, body: Markup) -> Page {
    ui.page(title, shell(ui, title, body))
}

// ---------- routes ----------

async fn index(ui: Ui) -> Page {
    let linen = ui.param("palette") == Some("linen");
    let page = page(
        &ui,
        "Components",
        html! {
            p class="nojs-lede" { (COMPONENTS.len()) " interactive components for Axum and Maud that work with JavaScript turned off. The HTML platform and plain form posts do the work. Each page loads one optional script, " code { "/nojs/enhance.js" } ", which updates the same markup in place instead of reloading. Block it and every page still works." }
            @if !ui.has(Cap::Probed) { p class="nojs-note" { "First visit: this page is the fallback variant. Reload and the server will know your browser." } }
            p class="nojs-note" { "Theme: " @if linen { a href="/" { "neutral" } " · linen and copper" } @else { "neutral · " a href="/?palette=linen" { "linen and copper" } } ", see " code { "docs/theming.md" } }
            div class="nojs-index" { @for group in GROUPS {
                h2 { (group) }
                ul { @for (href, title, _, feats, what) in COMPONENTS.iter().filter(|c| c.2 == group) {
                    li { a href=(href) { (title) } div { p { (what) } span { @for f in feats.split(", ") { code { (f) } " " } } } }
                } }
            } }
            // Idle-time fetch of every component page, so the click is served from cache.
            @for (href, ..) in COMPONENTS { link rel="prefetch" href=(href); }
        },
    );
    if linen { page.tokens(&LINEN) } else { page }
}

async fn dialog_page(ui: Ui) -> Page {
    page(
        &ui,
        "Dialog",
        html! {
            (ui.flash())
            // code: /dialog
            (ui.dialog("Delete account").id("confirm").title("Delete account?").small().danger()
                .confirm("Delete account", "/dialog/delete").cancel("Keep it")
                .body(html! {
                    p { "This cannot be undone. Everything you wrote goes with it." }
                    (ui.input("reason", "Tell us why (optional)").placeholder("Moving on"))
                }))
            // end code
            p class="nojs-note" { "Opened by an invoker button; the footer is a real form posting to " code { "/dialog/delete" } " with a hidden " code { "returns_to" } " so the server comes back here. Server-opened: " a href="/dialog?dialog=confirm" { "?dialog=confirm" } }
        },
    )
}

#[derive(Deserialize)]
struct DeleteForm {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    returns_to: String,
}

/// The confirm form's target: only ever redirects to a local path from `returns_to`.
async fn dialog_delete(ui: Ui, Form(f): Form<DeleteForm>) -> Redirect {
    let local = f.returns_to.starts_with('/') && !f.returns_to.starts_with("//");
    let msg = if f.reason.is_empty() {
        "Account deleted (not really)".to_string()
    } else {
        format!("Account deleted (not really). Reason: {}", f.reason)
    };
    ui.redirect(if local { &f.returns_to } else { "/dialog" })
        .flash(&msg)
}

async fn popover_page(ui: Ui) -> Page {
    page(
        &ui,
        "Popover menu",
        html! {
            (ui.flash())
            div class="nojs-popover-row" {
                // code: /popover
                (ui.menu("Account")
                    .heading("Signed in as Ada")
                    .link("Profile", "/popover").icon("@").shortcut("g p")
                    .link("Settings", "/settings").icon("\u{2699}").shortcut("g s")
                    .link("Billing", "/popover").icon("$").disabled()
                    .separator()
                    .submenu("Theme", [("Light", "/popover?theme=light"), ("Dark", "/popover?theme=dark")]).icon("\u{25d0}")
                    .separator()
                    .action("Sign out", "/popover/signout").icon("\u{2192}").danger())
                (ui.menu("More").align_end()
                    .link("Documentation", "/").icon("?")
                    .action("Clear cache", "/popover/signout"))
                // end code
            }
            p class="nojs-note" { "Links, a heading, a disabled item, a submenu that is another popover, and a " code { "<form method=\"post\">" } " action. Click outside or press Escape to close; the second menu opens end-aligned." }
        },
    )
}

/// A menu action: Post/Redirect/Get back to the menu page with a flash.
async fn popover_signout(ui: Ui) -> Redirect {
    ui.redirect("/popover").flash("Signed out (not really)")
}

async fn tabs_page(ui: Ui) -> Page {
    page(
        &ui,
        "Tabs",
        html! {
            // Hovering or focusing a tab title fetches it early; the click reuses the answer.
            // code: /tabs
            div data-nojs-prefetch { (ui.tabs("demo")
                .tab("Install", html! { p { code { "cargo add axum-nojs maud axum" } } })
                .tab("Use", html! { p { "Call a function, get " code { "Markup" } ", send it." } }).badge(3)
                // Lazy: the body is rendered only by the request that opens the tab.
                .lazy("Why", || html! { p { "Because the platform can do this without script now. (Rendered on demand.)" } })
                .select_below()) }
            // end code
            p class="nojs-note" { "Deep link: " a href="/tabs?tab.demo=2" { "?tab.demo=2" } ". Leave and come back: the tab is remembered. The third tab is lazy; under 40rem the strip becomes a select." }
            h2 { "Vertical" }
            (ui.tabs("side").vertical()
                .tab("General", html! { p { "Titles stack on the left; the open panel sits beside them." } })
                .tab("Members", html! { p { "Twelve members." } }).badge(12)
                .tab("Danger zone", html! { p { "Nothing here is destructive." } }))
        },
    )
}

async fn accordion_page(ui: Ui) -> Page {
    page(
        &ui,
        "Accordion",
        html! {
            // code: /accordion
            (ui.accordion("faq").multi().controls()
                .item("Does this need JavaScript?", html! { p { "No. Turn it off and reload: every control still works through links and form posts. The one script on the page only swaps the answer in place instead of reloading." } })
                    .icon("\u{1F50D}").summary("Every open and close is a link the server answers.")
                .item("Does it animate?", html! { p { "Yes, via ::details-content transitions where supported." } })
                    .icon("\u{1F3AC}").summary("Height animates to auto in Chrome; elsewhere it snaps.")
                .item("Can several be open?", html! {
                    p { "Yes: this group is " code { "multi" } ", so " code { "?open.faq=0,2" } " keeps two open. A body can hold another group:" }
                    (ui.accordion("faq-more")
                        .item("Nested", html! { p { "Its own key, " code { "open.faq-more" } "." } })
                        .item("Exclusive", html! { p { "This inner group opens one at a time." } }))
                }).icon("\u{1F4DA}").summary("Lists, links and a nested accordion."))
            // end code
            p class="nojs-note" { "Deep link: " a href="/accordion?open.faq=0,2" { "?open.faq=0,2" } ". Leave and come back: the open sections are remembered." }
        },
    )
}

async fn combobox_page(ui: Ui) -> Page {
    page(
        &ui,
        "Combobox",
        html! {
            (ui.flash())
            // One swap root around the form and its results: the script searches as you type.
            div id="langs" data-nojs="swap" {
                // code: /combobox
                (ui.combobox("q", "/combobox").multi().create("/combobox/new")
                    .label("Language").placeholder("Type a language")
                    .group("Systems", ["Rust", "Zig", "Swift"])
                    .group("Scripting", ["Ruby", "Python", "Racket"])
                    .options(["Prolog", "Scala"]))
                // end code
            }
            p class="nojs-note" { "Pick several: each result adds a chip, each chip's \u{d7} removes it, and the chips ride along with the next search. Type a language that is not here to get a Create row." }
        },
    )
}

#[derive(Deserialize)]
struct NewLang {
    name: String,
    #[serde(default)]
    sel: Vec<String>,
}

async fn combobox_new(ui: Ui, Form(f): Form<NewLang>) -> Redirect {
    let name = f.name.trim();
    let to: String = f
        .sel
        .iter()
        .map(String::as_str)
        .chain([name])
        .map(|s| format!("&sel={s}"))
        .collect();
    ui.redirect(&format!("/combobox?q={to}")).flash(&format!(
        "Added {name} (not really: the demo has no database)."
    ))
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

/// The files table's columns; the page and the CSV both read their sort and filter from it.
fn files_table(ui: &Ui) -> Table<'_> {
    // code: /table
    ui.table("files", "/table")
        .column("name", "Name")
        .sortable()
        .column("size", "Size")
        .sortable()
        .numeric()
        .width("7rem")
        .column("kind", "Kind")
        .sortable()
        .width("9rem")
    // end code
}

/// Thirty-six files sorted and filtered on the server, in one place for the page and the CSV.
fn files(t: &Table) -> Vec<(String, u32, &'static str)> {
    let q = t.filter().to_lowercase();
    let mut files: Vec<(String, u32, &str)> = ["src", "docs", "old"]
        .iter()
        .flat_map(|dir| {
            FILES
                .iter()
                .map(move |f| (format!("{dir}/{}", f.0), f.1 * (dir.len() as u32), f.2))
        })
        .filter(|f| q.is_empty() || f.0.contains(&q) || f.2.contains(&q))
        .collect();
    if let Some((key, desc)) = t.sort() {
        files.sort_by(|a, b| match key {
            "size" => a.1.cmp(&b.1),
            "kind" => a.2.cmp(b.2),
            _ => a.0.cmp(&b.0),
        });
        if desc {
            files.reverse();
        }
    }
    files
}

/// The table only renders and links; a row can expand, has its own menu and can be selected.
async fn table_page(ui: Ui) -> Page {
    let t = files_table(&ui);
    let files = files(&t);
    let rows = files.iter().map(|f| Row::new([html! { code { (f.0) } }, html! { (axum_nojs::paged_table::thousands(f.1 as usize / 1024)) " KB" }, html! { (f.2) }])
        .key(&f.0)
        .detail(html! { p { "A " (f.2) " of " (f.1) " bytes, in " code { (f.0.split('/').next().unwrap_or("")) } "." } })
        .menu([MenuItem::link("Open", "/table"), MenuItem::action("Delete", "/table/bulk").danger()]));
    // code: /table
    let t = t
        .rows(rows)
        .paged(files.len())
        .choose_columns()
        .csv("/table.csv")
        .bulk(
            "/table/bulk",
            [("archive", "Archive"), ("delete", "Delete")],
        )
        .empty("No files match this filter.")
        .loading(ui.param("loading") == Some("1"));
    // end code
    page(
        &ui,
        "Table",
        html! {
            (ui.flash())
            p { "Click a header to sort, again to flip. Type to filter. Hide columns, tick rows for the bulk form, open a row's menu or its detail. The page size you pick is remembered for your next visit. Every state is a URL, including " a href="/table?loading=1" { "the loading one" } "." }
            (t)
        },
    )
}

/// The same rows as text/csv, for the sort, filter and columns in the URL.
async fn table_csv(ui: Ui) -> impl IntoResponse {
    let t = files_table(&ui);
    let cols = t.visible();
    let mut csv = cols.join(",") + "\n";
    for f in files(&t) {
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

/// `row=<key>` per ticked box and `action=<value>` from the button: acknowledged with a flash.
async fn table_bulk(ui: Ui, Form(pairs): Form<Vec<(String, String)>>) -> Redirect {
    let rows = pairs.iter().filter(|(k, _)| k == "row").count();
    let action = pairs
        .iter()
        .find(|(k, _)| k == "action")
        .map_or("delete", |(_, v)| v.as_str());
    let msg = if rows == 0 {
        "Nothing selected: tick a row first.".to_string()
    } else {
        format!("{action}: {rows} file(s) (not really).")
    };
    ui.redirect("/table").flash(&msg)
}

/// What the wizard has collected so far, as the steps posted it.
#[derive(Default, Deserialize, Serialize)]
struct Signup(Vec<(String, String)>);

impl Signup {
    fn get(&self, k: &str) -> &str {
        self.0
            .iter()
            .find(|(n, _)| n == k)
            .map_or("", |(_, v)| v.trim())
    }
}

fn signup<'a>(ui: &'a Ui, s: &'a Signup, errors: &'a [(&'a str, &'a str)]) -> Wizard<'a> {
    // code: /wizard
    ui.wizard("signup", "/wizard")
        .step(
            "Account",
            ui.fields()
                .text("name", "Name")
                .required()
                .email("email", "Email")
                .required(),
        )
        .step(
            "Newsletter",
            ui.fields()
                .select("digest", "Digest", ["daily", "weekly", "never"])
                .text("topics", "Topics")
                .placeholder("rust, html"),
        )
        .optional()
        .review("Review")
        .values(&s.0)
        .errors(errors)
        .finish("Create account")
    // end code
}

/// Server rules for a wizard step: `(field, message)` per problem.
fn signup_errors(step: usize, s: &Signup) -> Vec<(&'static str, &'static str)> {
    let (name, email) = (s.get("name"), s.get("email"));
    match step {
        0 if name.is_empty() => vec![("name", "Enter your name.")],
        0 if !email.contains('@') => vec![("email", "Enter an email address with an @.")],
        0 if email.ends_with("@example.com") => {
            vec![("email", "example.com addresses are not accepted.")]
        }
        _ => Vec::new(),
    }
}

fn wizard_view(ui: &Ui, wizard: Wizard) -> Page {
    page(
        ui,
        "Wizard",
        html! {
            (ui.flash())
            p { "Three steps, one form each. The server checks every step; the second can be skipped. Close the tab and come back to " a href="/wizard" { "/wizard" } ": you resume where you left off." }
            (wizard)
        },
    )
}

async fn wizard_page(ui: Ui, Saved(s): Saved<Signup>) -> Page {
    wizard_view(&ui, signup(&ui, &s, &[]))
}

/// Check the posted step: answer 422 with the same step and its messages, or keep the fields
/// (a year, so closing the browser loses nothing) and redirect to the next step.
async fn wizard_submit(
    ui: Ui,
    Saved(mut s): Saved<Signup>,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Response {
    let posted = Posted::from_pairs(&pairs);
    for (k, v) in pairs
        .into_iter()
        .filter(|(k, _)| !posted.skip && k != "step" && k != "skip")
    {
        s.0.retain(|(n, _)| *n != k);
        s.0.push((k, v));
    }
    let errors = if posted.skip {
        Vec::new()
    } else {
        signup_errors(posted.step, &s)
    };
    let wizard = signup(&ui, &s, &errors).at(posted.step);
    if !errors.is_empty() {
        return (StatusCode::UNPROCESSABLE_ENTITY, wizard_view(&ui, wizard)).into_response();
    }
    if wizard.is_last(posted.step) {
        return ui
            .redirect(&wizard.link(0))
            .flash("Account created (well, the cookie was cleared).")
            .forget::<Signup>()
            .into_response();
    }
    ui.redirect(&wizard.link(posted.step + 1))
        .save(&s)
        .into_response()
}

fn form_view(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Page {
    let inline = ui.param("layout") == Some("inline");
    // code: /form
    let form = ui
        .form("/form")
        .submit("Sign up")
        .values(values)
        .errors(errors)
        .group("Account")
        .text("name", "Name")
        .required()
        .email("email", "Email")
        .required()
        .number("age", "Age", 13, 120)
        .required()
        .pattern(
            "handle",
            "Handle",
            "[a-z0-9_]{3,16}",
            "3–16 lowercase letters, digits or _",
        )
        .required()
        .select("plan", "Plan", ["Free", "Team", "Enterprise"])
        .group("Profile")
        .textarea("bio", "Bio", 3)
        .maxlength(160)
        .help("Grows as you type where the browser supports it.")
        .file("avatar", "Avatar", "image/png,image/jpeg")
        .help("PNG or JPEG.")
        .date("start", "Start date", "2026-01-01", "2027-12-31")
        .time("call", "Best time to call", "09:00", "17:00")
        .help("Office hours, 09:00 to 17:00.");
    // end code
    page(
        ui,
        "Validated form",
        html! {
            (ui.flash())
            p { "Labels " @if inline { "beside the fields. " a href="/form" { "Put them above" } } @else { "above the fields. " a href="/form?layout=inline" { "Put them beside" } } "." }
            @if !errors.is_empty() { p class="nojs-error" { "Server-side checks failed. Browser validation passed, these rules only live on the server." } }
            (if inline { form.inline() } else { form })
        },
    )
}

async fn form_page(ui: Ui) -> Page {
    form_view(&ui, &[], &[])
}

/// A multipart post (the avatar is a file): server rules, then PRG with a flash or the form again.
async fn form_submit(ui: Ui, mut parts: axum::extract::Multipart) -> Response {
    let (mut values, mut files) = (Vec::new(), Vec::new());
    while let Ok(Some(part)) = parts.next_field().await {
        let (name, file) = (
            part.name().unwrap_or("").to_string(),
            part.file_name().map(str::to_string),
        );
        match file {
            Some(f) => {
                let n = part.bytes().await.map_or(0, |b| b.len());
                if !f.is_empty() {
                    files.push(format!(" with {f} ({n} bytes)"));
                }
            }
            None => values.push((name, part.text().await.unwrap_or_default())),
        }
    }
    let get = |k: &str| {
        values
            .iter()
            .find(|(n, _)| n == k)
            .map_or("", |(_, v)| v.as_str())
    };
    let mut errors = Vec::new();
    if get("handle") == "admin" {
        errors.push(("handle", "That handle is reserved."));
    }
    if get("email").ends_with("@example.com") {
        errors.push(("email", "example.com addresses are not accepted."));
    }
    if errors.is_empty() {
        return ui
            .redirect("/form")
            .flash(&format!(
                "Signed up as {}{}.",
                get("handle"),
                files.concat()
            ))
            .into_response();
    }
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        form_view(&ui, &values, &errors),
    )
        .into_response()
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

/// The inputs page's values: from the query while filtering (unsaved), else saved.
#[derive(Deserialize, Serialize, Default)]
struct Inputs {
    size: Option<String>,
    volume: Option<i64>,
    accent: Option<String>,
    #[serde(rename = "accent-alpha")]
    alpha: Option<u8>,
    #[serde(rename = "accent-preset")]
    preset: Option<String>,
    price_min: Option<i64>,
    price_max: Option<i64>,
    country: Option<String>,
}

const SIZES: [(&str, &str, &str); 3] = [
    ("s", "Small", "🐭"),
    ("m", "Medium", "🐕"),
    ("l", "Large", "🐘"),
];
const ACCENTS: [&str; 5] = ["#1f6f5f", "#2f5bea", "#b3261e", "#8a5a00", "#6b3fa0"];
/// `(value, name, flag)`.
type Country = (&'static str, &'static str, &'static str);
const COUNTRIES: [(&str, [Country; 7]); 3] = [
    (
        "Europe",
        [
            ("es", "Spain", "🇪🇸"),
            ("fr", "France", "🇫🇷"),
            ("de", "Germany", "🇩🇪"),
            ("it", "Italy", "🇮🇹"),
            ("pt", "Portugal", "🇵🇹"),
            ("nl", "Netherlands", "🇳🇱"),
            ("se", "Sweden", "🇸🇪"),
        ],
    ),
    (
        "Americas",
        [
            ("us", "United States", "🇺🇸"),
            ("ca", "Canada", "🇨🇦"),
            ("mx", "Mexico", "🇲🇽"),
            ("br", "Brazil", "🇧🇷"),
            ("ar", "Argentina", "🇦🇷"),
            ("cl", "Chile", "🇨🇱"),
            ("co", "Colombia", "🇨🇴"),
        ],
    ),
    (
        "Asia",
        [
            ("jp", "Japan", "🇯🇵"),
            ("kr", "South Korea", "🇰🇷"),
            ("in", "India", "🇮🇳"),
            ("id", "Indonesia", "🇮🇩"),
            ("vn", "Vietnam", "🇻🇳"),
            ("th", "Thailand", "🇹🇭"),
            ("ph", "Philippines", "🇵🇭"),
        ],
    ),
];

/// Select, range and colour in one form, saved in `nojs-inputs`. The country filter is a GET
/// through the same form, so while filtering the values come from the query.
async fn inputs_page(ui: Ui, Query(q): Query<Inputs>, Saved(saved): Saved<Inputs>) -> Page {
    let v = if ui.param("country-q").is_some() {
        q
    } else {
        saved
    };
    page(
        &ui,
        "Select, range, colour",
        html! {
            (ui.flash())
            form id="inputs" data-nojs="swap" class="nojs-form" method="post" action="/inputs" {
                // code: /inputs
                (ui.select("size", v.size.as_deref().unwrap_or("m")).options(SIZES).label("Size"))
                (ui.select("country", v.country.as_deref().unwrap_or("es")).groups(COUNTRIES).search("/inputs").label("Country"))
                (ui.range("volume", v.volume.unwrap_or(40)).step(5).label("Volume"))
                (ui.range_pair("price", (v.price_min.unwrap_or(20), v.price_max.unwrap_or(80))).step(5).label("Price"))
                (ui.color("accent", v.accent.as_deref().unwrap_or("#1f6f5f")).presets(&ACCENTS).alpha(v.alpha.unwrap_or(100)).label("Accent"))
                // end code
                (ui.button("Save").primary())
            }
            p class="nojs-note" { "Without the enhancement script the outputs and the swatch show the last saved values and update on submit, and the country filter needs its button." }
        },
    )
}

/// Only known sizes, countries and `#rrggbb` colours are kept; numbers are clamped.
async fn inputs_submit(ui: Ui, Form(f): Form<Inputs>) -> Redirect {
    let known = |v: &Option<String>, ok: &dyn Fn(&str) -> bool| v.clone().filter(|v| ok(v));
    let hex = |c: &str| c.len() == 7 && c.starts_with('#');
    let (lo, hi) = axum_nojs::range::order(
        f.price_min.unwrap_or(20).clamp(0, 100),
        f.price_max.unwrap_or(80).clamp(0, 100),
    );
    let clean = Inputs {
        size: known(&f.size, &|s| SIZES.iter().any(|(v, ..)| *v == s)),
        country: known(&f.country, &|c| {
            COUNTRIES
                .iter()
                .flat_map(|(_, cs)| cs)
                .any(|(v, ..)| *v == c)
        }),
        accent: known(&f.preset, &hex).or(known(&f.accent, &hex)),
        alpha: Some(f.alpha.unwrap_or(100).min(100)),
        volume: Some(f.volume.unwrap_or(40).clamp(0, 100)),
        price_min: Some(lo),
        price_max: Some(hi),
        preset: None,
    };
    ui.redirect("/inputs").flash("Inputs saved.").save(&clean)
}

/// Toasts come back from a post like a flash: the one-shot cookie, several at once.
async fn toast_page(ui: Ui) -> Page {
    page(
        &ui,
        "Toasts",
        html! {
            p { "Each button posts, the server redirects back, and the answer shows in the corner. Calm ones fade after five seconds (hover to keep them); errors stay until dismissed." }
            form method="post" action="/toast" class="nojs-cluster" {
                (ui.button("Send invite").primary().name("kind").value("ok"))
                (ui.button("Copy link").name("kind").value("warn"))
                (ui.button("Sync now").name("kind").value("danger"))
                (ui.button("All three").name("kind").value("all"))
            }
            // code: /toast
            (ui.toasts().dismiss())
            // end code
        },
    )
}

#[derive(Deserialize)]
struct ToastForm {
    kind: String,
}

async fn toast_submit(ui: Ui, Form(f): Form<ToastForm>) -> Redirect {
    let wants = |k: &str| f.kind == "all" || f.kind == k;
    // code: /toast
    let mut back = ui.redirect("/toast");
    if wants("ok") {
        back = back.ok("Invite sent to ada@example.org.");
    }
    if wants("warn") {
        back = back.warn("Link copied; it expires in an hour.");
    }
    if wants("danger") {
        back = back.danger("Sync failed: the server did not answer.");
    }
    // end code
    back
}

/// A sidebar on wide screens, a drawer on narrow ones, and breadcrumbs above the content.
async fn nav_page(ui: Ui) -> Page {
    page(
        &ui,
        "Drawer and breadcrumbs",
        html! {
            // code: /nav
            (ui.drawer("Menu").id("site").title("axum-nojs").sidebar()
                .nav(html! { ul {
                    li { a href="/nav" aria-current="page" { "Overview" } }
                    li { a href="/table" { "Files" } } li { a href="/dashboard" { "Reports" } } li { a href="/settings" { "Settings" } }
                } })
                .body(html! {
                    (ui.breadcrumbs().link("Home", "/").link("Projects", "/nav").here("axum-nojs"))
                    p { "Wider than 60rem the navigation is a sidebar; narrower, the menu button opens it as a drawer. Escape or a click outside closes it." }
                    p { "A long trail folds its middle so both ends stay readable:" }
                    (ui.breadcrumbs().link("Home", "/").link("Projects", "/nav").link("axum-nojs", "/nav")
                        .link("Components", "/").link("Navigation", "/nav").here("Breadcrumbs"))
            // end code
                    p class="nojs-note" { "Server-opened: " a href="/nav?dialog=site" { "?dialog=site" } }
                }))
        },
    )
}

/// Stat cards over a list that may be empty (`?orders=none`).
async fn calendar_page(ui: Ui) -> Page {
    let soon = |days| Date::today().add_days(days).to_string();
    let (invoice, release) = (soon(6), soon(21));
    page(
        &ui,
        "Calendar",
        html! {
            // code: /calendar
            (ui.calendar("day")
                .disabled(|d| d.weekday() >= 5)
                .event(&invoice, "Invoice due")
                .event(&release, "Release"))
            // end code
            p class="nojs-note" { @match ui.param("day") {
                Some(d) => { "You picked " (d) ". Weekends cannot be picked; a dot marks an event." },
                None => { "Pick a weekday. The month links and the days are ordinary links: the page comes back with " code { "?day=" } " set." },
            } }
        },
    )
}

async fn button_page(ui: Ui) -> Page {
    let loading = ui.param("loading") == Some("1");
    page(
        &ui,
        "Buttons and badges",
        html! {
            (ui.stack(html! {
                // code: /button
                (ui.cluster(html! {
                    (ui.button("Save").primary().loading(loading))
                    (ui.button("Cancel"))
                    (ui.button("Delete").danger())
                    (ui.button("Skip").ghost())
                    (ui.button("Small").small())
                    (ui.button("\u{2026}").icon().ghost().label("More"))
                    (ui.link_button("Read the docs", "/"))
                }))
                (ui.cluster(html! {
                    (ui.badge("New")) (ui.badge("Draft").secondary()) (ui.badge("Failed").danger())
                    (ui.badge("rust").outline()) (ui.badge("Paid").ok()) (ui.badge("Pending").warn())
                }))
                (ui.cluster(html! { @for icon in Icon::ALL { (ui.icon(icon).label(icon.name())) } }).gap(3))
                // end code
                p class="nojs-note" { "The server decides a button is loading: " a href=(if loading { "/button" } else { "/button?loading=1" }) { @if loading { "stop" } @else { "start" } } "." }
            }))
        },
    )
}

async fn field_page(ui: Ui) -> Page {
    let email = ui.param("email").unwrap_or("");
    let bad = !email.is_empty() && !email.contains('@');
    page(
        &ui,
        "Fields",
        html! {
            form class="nojs-stack" method="get" action="/field" {
                // code: /field
                (ui.input("name", "Name").placeholder("Ada Lovelace").help("As it should appear on invoices."))
                (ui.input("email", "Email").email().required().value(email).error(if bad { "An email address needs an @." } else { "" }))
                (ui.checkbox("terms", "I accept the terms").required())
                (ui.switch("digest", "Weekly digest").checked(ui.param("digest").is_some()))
                (ui.radio_group("plan", "Plan").option("free", "Free").option("pro", "Pro").value(ui.param("plan").unwrap_or("free")))
                (ui.button("Check").primary())
                // end code
            }
        },
    )
}

async fn card_page(ui: Ui) -> Page {
    let team = [
        ("Ada Lovelace", "Owner"),
        ("Grace Hopper", "Admin"),
        ("Alan Turing", "Member"),
    ];
    page(
        &ui,
        "Cards and avatars",
        html! {
            // code: /card
            (ui.grid("16rem", html! {
                (ui.card().title("Team").description("3 people can edit this project.")
                    .header(html! { (ui.badge("Pro").secondary()) })
                    .body(html! { (ui.stack(html! { @for (name, role) in team {
                        (ui.cluster(html! { (ui.avatar(name)) span { (name) } (ui.badge(role).outline()) }))
                    } }).gap(3)) })
                    .footer(html! { (ui.button("Invite").primary()) (ui.button("Manage").ghost()) }))
                (ui.card().title("Storage").description("Resets on the 1st.")
                    .body(html! { p { "3.2 GB of 5 GB used." } })
                    .footer(html! { (ui.link_button("Upgrade", "/card")) }))
            }))
            // end code
        },
    )
}

async fn layout_page(ui: Ui) -> Page {
    let tile = |t: &str| html! { div class="nojs-layout-tile" { (t) } };
    page(
        &ui,
        "Layout",
        html! {
            // code: /layout
            (ui.stack(html! {
                (ui.cluster(html! { h3 { "Cluster" } (ui.cluster(html! { (ui.button("Export")) (ui.button("New").primary()) })) }).between())
                (ui.grid("8rem", html! { @for t in ["Grid", "fills", "the row", "then", "wraps"] { (tile(t)) } }).gap(2))
                (ui.split(html! { (tile("Split: side")) }, html! { (tile("main, stacks under the side when narrow")) }).side_width("12rem"))
            }).gap(6))
            // end code
        },
    )
}

async fn dashboard_page(ui: Ui) -> Page {
    let none = ui.param("orders") == Some("none");
    page(
        &ui,
        "Stats and empty states",
        html! {
            div class="nojs-stat-grid" {
                // code: /dashboard
                (ui.stat("Visitors", "12,480").delta("+8.2%").note("last 7 days"))
                (ui.stat("Orders", if none { "0" } else { "3" }).delta(if none { "-3" } else { "0" }))
                (ui.stat("Error rate", "0.4%").delta("-0.2 pt").down_is_good().href("/table"))
                (ui.stat("p95 latency", "38 ms").delta("+6 ms").down_is_good())
                // end code
            }
            h2 { "Recent orders" }
            @if none {
                // code: /dashboard
                (ui.empty_state("No orders yet").icon("\u{1f4e6}")
                    .text(html! { "Orders show up here as soon as a customer checks out." })
                    .link("Show sample orders", "/dashboard"))
                // end code
            } @else {
                ul { li { "#1042, Ada Lovelace, 3 items" } li { "#1041, Grace Hopper, 1 item" } li { "#1040, Alan Turing, 2 items" } }
                p class="nojs-note" { a href="/dashboard?orders=none" { "See the empty state" } }
            }
        },
    )
}

/// Every demo page as a command, plus a few deep links. An exact command name redirects;
/// anything else lists the matches.
async fn palette_page(ui: Ui) -> Response {
    // code: /palette
    let palette = ui
        .palette("/palette")
        .id("cmd")
        .group("Components")
        .commands(COMPONENTS.iter().map(|c| (c.1, c.0)))
        .group("Shortcuts")
        .command("Notification settings", "/settings?tab.settings=1")
        .keywords("email releases")
        .command("Largest files", "/table?sort=size&dir=desc")
        .keywords("sort size big")
        .command("Open the delete dialog", "/dialog?dialog=confirm")
        .keywords("account remove");
    if let Some(href) = palette.exact() {
        return ui.redirect(href).into_response();
    }
    // end code
    page(&ui, "Command palette", html! {
        p { "Open it with the button or the access key, type, pick a suggestion and press Enter. An exact name goes straight to the page; anything else lists what matches." }
        (palette)
    }).into_response()
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

#[derive(Deserialize)]
struct ThemeForm {
    theme: String,
}

/// Keep the picked theme and go back to the page the toggle was on.
async fn theme_submit(ui: Ui, headers: HeaderMap, Form(f): Form<ThemeForm>) -> Redirect {
    ui.redirect(&back_to(&headers))
        .theme(Theme::parse(&f.theme))
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
            let modern = Cap::ALL
                .map(|c| format!("nojs-cap-{}=1", c.name()))
                .join("; ");
            for cookie in ["", modern.as_str()] {
                let req = Request::get(path)
                    .header("cookie", cookie)
                    .body(Body::empty())
                    .unwrap();
                let res = router().oneshot(req).await.unwrap();
                assert_eq!(res.status(), 200, "{path}");
                let body = axum::body::to_bytes(res.into_body(), usize::MAX)
                    .await
                    .unwrap();
                let html = String::from_utf8(body.to_vec()).unwrap();
                assert_eq!(
                    html.matches("<script").count(),
                    1,
                    "{path}: exactly one script tag"
                );
                assert!(
                    html.contains(&tag),
                    "{path}: the tag is the enhancement script"
                );
                assert!(!html.contains("javascript:"), "{path}");
                let inline_handler = html.split('<').any(|tag| {
                    tag.split_whitespace()
                        .any(|a| a.starts_with("on") && a.contains('='))
                });
                assert!(!inline_handler, "{path}: inline event handler");
            }
        }
    }

    /// Every component page shows code cut from this file, so the snippet cannot drift from
    /// what runs; every `// code:` marker is closed and names a component page.
    #[tokio::test]
    async fn every_component_page_shows_its_code() {
        let opens = SOURCE
            .lines()
            .filter(|l| l.trim().starts_with("// code: "))
            .count();
        let closes = SOURCE.lines().filter(|l| l.trim() == "// end code").count();
        assert_eq!(opens, closes, "every // code: has its // end code");
        for l in SOURCE
            .lines()
            .filter_map(|l| l.trim().strip_prefix("// code: "))
        {
            assert!(
                COMPONENTS.iter().any(|c| c.0 == l),
                "{l} is not a component page"
            );
        }
        for (href, ..) in COMPONENTS {
            let code = snippet(href);
            assert!(!code.trim().is_empty(), "{href}: no snippet");
            assert!(
                code.lines().all(|l| SOURCE.contains(l)),
                "{href}: snippet not cut from the source"
            );
            let res = router()
                .oneshot(Request::get(href).body(Body::empty()).unwrap())
                .await
                .unwrap();
            let html = String::from_utf8(
                axum::body::to_bytes(res.into_body(), usize::MAX)
                    .await
                    .unwrap()
                    .to_vec(),
            )
            .unwrap();
            // The highlighted block, tags stripped, is exactly the code.
            let pre = html
                .split("class=\"nojs-snippet\"")
                .nth(1)
                .and_then(|s| s.split("<pre>").nth(1))
                .and_then(|s| s.split("</pre>").next());
            let text: String = pre
                .expect("{href}: no snippet box")
                .split('<')
                .map(|p| p.split_once('>').map_or(p, |(_, t)| t))
                .collect();
            assert_eq!(
                text,
                html! { (code) }.into_string(),
                "{href}: the box does not show its code"
            );
            assert!(
                pre.unwrap().contains("class=\"nojs-hl-"),
                "{href}: not highlighted"
            );
        }
        let code = highlight(
            r#"let t = ui.tabs("demo").badge(3); // lazy
html! { @if x { Some(Page) } }"#,
        )
        .into_string();
        for part in [
            r#"hl-k">let<"#,
            r#"hl-f">tabs<"#,
            r#"hl-s">&quot;demo&quot;<"#,
            r#"hl-n">3<"#,
            r#"hl-c">// lazy"#,
            r#"hl-m">html!<"#,
            r#"hl-k">if<"#,
            r#"hl-t">Some<"#,
        ] {
            assert!(code.contains(part), "{part} in {code}");
        }
    }

    #[tokio::test]
    async fn whole_pages_are_compressed_and_streams_are_not() {
        let gz = |path: &str| {
            Request::get(path)
                .header("accept-encoding", "gzip")
                .body(Body::empty())
                .unwrap()
        };
        let res = router().oneshot(gz("/table")).await.unwrap();
        assert_eq!(
            res.headers()["content-encoding"],
            "gzip",
            "a whole page is compressed"
        );
        let res = router().oneshot(gz("/stream")).await.unwrap();
        assert!(
            res.headers().get("content-encoding").is_none(),
            "a stream keeps its chunks"
        );
        let res = router()
            .oneshot(Request::get("/table").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert!(res.headers().get("content-encoding").is_none());
        assert!(
            axum::body::HttpBody::size_hint(res.body())
                .exact()
                .is_some(),
            "plain pages carry a Content-Length"
        );
    }

    #[tokio::test]
    async fn enhanced_requests_get_the_page_without_its_stylesheet() {
        let get = |enhanced: bool| {
            let req = Request::get("/tabs?tab.demo=1");
            let req = if enhanced {
                req.header("nojs-enhance", "1")
            } else {
                req
            };
            router().oneshot(req.body(Body::empty()).unwrap())
        };
        let full = get(false).await.unwrap();
        assert_eq!(full.headers()["vary"], "nojs-enhance, cookie");
        let full = String::from_utf8(
            axum::body::to_bytes(full.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        let slim = get(true).await.unwrap();
        assert_eq!(slim.headers()["vary"], "nojs-enhance, cookie");
        let slim = String::from_utf8(
            axum::body::to_bytes(slim.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(
            full.contains("<style>") && !slim.contains("<style>"),
            "the stylesheet stays home"
        );
        assert!(
            slim.contains("id=\"nojs-tabs-demo\"") && slim.contains("<title>"),
            "the swap root and title are still there"
        );
        assert!(
            slim.len() * 3 < full.len(),
            "slim is {} of {} bytes",
            slim.len(),
            full.len()
        );
    }

    #[tokio::test]
    async fn enhancement_script_is_served_immutable() {
        let req = Request::get(axum_nojs::enhance::script_url())
            .body(Body::empty())
            .unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 200);
        assert_eq!(
            res.headers()["content-type"],
            "text/javascript; charset=utf-8"
        );
        assert!(
            res.headers()["cache-control"]
                .to_str()
                .unwrap()
                .contains("immutable")
        );
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body, axum_nojs::enhance::served().as_bytes());
    }

    #[tokio::test]
    async fn caps_beacon_sets_one_cookie_per_flag() {
        let req = Request::get("/nojs/caps?flag=popover")
            .body(Body::empty())
            .unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 204);
        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(cookie.starts_with("nojs-cap-popover=1;"), "{cookie}");
        let req = Request::get("/nojs/caps?flag=nope")
            .body(Body::empty())
            .unwrap();
        assert_eq!(router().oneshot(req).await.unwrap().status(), 404);
    }

    /// Collect the body frames of `path` as they arrive.
    async fn frames(path: &str, cookie: &str) -> Vec<String> {
        use http_body_util::BodyExt;
        let req = Request::get(path)
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap();
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
        assert!(
            chunks.len() >= 6,
            "expected head + shell + 3 fills + suffix, got {}",
            chunks.len()
        );
        assert!(
            chunks[0].ends_with("</head>") && chunks[0].contains("<style>"),
            "the head, stylesheet included, goes first"
        );
        assert!(chunks[1].contains("<template shadowrootmode=\"open\">"));
        assert!(chunks[1].contains("<slot name=\"slow\">"));
        let order: Vec<&str> = chunks[2..5]
            .iter()
            .map(|c| {
                c.split("slot=\"")
                    .nth(1)
                    .unwrap()
                    .split('"')
                    .next()
                    .unwrap()
            })
            .collect();
        assert_eq!(order, ["fast", "medium", "slow"]);
        assert!(
            chunks.last().unwrap().ends_with("</script></body></html>"),
            "suffix carries the enhancement tag"
        );
    }

    #[tokio::test]
    async fn stream_fallback_is_in_document_order() {
        let chunks = frames("/stream", "nojs-cap-probed=1").await;
        let html = chunks.concat();
        assert!(!html.contains("<template") && !html.contains("<slot"));
        let pos = |s: &str| html.find(s).unwrap();
        assert!(
            pos("slow</strong>") < pos("medium</strong>")
                && pos("medium</strong>") < pos("fast</strong>")
        );
        assert!(
            chunks.len() >= 5,
            "streamed in pieces, got {}",
            chunks.len()
        );
        assert!(chunks[0].ends_with("</head>"), "the head goes first");
    }

    #[tokio::test]
    async fn palette_exact_name_redirects_and_toast_posts_stack() {
        let res = router()
            .oneshot(
                Request::get("/palette?q=largest%20FILES")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res.status(), 303);
        assert_eq!(
            res.headers().get("location").unwrap(),
            "/table?sort=size&dir=desc"
        );
        let res = router()
            .oneshot(Request::get("/palette?q=zzz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        let req = Request::post("/toast")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("kind=all"))
            .unwrap();
        let res = router().oneshot(req).await.unwrap();
        let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
        assert!(
            cookie.starts_with("nojs-flash=ok%3AInvite")
                && cookie.contains("%0Awarn%3A")
                && cookie.contains("%0Adanger%3A"),
            "{cookie}"
        );
    }

    #[tokio::test]
    async fn state_round_trip_through_prg_and_cookies() {
        // POST → 303 with a flash cookie and the values saved; the tab is in the nojs-ui cookie.
        let req = Request::post("/settings")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("name=Ada&notify=true"))
            .unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert_eq!(res.status(), 303);
        assert_eq!(res.headers().get("location").unwrap(), "/settings");
        let cookies: Vec<String> = res
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert!(
            cookies
                .iter()
                .any(|c| c.starts_with("nojs-flash=ok%3ASettings%20saved.")),
            "{cookies:?}"
        );
        assert!(
            cookies
                .iter()
                .any(|c| c.starts_with("nojs-settings=name%3DAda%26notify%3Dtrue;")),
            "{cookies:?}"
        );
        // GET the redirect target: flash shown and cleared, tab persisted to nojs-ui, values filled in.
        let req = Request::get("/settings?tab.settings=1")
            .header(
                "cookie",
                "nojs-flash=Settings%20saved.; nojs-settings=name%3DAda%26notify%3Dtrue",
            )
            .body(Body::empty())
            .unwrap();
        let res = router().oneshot(req).await.unwrap();
        let cookies: Vec<String> = res
            .headers()
            .get_all("set-cookie")
            .iter()
            .map(|v| v.to_str().unwrap().to_string())
            .collect();
        assert!(
            cookies
                .iter()
                .any(|c| c.starts_with("nojs-ui=tab.settings=1;")),
            "{cookies:?}"
        );
        assert!(
            cookies
                .iter()
                .any(|c| c.starts_with("nojs-flash=; Path=/; Max-Age=0")),
            "{cookies:?}"
        );
        let html = String::from_utf8(
            axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(
            html.contains("Settings saved.")
                && html.contains("value=\"Ada\"")
                && html.contains("checked")
        );
        assert!(
            html.contains("<details name=\"settings\" open>")
                && html.contains("href=\"/settings?tab.settings=0\"")
        );
        // Coming back with only the cookie: the tab is still open, nothing is rewritten.
        let req = Request::get("/settings")
            .header("cookie", "nojs-ui=tab.settings=1")
            .body(Body::empty())
            .unwrap();
        let res = router().oneshot(req).await.unwrap();
        assert!(res.headers().get("set-cookie").is_none());
        let html = String::from_utf8(
            axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(html.contains("<details name=\"settings\" open><summary><a href=\"/settings?tab.settings=1\">Notifications"));
    }

    #[tokio::test]
    async fn markup_follows_caps() {
        async fn body(path: &str, cookie: &str) -> String {
            let req = Request::get(path)
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap();
            let res = router().oneshot(req).await.unwrap();
            let body = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap();
            String::from_utf8(body.to_vec()).unwrap()
        }
        let old = body("/dialog", "").await;
        assert!(old.contains("href=\"#confirm\"") && !old.contains("commandfor"));
        assert!(old.contains("nojs-caps"), "unknown browser gets beacons");
        let new = body("/dialog", "nojs-cap-probed=1; nojs-cap-invokers=1").await;
        assert!(new.contains("commandfor=\"confirm\"") && !new.contains("href=\"#confirm\""));
        assert!(
            !new.contains("class=\"nojs-caps\""),
            "probed browser gets no beacons"
        );
        assert!(body("/popover", "").await.contains("nojs-popover-details"));
        assert!(
            body("/tabs", "nojs-cap-details_content=1")
                .await
                .contains("nojs-tabs-panel")
        );
        assert!(body("/tabs", "").await.contains("nojs-accordion-body"));
    }
}
