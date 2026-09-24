//! # Popover menu
//!
//! A dropdown menu that opens on click, closes on outside click or Escape, no script. Items
//! are links, `<form method="post">` actions, headings, separators and submenus; each item
//! can show an icon and a keyboard shortcut and be disabled or destructive.
//!
//! **Platform features:**
//! - `popover` attribute + `popovertarget` button (Chrome 114, Firefox 125, Safari 17). Light
//!   dismiss and top-layer stacking come for free; a submenu is a nested popover, which the
//!   platform keeps open with its parent.
//! - CSS anchor positioning `anchor-name` / `position-anchor` / `position-area`
//!   (Chrome 125, Firefox 147, Safari 26) to place the menu under its button, at its end, or
//!   to its right (`Placement`).
//!
//! **What it does not do without script:** position itself against the opener where anchor
//! positioning is missing; it is centred instead.
//!
//! **Fallback:** without `Caps::Anchor` the popover is UA-centred, which is still usable.
//! Without `Caps::Popover` the menu is a `<details>` dropdown (a submenu a nested one): it
//! opens and closes on click but has no light dismiss.
//!
//! **Without script:** a shortcut shown beside an item is a label; binding the key needs
//! script the crate does not ship. Arrow keys move between items only with the enhancement
//! script; Tab always works.
//!
//! ```rust
//! use axum_nojs::{Caps, popover_menu, popover_menu_with, popover::{MenuItem, Placement, PopoverOptions}};
//! // The id is derived from the label: this menu is `#account`.
//! let m = popover_menu(&Caps::all(), "Account", &[MenuItem::link("Profile", "/profile"), MenuItem::link("Sign out", "/logout")]);
//! assert!(m.into_string().contains(r#"id="account""#));
//! let m = popover_menu_with(&Caps::all(), "acct", "Account", &[
//!     MenuItem::heading("Signed in as Ada"),
//!     MenuItem::link("Profile", "/profile").icon("@").shortcut("g p"),
//!     MenuItem::link("Billing", "/billing").disabled(),
//!     MenuItem::separator(),
//!     MenuItem::submenu("Theme", "acct-theme", &[MenuItem::link("Light", "/?t=light"), MenuItem::link("Dark", "/?t=dark")]),
//!     MenuItem::separator(),
//!     MenuItem::action("Sign out", "/logout").danger(),
//! ], PopoverOptions::default().placement(Placement::BottomEnd));
//! let html = m.into_string();
//! assert!(html.contains("<form method=\"post\" action=\"/logout\""));
//! assert!(html.contains("position-area: bottom span-left"));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// Where the menu opens relative to its button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Placement {
    /// Below, left edges aligned.
    #[default]
    BottomStart,
    /// Below, right edges aligned.
    BottomEnd,
    /// To the right, top edges aligned.
    Right,
}

impl Placement {
    fn area(self) -> &'static str {
        match self {
            Placement::BottomStart => "bottom span-right",
            Placement::BottomEnd => "bottom span-left",
            Placement::Right => "right span-bottom",
        }
    }
    fn class(self) -> &'static str {
        match self {
            Placement::BottomStart => "nojs-popover-start",
            Placement::BottomEnd => "nojs-popover-end",
            Placement::Right => "nojs-popover-right",
        }
    }
}

/// Options for [`popover_menu`]; `Default::default()` opens below the button, start-aligned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PopoverOptions {
    /// Where the menu opens.
    pub placement: Placement,
}

