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
//!   to its right (`.align_end()`, `.open_right()`).
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
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! // The id is the label's slug: this menu is `#account`.
//! let m = ui.menu("Account").link("Profile", "/profile").link("Sign out", "/logout");
//! assert!(m.render().into_string().contains(r#"id="account""#));
//! // `icon`, `shortcut`, `disabled` and `danger` apply to the item just added.
//! let m = ui.menu("Account")
//!     .heading("Signed in as Ada")
//!     .link("Profile", "/profile").icon("@").shortcut("g p")
//!     .link("Billing", "/billing").disabled()
//!     .separator()
//!     .submenu("Theme", [("Light", "/?t=light"), ("Dark", "/?t=dark")])
//!     .separator()
//!     .action("Sign out", "/logout").danger()
//!     .align_end();
//! let html = m.render().into_string();
//! assert!(html.contains("<form method=\"post\" action=\"/logout\""));
//! assert!(html.contains("position-area: bottom span-left") && html.contains(r#"id="account-theme""#));
//! // The same in `nojs!`:
//! let same = nojs! { Menu("Account") align_end {
//!     heading "Signed in as Ada";
//!     link "Profile" "/profile" icon="@" shortcut="g p";
//!     link "Billing" "/billing" disabled;
//!     separator();
//!     submenu "Theme" ([("Light", "/?t=light"), ("Dark", "/?t=dark")]);
//!     separator();
//!     action "Sign out" "/logout" danger;
//! } };
//! assert_eq!(same.into_string(), html);
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::props::{Prop, PropKind};
use crate::{Cap, Caps, Icon, Ui, slug};

/// Where the menu opens relative to its button.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Placement {
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

#[derive(Clone, Debug, PartialEq, Eq)]
enum Kind<'a> {
    Link(&'a str),
    Action(&'a str),
    Heading,
    Separator,
    Submenu(Vec<MenuItem<'a>>),
}

/// One entry of a menu, for the places that take a list of them (a table row's menu). A
/// [`Menu`] builds its own with [`Menu::link`] and friends.
#[derive(Clone, Debug, PartialEq, Eq)]
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
        MenuItem {
            kind,
            text,
            icon: None,
            shortcut: None,
            disabled: false,
            danger: false,
        }
    }

    /// A link.
    pub const fn link(text: &'a str, href: &'a str) -> Self {
        Self::new(Kind::Link(href), text)
    }

    /// A `<form method="post">` button posting to `action`: for things that change state.
    pub const fn action(text: &'a str, action: &'a str) -> Self {
        Self::new(Kind::Action(action), text)
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

/// A button that opens a menu, made by [`Ui::menu`]. Items are added in order; `icon`,
/// `shortcut`, `disabled` and `danger` apply to the item added last. Opens below the button,
/// start-aligned, unless told otherwise.
///
/// **Setters.** Values and items: `.submenu(..)`, `.link(..)`, `.action(..)`, `.heading(..)`,
/// `.icon(..)`, `.shortcut(..)`, `.id(..)`; switches: `.separator()`, `.disabled()`,
/// `.danger()`, `.align_end()`, `.open_right()`.
#[derive(Clone, Debug)]
pub struct Menu<'a> {
    caps: Caps,
    id: String,
    label: &'a str,
    items: Vec<MenuItem<'a>>,
    placement: Placement,
}

impl Menu<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("link", PropKind::Item, "text: &'a str, href: &'a str").doc("A link."),
        Prop::new("action", PropKind::Item, "text: &'a str, action: &'a str")
            .attr("action")
            .doc("A `<form method=\"post\">` button posting to `action`."),
        Prop::new("heading", PropKind::Item, "text: &'a str").doc("A section heading."),
        Prop::new("separator", PropKind::Item, "").doc("A rule between groups."),
        Prop::new(
            "submenu",
            PropKind::Item,
            "text: &'a str, items: impl IntoIterator<Item = I>",
        )
        .doc("A nested menu of `items` (`MenuItem`s or `(text, href)` links)."),
        Prop::new("icon", PropKind::Modifier, "icon: &'a str").doc(
            "A glyph or emoji before the item's text (decorative, hidden from assistive tech).",
        ),
        Prop::new("shortcut", PropKind::Modifier, "keys: &'a str")
            .doc("A shortcut shown after the item's text, as `<kbd>`."),
        Prop::new("disabled", PropKind::Modifier, "")
            .attr("disabled")
            .doc("The item is shown but not usable."),
        Prop::new("danger", PropKind::Modifier, "").doc("The item is destructive."),
        Prop::new("id", PropKind::Value, "id: &str")
            .attr("id")
            .doc("The menu's id instead of the label's slug."),
        Prop::new("align_end", PropKind::Switch, "").doc(
            "Open below the button with right edges aligned, for a button at the end of a row.",
        ),
        Prop::new("open_right", PropKind::Switch, "").doc("Open to the right of the button."),
    ];
}

