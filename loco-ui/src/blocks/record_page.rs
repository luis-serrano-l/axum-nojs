//! # Record page
//!
//! One record: its title, its fields as a description list in a card, and the actions on it
//! (edit, delete) beside the title. The show page a scaffold writes, ready-made.
//!
//! **Platform features:** a `<dl>`; delete is a form that posts (Post/Redirect/Get).
//!
//! **Accessibility:** the title is the page's `<h1>`; labels and values are `<dt>`/`<dd>`
//! pairs; edit is a link, delete a real button. Checked by axe-core with the demo routes.
//!
//! **What it does not do without script:** ask "are you sure?" before deleting; put the
//! delete in a [`crate::dialog`] for that.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.record_page("Invoice 42").field("Customer", "Ada").field("Total", "€120")
//!     .edit("/invoices/42/edit").delete("/invoices/42/delete").render().into_string();
//! assert!(m.contains("<dt>Customer</dt><dd>Ada</dd>") && m.contains(r#"action="/invoices/42/delete""#));
//! // The same in `lui!`:
//! let same = lui! { RecordPage("Invoice 42") edit="/invoices/42/edit" delete="/invoices/42/delete" {
//!     field "Customer" "Ada"; field "Total" "€120";
//! } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::i18n::Text;
use crate::props::{Prop, PropKind};

/// A record's show page, made by [`Ui::record_page`].
///
/// **Setters.** Values and items: `.field(..)`, `.edit(..)`, `.delete(..)`, `.back(..)`.
#[derive(Clone, Debug)]
pub struct RecordPage<'a> {
    ui: &'a Ui,
    title: &'a str,
    fields: Vec<(&'a str, Markup)>,
    edit: Option<&'a str>,
    delete: Option<&'a str>,
    back: Option<&'a str>,
}

impl RecordPage<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets.
    pub const PROPS: &'static [Prop] = &[
        Prop::new(
            "field",
            PropKind::Item,
            "label: &'a str, value: impl Render",
        )
        .doc("A label and its value."),
        Prop::new("edit", PropKind::Value, "href: &'a str")
            .attr("href")
            .doc("An Edit link."),
        Prop::new("delete", PropKind::Value, "action: &'a str")
            .attr("action")
            .doc("A Delete button posting to `action`."),
        Prop::new("back", PropKind::Value, "href: &'a str")
            .attr("href")
            .doc("A Back link to the list."),
    ];
}

impl Ui {
    /// The page of the record titled `title`.
    pub fn record_page<'a>(&'a self, title: &'a str) -> RecordPage<'a> {
        RecordPage {
            ui: self,
            title,
            fields: Vec::new(),
            edit: None,
            delete: None,
            back: None,
        }
    }
}

impl<'a> RecordPage<'a> {
    /// A label and its value.
    pub fn field(mut self, label: &'a str, value: impl Render) -> Self {
        self.fields.push((label, value.render()));
        self
    }

    /// An Edit link.
    pub fn edit(mut self, href: &'a str) -> Self {
        self.edit = Some(href);
        self
    }

    /// A Delete button posting to `action`.
    pub fn delete(mut self, action: &'a str) -> Self {
        self.delete = Some(action);
        self
    }

    /// A Back link to the list.
    pub fn back(mut self, href: &'a str) -> Self {
        self.back = Some(href);
        self
    }
}

impl Render for RecordPage<'_> {
    fn render(&self) -> Markup {
        let ui = self.ui;
        html! {
            div class="lui-record" {
                div class="lui-record-head" {
                    h1 { (self.title) }
                    div class="lui-record-actions" {
                        @if let Some(b) = self.back { (ui.link_button(ui.text(Text::Back), b).ghost()) }
                        @if let Some(e) = self.edit { (ui.link_button(ui.text(Text::Edit), e)) }
                        @if let Some(d) = self.delete {
                            form method="post" action=(d) { (ui.button(ui.text(Text::Delete)).danger().submit()) }
                        }
                    }
                }
                dl class="lui-record-fields" {
                    @for (label, value) in &self.fields { dt { (label) } dd { (value) } }
                }
            }
        }
    }
}

/// Styles for this block; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-record { display: grid; gap: calc(var(--lui-space) * 2); }
.lui-record-head { display: flex; flex-wrap: wrap; align-items: center; justify-content: space-between; gap: calc(var(--lui-space) * 2); }
.lui-record-head h1 { margin: 0; }
.lui-record-actions { display: flex; gap: var(--lui-space); }
.lui-record-actions form { margin: 0; }
.lui-record-fields {
  display: grid; grid-template-columns: minmax(8rem, max-content) 1fr; gap: calc(var(--lui-space) * 1.5) calc(var(--lui-space) * 3); margin: 0;
  padding: calc(var(--lui-space) * 3); background: var(--lui-card); border: 1px solid var(--lui-line); border-radius: var(--lui-radius-lg);
}
.lui-record-fields dt { color: var(--lui-muted); font-size: 0.875rem; }
.lui-record-fields dd { margin: 0; }
"#;
