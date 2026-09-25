//! # Settings page
//!
//! A page of settings in sections: each has a title and a line on the left and its form on
//! the right (stacked on narrow screens), with a list of the sections at the top that jumps
//! to each.
//!
//! **Platform features:** fragment links to each section's `id`; a two-column grid that falls
//! back to one column under 48rem.
//!
//! **Accessibility:** one `<h1>`, an `<h2>` per section, the section list a `<nav>` named by
//! the page title; each section is a `<section>` labelled by its heading. Checked by axe-core
//! with the demo routes.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.settings_page("Settings")
//!     .section("Profile", "How others see you.", html! { p { "form" } })
//!     .section("Email", "Where we write to you.", html! { p { "form" } })
//!     .render().into_string();
//! assert!(m.contains(r##"<a href="#settings-email">Email</a>"##) && m.contains(r#"<section id="settings-email""#));
//! // The same in `lui!`:
//! let same = lui! { SettingsPage("Settings") {
//!     section "Profile" "How others see you." (html! { p { "form" } });
//!     section "Email" "Where we write to you." (html! { p { "form" } });
//! } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::props::{Prop, PropKind};
use crate::{Ui, slug};

/// Sections of settings, made by [`Ui::settings_page`].
///
/// **Setters.** Values and items: `.section(..)`.
#[derive(Clone, Debug)]
pub struct SettingsPage<'a> {
    title: &'a str,
    sections: Vec<(&'a str, &'a str, Markup)>,
}

impl SettingsPage<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets.
    pub const PROPS: &'static [Prop] = &[Prop::new(
        "section",
        PropKind::Item,
        "title: &'a str, description: &'a str, body: Markup",
    )
    .doc("A section: its title and a line on the left, `body` (a form) on the right.")];
}

impl Ui {
    /// A settings page titled `title`.
    pub fn settings_page<'a>(&self, title: &'a str) -> SettingsPage<'a> {
        SettingsPage {
            title,
            sections: Vec::new(),
        }
    }
}

impl<'a> SettingsPage<'a> {
    /// A section: its title and a line on the left, `body` (a form) on the right.
    pub fn section(mut self, title: &'a str, description: &'a str, body: Markup) -> Self {
        self.sections.push((title, description, body));
        self
    }
}

impl Render for SettingsPage<'_> {
    fn render(&self) -> Markup {
        let id = |t: &str| format!("settings-{}", slug(t));
        html! {
            div class="lui-settings-page" {
                h1 { (self.title) }
                nav class="lui-settings-page-nav" aria-label=(self.title) {
                    @for (t, ..) in &self.sections { a href={ "#" (id(t)) } { (t) } }
                }
                @for (t, d, body) in &self.sections {
                    section id=(id(t)) class="lui-settings-page-section" aria-labelledby={ (id(t)) "-title" } {
                        div {
                            h2 id={ (id(t)) "-title" } { (t) }
                            p { (d) }
                        }
                        div class="lui-settings-page-body" { (body) }
                    }
                }
            }
        }
    }
}

/// Styles for this block; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-settings-page { display: grid; gap: calc(var(--lui-space) * 3); }
.lui-settings-page h1 { margin: 0; }
.lui-settings-page-nav { display: flex; flex-wrap: wrap; gap: calc(var(--lui-space) * 2); font-size: 0.875rem; padding-bottom: calc(var(--lui-space) * 2); border-bottom: 1px solid var(--lui-line); }
.lui-settings-page-nav a { color: var(--lui-muted); text-decoration: none; }
.lui-settings-page-nav a:hover { color: var(--lui-fg); }
.lui-settings-page-section { display: grid; gap: calc(var(--lui-space) * 2); padding-bottom: calc(var(--lui-space) * 3); border-bottom: 1px solid var(--lui-line); scroll-margin-top: calc(var(--lui-space) * 2); }
.lui-settings-page-section h2 { margin: 0; font-size: 1rem; font-weight: 600; }
.lui-settings-page-section p { margin: 0.25rem 0 0; color: var(--lui-muted); font-size: 0.875rem; }
@media (min-width: 48rem) { .lui-settings-page-section { grid-template-columns: 16rem 1fr; } }
"#;