impl Ui {
    /// A menu behind a button labelled `label`; its id is the label's slug.
    pub fn menu<'a>(&self, label: &'a str) -> Menu<'a> {
        Menu {
            caps: self.caps,
            id: slug(label),
            label,
            items: Vec::new(),
            placement: Placement::default(),
        }
    }
}

impl<'a> Menu<'a> {
    fn push(mut self, item: MenuItem<'a>) -> Self {
        self.items.push(item);
        self
    }

    fn last(mut self, change: impl FnOnce(&mut MenuItem<'a>)) -> Self {
        if let Some(item) = self.items.last_mut() {
            change(item);
        }
        self
    }

    /// A link.
    pub fn link(self, text: &'a str, href: &'a str) -> Self {
        self.push(MenuItem::link(text, href))
    }

    /// A `<form method="post">` button posting to `action`: for things that change state.
    pub fn action(self, text: &'a str, action: &'a str) -> Self {
        self.push(MenuItem::action(text, action))
    }

    /// A section heading.
    pub fn heading(self, text: &'a str) -> Self {
        self.push(MenuItem::new(Kind::Heading, text))
    }

    /// A rule between groups.
    pub fn separator(self) -> Self {
        self.push(MenuItem::new(Kind::Separator, ""))
    }

    /// A nested menu of `items` ([`MenuItem`]s or `(text, href)` links); its id is this
    /// menu's id and the text's slug.
    pub fn submenu<I: Into<MenuItem<'a>>>(
        self,
        text: &'a str,
        items: impl IntoIterator<Item = I>,
    ) -> Self {
        self.push(MenuItem::new(
            Kind::Submenu(items.into_iter().map(Into::into).collect()),
            text,
        ))
    }

    /// A glyph or emoji before the item's text (decorative, hidden from assistive tech).
    pub fn icon(self, icon: &'a str) -> Self {
        self.last(|it| it.icon = Some(icon))
    }

    /// A shortcut shown after the item's text, as `<kbd>`; a label only.
    pub fn shortcut(self, keys: &'a str) -> Self {
        self.last(|it| it.shortcut = Some(keys))
    }

    /// The item is shown but not usable: a link without `href`, a button with `disabled`.
    pub fn disabled(self) -> Self {
        self.last(|it| it.disabled = true)
    }

    /// The item is destructive: coloured with `--nojs-danger`.
    pub fn danger(self) -> Self {
        self.last(|it| it.danger = true)
    }

    /// The menu's id instead of the label's slug.
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Open below the button with right edges aligned, for a button at the end of a row.
    pub fn align_end(mut self) -> Self {
        self.placement = Placement::BottomEnd;
        self
    }

    /// Open to the right of the button.
    pub fn open_right(mut self) -> Self {
        self.placement = Placement::Right;
        self
    }
}

impl Render for Menu<'_> {
    fn render(&self) -> Markup {
        menu(
            &self.caps,
            &self.id,
            self.label,
            &self.items,
            self.placement,
            false,
        )
    }
}

/// A button labelled `label` that toggles a menu of `items`. `compact` is a table row's
/// menu: a small ghost icon button, `label` its accessible name.
pub(crate) fn menu(
    caps: &Caps,
    id: &str,
    label: &str,
    items: &[MenuItem],
    placement: Placement,
    compact: bool,
) -> Markup {
    let popover = caps.has(Cap::Popover);
    let anchor = caps.has(Cap::Anchor);
    let list = html! { ul role="menu" { @for it in items { (item(id, it, popover, anchor)) } } };
    // A row menu repeats once per row, so its face is a text glyph rather than an SVG.
    let face = if compact {
        html! { "\u{22ef}" }
    } else {
        html! { (label) (Icon::ChevronDown) }
    };
    let trigger = Button::new(*caps, label)
        .popovertarget(id)
        .aria_haspopup("menu")
        .content(face.clone());
    let trigger = if compact {
        trigger.ghost().small().icon().label(label)
    } else {
        trigger
    };
    let summary_class = if compact {
        "nojs-button nojs-button-ghost nojs-button-small nojs-button-icon"
    } else {
        "nojs-button"
    };
    html! {
        @if !popover {
            details class={ "nojs-popover nojs-popover-details " (placement.class()) } id=(id) {
                summary class=(summary_class) aria-haspopup="menu" aria-label=[compact.then_some(label)] { (face) }
                nav { (list) }
            }
        } @else if anchor {
            div class={ "nojs-popover nojs-popover-anchored " (placement.class()) } style={ "anchor-name: --" (id) } {
                (trigger)
                nav id=(id) popover style={ "position-anchor: --" (id) "; position-area: " (placement.area()) } { (list) }
            }
        } @else {
            div class={ "nojs-popover " (placement.class()) } {
                (trigger)
                nav id=(id) popover { (list) }
            }
        }
    }
}

