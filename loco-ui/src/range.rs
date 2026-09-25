//! # Range
//!
//! A slider whose value the server remembers, no script.
//!
//! **Platform features:** `<input type="range">` (Chrome 4, Firefox 23, Safari 3.1) with
//! `min`, `max`, `step`, and an `<output>` that shows the value the server last saw. A
//! `list` of `<datalist>` ticks (Chrome 20, Firefox 110, Safari 12.1) labels the stops.
//!
//! [`Ui::range_pair`] is a min/max pair: two range inputs (`<name>_min`, `<name>_max`) laid over
//! one track in a CSS grid cell. The track ignores the pointer (`pointer-events: none`) and
//! only the thumbs take it (`::-webkit-slider-thumb`, `::-moz-range-thumb`), so either thumb
//! can be dragged. Each has its own `<output>`.
//!
//! **Accessibility:** native range inputs, each named (the pair's thumbs as "Minimum" and
//! "Maximum"); the value is shown in an `<output>`. Checked by axe-core in headless Firefox on
//! every demo route, both capability variants, light and dark (no serious or critical
//! violation).
//!
//! **What it does not do without script:** show the value while dragging (the `<output>` holds
//! the value the server last saw), or stop the two thumbs of a pair crossing; the server
//! reorders them with [`order`].
//!
//! **Fallback:** none needed. Without `list` support the ticks are simply not drawn. A pair
//! whose thumbs cross posts a low above the high; [`order`] swaps them back on the server.
//!
//! **Enhanced:** the enhancement script mirrors each slider into its `<output for>` while it
//! moves. Without it the `<output>` shows the value the server last saw.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let m = ui.range("volume", 40).step(5).label("Volume").render().into_string();
//! assert!(m.contains(r#"<label for="f-volume">Volume</label>"#) && m.contains("<output"));
//! // A pair posted the wrong way round is put back in order.
//! let m = ui.range_pair("price", (80, 20)).step(10).render().into_string();
//! assert!(m.contains("name=\"price_min\"") && m.contains("name=\"price_max\""));
//! assert!(m.contains("<output for=\"f-price_min\">20</output>"));
//!
//! // The same in `lui!`:
//! let same = lui! { RangePair("price", (80, 20)) step=10; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::i18n::{Strings, Text};
use crate::props::{Prop, PropKind};

/// A slider with the server's current value beside it, made by [`Ui::range`] or, as a
/// low/high pair over one track, by [`Ui::range_pair`]. 0 to 100 in steps of 1 unless told
/// otherwise.
///
/// **Setters.** Values and items: `.min(..)`, `.max(..)`, `.step(..)`, `.label(..)`.
#[derive(Clone, Debug)]
pub struct Range<'a> {
    name: &'a str,
    value: (i64, Option<i64>),
    min: i64,
    max: i64,
    step: i64,
    label: Option<&'a str>,
    strings: &'static Strings,
}

impl Range<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("min", PropKind::Number, "min: i64")
            .default("0")
            .attr("min")
            .doc("Lowest value."),
        Prop::new("max", PropKind::Number, "max: i64")
            .default("100")
            .attr("max")
            .doc("Highest value."),
        Prop::new("step", PropKind::Number, "step: i64")
            .default("1")
            .attr("step")
            .doc("Distance between allowed values (at least 1)."),
        Prop::new("label", PropKind::Value, "label: &'a str")
            .attr("label")
            .doc("A `<label>` above the slider, in a `div.lui-field` like a form field."),
    ];
}

impl Ui {
    /// A range input named `name` at `value`.
    pub fn range<'a>(&self, name: &'a str, value: i64) -> Range<'a> {
        Range {
            strings: self.strings,
            name,
            value: (value, None),
            min: 0,
            max: 100,
            step: 1,
            label: None,
        }
    }

    /// Two thumbs over one track, named `<name>_min` and `<name>_max`, at `(low, high)` (put
    /// in order if they crossed).
    pub fn range_pair<'a>(&self, name: &'a str, (a, b): (i64, i64)) -> Range<'a> {
        let (lo, hi) = order(a, b);
        Range {
            value: (lo, Some(hi)),
            ..self.range(name, lo)
        }
    }
}

impl<'a> Range<'a> {
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

