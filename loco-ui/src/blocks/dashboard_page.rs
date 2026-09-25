//! # Dashboard page
//!
//! A page of numbers: a title and a line, a row of stats that wraps to the width available,
//! and whatever goes under them (a table, a chart, recent activity).
//!
//! **Platform features:** those of [`crate::stat`]: an auto-fit grid, no media query.
//!
//! **Accessibility:** the title is the page's `<h1>`; each stat says its change in words.
//! Checked by axe-core with the demo routes.
//!
//! **What it does not do without script:** update live; the numbers are as fresh as the page.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.dashboard_page("Overview").description("Last 30 days.")
//!     .stat(ui.stat("Revenue", "$12,480").delta("+8%"))
//!     .stat(ui.stat("Customers", "312"))
//!     .body(html! { p { "Recent orders" } })
//!     .render().into_string();
//! assert!(m.contains("lui-stat-grid") && m.contains("$12,480") && m.contains("Recent orders"));
//! // The same in `lui!`:
//! let same = lui! { DashboardPage("Overview") description="Last 30 days." {
//!     stat (ui.stat("Revenue", "$12,480").delta("+8%"));
//!     stat (ui.stat("Customers", "312"));
//!     body (html! { p { "Recent orders" } });
//! } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};
use crate::stat::Stat;

/// A page of stats, made by [`Ui::dashboard_page`].
///
/// **Setters.** Values and items: `.description(..)`, `.stat(..)`, `.body(..)`.
#[derive(Clone, Debug)]
pub struct DashboardPage<'a> {
    title: &'a str,
    description: Option<&'a str>,
    stats: Vec<Stat<'a>>,
    body: Markup,
}

impl DashboardPage<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("description", PropKind::Value, "text: &'a str").doc("A line under the title."),
        Prop::new("stat", PropKind::Item, "stat: Stat<'a>").doc("A number in the row of stats."),
        Prop::new("body", PropKind::Value, "body: Markup").doc("What goes under the stats."),
    ];
}

impl Ui {
    /// A dashboard titled `title`.
    pub fn dashboard_page<'a>(&self, title: &'a str) -> DashboardPage<'a> {
        DashboardPage {
            title,
            description: None,
            stats: Vec::new(),
            body: Markup::default(),
        }
    }
}

impl<'a> DashboardPage<'a> {
    /// A line under the title.
    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }

    /// A number in the row of stats.
    pub fn stat(mut self, stat: Stat<'a>) -> Self {
        self.stats.push(stat);
        self
    }

    /// What goes under the stats.
    pub fn body(mut self, body: Markup) -> Self {
        self.body = body;
        self
    }
}

impl Render for DashboardPage<'_> {
    fn render(&self) -> Markup {
        html! {
            div class="lui-dashboard" {
                h1 { (self.title) }
                @if let Some(d) = self.description { p class="lui-dashboard-description" { (d) } }
                @if !self.stats.is_empty() { div class="lui-stat-grid" { @for s in &self.stats { (s) } } }
                (self.body)
            }
        }
    }
}

/// Styles for this block; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-dashboard h1 { margin: 0; }
.lui-dashboard-description { margin: 0.25rem 0 0; color: var(--lui-muted); }
"#;
