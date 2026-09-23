//! # Combobox
//!
//! A text input with suggestions, and a server-side filtered result list, no script.
//!
//! **Platform features:**
//! - `<input list>` + `<datalist>` (baseline 2020) gives native type-ahead suggestions from a
//!   fixed list the server already knows.
//! - `<search>` element (baseline 2023) for semantics.
//! - Submitting the form (Enter) re-renders with a filtered list from the server.
//!
//! **Fallback:** none needed; `Caps` is accepted for uniformity and unused.
//!
//! **Finding:** filtering *as you type* against server data needs script. The datalist covers
//! suggestions; results only update per round trip.
//!
//! ```rust
//! use webonsive::{Caps, combobox};
//! let m = combobox(&Caps::all(), "/combobox", "q", &["Rust", "Ruby"], "Ru");
//! ```

use maud::{Markup, html};

use crate::Caps;

/// A search form posting `name` to `action` (GET). `options` feed the datalist.
pub fn combobox(_caps: &Caps, action: &str, name: &str, options: &[&str], value: &str) -> Markup {
    let list_id = format!("{name}-options");
    html! {
        search class="wo-combobox" {
            form method="get" action=(action) {
                input type="search" name=(name) list=(list_id) value=(value)
                    placeholder="Type to search…" autocomplete="off";
                datalist id=(list_id) {
                    @for o in options { option value=(o) {} }
                }
                button type="submit" class="wo-primary" { "Search" }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-combobox form { display: flex; gap: var(--wo-space); }
.wo-combobox input { flex: 1; }
"#;
