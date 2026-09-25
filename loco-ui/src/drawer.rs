//! # Drawer
//!
//! Site navigation that is a sidebar on a wide screen and a drawer sliding in from the edge on
//! a narrow one, from the same markup: a menu button opens it, Escape, a click outside or the
//! close button shuts it. No script.
//!
//! **Platform features:**
//! - `<dialog>` opened as a modal by an invoker button, `command="show-modal"` (Chrome 135+,
//!   Firefox 144+, Safari 26.2+), closed by `command="close"` and by `closedby="any"` for
//!   Escape and light dismiss.
//! - `@starting-style` (Chrome 117, Firefox 129, Safari 17.5) for the slide in, off under
//!   `prefers-reduced-motion`.
//! - Sidebar mode: above 60rem a `@media` rule shows the closed `<dialog>` in a grid column
//!   and hides the menu button, so desktop gets a permanent sidebar with no request.
//!
//! **Fallback:** without `Caps` `Invokers`, the menu button is a link to `#id` and a `:target`
//! rule shows the drawer; its close control is a link to `#`. `?dialog=<id>` in the URL (or
//! `.open(true)`) renders it open from the server.
//!
//! **Accessibility:** a native modal `<dialog>` named by its title: focus moves in, Escape
//! closes it. Checked by axe-core in headless Firefox on every demo route, both capability
//! variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** swipe to close; focus is not trapped in the
//! `:target` fallback.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let nav = html! { ul { li { a href="/" { "Home" } } } };
//! // The id is the label's slug: this drawer is `#menu`.
//! let m = ui.drawer("Menu").nav(nav.clone()).body(html! { p { "Page" } }).render().into_string();
//! assert!(m.contains(r#"command="show-modal" commandfor="menu""#) && m.contains(r#"closedby="any""#));
//! // `?dialog=menu` opens it from the server.
//! let ui = Ui::from_request("/", "dialog=menu", "");
//! let m = ui.drawer("Menu").title("Browse").sidebar().nav(nav.clone()).render().into_string();
//! assert!(m.contains("lui-drawer-sidebar") && m.contains(" open>"));
//! // The same in `lui!`:
//! let same = lui! { Drawer("Menu") title="Browse" sidebar nav=(nav); };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::i18n::Text;
use crate::props::{Prop, PropKind};
use crate::{Cap, Icon, Ui, slug};

/// Navigation in a drawer beside the page's content, made by [`Ui::drawer`].
///
/// **Setters.** Values and items: `.nav(..)`, `.body(..)`, `.id(..)`, `.title(..)`; switches:
/// `.sidebar()`; from a condition: `.open(bool)`.
#[derive(Clone, Debug)]
pub struct Drawer<'a> {
    ui: &'a Ui,
    id: String,
    label: &'a str,
    nav: Markup,
    body: Markup,
    title: Option<&'a str>,
    sidebar: bool,
    open: bool,
}

impl Drawer<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("nav", PropKind::Value, "nav: Markup")
            .doc("The navigation inside the drawer, usually a `ul` of links."),
        Prop::new("body", PropKind::Value, "body: Markup")
            .doc("The page's content, beside the drawer."),
        Prop::new("id", PropKind::Value, "id: &str")
            .attr("id")
            .doc("The drawer's id instead of the label's slug."),
        Prop::new("title", PropKind::Value, "title: &'a str")
            .doc("A heading at the top of the panel (also its accessible name)."),
        Prop::new("sidebar", PropKind::Switch, "").doc("A permanent sidebar above 60rem."),
        Prop::new("open", PropKind::Condition, "open: bool")
            .attr("open")
            .doc("Render it open (non-modal) from the server."),
    ];
}

impl Ui {
    /// A drawer opened by a button labelled `label`; its id is the label's slug, and
    /// `?dialog=<id>` renders it open.
    pub fn drawer<'a>(&'a self, label: &'a str) -> Drawer<'a> {
        Drawer {
            ui: self,
            id: slug(label),
            label,
            nav: Markup::default(),
            body: Markup::default(),
            title: None,
            sidebar: false,
            open: false,
        }
    }
}

impl<'a> Drawer<'a> {
    /// The navigation inside the drawer, usually a `ul` of links.
    pub fn nav(mut self, nav: Markup) -> Self {
        self.nav = nav;
        self
    }

    /// The page's content, beside the drawer.
    pub fn body(mut self, body: Markup) -> Self {
        self.body = body;
        self
    }

    /// The drawer's id instead of the label's slug.
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// A heading at the top of the panel (also its accessible name); the label by default.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// A permanent sidebar above 60rem; a drawer below.
    pub fn sidebar(mut self) -> Self {
        self.sidebar = true;
        self
    }

    /// Render it open (non-modal) from the server.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
}

