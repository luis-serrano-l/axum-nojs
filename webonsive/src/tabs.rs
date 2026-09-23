//! # Tabs
//!
//! Tab strip where exactly one panel is open at a time, no script.
//!
//! **Platform features:**
//! - `<details name="group">` exclusive accordion (baseline 2024): opening one closes the rest.
//! - `display: contents` on each `<details>` so its summary and panel become flex items of the
//!   strip; `order` on `::details-content` (Chrome 131, Firefox 143, Safari 18.4) pushes
//!   every panel to a full-width row below all the summaries.
//!
//! **Fallback:** without `Caps::DetailsContent` the same `<details>` render as a stacked
//! accordion, reusing the accordion styles. Without `name` support exclusivity is lost.
//!
//! **Server persistence:** pass the active index from `?tab=n` to restore a tab after a
//! navigation. Clicking a summary toggles without a round trip.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, tabs};
//! let m = tabs(&Caps::all(), "docs", 0, &[("Install", html!{ p{"cargo add"} }), ("Use", html!{ p{"html!"} })]);
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// `active` is the zero-based index of the open panel.
pub fn tabs(caps: &Caps, name: &str, active: usize, panels: &[(&str, Markup)]) -> Markup {
    let strip = caps.has(Cap::DetailsContent);
    html! {
        div class=(if strip { "wo-tabs" } else { "wo-tabs wo-accordion" }) {
            @for (i, (title, body)) in panels.iter().enumerate() {
                details name=(name) open[i == active] {
                    summary { (title) }
                    div class=(if strip { "wo-tabs-panel" } else { "wo-accordion-body" }) { (body) }
                }
            }
        }
    }
}

pub const CSS: &str = r#"
.wo-tabs:not(.wo-accordion) { display: flex; flex-wrap: wrap; border-bottom: 1px solid var(--wo-line); }
.wo-tabs:not(.wo-accordion) summary {
  order: 0; list-style: none; cursor: pointer; padding: 0.5rem 1rem;
  border-bottom: 2px solid transparent; margin-bottom: -1px; color: var(--wo-muted);
}
.wo-tabs summary::-webkit-details-marker { display: none; }
.wo-tabs:not(.wo-accordion) details[open] summary { color: var(--wo-fg); border-bottom-color: var(--wo-accent); }
/* Push every panel to a full-width row under the strip. */
.wo-tabs details::details-content { order: 1; flex-basis: 100%; }
.wo-tabs .wo-tabs-panel { order: 1; flex-basis: 100%; padding: 1rem 0; }
.wo-tabs:not(.wo-accordion) details { display: contents; }
"#;
