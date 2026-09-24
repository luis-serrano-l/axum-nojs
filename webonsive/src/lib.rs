//! # webonsive
//!
//! Interactive HTML components for Rust servers that work with JavaScript turned off.
//!
//! Every component is a plain function that returns [`maud::Markup`]. Interactivity comes from
//! the HTML and CSS platform (dialog, popover, invokers, `<details name>`, datalist, view
//! transitions) and from ordinary form round trips. No page produced by this crate needs a
//! `<script>` tag; the one optional script in [`enhance`] only makes the same markup update
//! in place.
//!
//! Every component takes `&Caps` first and emits only the markup that browser needs: the
//! modern variant or the fallback, never both. See [`caps`] (the `wo-caps` crate) for how the
//! server learns it.
//!
//! One component lives in one file. Each file starts with a doc header that lists the platform
//! features it relies on, the browser baseline, and the fallback for older browsers.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, layout, dialog, Theme};
//!
//! // `Caps` says what the browser supports; the demo reads it from a cookie set by beacons.
//! let caps = Caps::all();
//! let page = layout(&caps, "Hello", Theme::Auto, html! {
//!     (dialog(&caps, "hi", "Say hi", html! { p { "Hello from a <dialog>." } }))
//! });
//! // The only script is the optional enhancement tag; the page works without it.
//! assert_eq!(page.into_string().matches("<script").count(), 1);
//! ```
//!
//! ## Without Maud templates
//!
//! [`maud::Markup`] is `PreEscaped<String>`: it implements [`maud::Render`] for nesting in
//! `html!`, and `.into_string()` (or `.0`) hands the HTML to anything else: another template
//! engine, a plain `String` body in any server, a file. [`stylesheet`] is a `String` too, so a
//! page can be assembled by concatenation with no `html!` anywhere.
//!
//! ```rust
//! use webonsive::{Caps, Theme, flash, stylesheet, theme_toggle};
//!
//! let caps = Caps::all();
//! let body: String = flash(&caps, Some("Saved.")).into_string()
//!     + &theme_toggle(&caps, "/theme", Theme::Auto).into_string();
//! let page = format!("<!DOCTYPE html><style>{}</style><main>{body}</main>", stylesheet());
//! assert!(page.contains("class=\"wo-theme\""));
//! assert!(!page.contains("<script"));
//! ```

// docs.rs builds with nightly and `--cfg docsrs`: feature-gated items get a "requires feature" badge.
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

#![warn(missing_docs)]

pub mod accordion;
pub mod breadcrumbs;
pub mod color;
pub mod combobox;
pub mod counter;
pub mod dialog;
pub mod drawer;
pub mod empty_state;
pub mod enhance;
pub mod flash;
pub mod form;
pub mod layout;
pub mod paged_table;
pub mod pager;
pub mod palette;
pub mod popover;
pub mod range;
pub mod select;
pub mod skeleton;
pub mod spec;
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

/// Server-side feature detection: the [`wo_caps`] crate, re-exported so `webonsive::caps`
/// keeps working.
pub use wo_caps as caps;

pub use accordion::{AccordionItem, AccordionOptions, accordion, accordion_with};
pub use breadcrumbs::breadcrumbs;
pub use wo_caps::{Cap, Caps};
pub use color::{ColorOptions, color, color_with};
pub use combobox::{ComboboxOptions, OptionGroup, combobox, combobox_with};
pub use counter::{CounterOptions, counter, counter_with};
pub use dialog::{DialogOptions, DialogSize, dialog, dialog_with};
pub use drawer::{DrawerOptions, drawer, drawer_with};
pub use empty_state::{EmptyOptions, empty_state, empty_state_with};
pub use flash::{FlashOptions, flash, flash_with};
pub use form::{Field, FieldGroup, FieldKind, FormLayout, FormOptions, form, form_with};
pub use layout::{Tokens, layout, layout_with};
pub use paged_table::{PagedTableOptions, paged_table, paged_table_with};
pub use pager::{PagerOptions, pager, pager_with};
pub use palette::{Command, PaletteOptions, command_palette, command_palette_with};
pub use popover::{MenuItem, Placement, PopoverOptions, popover_menu, popover_menu_with};
pub use range::{RangeOptions, range, range_pair, range_pair_with, range_with};
pub use select::{SelectOptions, select, select_with};
pub use skeleton::{SkeletonOptions, skeleton, skeleton_with};
pub use stat::{StatOptions, Trend, stat, stat_with};
#[cfg(feature = "http")]
pub use state::prg;
pub use state::UiState;
#[cfg(feature = "http")]
pub use stream::{Streamed, slot};
pub use table::{Column, Row, TableOptions, cols_from_query, sort_from_query, table, table_with};
pub use tabs::{Tab, TabsOptions, tabs, tabs_with};
pub use theme::{Theme, theme_toggle};
pub use ui::Ui;
pub use toast::{ToastOptions, toasts, toasts_with};
pub use wizard::{WizardOptions, wizard, wizard_with};