impl PopoverOptions {
    /// Where the menu opens.
    pub fn placement(mut self, placement: Placement) -> Self {
        self.placement = placement;
        self
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind<'a> {
    Link(&'a str),
    Action(&'a str),
    Heading,
    Separator,
    Submenu(&'a str, &'a [MenuItem<'a>]),
}

/// One entry of a [`popover_menu`]: build with [`MenuItem::link`], [`MenuItem::action`],
/// [`MenuItem::heading`], [`MenuItem::separator`] or [`MenuItem::submenu`], then set
/// `icon`, `shortcut`, `disabled` or `danger`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuItem<'a> {
    kind: Kind<'a>,
    text: &'a str,
    icon: Option<&'a str>,
    shortcut: Option<&'a str>,
    disabled: bool,
    danger: bool,
}

impl<'a> MenuItem<'a> {
    const fn new(kind: Kind<'a>, text: &'a str) -> Self {
        MenuItem { kind, text, icon: None, shortcut: None, disabled: false, danger: false }
    }
    /// A link.
    pub const fn link(text: &'a str, href: &'a str) -> Self {
        Self::new(Kind::Link(href), text)
    }
    /// A `<form method="post">` button posting to `action`: for things that change state.
    pub const fn action(text: &'a str, action: &'a str) -> Self {
        Self::new(Kind::Action(action), text)
    }
    /// A section heading.
    pub const fn heading(text: &'a str) -> Self {
        Self::new(Kind::Heading, text)
    }
    /// A rule between groups.
    pub const fn separator() -> Self {
        Self::new(Kind::Separator, "")
    }
    /// A nested menu; `id` must be unique on the page.
    pub const fn submenu(text: &'a str, id: &'a str, items: &'a [MenuItem<'a>]) -> Self {
        Self::new(Kind::Submenu(id, items), text)
    }
    /// A glyph or emoji shown before the text (decorative, hidden from assistive tech).
    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    /// A shortcut shown after the text, as `<kbd>`; a label only.
    pub const fn shortcut(mut self, keys: &'a str) -> Self {
        self.shortcut = Some(keys);
        self
    }
    /// Shown but not usable: a link without `href`, a button with `disabled`.
    pub const fn disabled(mut self) -> Self {
        self.disabled = true;
        self
    }
    /// Destructive: coloured with `--nojs-danger`.
    pub const fn danger(mut self) -> Self {
        self.danger = true;
        self
    }
}

/// `("Profile", "/profile")`: a link, its text and where it goes.
impl<'a> From<(&'a str, &'a str)> for MenuItem<'a> {
    fn from((text, href): (&'a str, &'a str)) -> Self {
        MenuItem::link(text, href)
    }
}

/// A menu whose id is derived from `label`; use [`popover_menu_with`] to name it.
/// [`popover_menu_with`] takes the options.
pub fn popover_menu(caps: &Caps, label: &str, items: &[MenuItem]) -> Markup {
    popover_menu_with(caps, &crate::slug(label), label, items, Default::default())
}

/// A button labelled `label` that toggles a menu of `items`. `id` must be unique on the page.
pub fn popover_menu_with(caps: &Caps, id: &str, label: &str, items: &[MenuItem], options: PopoverOptions) -> Markup {
    let popover = caps.has(Cap::Popover);
    let anchor = caps.has(Cap::Anchor);
    let list = html! { ul role="menu" { @for it in items { (item(it, popover, anchor)) } } };
    html! {
        @if !popover {
            details class={ "nojs-popover nojs-popover-details " (options.placement.class()) } id=(id) {
                summary { (label) " \u{25be}" }
                nav { (list) }
            }
        } @else if anchor {
            div class={ "nojs-popover nojs-popover-anchored " (options.placement.class()) } style={ "anchor-name: --" (id) } {
                button type="button" popovertarget=(id) { (label) " \u{25be}" }
                nav id=(id) popover style={ "position-anchor: --" (id) "; position-area: " (options.placement.area()) } { (list) }
            }
        } @else {
            div class={ "nojs-popover " (options.placement.class()) } {
                button type="button" popovertarget=(id) { (label) " \u{25be}" }
                nav id=(id) popover { (list) }
            }
        }
    }
}

fn item(it: &MenuItem, popover: bool, anchor: bool) -> Markup {
    let class = format!("nojs-popover-item{}{}", if it.danger { " nojs-popover-danger" } else { "" }, if it.disabled { " nojs-popover-disabled" } else { "" });
    let inner = html! {
        @if let Some(i) = it.icon { span class="nojs-popover-icon" aria-hidden="true" { (i) } }
        span class="nojs-popover-text" { (it.text) }
        @if let Some(k) = it.shortcut { kbd class="nojs-popover-kbd" { (k) } }
    };
    html! {
        @match it.kind {
            Kind::Heading => li role="presentation" class="nojs-popover-heading" { (it.text) },
            Kind::Separator => li role="separator" class="nojs-popover-sep" {},
            Kind::Link(href) => li role="none" {
                @if it.disabled {
                    a class=(class) role="menuitem" aria-disabled="true" { (inner) }
                } @else {
                    a class=(class) role="menuitem" href=(href) { (inner) }
                }
            },
            Kind::Action(action) => li role="none" {
                form method="post" action=(action) {
                    button type="submit" class=(class) role="menuitem" disabled[it.disabled] { (inner) }
                }
            },
            Kind::Submenu(sub_id, items) => li role="none" class="nojs-popover-sub" {
                @let list = html! { ul role="menu" { @for it in items { (item(it, popover, anchor)) } } };
                @if !popover {
                    details class="nojs-popover-details nojs-popover-right" id=(sub_id) {
                        summary class=(class) role="menuitem" aria-haspopup="menu" { (inner) " \u{25b8}" }
                        nav { (list) }
                    }
                } @else if anchor {
                    button type="button" class=(class) role="menuitem" aria-haspopup="menu" popovertarget=(sub_id) style={ "anchor-name: --" (sub_id) } { (inner) " \u{25b8}" }
                    nav id=(sub_id) popover class="nojs-popover-subnav" style={ "position-anchor: --" (sub_id) "; position-area: right span-bottom" } { (list) }
                } @else {
                    button type="button" class=(class) role="menuitem" aria-haspopup="menu" popovertarget=(sub_id) { (inner) " \u{25b8}" }
                    nav id=(sub_id) popover class="nojs-popover-subnav" { (list) }
                }
            },
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-popover { display: inline-block; position: relative; }
.nojs-popover nav {
  padding: var(--nojs-space) 0; min-width: 14rem;
  background: var(--nojs-surface); color: var(--nojs-fg);
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
  box-shadow: 0 8px 24px color-mix(in srgb, var(--nojs-fg) 14%, transparent);
}
.nojs-popover-anchored > nav, .nojs-popover-details > nav, .nojs-popover-subnav { margin: 0; }
.nojs-popover-anchored > nav { margin-top: 4px; }
/* Popover without anchor positioning: the UA centres it in the viewport; keep that. */
.nojs-popover ul { list-style: none; margin: 0; padding: 0; }
.nojs-popover form { margin: 0; }
.nojs-popover-item {
  display: flex; align-items: center; gap: var(--nojs-space); width: 100%; box-sizing: border-box;
  padding: 0.4rem 1rem; color: inherit; text-decoration: none; text-align: left; font: inherit;
  background: none; border: 0; border-radius: 0; cursor: pointer;
}
.nojs-popover-item:hover, .nojs-popover-item:focus-visible { background: var(--nojs-bg); outline-offset: -2px; }
.nojs-popover-icon { width: 1.25em; text-align: center; color: var(--nojs-muted); }
.nojs-popover-text { flex: 1; }
.nojs-popover-kbd { font: inherit; font-size: 0.8em; color: var(--nojs-muted); background: none; border: 0; padding: 0; }
.nojs-popover-danger { color: var(--nojs-danger); }
.nojs-popover-danger .nojs-popover-icon { color: inherit; }
.nojs-popover-disabled { color: var(--nojs-muted); cursor: default; }
.nojs-popover-disabled:hover { background: none; }
.nojs-popover-heading { padding: 0.25rem 1rem; font-size: 0.8rem; color: var(--nojs-muted); }
.nojs-popover-sep { margin: var(--nojs-space) 0; border-top: 1px solid var(--nojs-line); }
.nojs-popover-sub { position: relative; }
/* <details> fallback: summary styled as the button, menu absolutely positioned by placement. */
.nojs-popover-details > summary {
  list-style: none; cursor: pointer; background: var(--nojs-surface);
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); padding: 0.5rem 1rem;
}
.nojs-popover-details > summary::-webkit-details-marker { display: none; }
.nojs-popover-details > nav { position: absolute; z-index: 10; }
.nojs-popover-start.nojs-popover-details > nav { top: 100%; left: 0; margin-top: 4px; }
.nojs-popover-end.nojs-popover-details > nav { top: 100%; right: 0; margin-top: 4px; }
.nojs-popover-right.nojs-popover-details > nav { top: 0; left: 100%; margin-left: 4px; }
.nojs-popover-sub > .nojs-popover-details > summary { border: 0; border-radius: 0; background: none; }
"#;
