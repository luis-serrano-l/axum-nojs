//! # Cluster
//!
//! Children side by side, wrapping onto the next line when they run out of room: a row of
//! buttons, badges, filters or links.
//!
//! **Platform features:** `flex-wrap` with `gap` (Chrome 84, Firefox 63, Safari 14.1), so a
//! wrapped line keeps the same spacing and no child carries a margin.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.cluster(html! { (ui.badge("rust")) (ui.badge("maud")) }).render().into_string();
//! assert!(m.starts_with(r#"<div class="nojs-cluster">"#));
//! // A toolbar: the title on the left, the actions pushed to the right.
//! let m = ui.cluster(html! { h2 { "Orders" } (ui.button("New order").primary()) })
//!     .between()
//!     .gap(4);
//! assert!(m.render().into_string().contains(r#"class="nojs-cluster nojs-cluster-between nojs-gap-4""#));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// A wrapping row, made by [`Ui::cluster`].
///
/// **Setters.** Values and items: `.gap(..)`; switches: `.between()`, `.end()`.
#[derive(Clone, Debug)]
pub struct Cluster {
    content: Markup,
    justify: Option<&'static str>,
    gap: Option<u8>,
}

impl Cluster {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("between", PropKind::Switch, "")
            .doc("First child at the start, last at the end, the rest spread between."),
        Prop::new("end", PropKind::Switch, "")
            .doc("Children packed at the end of the row (a dialog's or card's actions)."),
        Prop::new("gap", PropKind::Number, "n: u8")
            .doc("The gap as a step of the `--nojs-space-*` scale."),
    ];
}

impl Ui {
    /// `content`'s top-level elements in a row that wraps, 8px apart by default, centred on
    /// each other vertically.
    pub fn cluster(&self, content: Markup) -> Cluster {
        Cluster {
            content,
            justify: None,
            gap: None,
        }
    }
}

impl Cluster {
    /// First child at the start, last at the end, the rest spread between.
    pub fn between(mut self) -> Self {
        self.justify = Some("nojs-cluster-between");
        self
    }

    /// Children packed at the end of the row (a dialog's or card's actions).
    pub fn end(mut self) -> Self {
        self.justify = Some("nojs-cluster-end");
        self
    }

    /// The gap as a step of the `--nojs-space-*` scale: 0, 1, 2, 3, 4, 6 or 8.
    pub fn gap(mut self, n: u8) -> Self {
        self.gap = Some(n);
        self
    }
}

impl Render for Cluster {
    fn render(&self) -> Markup {
        html! {
            div class={
                "nojs-cluster"
                @if let Some(j) = self.justify { " " (j) }
                @if let Some(n) = self.gap { " " (crate::gap_class(n)) }
            } { (self.content) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-cluster { display: flex; flex-wrap: wrap; align-items: center; }
:where(.nojs-cluster) { gap: var(--nojs-space-2); }
.nojs-cluster > * { margin: 0; }
.nojs-cluster-between { justify-content: space-between; }
.nojs-cluster-end { justify-content: flex-end; }
"#;