fn item(menu_id: &str, it: &MenuItem, popover: bool, anchor: bool) -> Markup {
    let class = format!(
        "nojs-popover-item{}{}",
        if it.danger {
            " nojs-popover-danger"
        } else {
            ""
        },
        if it.disabled {
            " nojs-popover-disabled"
        } else {
            ""
        }
    );
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
            Kind::Submenu(ref items) => li role="none" class="nojs-popover-sub" {
                @let sub_id = format!("{menu_id}-{}", slug(it.text));
                @let list = html! { ul role="menu" { @for it in items { (item(&sub_id, it, popover, anchor)) } } };
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
/* shadcn DropdownMenu: popover surface, p-1, rounded-md, shadow; items text-sm, rounded-sm,
   accent on hover and focus. */
.nojs-popover nav {
  padding: 0.25rem; min-width: 14rem;
  background: var(--nojs-popover); color: var(--nojs-fg);
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
  box-shadow: var(--nojs-shadow-lg);
}
.nojs-popover-anchored > nav, .nojs-popover-details > nav, .nojs-popover-subnav { margin: 0; }
.nojs-popover-anchored > nav { margin-top: 4px; }
/* Popover without anchor positioning: the UA centres it in the viewport; keep that. */
.nojs-popover ul { list-style: none; margin: 0; padding: 0; }
.nojs-popover form { margin: 0; }
.nojs-popover-item {
  display: flex; align-items: center; gap: var(--nojs-space); width: 100%; min-height: 0; box-sizing: border-box;
  padding: 0.375rem 0.5rem; color: inherit; text-decoration: none; text-align: left; font: inherit;
  font-size: 0.875rem; line-height: 1.25rem; font-weight: 400;
  background: none; border: 0; border-radius: var(--nojs-radius-sm); box-shadow: none; cursor: pointer; justify-content: flex-start;
}
.nojs-popover-item:hover, .nojs-popover-item:focus-visible { background: var(--nojs-accent); color: var(--nojs-on-accent); outline: none; }
.nojs-popover-icon { width: 1rem; text-align: center; color: var(--nojs-muted); }
.nojs-popover-text { flex: 1; }
.nojs-popover-kbd { font: inherit; font-size: 0.75rem; letter-spacing: 0.1em; color: var(--nojs-muted); background: none; border: 0; padding: 0; margin-left: auto; }
.nojs-popover-danger { color: var(--nojs-danger); }
.nojs-popover-danger:hover, .nojs-popover-danger:focus-visible { color: var(--nojs-danger); background: color-mix(in srgb, var(--nojs-danger) 10%, transparent); }
.nojs-popover-danger .nojs-popover-icon { color: inherit; }
.nojs-popover-disabled { opacity: 0.5; cursor: default; }
.nojs-popover-disabled:hover { background: none; color: inherit; }
.nojs-popover-heading { padding: 0.375rem 0.5rem; font-size: 0.875rem; font-weight: 500; color: var(--nojs-fg); }
.nojs-popover-sep { margin: 0.25rem -0.25rem; border-top: 1px solid var(--nojs-line); }
.nojs-popover-sub { position: relative; }
/* <details> fallback: the summary is a .nojs-button, the menu absolutely positioned by placement. */
.nojs-popover-details > summary { list-style: none; }
.nojs-popover-details > summary::-webkit-details-marker { display: none; }
.nojs-popover-details > nav { position: absolute; z-index: 10; }
.nojs-popover-start.nojs-popover-details > nav { top: 100%; left: 0; margin-top: 4px; }
.nojs-popover-end.nojs-popover-details > nav { top: 100%; right: 0; margin-top: 4px; }
.nojs-popover-right.nojs-popover-details > nav { top: 0; left: 100%; margin-left: 4px; }
.nojs-popover-sub > .nojs-popover-details > summary {
  display: flex; min-height: 0; padding: 0.375rem 0.5rem; font-weight: 400; border: 0; border-radius: var(--nojs-radius-sm); background: none; box-shadow: none;
}
"#;
