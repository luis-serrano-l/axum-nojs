//! # Range
//!
//! A slider whose value the server remembers, no script.
//!
//! **Platform features:** `<input type="range">` (Chrome 4, Firefox 23, Safari 3.1) with
//! `min`, `max`, `step`, and an `<output>` that shows the value the server last saw. A
//! `list` of `<datalist>` ticks (Chrome 20, Firefox 110, Safari 12.1) labels the stops.
//!
//! [`range_pair`] is a min/max pair: two range inputs (`<name>_min`, `<name>_max`) laid over
//! one track in a CSS grid cell. The track ignores the pointer (`pointer-events: none`) and
//! only the thumbs take it (`::-webkit-slider-thumb`, `::-moz-range-thumb`), so either thumb
//! can be dragged. Each has its own `<output>`.
//!
//! **What it does not do without script:** show the value while dragging (the `<output>` holds
//! the value the server last saw), or stop the two thumbs of a pair crossing; the server
//! reorders them with `range::order`.
//!
//! **Fallback:** none needed. Without `list` support the ticks are simply not drawn. A pair
//! whose thumbs cross posts a low above the high; [`order`] swaps them back on the server.
//!
//! **Enhanced:** the enhancement script mirrors each slider into its `<output for>` while it
//! moves. Without it the `<output>` shows the value the server last saw.
//!
//! ```rust
//! use webonsive::{Caps, range, range_with, range::{RangeOptions, order, range_pair, range_pair_with}};
//! let m = range(&Caps::all(), "volume", 40);
//! let m = range_with(&Caps::all(), "volume", 40, RangeOptions::default().min(0).max(100).step(5));
//! assert!(m.into_string().contains("<output"));
//! // A pair posted the wrong way round is put back in order.
//! let (lo, hi) = order(80, 20);
//! let m = range_pair_with(&Caps::all(), "price", (lo, hi), RangeOptions::default().step(10)).into_string();
//! assert!(m.contains("name=\"price_min\"") && m.contains("name=\"price_max\""));
//! assert!(m.contains("<output for=\"f-price_min\">20</output>"));
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

/// A 0 to 100 slider.
/// [`range_with`] takes the options.
pub fn range(caps: &Caps, name: &str, value: i64) -> Markup {
    range_with(caps, name, value, Default::default())
}

/// A range input named `name`, with the server's current `value` shown beside it.
pub fn range_with(_caps: &Caps, name: &str, value: i64, options: RangeOptions) -> Markup {
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

/// A 0 to 100 two-thumb range.
/// [`range_pair_with`] takes the options.
pub fn range_pair(caps: &Caps, name: &str, value: (i64, i64)) -> Markup {
    range_pair_with(caps, name, value, Default::default())
}

/// A low/high pair over one track, named `<name>_min` and `<name>_max`.
pub fn range_pair_with(_caps: &Caps, name: &str, (lo, hi): (i64, i64), options: RangeOptions) -> Markup {
    let RangeOptions { min, max, step } = options;
    let (lo_id, hi_id) = (format!("f-{name}_min"), format!("f-{name}_max"));
    html! {
        div class="wo-range wo-range-pair" {
            div class="wo-range-track" {
                input type="range" id=(lo_id) name={ (name) "_min" } min=(min) max=(max) step=(step) value=(lo) aria-label="Minimum";
                input type="range" id=(hi_id) name={ (name) "_max" } min=(min) max=(max) step=(step) value=(hi) aria-label="Maximum";
            }
            span class="wo-range-values" { output for=(lo_id) { (lo) } " – " output for=(hi_id) { (hi) } }
        }
    }
}

/// A posted pair in order: the thumbs can cross, the stored range should not.
pub fn order(a: i64, b: i64) -> (i64, i64) {
    (a.min(b), a.max(b))
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-range { display: flex; align-items: center; gap: var(--wo-space); }
.wo-range input { flex: 1; accent-color: var(--wo-accent); }
.wo-range output { min-width: 3ch; text-align: right; font-variant-numeric: tabular-nums; }
/* Two inputs share one grid cell; only their thumbs catch the pointer. */
.wo-range-track { flex: 1; display: grid; align-items: center; min-height: 1.5rem; }
.wo-range-track::before { content: ""; grid-area: 1 / 1; height: 4px; border-radius: 2px; background: var(--wo-line); }
.wo-range-track input {
  grid-area: 1 / 1; appearance: none; width: 100%; height: 1.5rem; margin: 0; padding: 0;
  border: 0; background: none; pointer-events: none;
}
.wo-range-track input::-webkit-slider-runnable-track { background: none; }
.wo-range-track input::-moz-range-track { background: none; }
.wo-range-track input::-moz-range-progress { background: none; }
.wo-range-track input::-webkit-slider-thumb {
  appearance: none; pointer-events: auto; cursor: pointer; width: 1.1rem; height: 1.1rem; border-radius: 50%;
  background: var(--wo-accent); border: 2px solid var(--wo-surface); box-shadow: 0 0 0 1px var(--wo-line);
}
.wo-range-track input::-moz-range-thumb {
  pointer-events: auto; cursor: pointer; width: 1.1rem; height: 1.1rem; border-radius: 50%; box-sizing: border-box;
  background: var(--wo-accent); border: 2px solid var(--wo-surface); box-shadow: 0 0 0 1px var(--wo-line);
}
.wo-range-track input:focus-visible { outline: none; }
.wo-range-track input:focus-visible::-webkit-slider-thumb { outline: 2px solid var(--wo-accent); outline-offset: 2px; }
.wo-range-track input:focus-visible::-moz-range-thumb { outline: 2px solid var(--wo-accent); outline-offset: 2px; }
.wo-range-values { text-wrap: nowrap; font-variant-numeric: tabular-nums; }
"#;
