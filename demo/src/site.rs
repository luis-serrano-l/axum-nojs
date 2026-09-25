//! What every page shares: the component index, the shell around each page (toolbar, title,
//! the stage and its code), the index page and the theme switch.

use crate::code::CODE;
use axum::{
    Form, Router,
    http::HeaderMap,
    routing::{get, post},
};
use loco_ui::layout::{Palette, Tokens};
use loco_ui::prelude::*;
use serde::Deserialize;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/theme", post(theme_submit))
        .route("/lang", post(lang_submit))
}

/// Every component in the index: path, title (what each route passes to `page`), group, the
/// platform features it is built on, and what it is for in plain words.
pub(crate) const COMPONENTS: [(&str, &str, &str, &str, &str); 36] = [
    (
        "/feedback",
        "Alerts, progress and tooltips",
        "Feedback",
        "role=alert, <progress>, <meter>, :hover/:focus-within, <hr>",
        "Callouts, bars and meters, a tooltip on hover or focus, and separators.",
    ),
    (
        "/app/signin",
        "Sign in",
        "Complete flows",
        "server validation, errors re-rendered, a session cookie, PRG",
        "A sign-in form whose mistakes come back from the server, next to the fields.",
    ),
    (
        "/app/notes",
        "Notes",
        "Complete flows",
        "create, edit in place, delete, filter, pages, flash, PRG",
        "A small app: add, rename and delete notes, with script off.",
    ),
    (
        "/pricing",
        "Pricing card",
        "Your own",
        "ui.card, ui.badge, ui.link_button, Icon, ui.grid, Page::css",
        "A component written in the demo crate, from the public primitives only.",
    ),
    (
        "/kanban",
        "Kanban",
        "Widgets",
        "form POST per move, PRG, view-transition-name, scroll-snap",
        "Cards in columns; each move is a form post the server keeps.",
    ),
    (
        "/upload",
        "Upload",
        "Widgets",
        "multipart POST, <input type=file>, drop on the input, <progress>, PRG",
        "Send files; the list under the form is what the server kept.",
    ),
    (
        "/calendar",
        "Calendar",
        "Widgets",
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
        ":user-invalid, <fieldset>, <output> counters, field-sizing, multipart, PRG, error summary with autofocus",
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
        "data-lui-target, data-lui-swap, data-lui-oob, data-lui-indicator, data-lui-push, Lui-Enhance header",
        "Update one part of the page without reloading it.",
    ),
    (
        "/blocks/shell",
        "App shell",
        "Blocks",
        "a sidebar that is a drawer on narrow screens, aria-current",
        "The frame of a signed-in app: navigation, who is signed in, the page.",
    ),
    (
        "/blocks/auth",
        "Auth page",
        "Blocks",
        "a card, a form that posts",
        "A sign-in or sign-up page, centred, with the links to the other account pages.",
    ),
    (
        "/blocks/settings",
        "Settings page",
        "Blocks",
        "fragment links, a two-column grid",
        "Settings in sections, each with its own form, and a list that jumps to each.",
    ),
    (
        "/blocks/record",
        "Record page",
        "Blocks",
        "<dl>, a delete form, PRG",
        "One record's fields and the actions on it.",
    ),
    (
        "/blocks/dashboard",
        "Dashboard page",
        "Blocks",
        "auto-fit grid",
        "A row of numbers and what goes under them.",
    ),
    (
        "/blocks/error",
        "Error page",
        "Blocks",
        "Router::fallback, HTTP status",
        "The 404 and 500 pages in the site's look; the demo's fallback.",
    ),
];

