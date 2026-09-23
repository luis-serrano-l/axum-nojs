//! # Counter
//!
//! The canonical "click me" demo done with a form round trip. State lives on the server
//! (here: a cookie), no script.
//!
//! **Platform features:** `<form method="post">`, `<button name value>` so one form carries
//! several actions, Post/Redirect/Get, and view transitions from the layout so the number
//! morphs instead of flashing.
//!
//! **Finding:** every click is a full navigation. It is fast on localhost and fine on a good
//! connection, but there is no optimistic update and no offline behaviour.
//!
//! ```rust
//! use webonsive::counter;
//! let m = counter("/counter", 3);
//! ```

use maud::{Markup, html};

/// Increment / decrement / reset buttons posting `op` to `action`.
pub fn counter(action: &str, value: i64) -> Markup {
    html! {
        form class="wo-counter" method="post" action=(action) {
            button type="submit" name="op" value="dec" aria-label="decrement" { "−" }
            output style="view-transition-name: wo-counter" { (value) }
            button type="submit" name="op" value="inc" aria-label="increment" { "+" }
            button type="submit" name="op" value="reset" { "reset" }
        }
    }
}

pub const CSS: &str = r#"
.wo-counter { display: inline-flex; align-items: center; gap: var(--wo-space); }
.wo-counter output { min-width: 3ch; text-align: center; font-size: 1.5rem; font-variant-numeric: tabular-nums; }
"#;
