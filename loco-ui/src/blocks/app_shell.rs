//! # App shell
//!
//! The frame of a signed-in app: its name and navigation in a sidebar (a drawer on narrow
//! screens), who is signed in with a sign-out button at the foot, and the page's content
//! beside it. The link to the current path is marked `aria-current="page"`.
//!
//! **Platform features:** those of [`crate::drawer`] (a modal `<dialog>` on narrow screens, a
//! plain sidebar when wide); the sign-out is a form that posts.
//!
//! **Accessibility:** the navigation is a `<nav>` inside the drawer, named by the app's name;
//! the current page's link has `aria-current="page"`; sign-out is a real button. Checked by
//! axe-core with the demo routes.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** that of the drawer: without invoker commands the menu opens through
//! `?dialog=`.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from_request("/", "", "");
//! let m = ui.app_shell("Acme").link("Dashboard", "/").user("Ada Lovelace", "/signout")
//!     .body(html! { h1 { "Hello" } }).render().into_string();
//! assert!(m.contains("Ada Lovelace") && m.contains(r#"action="/signout""#));
//! // The same in `lui!`:
//! let same = lui! { AppShell("Acme") user=("Ada Lovelace", "/signout") { link "Dashboard" "/"; body (html! { h1 { "Hello" } }); } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::i18n::Text;
use crate::props::{Prop, PropKind};

/// A sidebar and content, made by [`Ui::app_shell`].
///
/// **Setters.** Values and items: `.link(..)`, `.user(..)`, `.body(..)`.
#[derive(Clone, Debug)]
pub struct AppShell<'a> {
    ui: &'a Ui,
    name: &'a str,
    links: Vec<(&'a str, &'a str)>,
    user: Option<(&'a str, &'a str)>,
    body: Markup,
}

impl AppShell<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("link", PropKind::Item, "label: &'a str, href: &'a str")
            .doc("A link in the sidebar; the one to the current path is marked current."),
        Prop::new("user", PropKind::Value, "name: &'a str, signout: &'a str")
            .doc("Who is signed in, and where the sign-out button posts."),
        Prop::new("body", PropKind::Value, "body: Markup").doc("The page's content."),
    ];
}

impl Ui {
    /// The frame of an app called `name`.
    pub fn app_shell<'a>(&'a self, name: &'a str) -> AppShell<'a> {
        AppShell {
            ui: self,
            name,
            links: Vec::new(),
            user: None,
            body: Markup::default(),
        }
    }
}

impl<'a> AppShell<'a> {
    /// A link in the sidebar; the one to the current path is marked current.
    pub fn link(mut self, label: &'a str, href: &'a str) -> Self {
        self.links.push((label, href));
        self
    }

    /// Who is signed in, and where the sign-out button posts.
    pub fn user(mut self, name: &'a str, signout: &'a str) -> Self {
        self.user = Some((name, signout));
        self
    }

    /// The page's content.
    pub fn body(mut self, body: Markup) -> Self {
        self.body = body;
        self
    }
}

impl Render for AppShell<'_> {
    fn render(&self) -> Markup {
        let ui = self.ui;
        let here = ui.state.path();
        let nav = html! {
            ul class="lui-app-shell-links" {
                @for (label, href) in &self.links {
                    li { a href=(href) aria-current=[(*href == here).then_some("page")] { (label) } }
                }
            }
            @if let Some((name, signout)) = self.user {
                form method="post" action=(signout) class="lui-app-shell-user" {
                    (ui.avatar(name).small())
                    span { (name) }
                    (ui.button(ui.text(Text::SignOut)).ghost().small().submit())
                }
            }
        };
        html! {
            div class="lui-app-shell" {
                (ui.drawer(self.name).id("app").title(self.name).sidebar().nav(nav).body(self.body.clone()))
            }
        }
    }
}

/// Styles for this block; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-app-shell-links { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.125rem; }
.lui-app-shell-links a { display: block; padding: 0.375rem 0.5rem; border-radius: var(--lui-radius-sm); color: var(--lui-fg); text-decoration: none; font-size: 0.875rem; }
.lui-app-shell-links a:hover, .lui-app-shell-links a[aria-current="page"] { background: var(--lui-accent); color: var(--lui-on-accent); }
.lui-app-shell-links a[aria-current="page"] { font-weight: 500; }
.lui-app-shell-user { display: grid; grid-template-columns: auto 1fr; align-items: center; gap: 0.5rem; margin-top: calc(var(--lui-space) * 3); padding-top: calc(var(--lui-space) * 2); border-top: 1px solid var(--lui-line); font-size: 0.875rem; }
.lui-app-shell-user > span { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.lui-app-shell-user > .lui-button { grid-column: 1 / -1; justify-self: start; }
"#;
