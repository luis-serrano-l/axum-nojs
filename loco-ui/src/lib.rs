//! # loco-ui
//!
//! Interactive HTML components for Rust servers that work with JavaScript turned off.
//!
//! A handler takes one [`Ui`] (what the browser supports, its theme, the page's UI state and
//! query) and describes the page with it. Every component starts from `ui`, reads what it
//! needs from the request itself, and renders where `html!` splices it. Interactivity comes
//! from the HTML and CSS platform (dialog, popover, invokers, `<details name>`, datalist, view
//! transitions) and from ordinary form round trips. No page produced by this crate needs a
//! `<script>` tag; the one optional script in [`enhance`] only makes the same markup update
//! in place.
//!
//! ```rust
//! use loco_ui::prelude::*;
//!
//! // In Axum, `ui: Ui` is an extractor; anywhere else, build it from the request.
//! let ui = Ui::from_request("/", "", "");
//! let page = ui.page("Hello", html! {
//!     (ui.dialog("Say hi").body(html! { p { "Hello from a <dialog>." } }))
//!     (ui.tabs("intro")
//!         .tab("One", html! { p { "First." } })
//!         .tab("Two", html! { p { "Second." } }))
//! });
//! // The only script is the optional enhancement tag; the page works without it.
//! assert_eq!(page.into_string().matches("<script").count(), 1);
//! ```
//!
//! Each component emits only the markup that browser needs: the modern variant or the
//! fallback, never both. See [`caps`] (the `loco-ui-caps` crate) for how the server learns
//! it. One component lives in one file; each file starts with a doc header that lists the
//! platform features it relies on, the browser baseline, and the fallback.
//!
//! ## Without Maud templates
//!
//! A component renders to [`maud::Markup`], which is `PreEscaped<String>`: `.render()` then
//! `.into_string()` hands the HTML to anything else (another template engine, a plain
//! `String` body in any server, a file). [`stylesheet`] is a `String` too.
//!
//! ```rust
//! use loco_ui::{prelude::*, stylesheet};
//!
//! let ui = Ui::from_request("/", "", "lui-flash=Saved.");
//! let body = ui.flash().render().into_string() + &ui.theme_toggle("/theme").render().into_string();
//! let page = format!("<!DOCTYPE html><style>{}</style><main>{body}</main>", stylesheet());
//! assert!(page.contains("class=\"lui-theme\"") && page.contains("Saved."));
//! assert!(!page.contains("<script"));
//! ```

// docs.rs builds with nightly and `--cfg docsrs`: feature-gated items get a "requires feature" badge.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub mod accordion;
pub mod alert;
pub mod avatar;
pub mod badge;
pub mod blocks;
pub mod breadcrumbs;
pub mod button;
pub mod calendar;
pub mod card;
pub mod chart;
pub mod cluster;
pub mod color;
pub mod combobox;
pub mod context_menu;
pub mod counter;
pub mod date_picker;
pub mod description_list;
pub mod dialog;
pub mod drawer;
pub mod empty_state;
pub mod enhance;
pub mod error_summary;
pub mod flash;
pub mod form;
pub mod grid;
pub mod i18n;
pub mod icon;
pub mod input;
pub mod input_otp;
pub mod kanban;
pub mod layout;
#[cfg(feature = "loco")]
pub mod loco;
pub mod meter;
pub mod nav_menu;
pub mod paged_table;
pub mod pager;
pub mod palette;
pub mod popover;
pub mod progress;
pub mod props;
pub mod range;
#[cfg(feature = "axum")]
pub mod saved;
pub mod select;
pub mod separator;
pub mod sidebar;
pub mod skeleton;
pub mod spec;
pub mod split;
pub mod stack;
pub mod stat;
pub mod state;
#[cfg(feature = "http")]
pub mod stream;
pub mod table;
pub mod tabs;
pub mod theme;
pub mod toast;
pub mod toggle_group;
pub mod tooltip;
pub mod ui;
pub mod upload;
pub mod wizard;

/// Server-side feature detection: the [`loco_ui_caps`] crate, re-exported so `loco_ui::caps`
/// keeps working.
pub use loco_ui_caps as caps;

