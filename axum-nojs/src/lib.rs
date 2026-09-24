//! # axum-nojs
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
//! use axum_nojs::prelude::*;
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
//! fallback, never both. See [`caps`] (the `axum-nojs-caps` crate) for how the server learns
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
//! use axum_nojs::{prelude::*, stylesheet};
//!
//! let ui = Ui::from_request("/", "", "nojs-flash=Saved.");
//! let body = ui.flash().render().into_string() + &ui.theme_toggle("/theme").render().into_string();
//! let page = format!("<!DOCTYPE html><style>{}</style><main>{body}</main>", stylesheet());
//! assert!(page.contains("class=\"nojs-theme\"") && page.contains("Saved."));
//! assert!(!page.contains("<script"));
//! ```

// docs.rs builds with nightly and `--cfg docsrs`: feature-gated items get a "requires feature" badge.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![warn(missing_docs)]

pub mod accordion;
pub mod avatar;
pub mod badge;
pub mod breadcrumbs;
pub mod button;
pub mod calendar;
pub mod card;
pub mod cluster;
pub mod color;
pub mod combobox;
pub mod counter;
pub mod dialog;
pub mod drawer;
pub mod empty_state;
pub mod enhance;
pub mod flash;
pub mod form;
pub mod grid;
pub mod icon;
pub mod input;
pub mod layout;
pub mod paged_table;
pub mod pager;
pub mod palette;
pub mod popover;
pub mod range;
#[cfg(feature = "axum")]
pub mod saved;
pub mod select;
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
pub mod ui;
pub mod wizard;

/// Server-side feature detection: the [`axum_nojs_caps`] crate, re-exported so `axum_nojs::caps`
/// keeps working.
pub use axum_nojs_caps as caps;

pub use axum_nojs_caps::{Cap, Caps};
pub use icon::Icon;
pub use popover::MenuItem;
#[cfg(feature = "axum")]
pub use saved::Saved;
pub use state::UiState;
#[cfg(feature = "http")]
pub use stream::Streamed;
pub use table::Row;
pub use theme::Theme;
pub use ui::{Page, Redirect, Ui};

/// Everything a handler needs, in one import: `use axum_nojs::prelude::*;`.
pub mod prelude {
    #[cfg(feature = "axum")]
    pub use crate::Saved;
    pub use crate::{Cap, Caps, Icon, MenuItem, Page, Redirect, Theme, Ui};
    pub use maud::{Markup, Render, html};
}

/// A key made safe for an `id`: anything but letters, digits, `-` and `_` becomes `-`, and
/// ASCII letters are lowercased, so `"Account"` gives `account`.
pub(crate) fn slug(key: &str) -> String {
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

/// A control under its label in a `div.nojs-field`, as the form component lays out its
/// fields; the control alone when there is no label. `id` is the control's id.
pub(crate) fn labelled(label: Option<&str>, id: &str, control: maud::Markup) -> maud::Markup {
    match label {
        Some(text) => {
            maud::html! { div class="nojs-field" { label for=(id) { (text) } (control) } }
        }
        None => control,
    }
}

/// The `nojs-gap-<n>` class for a layout primitive's `.gap(n)`: the step of the
/// `--nojs-space-*` scale (0, 1, 2, 3, 4, 6, 8) nearest `n`, rounding down between two.
pub(crate) fn gap_class(n: u8) -> &'static str {
    match n {
        0 => "nojs-gap-0",
        1 => "nojs-gap-1",
        2 => "nojs-gap-2",
        3 => "nojs-gap-3",
        4 | 5 => "nojs-gap-4",
        6 | 7 => "nojs-gap-6",
        _ => "nojs-gap-8",
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
/// `var(--nojs-*)` only; a test below checks that no literal slips in.
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
    dialog::CSS,
    popover::CSS,
    tabs::CSS,
    accordion::CSS,
    combobox::CSS,
    pager::CSS,
    form::CSS,
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

    /// Theming is tokens only: every colour in component CSS is a `var(--nojs-*)`, so a palette
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

    /// Buttons and inputs are styled in one place each: no other component's CSS selects a
    /// bare `button` or `input` (anywhere in a selector, `:is()` and `:where()` included), so
    /// a change to the primitive reaches every component. A component styles its own parts
    /// by class (`.nojs-counter-input`, `.nojs-dialog-close`).
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
        assert!(
            !selects_bare(".nojs-button", "button") && !selects_bare("[type=button]", "button")
        );
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
        let ui = Ui::from_request("/", "", "nojs-flash=hi");
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
