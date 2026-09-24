//! # Grid
//!
//! As many equal columns as fit, each at least a given width: cards, stats, a gallery. It
//! drops to fewer columns on a narrow screen with no media query.
//!
//! **Platform features:** `grid-template-columns: repeat(auto-fill, minmax(min(<min>, 100%),
//! 1fr))` (CSS Grid, baseline 2017; `min()`, baseline 2020), so a column never overflows a
//! screen narrower than its minimum. The minimum travels in a `--nojs-grid-min` custom
//! property on the element.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.grid("15rem", html! { (ui.card().title("A")) (ui.card().title("B")) }).render().into_string();
//! assert!(m.starts_with(r#"<div class="nojs-grid" style="--nojs-grid-min: 15rem">"#));
//! let m = ui.grid("10rem", html! { p { "x" } }).gap(2).render().into_string();
//! assert!(m.contains("nojs-grid nojs-gap-2"));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;

/// A responsive grid, made by [`Ui::grid`].
#[derive(Clone, Debug)]
pub struct Grid<'a> {
    min: &'a str,
    content: Markup,
    gap: Option<u8>,
}

impl Ui {
    /// `content`'s top-level elements in columns at least `min` wide (any CSS length:
    /// `"15rem"`, `"240px"`), 16px apart by default.
    pub fn grid<'a>(&self, min: &'a str, content: Markup) -> Grid<'a> {
        Grid {
            min,
            content,
            gap: None,
        }
    }
}

impl Grid<'_> {
    /// The gap as a step of the `--nojs-space-*` scale: 0, 1, 2, 3, 4, 6 or 8.
    pub fn gap(mut self, n: u8) -> Self {
        self.gap = Some(n);
        self
    }
}

impl Render for Grid<'_> {
    fn render(&self) -> Markup {
        html! {
            div class={ "nojs-grid" @if let Some(n) = self.gap { " " (crate::gap_class(n)) } }
                style={ "--nojs-grid-min: " (self.min) } { (self.content) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(min(var(--nojs-grid-min, 15rem), 100%), 1fr)); }
:where(.nojs-grid) { gap: var(--nojs-space-4); }
.nojs-grid > * { margin: 0; min-width: 0; }
"#;
