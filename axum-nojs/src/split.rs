//! # Split
//!
//! A narrow side beside a wide main part, the side stacking on top when the main part would
//! get narrower than half the width: a settings nav beside its form, filters beside results.
//!
//! **Platform features:** the "sidebar" flex pattern: `flex-wrap`, the side at its width and
//! the main part growing with `flex-grow: 999` and `min-inline-size: 50%`, so the switch to
//! one column happens by container width, with no media query.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.split(html! { nav { "Settings" } }, html! { p { "Form" } }).render().into_string();
//! assert!(m.contains(r#"<div class="nojs-split-side"><nav>Settings</nav></div>"#));
//! // A wider side, placed after the main part.
//! let m = ui.split(html! { aside { "Filters" } }, html! { p { "Results" } }).side_width("20rem").side_end();
//! let m = m.render().into_string();
//! assert!(m.contains("--nojs-split-side: 20rem") && m.find("Results") < m.find("Filters"));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;

/// A side-and-main layout, made by [`Ui::split`].
///
/// **Setters.** Values and items: `.side_width(..)`, `.gap(..)`; switches: `.side_end()`.
#[derive(Clone, Debug)]
pub struct Split<'a> {
    side: Markup,
    main: Markup,
    width: Option<&'a str>,
    side_end: bool,
    gap: Option<u8>,
}

impl Ui {
    /// `side` (15rem wide by default) beside `main`, which takes the rest.
    pub fn split<'a>(&self, side: Markup, main: Markup) -> Split<'a> {
        Split {
            side,
            main,
            width: None,
            side_end: false,
            gap: None,
        }
    }
}

impl<'a> Split<'a> {
    /// The side's width, any CSS length.
    pub fn side_width(mut self, width: &'a str) -> Self {
        self.width = Some(width);
        self
    }

    /// The side after the main part, in the markup and on screen.
    pub fn side_end(mut self) -> Self {
        self.side_end = true;
        self
    }

    /// The gap as a step of the `--nojs-space-*` scale: 0, 1, 2, 3, 4, 6 or 8.
    pub fn gap(mut self, n: u8) -> Self {
        self.gap = Some(n);
        self
    }
}

impl Render for Split<'_> {
    fn render(&self) -> Markup {
        let side = html! { div class="nojs-split-side" { (self.side) } };
        let main = html! { div class="nojs-split-main" { (self.main) } };
        html! {
            div class={ "nojs-split" @if let Some(n) = self.gap { " " (crate::gap_class(n)) } }
                style=[self.width.map(|w| format!("--nojs-split-side: {w}"))] {
                @if self.side_end { (main) (side) } @else { (side) (main) }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-split { display: flex; flex-wrap: wrap; }
:where(.nojs-split) { gap: var(--nojs-space-4); }
.nojs-split-side { flex-basis: var(--nojs-split-side, 15rem); flex-grow: 1; }
.nojs-split-main { flex-basis: 0; flex-grow: 999; min-inline-size: 50%; }
"#;
