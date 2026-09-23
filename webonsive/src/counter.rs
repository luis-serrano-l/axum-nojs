//! # Counter
//!
//! The canonical "click me" demo done with a form round trip. State lives on the server
//! (here: a cookie), no script.
//!
//! **Platform features:** `<form method="post">`, `<button name value>` so one form carries
//! several actions, Post/Redirect/Get, and `view-transition-name` (only when `Caps` says the
//! browser has view transitions) so the number morphs instead of flashing.
//!
//! **Fallback:** without view transitions the page simply reloads.
//!
//! **Finding:** every click is a full navigation. It is fast on localhost and fine on a good
//! connection, but there is no optimistic update and no offline behaviour.
//!
//! ```rust
//! use webonsive::{Caps, counter};
//! let m = counter(&Caps::all(), "/counter", 3);
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// Increment / decrement / reset buttons posting `op` to `action`.
pub fn counter(caps: &Caps, action: &str, value: i64) -> Markup {
    let vt = caps.has(Cap::ViewTransitions).then_some("view-transition-name: wo-counter");
    html! {
        form class="wo-counter" method="post" action=(action) {
            button type="submit" name="op" value="dec" aria-label="decrement" { "−" }
            output style=[vt] { (value) }
            button type="submit" name="op" value="inc" aria-label="increment" { "+" }
            button type="submit" name="op" value="reset" { "reset" }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-counter { display: inline-flex; align-items: center; gap: var(--wo-space); }
.wo-counter output { min-width: 3ch; text-align: center; font-size: 1.5rem; font-variant-numeric: tabular-nums; }
"#;
