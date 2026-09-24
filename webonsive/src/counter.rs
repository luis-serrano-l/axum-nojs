//! # Counter
//!
//! The canonical "click me" demo done with a form round trip. State lives on the server
//! (here: a cookie), no script.
//!
//! **Platform features:** `<form method="post">`, `<button name value>` so one form carries
//! several actions, Post/Redirect/Get, and `view-transition-name` (only when `Caps` says the
//! browser has view transitions) so the number morphs instead of flashing. With
//! [`CounterOptions`] the buttons are `disabled` at `min` and `max`, step by `step`, and the
//! value can be typed into an `<input type="number">` with `min`, `max`, `step` (baseline 2015), posted
//! with `op=set`; the browser refuses a typed value off the bounds or the step.
//!
//! **What it does not do without script:** change the number without a round trip; each step is
//! a POST and a redirect.
//!
//! **Fallback:** without view transitions the page simply reloads.
//!
//! **Enhanced:** the form is a swap root (`data-wo="swap"`), so with the [`crate::enhance`]
//! script each click is a background POST and only the form is replaced; rapid clicks queue.
//!
//! **Finding:** without the script every click is a full navigation, and a click that lands
//! while the page unloads is dropped. There is no optimistic update and no offline behaviour.
//!
//! ```rust
//! use webonsive::{Caps, counter, counter_with, counter::CounterOptions};
//! let m = counter(&Caps::all(), "/counter", 3);
//! let m = counter_with(&Caps::all(), "/counter", 10, CounterOptions::default().min(0).max(10).step(2).typed());
//! let html = m.into_string();
//! assert!(html.contains("value=\"inc\" aria-label=\"increment\" disabled"));
//! assert!(html.contains("type=\"number\" name=\"value\" min=\"0\" max=\"10\" step=\"2\" value=\"10\""));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps, enhance};

/// Options for [`counter`]; `Default::default()` is unbounded, steps by 1, no typed value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CounterOptions {
    /// Lowest value; the decrement button is disabled at it.
    pub min: Option<i64>,
    /// Highest value; the increment button is disabled at it.
    pub max: Option<i64>,
    /// How far one click moves.
    pub step: i64,
    /// A number field and a Set button posting `op=set&value=n`.
    pub typed: bool,
}

impl Default for CounterOptions {
    fn default() -> Self {
        CounterOptions { min: None, max: None, step: 1, typed: false }
    }
}

impl CounterOptions {
    /// Lowest value.
    pub fn min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }
    /// Highest value.
    pub fn max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }
    /// How far one click moves (at least 1).
    pub fn step(mut self, step: i64) -> Self {
        self.step = step.max(1);
        self
    }
    /// Offer a number field to type the value.
    pub fn typed(mut self) -> Self {
        self.typed = true;
        self
    }
    /// Apply a posted `op` (`inc`, `dec`, `reset`, `set` with `typed`) to `value`, clamped to
    /// the bounds: the handler's half of the component.
    pub fn apply(&self, value: i64, op: &str, typed: Option<i64>) -> i64 {
        let next = match op {
            "inc" => value.saturating_add(self.step),
            "dec" => value.saturating_sub(self.step),
            "set" => typed.unwrap_or(value),
            _ => self.min.unwrap_or(0).max(0).min(self.max.unwrap_or(i64::MAX)),
        };
        next.clamp(self.min.unwrap_or(i64::MIN), self.max.unwrap_or(i64::MAX))
    }
}

/// A counter with the default options.
/// [`counter_with`] takes the options.
pub fn counter(caps: &Caps, action: &str, value: i64) -> Markup {
    counter_with(caps, action, value, Default::default())
}

/// Increment / decrement / reset buttons posting `op` to `action`.
pub fn counter_with(caps: &Caps, action: &str, value: i64, options: CounterOptions) -> Markup {
    let CounterOptions { min, max, step, typed } = options;
    let vt = caps.has(Cap::ViewTransitions).then_some("view-transition-name: wo-counter");
    let at_min = min.is_some_and(|m| value <= m);
    let at_max = max.is_some_and(|m| value >= m);
    html! {
        form id=(enhance::swap_id("wo-counter", action)) data-wo="swap" class="wo-counter" method="post" action=(action) {
            button type="submit" name="op" value="dec" aria-label="decrement" disabled[at_min] { "−" }
            output style=[vt] { (value) }
            button type="submit" name="op" value="inc" aria-label="increment" disabled[at_max] { "+" }
            button type="submit" name="op" value="reset" { "reset" }
            @if typed {
                label class="wo-counter-typed" {
                    span class="wo-sr" { "Value" }
                    input type="number" name="value" min=[min] max=[max] step=(step) value=(value) inputmode="numeric";
                }
                button type="submit" name="op" value="set" { "Set" }
            }
            @if min.is_some() || max.is_some() {
                small class="wo-counter-bounds" {
                    @match (min, max) {
                        (Some(a), Some(b)) => { (a) " to " (b) },
                        (Some(a), None) => { "at least " (a) },
                        (None, Some(b)) => { "at most " (b) },
                        _ => {},
                    }
                    @if step != 1 { ", in steps of " (step) }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-counter { display: inline-flex; flex-wrap: wrap; align-items: center; gap: var(--wo-space); }
.wo-counter output { min-width: 3ch; text-align: center; font-size: 1.5rem; font-variant-numeric: tabular-nums; }
.wo-counter button:disabled { opacity: 0.45; cursor: not-allowed; }
.wo-counter-typed input { width: 6em; }
.wo-counter-bounds { flex-basis: 100%; color: var(--wo-muted); }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_disable_and_clamp() {
        let o = CounterOptions::default().min(0).max(10).step(3);
        let m = counter_with(&Caps::NONE, "/c", 0, o).into_string();
        assert!(m.contains("aria-label=\"decrement\" disabled") && !m.contains("aria-label=\"increment\" disabled"), "{m}");
        assert!(m.contains("0 to 10, in steps of 3"));
        assert_eq!(o.apply(9, "inc", None), 10);
        assert_eq!(o.apply(1, "dec", None), 0);
        assert_eq!(o.apply(5, "set", Some(42)), 10);
        assert_eq!(o.apply(5, "reset", None), 0);
        assert_eq!(CounterOptions::default().min(5).apply(9, "reset", None), 5);
    }
}
