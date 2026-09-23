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
//! **Server persistence:** with a `UiState`, `active` comes from `state.tab(name)` and each
//! title is a link to `?tab.<name>=i`, so the choice survives navigation (query first, cookie
//! after). The link fills the summary, so every click is a round trip: a native toggle would
//! be undone by the next render.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, UiState, tabs};
//! let state = UiState::parse("/docs", "tab.docs=1", "");
//! let m = tabs(&Caps::all(), "docs", &[("Install", html!{ p{"cargo add"} }), ("Use", html!{ p{"html!"} })], Some(&state));
//! assert!(m.into_string().contains("href=\"/docs?tab.docs=0\""));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps, UiState};

/// Tab strip `name`. The open panel is `state.tab(name)`, or the first one without state.
pub fn tabs(caps: &Caps, name: &str, panels: &[(&str, Markup)], state: Option<&UiState>) -> Markup {
    let strip = caps.has(Cap::DetailsContent);
    let active = state.map(|s| s.tab(name)).unwrap_or(0);
    let key = format!("tab.{name}");
    html! {
        div class=(if strip { "wo-tabs" } else { "wo-tabs wo-accordion" }) {
            @for (i, (title, body)) in panels.iter().enumerate() {
                details name=(name) open[i == active] {
                    summary {
                        @match state {
                            Some(s) => a href=(s.link(&key, &i.to_string())) { (title) },
                            None => (title),
                        }
                    }
                    div class=(if strip { "wo-tabs-panel" } else { "wo-accordion-body" }) { (body) }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-tabs:not(.wo-accordion) { display: flex; flex-wrap: wrap; border-bottom: 1px solid var(--wo-line); }
.wo-tabs:not(.wo-accordion) summary {
  order: 0; list-style: none; cursor: pointer; padding: 0.5rem 1rem;
  border-bottom: 2px solid transparent; margin-bottom: -1px; color: var(--wo-muted);
}
.wo-tabs summary::-webkit-details-marker { display: none; }
/* The link fills the summary, so every click goes through the server (a click on bare summary
   padding would toggle natively and be undone by the next render). */
.wo-tabs summary a { display: block; color: inherit; text-decoration: none; }
.wo-tabs:not(.wo-accordion) summary a { margin: -0.5rem -1rem; padding: 0.5rem 1rem; }
.wo-tabs:not(.wo-accordion) details[open] summary { color: var(--wo-fg); border-bottom-color: var(--wo-accent); }
/* Push every panel to a full-width row under the strip. */
.wo-tabs details::details-content { order: 1; flex-basis: 100%; }
.wo-tabs .wo-tabs-panel { order: 1; flex-basis: 100%; padding: 1rem 0; }
.wo-tabs:not(.wo-accordion) details { display: contents; }
"#;
