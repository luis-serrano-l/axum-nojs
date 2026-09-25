//! # Description list
//!
//! Terms and what they are: a record's fields, a summary before confirming, key facts. Terms
//! sit in a column beside their details, or above them with `.stacked()`.
//!
//! **Platform features:** `<dl>` with `<dt>`/`<dd>` pairs (every browser); CSS grid for the
//! two columns.
//!
//! **Accessibility:** a real description list, so screen readers announce the pairs and their
//! count; a detail may hold any markup (a badge, a link). Checked by axe-core in headless
//! Firefox on every demo route, both capability variants, light and dark (no serious or
//! critical violation).
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** without grid the terms stack above their details.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.description_list().item("Plan", "Team").item("Seats", "12").render().into_string();
//! assert!(m.contains("<dt>Plan</dt><dd>Team</dd>"));
//!
//! let m = ui.description_list().stacked().item("Status", ui.badge("Active").ok());
//! let html = m.render().into_string();
//! assert!(html.contains("lui-description-list lui-description-list-stacked") && html.contains("Active"));
//! // The same in `lui!`:
//! let same = lui! { DescriptionList stacked { item "Status" (ui.badge("Active").ok()); } };
//! assert_eq!(same.into_string(), html);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// Terms and their details, made by [`Ui::description_list`].
///
/// **Setters.** Values and items: `.item(..)`; switches: `.stacked()`.
#[derive(Clone, Debug, Default)]
pub struct DescriptionList<'a> {
    items: Vec<(&'a str, Markup)>,
    stacked: bool,
}

impl DescriptionList<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("item", PropKind::Item, "term: &'a str, detail: impl Render")
            .doc("A term and its detail (text, or markup such as a badge)."),
        Prop::new("stacked", PropKind::Switch, "")
            .doc("Each term above its detail instead of beside it."),
    ];
}

impl Ui {
    /// An empty description list; add pairs with [`DescriptionList::item`].
    pub fn description_list<'a>(&self) -> DescriptionList<'a> {
        DescriptionList::default()
    }
}

impl<'a> DescriptionList<'a> {
    /// A term and its detail (text, or markup such as a badge).
    pub fn item(mut self, term: &'a str, detail: impl Render) -> Self {
        self.items.push((term, detail.render()));
        self
    }

    /// Each term above its detail instead of beside it.
    pub fn stacked(mut self) -> Self {
        self.stacked = true;
        self
    }
}

impl Render for DescriptionList<'_> {
    fn render(&self) -> Markup {
        html! {
            dl class={ "lui-description-list" @if self.stacked { " lui-description-list-stacked" } } {
                @for (term, detail) in &self.items { dt { (term) } dd { (detail) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-description-list { display: grid; grid-template-columns: minmax(8rem, max-content) 1fr; gap: calc(var(--lui-space) * 1.5) calc(var(--lui-space) * 3); margin: 0; }
.lui-description-list dt { color: var(--lui-muted); font-size: 0.875rem; }
.lui-description-list dd { margin: 0; }
.lui-description-list-stacked { grid-template-columns: 1fr; gap: 0.25rem; }
.lui-description-list-stacked dd + dt { margin-top: calc(var(--lui-space) * 1.5); }
"#;