impl Render for Drawer<'_> {
    fn render(&self) -> Markup {
        let Drawer {
            ui,
            ref id,
            label,
            ref nav,
            ref body,
            title,
            sidebar,
            open,
        } = *self;
        let invokers = ui.has(Cap::Invokers);
        let open = open || ui.state.dialog() == Some(id);
        let title = title.unwrap_or(label);
        let title_id = format!("{id}-title");
        let open_href = format!("#{id}");
        html! {
            div class={ "lui-drawer" @if sidebar { " lui-drawer-sidebar" } } {
                @let menu = html! { (Icon::Menu) (label) };
                @if invokers {
                    (ui.button(label).class("lui-drawer-open").content(menu).command("show-modal", id).aria_haspopup("dialog"))
                } @else {
                    (ui.link_button(label, &open_href).class("lui-drawer-open").content(menu).role("button"))
                }
                dialog id=(id) class="lui-drawer-panel" closedby="any" aria-labelledby=(title_id) open[open] {
                    div class="lui-drawer-head" {
                        p id=(title_id) class="lui-drawer-title" { (title) }
                        @let x = html! { (Icon::X) };
                        @if invokers {
                            (ui.button("").ghost().small().icon().class("lui-drawer-close").label(ui.text(Text::Close)).content(x).command("close", id))
                        } @else {
                            (ui.link_button("", "#").ghost().small().icon().class("lui-drawer-close").label(ui.text(Text::Close)).content(x))
                        }
                    }
                    nav aria-labelledby=(title_id) { (nav) }
                }
                div class="lui-drawer-content" { (body) }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-drawer { display: grid; gap: calc(var(--lui-space) * 2); }
.lui-drawer-open { justify-self: start; }
/* shadcn Sheet, side="left": full height, max-w-sm, border on the open edge, shadow-lg. */
.lui-drawer-panel {
  box-sizing: border-box; margin: 0; padding: calc(var(--lui-space) * 2);
  color: var(--lui-fg); background: var(--lui-popover); border: 0; border-inline-end: 1px solid var(--lui-line);
}
.lui-drawer-panel:modal, .lui-drawer-panel:target {
  display: block; position: fixed; inset: 0 auto 0 0; height: 100dvh; max-height: none; width: min(24rem, 75vw); z-index: 10;
  padding: calc(var(--lui-space) * 3); box-shadow: var(--lui-shadow-lg), var(--lui-highlight);
  transition: translate 0.3s ease-in-out, display 0.3s allow-discrete, overlay 0.3s allow-discrete;
}
.lui-drawer-panel:target { box-shadow: var(--lui-shadow-lg), var(--lui-highlight), 0 0 0 100vmax var(--lui-overlay); }
@starting-style { .lui-drawer-panel:modal { translate: -100% 0; } }
.lui-drawer-panel::backdrop { background: var(--lui-overlay); }
.lui-drawer-panel:not(:modal):not(:target)[open] { position: static; width: auto; border: 1px solid var(--lui-line); border-radius: var(--lui-radius); }
.lui-drawer-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: calc(var(--lui-space) * 2); }
.lui-drawer-title { margin: 0; font-weight: 600; }
.lui-drawer-close { opacity: 0.7; }
.lui-drawer-close:hover { opacity: 1; }
/* Links as shadcn sidebar menu buttons: text-sm, rounded-md, accent on hover and when current. */
.lui-drawer-panel ul { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.25rem; }
.lui-drawer-panel li a {
  display: block; padding: 0.375rem 0.5rem; border-radius: var(--lui-radius-sm);
  font-size: 0.875rem; line-height: 1.25rem; color: var(--lui-fg); text-decoration: none;
}
.lui-drawer-panel li a:hover { background: var(--lui-accent); color: var(--lui-on-accent); }
.lui-drawer-panel li a[aria-current] { background: var(--lui-accent); color: var(--lui-on-accent); font-weight: 500; }
.lui-drawer-content { min-width: 0; }
@media (prefers-reduced-motion: reduce) { .lui-drawer-panel:modal { transition: none; } }
@media (min-width: 60rem) {
  .lui-drawer-sidebar { grid-template-columns: 14rem 1fr; align-items: start; }
  .lui-drawer-sidebar > .lui-drawer-open { display: none; }
  .lui-drawer-sidebar > .lui-drawer-panel:not(:modal) {
    display: block; position: sticky; top: calc(var(--lui-space) * 2); width: auto; height: auto; box-shadow: none;
    padding: var(--lui-space); background: var(--lui-surface);
    border: 1px solid var(--lui-line); border-radius: var(--lui-radius); z-index: auto;
  }
  .lui-drawer-sidebar .lui-drawer-close { display: none; }
}
"#;
