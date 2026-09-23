//! # Accordion
//!
//! Stacked disclosure sections; at most one open at a time, no script.
//!
//! **Platform features:** `<details name="group">` (baseline 2024). Omit `name` behaviour by
//! passing an empty group name to allow several open at once.
//!
//! **Fallback:** `<details>` alone (baseline 2020) still toggles; only the exclusivity is lost.
//!
//! ```rust
//! use maud::html;
//! use webonsive::accordion;
//! let m = accordion("faq", &[("What?", html!{ p{"A"} }), ("Why?", html!{ p{"B"} })]);
//! ```

use maud::{Markup, html};

/// `group` empty means sections open independently.
pub fn accordion(group: &str, items: &[(&str, Markup)]) -> Markup {
    html! {
        div class="wo-accordion" {
            @for (title, body) in items {
                details name=[(!group.is_empty()).then_some(group)] {
                    summary { (title) }
                    div class="wo-accordion-body" { (body) }
                }
            }
        }
    }
}

pub const CSS: &str = r#"
.wo-accordion { border: 1px solid var(--wo-line); border-radius: var(--wo-radius); overflow: hidden; }
.wo-accordion details + details { border-top: 1px solid var(--wo-line); }
.wo-accordion summary { cursor: pointer; padding: 0.75rem 1rem; font-weight: 600; }
.wo-accordion summary:hover { background: var(--wo-surface); }
.wo-accordion-body { padding: 0 1rem 1rem; }
.wo-accordion details::details-content { transition: height 0.2s, content-visibility 0.2s allow-discrete; height: 0; overflow: hidden; }
.wo-accordion details[open]::details-content { height: auto; }
"#;