pub use loco_ui_caps::{Cap, Caps};
/// Every builder with its constructors and setters: see [`props`](mod@props).
pub fn props() -> &'static [props::Component] {
    props::COMPONENTS
}

pub use icon::Icon;
/// Maud's `html!` with components written like elements: see [`loco_ui_macros`].
pub use loco_ui_macros::lui;
pub use popover::MenuItem;
#[cfg(feature = "axum")]
pub use saved::Saved;
pub use state::UiState;
#[cfg(feature = "http")]
pub use stream::Streamed;
pub use table::Row;
pub use theme::Theme;
pub use ui::{Page, Redirect, Ui};

/// `docs/components.md`, whose code runs as a doctest so the guide keeps compiling.
#[cfg(doctest)]
#[doc = include_str!("../../docs/components.md")]
pub struct ComponentsGuide;

/// Everything a handler needs, in one import: `use loco_ui::prelude::*;`.
pub mod prelude {
    #[cfg(feature = "axum")]
    pub use crate::Saved;
    pub use crate::lui;
    pub use crate::{Cap, Caps, Icon, MenuItem, Page, Redirect, Theme, Ui};
    pub use maud::{Markup, Render, html};
}

/// A key made safe for an `id`: anything but letters, digits, `-` and `_` becomes `-`, and
/// ASCII letters are lowercased, so `"Account"` gives `account`. Components derive the ids
/// a caller does not care about from a label this way; so can yours.
///
/// ```rust
/// assert_eq!(loco_ui::slug("Billing & plans"), "billing---plans");
/// ```
pub fn slug(key: &str) -> String {
    key.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect()
}

/// A control under its label in a `div.lui-field`, as the form component lays out its
/// fields; the control alone when there is no label. `id` is the control's id.
pub(crate) fn labelled(label: Option<&str>, id: &str, control: maud::Markup) -> maud::Markup {
    match label {
        Some(text) => {
            maud::html! { div class="lui-field" { label for=(id) { (text) } (control) } }
        }
        None => control,
    }
}

/// The `lui-gap-<n>` class for a layout primitive's `.gap(n)`: the step of the
/// `--lui-space-*` scale (0, 1, 2, 3, 4, 6, 8) nearest `n`, rounding down between two.
pub(crate) fn gap_class(n: u8) -> &'static str {
    match n {
        0 => "lui-gap-0",
        1 => "lui-gap-1",
        2 => "lui-gap-2",
        3 => "lui-gap-3",
        4 | 5 => "lui-gap-4",
        6 | 7 => "lui-gap-6",
        _ => "lui-gap-8",
    }
}

/// All component stylesheets, concatenated once per process. `layout` inlines this once per page.
pub fn stylesheet() -> &'static str {
    static CSS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    CSS.get_or_init(|| {
        let beacons = caps::beacon_css();
        let tokens = layout::Tokens::default().css();
        let mut parts = vec![tokens.as_str()];
        parts.extend(COMPONENT_CSS);
        parts.extend(blocks::CSS);
        parts.push(beacons.as_str());
        minify_css(&parts.join("\n"))
    })
}

/// Strip comments and collapse whitespace in a stylesheet, leaving quoted strings alone. A
/// space survives only where CSS needs one: between two words (`0 8px`, `.a .b`, `and (`),
/// never next to `{ } ; , >`, never after `(` or `:` and never before `)`. A space before
/// `:` stays, since `.a :focus` and `.a:focus` differ. [`stylesheet`] applies it once.
pub fn minify_css(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut chars = css.chars().peekable();
    let mut space = false;
    while let Some(c) = chars.next() {
        match c {
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut prev = ' ';
                for c in chars.by_ref() {
                    if prev == '*' && c == '/' {
                        break;
                    }
                    prev = c;
                }
                space = true;
            }
            c if c.is_whitespace() => space = true,
            '"' | '\'' => {
                if space && !out.is_empty() && !out.ends_with(|p: char| "{};,>(:".contains(p)) {
                    out.push(' ');
                }
                space = false;
                out.push(c);
                let mut escaped = false;
                for d in chars.by_ref() {
                    out.push(d);
                    if d == c && !escaped {
                        break;
                    }
                    escaped = d == '\\' && !escaped;
                }
            }
            _ => {
                let tight = |p: char| "{};,>".contains(p);
                if space
                    && !out.is_empty()
                    && !tight(c)
                    && c != ')'
                    && !out.ends_with(|p: char| tight(p) || p == '(' || p == ':')
                {
                    out.push(' ');
                }
                space = false;
                if c == '}' && out.ends_with(';') {
                    out.pop();
                }
                out.push(c);
            }
        }
    }
    out
}