/// The index's layers, bottom up, each with the groups it holds (as in `docs/layers.svg`).
pub(crate) const LAYERS: [(&str, &str, &[&str]); 6] = [
    (
        "Primitives",
        "The parts every component is built from.",
        &["Primitives"],
    ),
    (
        "Components",
        "Built from the primitives: one change to the button restyles them all.",
        &[
            "Overlays",
            "Disclosure",
            "Navigation",
            "Input",
            "Feedback",
            "Server state",
        ],
    ),
    (
        "Widgets",
        "Larger pieces built from components and primitives.",
        &["Widgets"],
    ),
    (
        "Blocks",
        "Whole pages from the components: fill one in, or copy its file when yours differs.",
        &["Blocks"],
    ),
    (
        "Your own",
        "A component written in the demo crate, the way you would write one.",
        &["Your own"],
    ),
    (
        "Complete flows",
        "Whole tasks, end to end, with JavaScript off: the parts working together.",
        &["Complete flows"],
    ),
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

/// The row above every title: the way back to the index (not on the index), the language of
/// the components' own words, and the theme switch.
fn toolbar(ui: &Ui, back: bool) -> Markup {
    let languages = html! {
        form method="post" action="/lang" class="lui-lang" {
            @for (tag, name) in [("en", "English"), ("es", "Español")] {
                (ui.button(name).small().ghost().name("lang").value(tag).pressed(ui.lang() == tag))
            }
        }
    };
    html! { nav class="lui-toolbar" {
        @if back { a class="lui-back" href="/" { "All components" } } @else { span {} }
        (ui.cluster(html! { (languages) (ui.theme_toggle("/theme")) }))
    } }
}

/// What every page shows around its body: the toolbar and the title, and on a component page
/// what it is for and built on, then the body on a stage with the code that drew it underneath.
pub(crate) fn shell(ui: &Ui, title: &str, body: Markup) -> Markup {
    let component = COMPONENTS.iter().find(|c| c.1 == title);
    let Some(c) = component else {
        return html! { (toolbar(ui, false)) h1 { (title) } (body) };
    };
    html! {
        (toolbar(ui, true))
        h1 { (title) }
        p class="lui-lede" { (c.4) }
        p class="lui-built" { "Built on " @for f in c.3.split(", ") { code { (f) } " " } }
        // The live component and the code that drew it, joined as one plate.
        div class="lui-plate" {
            div class="lui-stage" { (body) }
            figure class="lui-snippet" {
                @if let Some((_, path, code, _)) = CODE.iter().find(|h| h.0 == c.0) {
                    figcaption { span { (path) } span { "The code behind the component above" } }
                    pre tabindex="0" aria-label=(path) { code { (maud::PreEscaped(code)) } }
                }
            }
        }
        @if let Some((.., builders)) = CODE.iter().find(|h| h.0 == c.0).filter(|h| !h.3.is_empty()) {
            (props(builders))
        }
    }
}

/// `text` with each `` `span` `` as `<code>`, as rustdoc shows it.
fn inline_code(text: &str) -> Markup {
    html! { @for (i, part) in text.split('`').enumerate() { @if i % 2 == 1 { code { (part) } } @else { (part) } } }
}

/// What each builder on the page accepts, from `loco_ui::props()`: one `<details>` per
/// builder (the first open), its constructors in the summary and a table of its setters inside.
fn props(builders: &[&loco_ui::props::Component]) -> Markup {
    html! {
        section class="lui-props" {
            h2 { "Props" }
            @for (i, b) in builders.iter().enumerate() {
                details open[i == 0] {
                    summary {
                        code { (b.lui()) }
                        @for call in b.calls { " " code { (call) } }
                        span { (b.props.len()) @if b.props.len() == 1 { " prop" } @else { " props" } }
                    }
                    @if !b.props.is_empty() {
                        div class="lui-props-scroll" tabindex="0" role="region" aria-label={ (b.builder) " props" } { table {
                            thead { tr { th { "Prop" } th { "Kind" } th { "Arguments" } th { "Default" } th { "HTML" } th { "What it does" } } }
                            tbody { @for p in b.props { tr {
                                td { code { (p.name) } }
                                td { (p.kind.as_str()) }
                                td { @if !p.args.is_empty() { code { (p.args) } } }
                                td { @if !p.default.is_empty() { code { (p.default) } } }
                                td { @if !p.attr.is_empty() { code { (p.attr) } } }
                                td { (inline_code(p.doc)) }
                            } } }
                        } }
                    }
                }
            }
        }
    }
}

pub(crate) fn page(ui: &Ui, title: &str, body: Markup) -> Page {
    let page = ui.page(title, shell(ui, title, body));
    // `?script=off`: the same page without the enhancement script, served under
    // `script-src 'none'`, so the no-script path can be tried in any browser.
    if ui.param("script") == Some("off") {
        page.without_script()
    } else {
        page
    }
}

async fn index(ui: Ui) -> Page {
    let linen = ui.param("palette") == Some("linen");
    let page = page(
        &ui,
        "Components",
        html! {
            @let count = |groups: &[&str]| COMPONENTS.iter().filter(|c| groups.contains(&c.2)).count();
            p class="lui-lede" { (count(LAYERS[0].2)) " primitives, " (count(LAYERS[1].2)) " components, " (count(LAYERS[2].2)) " widgets and one of your own, for Axum and Maud, all working with JavaScript turned off. The HTML platform and plain form posts do the work. Each page loads one optional script, " code { "/lui/enhance.js" } ", which updates the same markup in place instead of reloading. Block it and every page still works." }
            @if !ui.has(Cap::Probed) { p class="lui-note" { "First visit: this page is the fallback variant. Reload and the server will know your browser." } }
            p class="lui-note" { "Theme: " @if linen { a href="/" { "neutral" } " · linen and copper" } @else { "neutral · " a href="/?palette=linen" { "linen and copper" } } ", see " code { "docs/theming.md" } }
            div class="lui-index" { @for (layer, blurb, groups) in LAYERS {
                h2 { (layer) }
                p class="lui-index-layer" { (blurb) }
                @for group in groups.iter() {
                    @if groups.len() > 1 { h3 { (group) } }
                    ul { @for (href, title, _, feats, what) in COMPONENTS.iter().filter(|c| c.2 == *group) {
                        li { a href=(href) { (title) } div { p { (what) } span { @for f in feats.split(", ") { code { (f) } " " } } } }
                    } }
                }
            } }
            // Idle-time fetch of every component page, so the click is served from cache.
            @for (href, ..) in COMPONENTS { link rel="prefetch" href=(href); }
        },
    );
    if linen { page.tokens(&LINEN) } else { page }
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

#[derive(Deserialize)]
struct LangForm {
    lang: String,
}

/// Keep the picked language and go back to the page the switch was on.
async fn lang_submit(ui: Ui, headers: HeaderMap, Form(f): Form<LangForm>) -> Redirect {
    ui.redirect(&back_to(&headers)).lang(&f.lang)
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
