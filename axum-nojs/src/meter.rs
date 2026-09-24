//! # Meter
//!
//! A measurement within a known range, like disk space or a password's strength: unlike a
//! progress bar it is not a task moving forward, and it turns warn or danger past thresholds.
//!
//! **Platform features:** `<meter>` (Chrome 6, Firefox 16, Safari 6) with `low`, `high` and
//! `optimum`, so the browser picks the good, average or bad colour itself; the
//! `::-webkit-meter-*` and `::-moz-meter-bar` pseudo-elements map those to `--nojs-ok`,
//! `--nojs-warn` and `--nojs-danger`.
//!
//! **What it does not do without script:** update live.
//!
//! **Fallback:** without the pseudo-elements a browser draws its own meter, in its own colours.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.meter(82, 0, 100).label("Disk").low(60).high(80).optimum(0).render().into_string();
//! assert!(m.contains(r#"value="82" min="0" max="100" low="60" high="80" optimum="0""#));
//! ```

use maud::{Markup, Render, html};

use crate::{Ui, slug};

/// A meter, made by [`Ui::meter`].
///
/// **Setters.** Values and items: `.label(..)`, `.low(..)`, `.high(..)`, `.optimum(..)`.
#[derive(Clone, Debug)]
pub struct Meter<'a> {
    value: i64,
    min: i64,
    max: i64,
    low: Option<i64>,
    high: Option<i64>,
    optimum: Option<i64>,
    label: Option<&'a str>,
}

impl Ui {
    /// `value` between `min` and `max`.
    pub fn meter<'a>(&self, value: i64, min: i64, max: i64) -> Meter<'a> {
        Meter {
            value,
            min,
            max,
            low: None,
            high: None,
            optimum: None,
            label: None,
        }
    }
}

impl<'a> Meter<'a> {
    /// A label above the meter, with the value beside it.
    pub fn label(mut self, text: &'a str) -> Self {
        self.label = Some(text);
        self
    }

    /// Below this is "low".
    pub fn low(mut self, low: i64) -> Self {
        self.low = Some(low);
        self
    }

    /// Above this is "high".
    pub fn high(mut self, high: i64) -> Self {
        self.high = Some(high);
        self
    }

    /// The best value: which end of the range is good (0 for disk use, the max for a score).
    pub fn optimum(mut self, optimum: i64) -> Self {
        self.optimum = Some(optimum);
        self
    }
}

impl Render for Meter<'_> {
    fn render(&self) -> Markup {
        let id = format!("nojs-meter-{}", slug(self.label.unwrap_or("meter")));
        html! {
            div class="nojs-progress-field" {
                @if let Some(l) = self.label {
                    label for=(id) { span { (l) } span class="nojs-progress-value" { (self.value) " / " (self.max) } }
                }
                meter id=(id) class="nojs-meter" value=(self.value) min=(self.min) max=(self.max)
                    low=[self.low] high=[self.high] optimum=[self.optimum] { (self.value) " / " (self.max) }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. The progress bar's shape;
/// the browser's good / average / bad choice picks ok, warn or danger.
pub const CSS: &str = r#"
.nojs-meter {
  appearance: none; display: block; width: 100%; height: 0.5rem; border: 0; border-radius: 9999px; overflow: hidden;
  background: var(--nojs-secondary);
}
.nojs-meter::-webkit-meter-bar { background: var(--nojs-secondary); border: 0; border-radius: 9999px; height: 0.5rem; }
.nojs-meter::-webkit-meter-optimum-value { background: var(--nojs-ok); border-radius: 9999px; }
.nojs-meter::-webkit-meter-suboptimum-value { background: var(--nojs-warn); border-radius: 9999px; }
.nojs-meter::-webkit-meter-even-less-good-value { background: var(--nojs-danger); border-radius: 9999px; }
.nojs-meter::-moz-meter-bar { background: var(--nojs-ok); border-radius: 9999px; }
.nojs-meter:-moz-meter-sub-optimum::-moz-meter-bar { background: var(--nojs-warn); }
.nojs-meter:-moz-meter-sub-sub-optimum::-moz-meter-bar { background: var(--nojs-danger); }
"#;
