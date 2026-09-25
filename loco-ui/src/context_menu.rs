//! # Context menu
//!
//! Actions on one thing (a file, a card, a row) from a small "more" button in its corner:
//! the [popover menu](crate::popover) the way a right-click menu would offer it. A right-click
//! cannot be caught without script, so the actions sit behind a visible button instead.
//!
//! **Platform features:** those of the popover menu: `popover`, `popovertarget` and anchor
//! positioning, a `<details>` dropdown where popovers are missing.
//!
//! **Accessibility:** the button is named "More actions" (or the label given) and says it opens
//! a menu; the menu is the popover menu's (`role="menu"`, Escape closes). Checked by axe-core in
//! headless Firefox on every demo route, both capability variants, light and dark (no serious
//! or critical violation).
//!
//! **What it does not do without script:** open on right-click or a long press; the button
//! does it.
//!
//! **Fallback:** that of the popover menu.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from(Caps::all());
//! // The items are the popover menu's: `icon`, `shortcut`, `disabled` and `danger` apply to
//! // the item just added.
//! let m = ui.context_menu("report.pdf")
//!     .link("Open", "/files/1").shortcut("o")
//!     .separator()
//!     .action("Delete", "/files/1/delete").danger()
//!     .body(html! { p { "report.pdf" } });
//! let html = m.render().into_string();
//! assert!(html.contains(r#"aria-label="More actions: report.pdf""#) && html.contains(r#"action="/files/1/delete""#));
//! // The same in `lui!`:
//! let same = lui! { ContextMenu("report.pdf") {
//!     link "Open" "/files/1" shortcut="o";
//!     separator();
//!     action "Delete" "/files/1/delete" danger;
//!     body() { p { "report.pdf" } }
//! } };
//! assert_eq!(same.into_string(), html);
//! ```

use maud::{Markup, Render, html};

use crate::i18n::Text;
use crate::icon::Glyph;
use crate::popover::{MenuItem, Placement, menu};
use crate::props::{Prop, PropKind};
use crate::{Ui, slug};

/// A thing with a menu of actions, made by [`Ui::context_menu`].
///
/// **Setters.** Values and items: `.link(..)`, `.action(..)`, `.group(..)`, `.icon(..)`,
/// `.shortcut(..)`, `.body(..)`, `.id(..)`; switches: `.separator()`, `.disabled()`,
/// `.danger()`.
#[derive(Clone, Debug)]
pub struct ContextMenu<'a> {
    ui: &'a Ui,
    label: &'a str,
    items: Vec<MenuItem<'a>>,
    body: Markup,
    id: Option<&'a str>,
}

impl ContextMenu<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("link", PropKind::Item, "text: &'a str, href: &'a str").doc("A link."),
        Prop::new("action", PropKind::Item, "text: &'a str, action: &'a str")
            .attr("action")
            .doc("A `<form method=\"post\">` button posting to `action`."),
        Prop::new("group", PropKind::Item, "text: &'a str").doc("A heading over the items after it."),
        Prop::new("separator", PropKind::Item, "").doc("A rule between groups."),
        Prop::new("icon", PropKind::Modifier, "icon: impl Into<Glyph<'a>>").doc(
            "An icon, or a glyph or emoji, before the item's text (decorative, hidden from assistive tech).",
        ),
        Prop::new("shortcut", PropKind::Modifier, "keys: &'a str")
            .doc("A shortcut shown after the item's text, as `<kbd>`."),
        Prop::new("disabled", PropKind::Modifier, "")
            .attr("disabled")
            .doc("The item is shown but not usable."),
        Prop::new("danger", PropKind::Modifier, "").doc("The item is destructive."),
        Prop::new("body", PropKind::Value, "body: Markup").doc("The thing the actions are on."),
        Prop::new("id", PropKind::Value, "id: &'a str")
            .attr("id")
            .doc("The menu's id instead of `ctx-<label>`."),
    ];
}

impl Ui {
    /// A menu of actions on the thing named `label` (the button says "More actions: label").
    pub fn context_menu<'a>(&'a self, label: &'a str) -> ContextMenu<'a> {
        ContextMenu {
            ui: self,
            label,
            items: Vec::new(),
            body: Markup::default(),
            id: None,
        }
    }
}

impl<'a> ContextMenu<'a> {
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

    /// A heading over the items after it.
    pub fn group(self, text: &'a str) -> Self {
        self.push(MenuItem::group(text))
    }

    /// A rule between groups.
    pub fn separator(self) -> Self {
        self.push(MenuItem::separator())
    }

    /// An icon, or a glyph or emoji, before the item's text (decorative, hidden from
    /// assistive tech).
    pub fn icon(self, icon: impl Into<Glyph<'a>>) -> Self {
        let icon = icon.into();
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

    /// The item is destructive: coloured with `--lui-danger`.
    pub fn danger(self) -> Self {
        self.last(|it| it.danger = true)
    }

    /// The actions as a list, replacing any added so far; kept for one release.
    #[deprecated(note = "use the adders: .link(), .action(), .group(), .separator()")]
    pub fn items(mut self, items: impl IntoIterator<Item = MenuItem<'a>>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// The thing the actions are on.
    pub fn body(mut self, body: Markup) -> Self {
        self.body = body;
        self
    }

    /// The menu's id instead of `ctx-<label>`.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }
}

impl Render for ContextMenu<'_> {
    fn render(&self) -> Markup {
        let id = self
            .id
            .map_or_else(|| format!("ctx-{}", slug(self.label)), str::to_string);
        let name = format!("{}: {}", self.ui.text(Text::MoreActions), self.label);
        html! {
            div class="lui-context-menu" {
                div class="lui-context-menu-body" { (self.body) }
                (menu(&self.ui.caps, &id, &name, &self.items, Placement::BottomEnd, true))
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-context-menu { position: relative; display: flex; align-items: flex-start; gap: var(--lui-space); padding: calc(var(--lui-space) * 1.5); border: 1px solid var(--lui-line); border-radius: var(--lui-radius); background: var(--lui-card); }
.lui-context-menu-body { flex: 1; min-width: 0; }
.lui-context-menu-body > :first-child { margin-top: 0; }
.lui-context-menu-body > :last-child { margin-bottom: 0; }
"#;