    /// Distance between allowed values (at least 1).
    pub fn step(mut self, step: i64) -> Self {
        self.step = step.max(1);
        self
    }

    /// A `<label>` above the slider, in a `div.lui-field` like a form field.
    pub fn label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

impl Render for Range<'_> {
    fn render(&self) -> Markup {
        let Range {
            strings: _,
            name,
            value,
            min,
            max,
            step,
            label,
        } = *self;
        let control = match value {
            (value, None) => {
                let (list, id) = (format!("{name}-ticks"), format!("f-{name}"));
                html! {
                    div class="lui-range" {
                        input type="range" class="lui-range-input" id=(id) name=(name) min=(min) max=(max) step=(step) value=(value) list=(list);
                        datalist id=(list) {
                            option value=(min) label=(min) {}
                            option value=((min + max) / 2) {}
                            option value=(max) label=(max) {}
                        }
                        output for=(id) { (value) }
                    }
                }
            }
            (lo, Some(hi)) => {
                let (lo_id, hi_id) = (format!("f-{name}_min"), format!("f-{name}_max"));
                html! {
                    div class="lui-range lui-range-pair" {
                        div class="lui-range-track" {
                            input type="range" class="lui-range-input" id=(lo_id) name={ (name) "_min" } min=(min) max=(max) step=(step) value=(lo) aria-label=(self.strings.get(Text::Minimum));
                            input type="range" class="lui-range-input" id=(hi_id) name={ (name) "_max" } min=(min) max=(max) step=(step) value=(hi) aria-label=(self.strings.get(Text::Maximum));
                        }
                        span class="lui-range-values" { output for=(lo_id) { (lo) } " – " output for=(hi_id) { (hi) } }
                    }
                }
            }
        };
        // A pair's label names the group; it points at the low thumb.
        let id = if value.1.is_some() {
            format!("f-{name}_min")
        } else {
            format!("f-{name}")
        };
        crate::labelled(label, &id, control)
    }
}
/// A posted pair in order: the thumbs can cross, the stored range should not.
pub fn order(a: i64, b: i64) -> (i64, i64) {
    (a.min(b), a.max(b))
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-range { display: flex; align-items: center; gap: var(--lui-space); }
.lui-range-input { flex: 1; accent-color: var(--lui-primary); }
.lui-range output { min-width: 3ch; text-align: right; font-variant-numeric: tabular-nums; }
/* Two inputs share one grid cell; only their thumbs catch the pointer. */
.lui-range-track { flex: 1; display: grid; align-items: center; min-height: 1.5rem; }
.lui-range-track::before { content: ""; grid-area: 1 / 1; height: 6px; border-radius: 3px; background: var(--lui-secondary); }
.lui-range-track .lui-range-input {
  grid-area: 1 / 1; appearance: none; width: 100%; height: 1.5rem; margin: 0; padding: 0;
  border: 0; background: none; pointer-events: none;
}
.lui-range-track .lui-range-input::-webkit-slider-runnable-track { background: none; }
.lui-range-track .lui-range-input::-moz-range-track { background: none; }
.lui-range-track .lui-range-input::-moz-range-progress { background: none; }
.lui-range-track .lui-range-input::-webkit-slider-thumb {
  appearance: none; pointer-events: auto; cursor: pointer; width: 1rem; height: 1rem; border-radius: 50%;
  background: var(--lui-bg); border: 1px solid var(--lui-primary); box-shadow: var(--lui-shadow-xs);
}
.lui-range-track .lui-range-input::-moz-range-thumb {
  pointer-events: auto; cursor: pointer; width: 1rem; height: 1rem; border-radius: 50%; box-sizing: border-box;
  background: var(--lui-bg); border: 1px solid var(--lui-primary); box-shadow: var(--lui-shadow-xs);
}
.lui-range-track .lui-range-input:focus-visible { outline: none; }
.lui-range-track .lui-range-input:focus-visible::-webkit-slider-thumb { outline: 4px solid color-mix(in srgb, var(--lui-ring) 50%, transparent); outline-offset: 0; }
.lui-range-track .lui-range-input:focus-visible::-moz-range-thumb { outline: 4px solid color-mix(in srgb, var(--lui-ring) 50%, transparent); outline-offset: 0; }
.lui-range-values { text-wrap: nowrap; font-variant-numeric: tabular-nums; }
"#;
