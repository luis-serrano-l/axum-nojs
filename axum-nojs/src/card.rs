//! # Card
//!
//! A bordered box with a header (title, description, and anything else such as an action),
//! a body and a footer: shadcn's Card. Settings panels, pricing tiers, sign-in boxes.
//!
//! **Platform features:** plain `<div>`s and a heading; the header is a grid, so an action
//! put in `.header(..)` sits at the top right beside the title.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.card().title("Team").body(html! { p { "3 members" } }).render().into_string();
//! assert!(m.contains(r#"<h3 class="nojs-card-title">Team</h3>"#) && m.contains("3 members"));
//! // A description, an action in the header, and a footer of buttons.
//! let m = ui.card()
//!     .title("Plan")
//!     .description("Billed monthly.")
//!     .header(html! { (ui.badge("Current").secondary()) })
//!     .body(html! { p { "Pro, 12 seats" } })
//!     .footer(html! { (ui.button("Change plan").primary()) });
//! let m = m.render().into_string();
//! assert!(m.contains("nojs-card-description") && m.contains("nojs-card-action") && m.contains("nojs-card-footer"));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;

/// A card, made by [`Ui::card`].
#[derive(Clone, Debug, Default)]
pub struct Card<'a> {
    title: Option<&'a str>,
    description: Option<&'a str>,
    header: Option<Markup>,
    body: Option<Markup>,
    footer: Option<Markup>,
    id: Option<&'a str>,
}

impl Ui {
    /// An empty card; give it a `.title()`, a `.body()` and so on.
    pub fn card<'a>(&self) -> Card<'a> {
        Card::default()
    }
}

impl<'a> Card<'a> {
    /// The heading at the top (`h3`).
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Muted text under the title.
    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }

    /// More header content, placed at the top right beside the title: a badge, a menu, a
    /// link.
    pub fn header(mut self, markup: Markup) -> Self {
        self.header = Some(markup);
        self
    }

    /// The main content.
    pub fn body(mut self, markup: Markup) -> Self {
        self.body = Some(markup);
        self
    }

    /// A row at the bottom, usually buttons.
    pub fn footer(mut self, markup: Markup) -> Self {
        self.footer = Some(markup);
        self
    }

    /// The root's id, for a link to the card or a swap target.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }
}

impl Render for Card<'_> {
    fn render(&self) -> Markup {
        let has_header =
            self.title.is_some() || self.description.is_some() || self.header.is_some();
        html! {
            div class="nojs-card" id=[self.id] {
                @if has_header {
                    div class="nojs-card-header" {
                        @if let Some(t) = self.title { h3 class="nojs-card-title" { (t) } }
                        @if let Some(d) = self.description { p class="nojs-card-description" { (d) } }
                        @if let Some(h) = &self.header { div class="nojs-card-action" { (h) } }
                    }
                }
                @if let Some(b) = &self.body { div class="nojs-card-body" { (b) } }
                @if let Some(f) = &self.footer { div class="nojs-card-footer" { (f) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn: py-6, gap-6, px-6.
pub const CSS: &str = r#"
.nojs-card {
  display: flex; flex-direction: column; gap: 1.5rem; padding-block: 1.5rem;
  background: var(--nojs-card); color: var(--nojs-fg);
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius-lg); box-shadow: var(--nojs-shadow-xs);
}
.nojs-card-header { display: grid; grid-template-columns: 1fr auto; row-gap: 0.375rem; column-gap: 1rem; padding-inline: 1.5rem; }
.nojs-card-header > :not(.nojs-card-action) { grid-column: 1; }
.nojs-card-title { margin: 0; font-size: 1rem; line-height: 1.25; font-weight: 600; }
.nojs-card-description { margin: 0; color: var(--nojs-muted); font-size: 0.875rem; }
.nojs-card-action { grid-column: 2; grid-row: 1 / span 2; align-self: start; justify-self: end; }
.nojs-card-body { padding-inline: 1.5rem; }
.nojs-card-body > :first-child { margin-top: 0; }
.nojs-card-body > :last-child { margin-bottom: 0; }
.nojs-card-footer { display: flex; align-items: center; flex-wrap: wrap; gap: 0.5rem; padding-inline: 1.5rem; }
"#;
