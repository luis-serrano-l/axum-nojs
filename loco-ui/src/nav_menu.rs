//! # Navigation menu
//!
//! A site's top navigation: a row of links, some of which open a panel of more links. Panels
//! are the [popover menu](crate::popover)'s: they open on click, close on a click outside or
//! Escape, and are `<details>` dropdowns where popovers are missing.
//!
//! **Platform features:** a `<nav>` of lists; the panels use `popover` and anchor positioning
//! as the popover menu does (see its header); `aria-current="page"` on the link to the current
//! path.
//!
//! **Accessibility:** a `<nav>` named by its label; plain links are links, and a panel's button
//! says it opens a menu (`aria-haspopup`); the current page's link has `aria-current="page"`.
//! Checked by axe-core in headless Firefox on every demo route, both capability variants, light
//! and dark (no serious or critical violation).
//!
//! **What it does not do without script:** open a panel on hover; a click opens it.
//!
//! **Fallback:** that of the popover menu: a `<details>` dropdown without `popover`, a centred
//! panel without anchor positioning.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let mut ui = Ui::from_request("/pricing", "", "");
//! ui.caps = Caps::all();
//! let m = ui.nav_menu("Main").link("Pricing", "/pricing").render().into_string();
//! assert!(m.contains(r#"<a class="lui-nav-menu-link" href="/pricing" aria-current="page">"#));
//!
//! let m = ui.nav_menu("Main")
//!     .panel("Products", [("Mail", "/mail"), ("Calendar", "/calendar")])
//!     .link("Pricing", "/pricing");
//! let html = m.render().into_string();
//! assert!(html.contains(r#"popovertarget="nav-main-products""#) && html.contains(r#"href="/calendar""#));
//! // The same in `lui!`:
//! let same = lui! { NavMenu("Main") {
//!     panel "Products" ([("Mail", "/mail"), ("Calendar", "/calendar")]);
//!     link "Pricing" "/pricing";
//! } };
//! assert_eq!(same.into_string(), html);
//! ```

use maud::{Markup, Render, html};

use crate::popover::{MenuItem, Placement, menu};
use crate::props::{Prop, PropKind};
use crate::{Caps, Ui, slug};

/// One entry of the row: a link, or a panel of links.
#[derive(Clone, Debug)]
enum Entry<'a> {
    Link(&'a str, &'a str),
    Panel(&'a str, Vec<MenuItem<'a>>),
}

/// A row of links and panels, made by [`Ui::nav_menu`].
///
/// **Setters.** Values and items: `.link(..)`, `.panel(..)`, `.id(..)`.
#[derive(Clone, Debug)]
pub struct NavMenu<'a> {
    caps: Caps,
    label: &'a str,
    here: String,
    entries: Vec<Entry<'a>>,
    id: Option<&'a str>,
}

impl NavMenu<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("link", PropKind::Item, "text: &'a str, href: &'a str")
            .attr("href")
            .doc("A link in the row; the one to the current path is marked current."),
        Prop::new(
            "panel",
            PropKind::Item,
            "text: &'a str, items: impl IntoIterator<Item = I>",
        )
        .doc("A button that opens a panel of `items` (`MenuItem`s or `(text, href)` links)."),
        Prop::new("id", PropKind::Value, "id: &'a str")
            .attr("id")
            .doc("The prefix of the panels' ids instead of `nav-<label>`."),
    ];
}

impl Ui {
    /// Top navigation named `label` (for assistive tech).
    pub fn nav_menu<'a>(&self, label: &'a str) -> NavMenu<'a> {
        NavMenu {
            caps: self.caps,
            label,
            here: self.state.path().to_string(),
            entries: Vec::new(),
            id: None,
        }
    }
}

impl<'a> NavMenu<'a> {
    /// A link in the row; the one to the current path is marked current.
    pub fn link(mut self, text: &'a str, href: &'a str) -> Self {
        self.entries.push(Entry::Link(text, href));
        self
    }

    /// A button that opens a panel of `items` (`MenuItem`s or `(text, href)` links).
    pub fn panel<I: Into<MenuItem<'a>>>(
        mut self,
        text: &'a str,
        items: impl IntoIterator<Item = I>,
    ) -> Self {
        self.entries.push(Entry::Panel(
            text,
            items.into_iter().map(Into::into).collect(),
        ));
        self
    }

    /// The prefix of the panels' ids instead of `nav-<label>`.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }
}

impl Render for NavMenu<'_> {
    fn render(&self) -> Markup {
        let root = self
            .id
            .map_or_else(|| format!("nav-{}", slug(self.label)), str::to_string);
        html! {
            nav class="lui-nav-menu" aria-label=(self.label) {
                ul {
                    @for e in &self.entries {
                        li {
                            @match e {
                                Entry::Link(text, href) => a class="lui-nav-menu-link" href=(href) aria-current=[(*href == self.here).then_some("page")] { (text) },
                                Entry::Panel(text, items) => (menu(&self.caps, &format!("{root}-{}", slug(text)), text, items, Placement::BottomStart, false)),
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn NavigationMenu: a row
/// of quiet triggers and links that light up on hover.
pub const CSS: &str = r#"
.lui-nav-menu > ul { list-style: none; margin: 0; padding: 0; display: flex; flex-wrap: wrap; align-items: center; gap: 0.25rem; }
.lui-nav-menu .lui-popover > .lui-button, .lui-nav-menu .lui-popover > summary.lui-button { background: transparent; border-color: transparent; box-shadow: none; }
.lui-nav-menu-link { display: inline-flex; align-items: center; height: 2.25rem; padding: 0 1rem; border-radius: var(--lui-radius); font-size: 0.875rem; font-weight: 500; color: var(--lui-fg); text-decoration: none; }
.lui-nav-menu-link:hover, .lui-nav-menu-link[aria-current="page"] { background: var(--lui-accent); color: var(--lui-on-accent); }
"#;
