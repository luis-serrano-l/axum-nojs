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
//! rule shows the drawer; its close control is a link to `#`. `DrawerOptions::open` renders it
//! open from the server (`UiState::dialog()` after `?dialog=<id>`).
//!
//! **What it does not do without script:** swipe to close; focus is not trapped in the
//! `:target` fallback.
//!
//! ```rust
//! use maud::html;
//! use axum_nojs::{Caps, drawer, drawer_with, drawer::DrawerOptions};
//! let nav = html! { ul { li { a href="/" { "Home" } } } };
//! // The id is derived from the label: this drawer is `#menu`.
//! let m = drawer(&Caps::all(), "Menu", nav.clone(), html! { p { "Page" } }).into_string();
//! assert!(m.contains(r#"command="show-modal" commandfor="menu""#) && m.contains(r#"closedby="any""#));
//! let m = drawer_with(&Caps::all(), "site", "Menu", nav, html! { p { "Page" } },
//!     DrawerOptions::default().title("Browse").sidebar().open(true)).into_string();
//! assert!(m.contains("nojs-drawer-sidebar") && m.contains(" open>"));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// Options for [`drawer`].
#[derive(Clone, Debug, Default)]
pub struct DrawerOptions<'a> {
    /// Heading at the top of the panel (also its accessible name).
    pub title: Option<&'a str>,
    /// Permanent sidebar above 60rem; a drawer below.
    pub sidebar: bool,
    /// Render it open (non-modal) from the server.
    pub open: bool,
}

impl<'a> DrawerOptions<'a> {
    /// Heading at the top of the panel.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }
    /// Sidebar on wide screens.
    pub fn sidebar(mut self) -> Self {
        self.sidebar = true;
        self
    }
    /// Open on arrival.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
}

/// A drawer whose id is derived from `label`; use [`drawer_with`] to name it.
/// [`drawer_with`] takes the options.
pub fn drawer(caps: &Caps, label: &str, nav: Markup, content: Markup) -> Markup {
    drawer_with(caps, &crate::slug(label), label, nav, content, Default::default())
}

/// A navigation drawer `id` opened by a button labelled `label`, holding `nav`, beside `content`.
pub fn drawer_with(caps: &Caps, id: &str, label: &str, nav: Markup, content: Markup, options: DrawerOptions) -> Markup {
    let invokers = caps.has(Cap::Invokers);
    let title = options.title.unwrap_or(label);
    let title_id = format!("{id}-title");
    html! {
        div class={ "nojs-drawer" @if options.sidebar { " nojs-drawer-sidebar" } } {
            @if invokers {
                button type="button" class="nojs-drawer-open" command="show-modal" commandfor=(id) aria-haspopup="dialog" { "\u{2630} " (label) }
            } @else {
                a class="nojs-drawer-open" role="button" href={ "#" (id) } { "\u{2630} " (label) }
            }
            dialog id=(id) class="nojs-drawer-panel" closedby="any" aria-labelledby=(title_id) open[options.open] {
                div class="nojs-drawer-head" {
                    p id=(title_id) class="nojs-drawer-title" { (title) }
                    @if invokers {
                        button type="button" class="nojs-drawer-close" command="close" commandfor=(id) aria-label="Close" { "\u{d7}" }
                    } @else {
                        a href="#" class="nojs-drawer-close" aria-label="Close" { "\u{d7}" }
                    }
                }
                nav aria-labelledby=(title_id) { (nav) }
            }
            div class="nojs-drawer-content" { (content) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-drawer { display: grid; gap: calc(var(--nojs-space) * 2); }
.nojs-drawer-open {
  justify-self: start; display: inline-block; padding: 0.5rem 1rem; color: inherit; text-decoration: none;
  background: var(--nojs-surface); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
}
.nojs-drawer-panel {
  box-sizing: border-box; margin: 0; padding: calc(var(--nojs-space) * 2);
  color: var(--nojs-fg); background: var(--nojs-surface); border: 0; border-inline-end: 1px solid var(--nojs-line);
}
.nojs-drawer-panel:modal, .nojs-drawer-panel:target {
  display: block; position: fixed; inset: 0 auto 0 0; height: 100dvh; max-height: none; width: min(20rem, 85vw); z-index: 10;
  transition: translate 0.2s ease-out, display 0.2s allow-discrete, overlay 0.2s allow-discrete;
}
.nojs-drawer-panel:target { box-shadow: 0 0 0 100vmax color-mix(in srgb, var(--nojs-fg) 45%, transparent); }
@starting-style { .nojs-drawer-panel:modal { translate: -100% 0; } }
.nojs-drawer-panel::backdrop { background: color-mix(in srgb, var(--nojs-fg) 45%, transparent); }
.nojs-drawer-panel:not(:modal):not(:target)[open] { position: static; width: auto; border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); }
.nojs-drawer-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--nojs-space); }
.nojs-drawer-title { margin: 0; font-weight: 700; }
.nojs-drawer-close {
  width: 2rem; height: 2rem; padding: 0; font-size: 1.25rem; line-height: 1; display: inline-flex; align-items: center; justify-content: center;
  color: var(--nojs-muted); background: none; border: 1px solid transparent; border-radius: var(--nojs-radius); text-decoration: none;
}
.nojs-drawer-close:hover { color: var(--nojs-fg); border-color: var(--nojs-line); }
.nojs-drawer-panel ul { list-style: none; margin: 0; padding: 0; }
.nojs-drawer-panel li a { display: block; padding: 0.4rem 0.75rem; border-radius: var(--nojs-radius); color: var(--nojs-fg); text-decoration: none; }
.nojs-drawer-panel li a:hover { background: var(--nojs-bg); }
.nojs-drawer-panel li a[aria-current] { background: var(--nojs-accent); color: var(--nojs-on-accent); }
.nojs-drawer-content { min-width: 0; }
@media (prefers-reduced-motion: reduce) { .nojs-drawer-panel:modal { transition: none; } }
@media (min-width: 60rem) {
  .nojs-drawer-sidebar { grid-template-columns: 14rem 1fr; align-items: start; }
  .nojs-drawer-sidebar > .nojs-drawer-open { display: none; }
  .nojs-drawer-sidebar > .nojs-drawer-panel:not(:modal) {
    display: block; position: sticky; top: calc(var(--nojs-space) * 2); width: auto; height: auto; box-shadow: none;
    border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); z-index: auto;
  }
  .nojs-drawer-sidebar .nojs-drawer-close { display: none; }
}
"#;
