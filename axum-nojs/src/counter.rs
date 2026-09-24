//! # Counter
//!
//! The canonical "click me" demo done with a form round trip. State lives on the server
//! (here: a cookie), no script.
//!
//! **Platform features:** `<form method="post">`, `<button name value>` so one form carries
//! several actions, Post/Redirect/Get, and `view-transition-name` (only when `Caps` says the
//! browser has view transitions) so the number morphs instead of flashing. With
//! `.min()`, `.max()`, `.step()` and `.typed()` the buttons are `disabled` at `min` and `max`, step by `step`, and the
//! value can be typed into an `<input type="number">` with `min`, `max`, `step` (baseline 2015), posted
//! with `op=set`; the browser refuses a typed value off the bounds or the step.
//!
//! **What it does not do without script:** change the number without a round trip; each step is
//! a POST and a redirect.
//!
//! **Fallback:** without view transitions the page simply reloads.
//!
//! **Enhanced:** the form is a swap root (`data-nojs="swap"`), so with the [`crate::enhance`]
//! script each click is a background POST and only the form is replaced; rapid clicks queue.
//!
//! **Finding:** without the script every click is a full navigation, and a click that lands
//! while the page unloads is dropped. There is no optimistic update and no offline behaviour.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let plain = ui.counter("/counter", 3);
//! let bounded = ui.counter("/counter", 10).min(0).max(10).step(2).typed();
//! let html = bounded.render().into_string();
//! assert!(html.contains("value=\"inc\" aria-label=\"increment\" disabled"));
//! assert!(html.contains("type=\"number\" name=\"value\" min=\"0\" max=\"10\" step=\"2\" value=\"10\""));
//! // The handler's half: the same counter applies the posted `op`, clamped to its bounds.
//! assert_eq!(bounded.apply("dec", None), 8);
//! # let _ = plain;
//! ```

use maud::{Markup, Render, html};

use crate::{Cap, Caps, Ui, enhance};

/// A number with buttons posting `op` to `action`, made by [`Ui::counter`]. Unbounded and
/// stepping by 1 unless told otherwise.
#[derive(Clone, Debug)]
pub struct Counter<'a> {
    caps: Caps,
    action: &'a str,
    value: i64,
    min: Option<i64>,
    max: Option<i64>,
    step: i64,
    typed: bool,
}

impl Ui {
    /// A counter showing `value`, its buttons posting to `action`.
    pub fn counter<'a>(&self, action: &'a str, value: i64) -> Counter<'a> {
        Counter {
            caps: self.caps,
            action,
            value,
            min: None,
            max: None,
            step: 1,
            typed: false,
        }
    }
}

impl Counter<'_> {
    /// Lowest value; the decrement button is disabled at it.
    pub fn min(mut self, min: i64) -> Self {
        self.min = Some(min);
        self
    }

    /// Highest value; the increment button is disabled at it.
    pub fn max(mut self, max: i64) -> Self {
        self.max = Some(max);
        self
    }

    /// How far one click moves (at least 1).
    pub fn step(mut self, step: i64) -> Self {
        self.step = step.max(1);
        self
    }

    /// A number field and a Set button posting `op=set&value=n`.
    pub fn typed(mut self) -> Self {
        self.typed = true;
        self
    }

    /// The value after a posted `op` (`inc`, `dec`, `reset`, `set` with the typed value),
    /// clamped to the bounds: the handler's half of the component.
    pub fn apply(&self, op: &str, typed: Option<i64>) -> i64 {
        let (value, min, max) = (self.value, self.min, self.max);
        let next = match op {
            "inc" => value.saturating_add(self.step),
            "dec" => value.saturating_sub(self.step),
            "set" => typed.unwrap_or(value),
            _ => min.unwrap_or(0).max(0).min(max.unwrap_or(i64::MAX)),
        };
        next.clamp(min.unwrap_or(i64::MIN), max.unwrap_or(i64::MAX))
    }
}

impl Render for Counter<'_> {
    fn render(&self) -> Markup {
        let Counter {
            caps,
            action,
            value,
            min,
            max,
            step,
            typed,
        } = *self;
        let vt = caps
            .has(Cap::ViewTransitions)
            .then_some("view-transition-name: nojs-counter");
        let at_min = min.is_some_and(|m| value <= m);
        let at_max = max.is_some_and(|m| value >= m);
        html! {
            form id=(enhance::swap_id("nojs-counter", action)) data-nojs="swap" class="nojs-counter" method="post" action=(action) {
                button type="submit" name="op" value="dec" aria-label="decrement" disabled[at_min] { "−" }
                output style=[vt] { (value) }
                button type="submit" name="op" value="inc" aria-label="increment" disabled[at_max] { "+" }
                button type="submit" name="op" value="reset" { "reset" }
                @if typed {
                    label class="nojs-counter-typed" {
                        span class="nojs-sr" { "Value" }
                        input type="number" name="value" min=[min] max=[max] step=(step) value=(value) inputmode="numeric";
                    }
                    button type="submit" name="op" value="set" { "Set" }
                }
                @if min.is_some() || max.is_some() {
                    small class="nojs-counter-bounds" {
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
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-counter { display: inline-flex; flex-wrap: wrap; align-items: center; gap: var(--nojs-space); }
.nojs-counter output { min-width: 3ch; text-align: center; font-size: 1.5rem; font-variant-numeric: tabular-nums; }
.nojs-counter button:disabled { opacity: 0.45; cursor: not-allowed; }
.nojs-counter-typed input { width: 6em; }
.nojs-counter-bounds { flex-basis: 100%; color: var(--nojs-muted); }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounds_disable_and_clamp() {
        let ui = Ui::from(Caps::NONE);
        let c = |n| ui.counter("/c", n).min(0).max(10).step(3);
        let m = c(0).render().into_string();
        assert!(
            m.contains("aria-label=\"decrement\" disabled")
                && !m.contains("aria-label=\"increment\" disabled"),
            "{m}"
        );
        assert!(m.contains("0 to 10, in steps of 3"));
        assert_eq!(c(9).apply("inc", None), 10);
        assert_eq!(c(1).apply("dec", None), 0);
        assert_eq!(c(5).apply("set", Some(42)), 10);
        assert_eq!(c(5).apply("reset", None), 0);
        assert_eq!(ui.counter("/c", 9).min(5).apply("reset", None), 5);
    }
}