/// Every component's `CSS`, in the order the stylesheet includes them. Colours in here are
/// `var(--lui-*)` only; a test below checks that no literal slips in.
pub const COMPONENT_CSS: &[&str] = &[
    layout::CSS,
    button::CSS,
    input::CSS,
    badge::CSS,
    card::CSS,
    icon::CSS,
    avatar::CSS,
    stack::CSS,
    cluster::CSS,
    grid::CSS,
    split::CSS,
    calendar::CSS,
    date_picker::CSS,
    upload::CSS,
    kanban::CSS,
    tooltip::CSS,
    alert::CSS,
    progress::CSS,
    meter::CSS,
    separator::CSS,
    dialog::CSS,
    popover::CSS,
    tabs::CSS,
    accordion::CSS,
    combobox::CSS,
    pager::CSS,
    form::CSS,
    error_summary::CSS,
    counter::CSS,
    theme::CSS,
    flash::CSS,
    select::CSS,
    range::CSS,
    color::CSS,
    #[cfg(feature = "http")]
    stream::CSS,
    table::CSS,
    paged_table::CSS,
    wizard::CSS,
    toast::CSS,
    breadcrumbs::CSS,
    skeleton::CSS,
    empty_state::CSS,
    stat::CSS,
    chart::CSS,
    sidebar::CSS,
    nav_menu::CSS,
    description_list::CSS,
    toggle_group::CSS,
    context_menu::CSS,
    input_otp::CSS,
    drawer::CSS,
    palette::CSS,
];

#[cfg(test)]
mod tests {
    use super::*;
    use maud::{Render, html};

    #[test]
    fn tuples_build_the_same_items_as_the_constructors() {
        let ui = Ui::from(Caps::all());
        let render = |r: &dyn Render| r.render().into_string();
        assert_eq!(
            MenuItem::from(("Profile", "/p")),
            MenuItem::link("Profile", "/p")
        );
        let sizes = [("s", "Small", "🐭")];
        assert_eq!(
            render(&ui.select("size", "s").options(sizes)),
            render(
                &ui.select("size", "s")
                    .options([select::SelectOption::new("s", "Small").icon("🐭")])
            )
        );
        assert_eq!(
            render(&ui.menu("M").submenu("Sub", [("A", "/a")])),
            render(&ui.menu("M").submenu("Sub", [MenuItem::link("A", "/a")]))
        );
    }

