//! # Tabs
//!
//! Tab strip where exactly one panel is open at a time, no script.
//!
//! **Platform features:**
//! - `<details name="group">` exclusive accordion (baseline 2024): opening one closes the rest.
//! - `display: contents` on each `<details>` so its summary and panel become flex items of the
//!   strip; `order` on `::details-content` (Chrome 131+, Firefox 138+, Safari 18.4+) pushes
//!   every panel to a full-width row below all the summaries.
//!
//! **Fallback:** `<details>` without `name` support still toggles, only exclusivity is lost.
//!
//! **Server persistence:** each tab `<summary>` wraps a link to `?tab=n`, so navigating away
//! and back can restore the active tab. Clicking the summary toggles without a round trip.
//!
//! ```rust
//! use maud::html;
//! use webonsive::tabs;
//! let m = tabs("docs", 0, &[("Install", html!{ p{"cargo add"} }), ("Use", html!{ p{"html!"} })]);
//! ```

use maud::{Markup, html};

/// `active` is the zero-based index of the open panel.
pub fn tabs(name: &str, active: usize, panels: &[(&str, Markup)]) -> Markup {
    html! {
        div class="wo-tabs" {
            @for (i, (title, body)) in panels.iter().enumerate() {
                details name=(name) open[i == active] {
                    summary { (title) }
                    div class="wo-tabs-panel" { (body) }
                }
            }
        }
    }
}

pub const CSS: &str = r#"
.wo-tabs { display: flex; flex-wrap: wrap; border-bottom: 1px solid var(--wo-line); }
.wo-tabs summary {
  order: 0; list-style: none; cursor: pointer; padding: 0.5rem 1rem;
  border-bottom: 2px solid transparent; margin-bottom: -1px; color: var(--wo-muted);
}
.wo-tabs summary::-webkit-details-marker { display: none; }
.wo-tabs details[open] summary { color: var(--wo-fg); border-bottom-color: var(--wo-accent); }
/* Push every panel to a full-width row under the strip. */
.wo-tabs details::details-content { order: 1; flex-basis: 100%; }
.wo-tabs .wo-tabs-panel { order: 1; flex-basis: 100%; padding: 1rem 0; }
.wo-tabs details { display: contents; }
"#;
