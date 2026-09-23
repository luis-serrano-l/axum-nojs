//! # Range
//!
//! A slider whose value the server remembers, no script.
//!
//! **Platform features:** `<input type="range">` (Chrome 4, Firefox 23, Safari 3.1) with
//! `min`, `max`, `step`, and an `<output>` that shows the value the server last saw. A
//! `list` of `<datalist>` ticks (Chrome 20, Firefox 110, Safari 12.1) labels the stops.
//!
//! **Fallback:** none needed. Without `list` support the ticks are simply not drawn.
//!
//! **Finding:** the `<output>` only updates on submit. Mirroring the slider as it moves is the
//! one line of script this crate refuses to ship; the value is still correct after every
//! round trip.
//!
//! ```rust
//! use webonsive::{Caps, range, range::RangeOptions};
//! let m = range(&Caps::all(), "volume", 40, Default::default());
//! let m = range(&Caps::all(), "volume", 40, RangeOptions::default().min(0).max(100).step(5));
//! assert!(m.into_string().contains("<output"));
//! ```

use maud::{Markup, html};

use crate::Caps;

/// Options for [`range`]; `Default::default()` is 0 to 100 in steps of 1.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeOptions {
    /// Lowest value.
    pub min: i64,
    /// Highest value.
    pub max: i64,
    /// Distance between allowed values.
    pub step: i64,
}

impl Default for RangeOptions {
    fn default() -> Self {
        RangeOptions { min: 0, max: 100, step: 1 }
    }
}

impl RangeOptions {
    /// Lowest value.
    pub fn min(mut self, min: i64) -> Self {
        self.min = min;
        self
    }

    /// Highest value.
    pub fn max(mut self, max: i64) -> Self {
        self.max = max;
        self
    }

    /// Distance between allowed values.
    pub fn step(mut self, step: i64) -> Self {
        self.step = step.max(1);
        self
    }
}

/// A range input named `name`, with the server's current `value` shown beside it.
pub fn range(_caps: &Caps, name: &str, value: i64, options: RangeOptions) -> Markup {
    let RangeOptions { min, max, step } = options;
    let list = format!("{name}-ticks");
    let id = format!("f-{name}");
    html! {
        div class="wo-range" {
            input type="range" id=(id) name=(name) min=(min) max=(max) step=(step) value=(value) list=(list);
            datalist id=(list) {
                option value=(min) label=(min) {}
                option value=((min + max) / 2) {}
                option value=(max) label=(max) {}
            }
            output for=(id) { (value) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-range { display: flex; align-items: center; gap: var(--wo-space); }
.wo-range input { flex: 1; accent-color: var(--wo-accent); }
.wo-range output { min-width: 3ch; text-align: right; font-variant-numeric: tabular-nums; }
"#;