    #[test]
    fn ids_come_from_labels() {
        let ui = Ui::from(Caps::all());
        let html = |r: &dyn Render| r.render().into_string();
        assert!(html(&ui.menu("My account")).contains(r#"id="my-account""#));
        assert!(html(&ui.drawer("Menu")).contains(r#"id="menu""#));
        assert!(html(&ui.dialog("Delete account").id("confirm")).contains(r#"id="confirm""#));
    }

    #[test]
    fn minified_stylesheet_keeps_every_rule() {
        let css = minify_css(
            "/* note */ .a  .b > p ,\n a:hover { margin: 0  8px ; content: \"  ← \" }\n@media (min-width: 60rem) and (x) { .c { top: calc(1px + 2px); } }",
        );
        assert_eq!(
            css,
            r#".a .b>p,a:hover{margin:0 8px;content:"  ← "}@media (min-width:60rem) and (x){.c{top:calc(1px + 2px)}}"#
        );
        let full = [
            layout::Tokens::default().css().as_str(),
            &COMPONENT_CSS.concat(),
            &caps::beacon_css(),
        ]
        .concat();
        let min = stylesheet();
        assert!(
            min.len() * 10 < full.len() * 9,
            "at least a tenth smaller: {} of {}",
            min.len(),
            full.len()
        );
        for pair in [('{', '}'), ('(', ')')] {
            assert_eq!(
                min.matches(pair.0).count(),
                min.matches(pair.1).count(),
                "balanced {pair:?}"
            );
        }
        assert!(!min.contains("/*"), "no comments left");
        assert_eq!(minify_css(min), min, "minifying twice changes nothing");
    }

    /// Theming is tokens only: every colour in component CSS is a `var(--lui-*)`, so a palette
    /// passed to `layout_with` reaches everything. Literals live in `layout::Tokens` alone.
    #[test]
    fn no_colour_literal_outside_tokens() {
        for css in COMPONENT_CSS {
            for line in css.lines() {
                let hex = line.char_indices().any(|(i, c)| {
                    c == '#'
                        && line[i + 1..]
                            .chars()
                            .take_while(|c| c.is_ascii_hexdigit())
                            .count()
                            >= 3
                });
                let func = ["rgb(", "rgba(", "hsl(", "hsla(", "oklch(", "light-dark("]
                    .iter()
                    .any(|f| line.contains(f));
                // `white-space` is a property, not a colour.
                let named = line
                    .replace("white-space", "")
                    .split(|c: char| !c.is_ascii_alphabetic())
                    .any(|w| {
                        ["white", "black", "gray", "grey", "red", "blue", "green"].contains(&w)
                    });
                assert!(
                    !hex && !func && !named,
                    "colour literal in component CSS: {line}"
                );
            }
        }
    }

    /// Builders are plain data: every public struct derives or implements `Clone` and `Debug`,
    /// so a route can keep one in a variable, build it in a loop or print it. `Streamed` owns
    /// futures and is `Debug` only; the doctest marker `ComponentsGuide` is not a type anyone
    /// holds.
    #[test]
    fn every_builder_is_clone_and_debug() {
        let mut missing = Vec::new();
        for path in library_sources() {
            let source = std::fs::read_to_string(&path).unwrap();
            let mut attrs = String::new();
            for line in source.lines().map(str::trim) {
                if line.starts_with("#[") {
                    attrs.push_str(line);
                } else if let Some(rest) = line.strip_prefix("pub struct ") {
                    let name: String = rest
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    let has = |t: &str| {
                        attrs.contains(t)
                            || source.contains(&format!("impl {t} for {name}"))
                            || source.contains(&format!("impl std::fmt::{t} for {name}"))
                            || source.contains(&format!("impl fmt::{t} for {name}"))
                    };
                    let exempt_clone = name == "Streamed" || name == "ComponentsGuide";
                    if (!has("Clone") && !exempt_clone)
                        || (!has("Debug") && name != "ComponentsGuide")
                    {
                        missing.push(format!("{}: {name}", path.display()));
                    }
                    attrs.clear();
                } else if !line.starts_with("///") && !line.starts_with("//") {
                    attrs.clear();
                }
            }
        }
        assert!(missing.is_empty(), "not Clone + Debug: {missing:?}");
    }

    /// A builder with a `**Setters.**` paragraph, read from the source: its name, doc, the
    /// file's source and every setter (`pub fn` taking `self` and returning `Self`) of its
    /// `impl` blocks as (name, arguments after `self`).
    /// Every word a component writes by itself comes from `i18n::Strings`, so a translated
    /// table reaches every part. This reads each component's code (not its docs, tests, CSS or
    /// `PROPS`) for string literals that look like text for people: a capitalised word, or two
    /// words with a space. What is left is markup syntax, attribute values and keys.
    #[test]
    fn components_write_no_english_outside_the_string_table() {
        // Files that are not components, or whose text is not for the visitor: the table
        // itself, the spec and props lists, the script, Loco glue (its form messages are
        // `FieldErrors`' own, overridable by the app), icons (SVG paths), the site header in
        // `layout` (the demo's brand line) and debug output.
        const SKIP: [&str; 8] = [
            "i18n.rs",
            "spec.rs",
            "props.rs",
            "lib.rs",
            "enhance.rs",
            "loco.rs",
            "icon.rs",
            "layout.rs",
        ];
        let mut found = Vec::new();
        for path in library_sources() {
            let name = path.file_name().unwrap().to_string_lossy().to_string();
            if SKIP.contains(&name.as_str()) {
                continue;
            }
            let source = std::fs::read_to_string(&path).unwrap();
            let code = source.split("#[cfg(test)]\nmod tests").next().unwrap();
            let (mut in_css, mut in_props) = (false, false);
            for (n, line) in code.lines().enumerate() {
                let t = line.trim();
                if t.starts_with("pub const CSS") {
                    in_css = true;
                }
                if in_css {
                    in_css = !t.starts_with("\"#;");
                    continue;
                }
                if t.starts_with("pub const PROPS") {
                    in_props = true;
                }
                if in_props {
                    in_props = t != "];";
                    continue;
                }
                let skip = t.starts_with("//")
                    || t.starts_with("Prop::new")
                    || t.starts_with(".doc(")
                    || t.starts_with(".default(")
                    || t.contains("expect(")
                    || t.contains("debug_struct(")
                    || t.contains("write_str(")
                    || t.contains("debug_tuple(")
                    || t.contains("r#\"") // raw strings are SVG or CSS written by hand
                    || t.contains("write!(f");
                if skip {
                    continue;
                }
                for literal in literals(t) {
                    if looks_like_text(literal) {
                        found.push(format!("{name}:{}: \"{literal}\"", n + 1));
                    }
                }
            }
        }
        assert!(
            found.is_empty(),
            "English outside i18n::Strings:\n{}",
            found.join("\n")
        );
    }

    /// The string literals on one line (not raw strings; escapes kept as written).
    fn literals(line: &str) -> Vec<&str> {
        let mut out = Vec::new();
        let mut rest = line;
        while let Some(start) = rest.find('"') {
            let body = &rest[start + 1..];
            let mut end = None;
            let mut escaped = false;
            for (i, c) in body.char_indices() {
                match c {
                    '\\' if !escaped => escaped = true,
                    '"' if !escaped => {
                        end = Some(i);
                        break;
                    }
                    _ => escaped = false,
                }
            }
            let Some(end) = end else { break };
            out.push(&body[..end]);
            rest = &body[end + 1..];
        }
        out
    }

    /// A capitalised word followed by lowercase ("Next"), or two words with a space
    /// ("Load more", " of "), once format placeholders are taken out.
    fn looks_like_text(literal: &str) -> bool {
        let mut plain = String::new();
        let mut depth = 0;
        for c in literal.chars() {
            match c {
                '{' => depth += 1,
                '}' => depth -= 1,
                _ if depth == 0 => plain.push(c),
                _ => {}
            }
        }
        // A class, inline CSS, markup, a cookie or header value, a key shortcut.
        let syntax = plain.trim_start().starts_with("lui-")
            || plain.contains("--")
            || plain.contains('<')
            || plain.contains('>')
            || plain.contains("Path=/")
            || plain.contains("charset")
            || plain.contains("Alt+")
            || plain.contains("span-")
            || plain.contains("position-area")
            || plain.split_once(": ").is_some_and(|(k, _)| {
                !k.is_empty() && k.chars().all(|c| c.is_ascii_lowercase() || c == '-')
            });
        if syntax {
            return false;
        }
        let words: Vec<&str> = plain
            .split(|c: char| !c.is_ascii_alphabetic())
            .filter(|w| w.len() >= 2)
            .collect();
        let capitalised = words.iter().any(|w| {
            let mut cs = w.chars();
            cs.next().is_some_and(|c| c.is_ascii_uppercase())
                && cs.next().is_some_and(|c| c.is_ascii_lowercase())
        });
        let sentence = plain.contains(' ')
            && words
                .iter()
                .any(|w| w.chars().all(|c| c.is_ascii_lowercase()));
        let aside = plain.starts_with('(') && plain.ends_with(')') && words.len() == 1; // "(skipped)"
        capitalised || sentence || aside
    }

    /// Every component header says what it offers assistive tech and that axe checked it.
    #[test]
    fn every_component_header_has_an_accessibility_line() {
        let mut missing = Vec::new();
        for path in library_sources() {
            if path.ends_with("loco.rs") {
                continue; // Loco glue, not a component
            }
            let source = std::fs::read_to_string(&path).unwrap();
            let header: String = source
                .lines()
                .take_while(|l| l.starts_with("//!"))
                .collect();
            if header.contains("**What it does not do without script:**")
                && !header.contains("**Accessibility:**")
            {
                missing.push(path.file_name().unwrap().to_string_lossy().to_string());
            }
        }
        assert!(
            missing.is_empty(),
            "no **Accessibility:** line: {missing:?}"
        );
    }

    /// The library's own files, `src/*.rs` and `src/blocks/*.rs` (not `src/bin/`, the
    /// installer).
    fn library_sources() -> Vec<std::path::PathBuf> {
        let rs = |dir: &str| {
            let entries = std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().path());
            entries
                .filter(|p| p.extension().is_some_and(|x| x == "rs"))
                .collect::<Vec<_>>()
        };
        let mut files = rs(concat!(env!("CARGO_MANIFEST_DIR"), "/src"));
        files.extend(rs(concat!(env!("CARGO_MANIFEST_DIR"), "/src/blocks")));
        files
    }

    struct Builder {
        name: String,
        doc: String,
        source: String,
        setters: Vec<(String, String)>,
    }

    fn builders() -> Vec<Builder> {
        let mut found = Vec::new();
        for path in library_sources() {
            let source = std::fs::read_to_string(path).unwrap();
            let lines: Vec<&str> = source.lines().collect();
            for (i, line) in lines.iter().enumerate() {
                let Some(rest) = line.strip_prefix("pub struct ") else {
                    continue;
                };
                let name: String = rest
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();
                let doc: String = lines[..i]
                    .iter()
                    .rev()
                    .take_while(|l| l.starts_with("///") || l.starts_with("#["))
                    .copied()
                    .collect();
                if !doc.contains("**Setters.**") {
                    continue;
                }
                // Each `impl Name` / `impl<'a> Name<'a>` block up to its closing `}`.
                let mut in_impl = false;
                let mut body = String::new();
                for l in &lines {
                    if l.starts_with("impl")
                        && !l.contains(" for ")
                        && (l.contains(&format!(" {name} ")) || l.contains(&format!(" {name}<")))
                    {
                        in_impl = true;
                    } else if in_impl && *l == "}" {
                        in_impl = false;
                    } else if in_impl {
                        body.push_str(l);
                        body.push('\n');
                    }
                }
                let mut setters = Vec::new();
                for (at, _) in body
                    .match_indices("pub fn ")
                    .chain(body.match_indices("pub const fn "))
                {
                    let sig = &body[at..body[at..].find('{').map_or(body.len(), |e| at + e)];
                    let setter: String = sig[sig.find("fn ").unwrap() + "fn ".len()..]
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    if !(sig.contains("self") && sig.contains("-> Self")) {
                        continue;
                    }
                    // The arguments after `self`, on one line and without a trailing comma.
                    let open = sig.find('(').unwrap();
                    let close = sig.rfind(") ->").unwrap();
                    let args = sig[open + 1..close]
                        .split_whitespace()
                        .collect::<Vec<_>>()
                        .join(" ");
                    let args = args
                        .trim_start_matches("mut ")
                        .trim_start_matches("self")
                        .trim_start_matches(',')
                        .trim()
                        .trim_end_matches(',')
                        .replace("( ", "(")
                        .replace(" )", ")");
                    setters.push((setter, args));
                }
                found.push(Builder {
                    name,
                    doc,
                    source: source.clone(),
                    setters,
                });
            }
        }
        assert!(found.len() > 30, "found only {} builders", found.len());
        found
    }

    /// Every option discoverable in one place: a builder whose doc has a `**Setters.**`
    /// paragraph names every setter of its `impl` blocks there (grouped by the convention:
    /// values and items, no-argument switches, `bool` conditions).
    #[test]
    fn every_setter_is_listed_on_its_builder() {
        let mut missing = Vec::new();
        for b in builders() {
            for (setter, _) in &b.setters {
                if !b.doc.contains(&format!("`.{setter}(")) {
                    missing.push(format!("{}::{setter}", b.name));
                }
            }
        }
        assert!(
            missing.is_empty(),
            "setters missing from their builder's **Setters.** list: {missing:?}"
        );
    }

    /// `props()` lists every builder that has setters, and every `ui.<name>(..)` that returns
    /// a listed builder is among its constructors.
    #[test]
    fn props_lists_every_builder_and_constructor() {
        let listed = |name: &str| crate::props().iter().find(|c| c.builder == name);
        let mut missing = Vec::new();
        for b in builders() {
            if listed(&b.name).is_none() {
                missing.push(b.name);
            }
        }
        for path in library_sources() {
            let source = std::fs::read_to_string(path).unwrap();
            for block in source.split("\nimpl Ui {\n").skip(1) {
                let block = &block[..block.find("\n}\n").unwrap_or(block.len())];
                for sig in block.split("pub fn ").skip(1) {
                    let sig = &sig[..sig.find('{').unwrap_or(sig.len())];
                    let method: String = sig
                        .chars()
                        .take_while(|c| c.is_alphanumeric() || *c == '_')
                        .collect();
                    let ret: String = sig
                        .rsplit("-> ")
                        .next()
                        .unwrap()
                        .chars()
                        .take_while(|c| c.is_alphanumeric())
                        .collect();
                    if let Some(c) = listed(&ret)
                        && !c
                            .calls
                            .iter()
                            .any(|call| call.starts_with(&format!("ui.{method}(")))
                    {
                        missing.push(format!("ui.{method} -> {ret}"));
                    }
                }
            }
        }
        assert!(missing.is_empty(), "not in props(): {missing:?}");
    }

    /// Every component's header shows it both ways: its doctest builds it with the dot form and
    /// again with `lui!`, and asserts the two render the same HTML.
    #[test]
    fn every_component_header_shows_the_lui_form() {
        // Spec entries with no `ui.<component>(..)` of their own.
        let without_builder = ["enhance", "layout", "caps", "state", "paged_table"];
        let mut missing = Vec::new();
        for spec in crate::spec::SPECS {
            if without_builder.contains(&spec.module) {
                continue;
            }
            let path = format!("{}/src/{}.rs", env!("CARGO_MANIFEST_DIR"), spec.module);
            let source = std::fs::read_to_string(path).unwrap();
            let header: String = source
                .lines()
                .take_while(|l| l.starts_with("//!") || l.is_empty())
                .collect::<Vec<_>>()
                .join("\n");
            if !(header.contains("lui!") && header.contains("assert_eq!(")) {
                missing.push(spec.module);
            }
        }
        assert!(
            missing.is_empty(),
            "headers without a `lui!` twin: {missing:?}"
        );
    }

    /// Every setter is in its builder's `PROPS` with the arguments it takes, nothing else is,
    /// and the kind agrees with the arguments (a switch takes none, a condition one `bool`).
    #[test]
    fn every_setter_is_in_props() {
        let mut wrong = Vec::new();
        for b in builders() {
            // The builder's own `PROPS` block: an `impl Name` holding `pub const PROPS`.
            let head = [
                format!("\nimpl {} {{", b.name),
                format!("\nimpl {}<'_> {{", b.name),
            ];
            let block = head.iter().find_map(|h| {
                b.source.match_indices(h.as_str()).find_map(|(at, _)| {
                    let block = &b.source[at..at + b.source[at..].find("\n}").unwrap()];
                    block.contains("pub const PROPS").then_some(block)
                })
            });
            let Some(block) = block else {
                wrong.push(format!("{}: no PROPS", b.name));
                continue;
            };
            // The string literals of `text`, in order, with `\"` unescaped.
            let literals = |text: &str| {
                let mut found = Vec::new();
                let mut chars = text.chars();
                while let Some(c) = chars.next() {
                    if c != '"' {
                        continue;
                    }
                    let mut lit = String::new();
                    while let Some(c) = chars.next() {
                        match c {
                            '\\' => lit.push(chars.next().unwrap()),
                            '"' => break,
                            c => lit.push(c),
                        }
                    }
                    found.push(lit);
                }
                found
            };
            let mut listed = Vec::new();
            for entry in block.split("Prop::new(").skip(1) {
                let lits = literals(entry);
                let (name, args) = (lits[0].clone(), lits[1].clone());
                let kind = entry.split("PropKind::").nth(1).unwrap();
                let kind = &kind[..kind.find(',').unwrap()];
                listed.push((name.clone(), args.clone()));
                let fits = match kind {
                    "Switch" => args.is_empty(),
                    "Condition" => args.ends_with(": bool") && !args.contains(','),
                    _ => true,
                };
                if !fits {
                    wrong.push(format!("{}::{name}: {kind} does not fit `{args}`", b.name));
                }
            }
            for setter in &b.setters {
                if !listed.contains(setter) {
                    wrong.push(format!(
                        "{}::{}({}) not in PROPS",
                        b.name, setter.0, setter.1
                    ));
                }
            }
            for prop in &listed {
                if !b.setters.contains(prop) {
                    wrong.push(format!(
                        "{}::{}({}) in PROPS but not a setter",
                        b.name, prop.0, prop.1
                    ));
                }
            }
        }
        assert!(
            wrong.is_empty(),
            "PROPS out of step with the setters: {wrong:#?}"
        );
    }

    /// Every component's CSS together, inlined once per page, stays under 64 KB (57.6 KB and
    /// 10.3 KB gzipped at M26; README "What a page weighs").
    #[test]
    fn stylesheet_stays_under_its_budget() {
        // 64 KB until M29 added blocks, a chart and six components (README: "What a page weighs").
        assert!(
            stylesheet().len() < 72 * 1024,
            "stylesheet() is {} bytes",
            stylesheet().len()
        );
    }

    /// Buttons and inputs are styled in one place each: no other component's CSS selects a
    /// bare `button` or `input` (anywhere in a selector, `:is()` and `:where()` included), so
    /// a change to the primitive reaches every component. A component styles its own parts
    /// by class (`.lui-counter-input`, `.lui-dialog-close`).
    #[test]
    fn only_the_primitives_select_bare_buttons_and_inputs() {
        fn preludes(css: &str) -> Vec<String> {
            let (mut out, mut buf) = (Vec::new(), String::new());
            for c in minify_css(css).chars() {
                match c {
                    '{' => out.push(std::mem::take(&mut buf)),
                    '}' | ';' => buf.clear(),
                    c => buf.push(c),
                }
            }
            out.retain(|p| !p.starts_with('@'));
            out
        }
        fn selects_bare(selector: &str, element: &str) -> bool {
            selector.match_indices(element).any(|(i, _)| {
                let before = selector[..i].chars().next_back();
                let after = selector[i + element.len()..].chars().next();
                let starts = before.is_none_or(|c| " ,>+~(".contains(c));
                let ends =
                    after.is_none_or(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'));
                starts && ends
            })
        }
        assert!(
            selects_bare(".x :is(button, a)", "button")
                && selects_bare("input[type=range]", "input")
        );
        assert!(!selects_bare(".lui-button", "button") && !selects_bare("[type=button]", "button"));
        for css in COMPONENT_CSS {
            if *css == button::CSS || *css == input::CSS {
                continue;
            }
            for p in preludes(css) {
                for element in ["button", "input"] {
                    assert!(
                        !selects_bare(&p, element),
                        "`{p}` styles a bare {element}: style the part by class, or change {element}.rs"
                    );
                }
            }
        }
    }

    /// Every component nests in `html!` and converts to a plain `String`.
    #[test]
    fn components_render_and_stringify() {
        let ui = Ui::from_request("/", "", "lui-flash=hi");
        let parts: [&dyn Render; 3] = [
            &ui.flash(),
            &ui.counter("/counter", 3),
            &ui.theme_toggle("/theme"),
        ];
        for part in parts {
            let nested = html! { (part) }.into_string();
            assert!(nested.starts_with('<') && nested == part.render().into_string());
        }
    }
}
