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
//! use webonsive::{Caps, range};
//! let m = range(&Caps::all(), "volume", 0, 100, 5, 40);
//! assert!(m.into_string().contains("<output"));
//! ```

use maud::{Markup, html};

use crate::Caps;

/// A range input named `name`, with the server's current `value` shown beside it.
pub fn range(_caps: &Caps, name: &str, min: i64, max: i64, step: i64, value: i64) -> Markup {
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
