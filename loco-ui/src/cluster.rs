//! # Cluster
//!
//! Children side by side, wrapping onto the next line when they run out of room: a row of
//! buttons, badges, filters or links.
//!
//! **Platform features:** `flex-wrap` with `gap` (Chrome 84, Firefox 63, Safari 14.1), so a
//! wrapped line keeps the same spacing and no child carries a margin.
//!
//! **Accessibility:** layout only: no roles, reading order is source order. Checked by axe-core
//! in headless Firefox on every demo route, both capability variants, light and dark (no
//! serious or critical violation).
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.cluster().body(html! { (ui.badge("rust")) (ui.badge("maud")) }).render().into_string();
//! assert!(m.starts_with(r#"<div class="lui-cluster">"#));
//! // A toolbar: the title on the left, the actions pushed to the right.
//! let m = ui.cluster()
//!     .between()
//!     .gap(4)
//!     .body(html! { h2 { "Orders" } (ui.button("New order").primary()) });
//! let m = m.render().into_string();
//! assert!(m.contains(r#"class="lui-cluster lui-cluster-between lui-gap-4""#));
//! // The same in `lui!`, where the block is the body:
//! let same = lui! { Cluster between gap=4 { h2 { "Orders" } Button("New order") primary; } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// A wrapping row, made by [`Ui::cluster`].
///
/// **Setters.** Values and items: `.body(..)`, `.gap(..)`; switches: `.between()`, `.end()`.
#[derive(Clone, Debug)]
pub struct Cluster {
    content: Markup,
    justify: Option<&'static str>,
    gap: Option<u8>,
}

impl Cluster {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("body", PropKind::Value, "markup: Markup")
            .doc("What goes in the row: each top-level element beside the one before."),
        Prop::new("between", PropKind::Switch, "")
            .doc("First child at the start, last at the end, the rest spread between."),
        Prop::new("end", PropKind::Switch, "")
            .doc("Children packed at the end of the row (a dialog's or card's actions)."),
        Prop::new("gap", PropKind::Number, "n: u8")
            .doc("The gap as a step of the `--lui-space-*` scale."),
    ];
}

impl Ui {
    /// A cluster: the top-level elements of its [`Cluster::body`] in a row that wraps, 8px
    /// apart by default, centred on each other vertically.
    pub fn cluster(&self) -> Cluster {
        Cluster {
            content: Markup::default(),
            justify: None,
            gap: None,
        }
    }
}

impl Cluster {
    /// What goes in the row: each top-level element beside the one before.
    pub fn body(mut self, markup: Markup) -> Self {
        self.content = markup;
        self
    }

    /// First child at the start, last at the end, the rest spread between.
    pub fn between(mut self) -> Self {
        self.justify = Some("lui-cluster-between");
        self
    }

    /// Children packed at the end of the row (a dialog's or card's actions).
    pub fn end(mut self) -> Self {
        self.justify = Some("lui-cluster-end");
        self
    }

    /// The gap as a step of the `--lui-space-*` scale: 0, 1, 2, 3, 4, 6 or 8.
    pub fn gap(mut self, n: u8) -> Self {
        self.gap = Some(n);
        self
    }
}

impl Render for Cluster {
    fn render(&self) -> Markup {
        html! {
            div class={
                "lui-cluster"
                @if let Some(j) = self.justify { " " (j) }
                @if let Some(n) = self.gap { " " (crate::gap_class(n)) }
            } { (self.content) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-cluster { display: flex; flex-wrap: wrap; align-items: center; }
:where(.lui-cluster) { gap: var(--lui-space-2); }
.lui-cluster > * { margin: 0; }
.lui-cluster-between { justify-content: space-between; }
.lui-cluster-end { justify-content: flex-end; }
"#;