/// A key made safe for an `id`: anything but letters, digits, `-` and `_` becomes `-`, and
/// ASCII letters are lowercased, so `"Account"` gives `account`.
pub(crate) fn slug(key: &str) -> String {
    key.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c.to_ascii_lowercase() } else { '-' }).collect()
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
                if space && !out.is_empty() && !tight(c) && c != ')' && !out.ends_with(|p: char| tight(p) || p == '(' || p == ':') {
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
/// `var(--wo-*)` only; a test below checks that no literal slips in.
pub const COMPONENT_CSS: &[&str] = &[
    layout::CSS,
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
    use maud::html;

    #[test]
    fn tuples_build_the_same_items_as_the_constructors() {
        use select::SelectOption;
        let body = || html! { p { "Body" } };
        assert_eq!(MenuItem::from(("Profile", "/p")), MenuItem::link("Profile", "/p"));
        assert_eq!(Command::from(("Settings", "/s")), Command::new("Settings", "/s"));
        assert_eq!(Column::from(("note", "Note")), Column::plain("note", "Note"));
        let o = SelectOption::from(("m", "Medium", "🐕"));
        assert_eq!((o.value, o.text, o.icon), ("m", "Medium", Some("🐕")));
        let options = [o];
        let g: select::Group = ("Sizes", &options[..]).into();
        assert_eq!(g.label, Some("Sizes"));
        // Items that hold markup: compare what they render.
        let tabs_of = |t: Tab| tabs(&Caps::all(), "t", &[t]).into_string();
        assert_eq!(tabs_of(("One", body()).into()), tabs_of(Tab::new("One", body())));
        let acc_of = |a: AccordionItem| accordion(&Caps::all(), "a", &[a]).into_string();
        assert_eq!(acc_of(("Q", body()).into()), acc_of(AccordionItem::new("Q", body())));
        let fields = [Field::new("n", "Name", FieldKind::Text)];
        let fg: FieldGroup = ("Account", &fields[..]).into();
        assert_eq!(fg.legend, Some("Account"));
    }

    #[test]
    fn short_forms_are_the_full_forms_with_default_options() {
        let caps = Caps::all();
        let body = || html! { p { "Body" } };
        assert_eq!(dialog(&caps, "d", "Open", body()).0, dialog_with(&caps, "d", "Open", body(), Default::default()).0);
        assert_eq!(stat(&caps, "Visitors", "12").0, stat_with(&caps, "Visitors", "12", Default::default()).0);
        assert_eq!(flash(&caps, Some("Saved.")).0, flash_with(&caps, Some("Saved."), Default::default()).0);
        // An id the caller does not name is the label's slug.
        let items = [MenuItem::link("Profile", "/p")];
        assert_eq!(popover_menu(&caps, "My account", &items).0, popover_menu_with(&caps, "my-account", "My account", &items, Default::default()).0);
        assert_eq!(drawer(&caps, "Menu", body(), body()).0, drawer_with(&caps, "menu", "Menu", body(), body(), Default::default()).0);
    }

    #[test]
    fn minified_stylesheet_keeps_every_rule() {
        let css = minify_css("/* note */ .a  .b > p ,\n a:hover { margin: 0  8px ; content: \"  ← \" }\n@media (min-width: 60rem) and (x) { .c { top: calc(1px + 2px); } }");
        assert_eq!(css, r#".a .b>p,a:hover{margin:0 8px;content:"  ← "}@media (min-width:60rem) and (x){.c{top:calc(1px + 2px)}}"#);
        let full = [layout::Tokens::default().css().as_str(), &COMPONENT_CSS.concat(), &caps::beacon_css()].concat();
        let min = stylesheet();
        assert!(min.len() * 10 < full.len() * 9, "at least a tenth smaller: {} of {}", min.len(), full.len());
        for pair in [('{', '}'), ('(', ')')] {
            assert_eq!(min.matches(pair.0).count(), min.matches(pair.1).count(), "balanced {pair:?}");
        }
        assert!(!min.contains("/*"), "no comments left");
        assert_eq!(minify_css(min), min, "minifying twice changes nothing");
    }

    /// Theming is tokens only: every colour in component CSS is a `var(--wo-*)`, so a palette
    /// passed to `layout_with` reaches everything. Literals live in `layout::Tokens` alone.
    #[test]
    fn no_colour_literal_outside_tokens() {
        for css in COMPONENT_CSS {
            for line in css.lines() {
                let hex = line.char_indices().any(|(i, c)| {
                    c == '#' && line[i + 1..].chars().take_while(|c| c.is_ascii_hexdigit()).count() >= 3
                });
                let func = ["rgb(", "rgba(", "hsl(", "hsla(", "oklch(", "light-dark("].iter().any(|f| line.contains(f));
                let named = line
                    .split(|c: char| !c.is_ascii_alphabetic())
                    .any(|w| ["white", "black", "gray", "grey", "red", "blue", "green"].contains(&w));
                assert!(!hex && !func && !named, "colour literal in component CSS: {line}");
            }
        }
    }

    /// Every component's return type nests in `html!` and converts to a plain `String`.
    #[test]
    fn components_render_and_stringify() {
        fn renders<T: maud::Render>(_: &T) {}
        let caps = Caps::all();
        let parts = [
            flash(&caps, Some("hi")),
            counter(&caps, "/counter", 3),
            theme_toggle(&caps, "/theme", Theme::Auto),
        ];
        for part in &parts {
            renders(part);
            assert!(part.clone().into_string().starts_with('<'));
        }
    }
}
