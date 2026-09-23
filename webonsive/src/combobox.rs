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
//! **Finding:** filtering *as you type* against server data needs script. The datalist covers
//! suggestions; results only update per round trip.
//!
//! ```rust
//! use webonsive::combobox;
//! let m = combobox("/combobox", "q", &["Rust", "Ruby"], "Ru");
//! ```

use maud::{Markup, html};

/// A search form posting `name` to `action` (GET). `options` feed the datalist.
pub fn combobox(action: &str, name: &str, options: &[&str], value: &str) -> Markup {
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

pub const CSS: &str = r#"
.wo-combobox form { display: flex; gap: var(--wo-space); }
.wo-combobox input { flex: 1; }
"#;
