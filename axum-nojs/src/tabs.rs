//! # Tabs
//!
//! Tab strip where exactly one panel is open at a time, no script. Tabs can carry a badge
//! count, be lazy (their body rendered only when open), run vertically, and collapse to a
//! `<select>` on narrow screens; the open tab morphs to the next one on switch.
//!
//! **Platform features:**
//! - `<details name="group">` exclusive accordion (baseline 2024): opening one closes the rest.
//! - `display: contents` on each `<details>` so its summary and panel become flex or grid
//!   items of the strip; `order` on `::details-content` (Chrome 131, Firefox 143,
//!   Safari 18.4) pushes every panel to a full-width row below all the summaries, or to the
//!   column beside them when `vertical`.
//! - `view-transition-name` (Chrome 111, Firefox 144, Safari 18) on the open tab's chip
//!   (an empty `.nojs-tabs-mark`, never the title, so no text moves): the raised chip slides to
//!   the new tab, across documents through the layout's `@view-transition` rule and in place
//!   with the enhancement script.
//!
//! **What it does not do without script:** arrow keys between tabs (the WAI-ARIA tabs pattern);
//! each tab is a `<summary>` reached by Tab.
//!
//! **Fallback:** without `Caps::DetailsContent` the same `<details>` render as a stacked
//! accordion, reusing the accordion styles. Without `name` support exclusivity is lost.
//!
//! **Server persistence:** the open tab comes from the request's `?tab.<name>=` (see
//! [`crate::UiState::tab`]) and each title is a link to `?tab.<name>=i`, so the choice survives
//! navigation (query first, cookie after). The link fills the summary, so every click is a
//! round trip: a native toggle would be undone by the next render. The strip is a
//! swap root, so the [`crate::enhance`] script replaces just the strip instead of the page;
//! it also opens the clicked tab before the request, so the switch shows on the click.
//! Because every switch is a request, a lazy tab ([`Tabs::lazy`]) costs nothing until
//! opened: the component calls its closure only when it is the open tab.
//!
//! **Without script:** the narrow-screen `<select>` needs its "Go" button; the script submits
//! it on change.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let mut ui = Ui::from_request("/docs", "tab.docs=1", "");
//! ui.caps = Caps::all();
//! // `badge` applies to the tab added last.
//! let m = ui.tabs("docs")
//!     .tab("Install", html! { p { "cargo add" } })
//!     .tab("Use", html! { p { "html!" } }).badge(3)
//!     .lazy("Changelog", || html! { p { "(long)" } })
//!     .vertical()
//!     .select_below();
//! let html = m.render().into_string();
//! assert!(html.contains("href=\"/docs?tab.docs=0\""));
//! assert!(html.contains("view-transition-name: nojs-tabs-docs"));
//! assert!(html.contains("<select name=\"tab.docs\""));
//! ```

use maud::{Markup, Render, html};

use crate::{Cap, Ui};

/// One tab: a title, a panel (ready or rendered on demand), an optional badge count.
struct Tab<'a> {
    title: &'a str,
    body: Body<'a>,
    badge: Option<usize>,
}

/// A panel: rendered already, or rendered on demand.
enum Body<'a> {
    Ready(Markup),
    Lazy(Box<dyn Fn() -> Markup + 'a>),
}

/// A tab strip, made by [`Ui::tabs`]: the open tab is `?tab.<name>=i` (or the cookie's
/// memory of it), and each title links to its own. Horizontal unless told otherwise.
pub struct Tabs<'a> {
    ui: &'a Ui,
    name: &'a str,
    tabs: Vec<Tab<'a>>,
    vertical: bool,
    select_below: bool,
}

impl Ui {
    /// An empty strip named `name` (the key in `?tab.<name>=`); add tabs with [`Tabs::tab`].
    pub fn tabs<'a>(&'a self, name: &'a str) -> Tabs<'a> {
        Tabs {
            ui: self,
            name,
            tabs: Vec::new(),
            vertical: false,
            select_below: false,
        }
    }
}

impl<'a> Tabs<'a> {
    /// A tab titled `title` with its panel.
    pub fn tab(mut self, title: &'a str, body: Markup) -> Self {
        self.tabs.push(Tab {
            title,
            body: Body::Ready(body),
            badge: None,
        });
        self
    }

    /// A tab whose panel is rendered only when it is the open one: an expensive panel costs
    /// nothing until the round trip that opens it.
    pub fn lazy(mut self, title: &'a str, body: impl Fn() -> Markup + 'a) -> Self {
        self.tabs.push(Tab {
            title,
            body: Body::Lazy(Box::new(body)),
            badge: None,
        });
        self
    }

    /// A count after the title of the tab added last.
    pub fn badge(mut self, count: usize) -> Self {
        if let Some(t) = self.tabs.last_mut() {
            t.badge = Some(count);
        }
        self
    }

    /// Titles in a column on the left, the open panel beside them.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }

    /// On screens under 40rem the titles give way to a `<select>`.
    pub fn select_below(mut self) -> Self {
        self.select_below = true;
        self
    }
}

impl Render for Tabs<'_> {
    fn render(&self) -> Markup {
        let Tabs {
            ui,
            name,
            ref tabs,
            vertical,
            select_below,
        } = *self;
        let s = &ui.state;
        let strip = ui.has(Cap::DetailsContent);
        let active = s.tab(name);
        let key = format!("tab.{name}");
        let class = match (strip, vertical) {
            (false, _) => "nojs-tabs nojs-accordion",
            (true, false) => "nojs-tabs",
            (true, true) => "nojs-tabs nojs-tabs-vertical",
        };
        html! {
            div id={ "nojs-tabs-" (name) } data-nojs="swap"
                class=(class) style=[(strip && vertical).then(|| format!("--nojs-tabs-n: {}", tabs.len()))] {
                @if select_below {
                    form method="get" action=(s.path()) class="nojs-tabs-select" {
                        @for (k, v) in s.entries() { @if k != key { input type="hidden" name=(k) value=(v); } }
                        select name=(key) aria-label="Tab" {
                            @for (i, t) in tabs.iter().enumerate() {
                                option value=(i) selected[i == active] { (t.title) @if let Some(n) = t.badge { " (" (n) ")" } }
                            }
                        }
                        (ui.button("Go"))
                    }
                }
                @for (i, t) in tabs.iter().enumerate() {
                    details name=(name) open[i == active] {
                        summary {
                            a href=(s.link(&key, &i.to_string())) { (t.title) (badge(t)) }
                            @if i == active && strip {
                                span class="nojs-tabs-mark" style=(format!("view-transition-name: nojs-tabs-{name}")) {}
                            }
                        }
                        div class=(if strip { "nojs-tabs-panel" } else { "nojs-accordion-body" }) {
                            @match (&t.body, i == active) {
                                (Body::Ready(body), _) => (body),
                                (Body::Lazy(render), true) => (render()),
                                (Body::Lazy(_), false) => span class="nojs-tabs-lazy" {},
                            }
                        }
                    }
                }
            }
        }
    }
}
fn badge(t: &Tab) -> Markup {
    html! { @if let Some(n) = t.badge { " " span class="nojs-tabs-badge" { (n) } } }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-tabs:not(.nojs-accordion) { display: flex; flex-wrap: wrap; }
/* Fills the rest of the titles' row so the panels start on the next one. */
.nojs-tabs:not(.nojs-accordion):not(.nojs-tabs-vertical)::after { content: ""; order: 0; flex: 1; }
/* shadcn Tabs: the titles sit in a muted pill (secondary, 3px padding, rounded-lg); the open
   one is a raised chip on the page background. Each summary paints its slice of the pill. */
.nojs-tabs:not(.nojs-accordion) summary {
  order: 0; position: relative; list-style: none; cursor: pointer; padding: 3px;
  background: var(--nojs-secondary); color: var(--nojs-muted);
}
.nojs-tabs:not(.nojs-accordion):not(.nojs-tabs-vertical) > details:first-of-type > summary { border-radius: var(--nojs-radius) 0 0 var(--nojs-radius); }
.nojs-tabs:not(.nojs-accordion):not(.nojs-tabs-vertical) > details:last-of-type > summary { border-radius: 0 var(--nojs-radius) var(--nojs-radius) 0; }
.nojs-tabs:not(.nojs-accordion):not(.nojs-tabs-vertical) > details:only-of-type > summary { border-radius: var(--nojs-radius); }
.nojs-tabs summary::-webkit-details-marker { display: none; }
/* The link fills the summary, so every click goes through the server (a click on bare summary
   padding would toggle natively and be undone by the next render). */
.nojs-tabs summary a { display: block; color: inherit; text-decoration: none; }
.nojs-tabs:not(.nojs-accordion) summary a {
  position: relative; z-index: 1; display: flex; align-items: center; gap: 0.375rem;
  padding: 0.25rem 0.75rem; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; white-space: nowrap;
}
.nojs-tabs:not(.nojs-accordion) summary a:hover { color: var(--nojs-fg); }
.nojs-tabs:not(.nojs-accordion) details[open] summary { color: var(--nojs-fg); }
/* The chip is its own empty element so the view transition slides it, not the text. */
.nojs-tabs-mark {
  position: absolute; inset: 3px; border-radius: var(--nojs-radius-sm);
  background: var(--nojs-bg); box-shadow: var(--nojs-shadow-xs);
}
.nojs-tabs-badge {
  display: inline-block; min-width: 1.25rem; padding: 0 0.3rem; border-radius: 1em; text-align: center;
  font-size: 0.75rem; font-weight: 500; line-height: 1.25rem; background: var(--nojs-line); color: var(--nojs-fg);
}
.nojs-tabs details[open] .nojs-tabs-badge { background: var(--nojs-primary); color: var(--nojs-on-primary); }
/* Push every panel to a full-width row under the strip. */
.nojs-tabs details::details-content { order: 1; flex-basis: 100%; }
.nojs-tabs .nojs-tabs-panel { order: 1; flex-basis: 100%; padding: 1rem 0; }
.nojs-tabs:not(.nojs-accordion) details { display: contents; }
/* Vertical: a plain list with a rule, the open title marked by a bar on the rule. The titles
   are in the first column, the open panel spans every row of the second. The rule goes on
   ::details-content, the grid item, so it runs the full height; the padding stays on the
   panel (Blitz builds no ::details-content box, see FINDINGS). */
.nojs-tabs.nojs-tabs-vertical { display: grid; grid-template-columns: max-content 1fr; }
.nojs-tabs.nojs-tabs-vertical summary {
  grid-column: 1; padding: 0; background: none; border-right: 1px solid var(--nojs-line); margin: 0 -1px 0 0;
}
.nojs-tabs.nojs-tabs-vertical summary a { padding: 0.5rem 1rem; }
.nojs-tabs.nojs-tabs-vertical .nojs-tabs-mark { inset: 0 -1px 0 auto; width: 2px; border-radius: 0; background: var(--nojs-fg); box-shadow: none; }
.nojs-tabs.nojs-tabs-vertical details::details-content {
  grid-column: 2; grid-row: 1 / span var(--nojs-tabs-n, 1); border-left: 1px solid var(--nojs-line);
}
.nojs-tabs.nojs-tabs-vertical .nojs-tabs-panel {
  grid-column: 2; grid-row: 1 / span var(--nojs-tabs-n, 1); padding: 0 0 0 calc(var(--nojs-space) * 3);
}
.nojs-tabs.nojs-tabs-vertical .nojs-tabs-select { grid-column: 1 / -1; }
/* Narrow screens: the titles give way to the select (needs the enhancement script for
   submit-on-change; the Go button is always there). */
.nojs-tabs-select { display: none; gap: var(--nojs-space); flex-basis: 100%; margin-bottom: var(--nojs-space); }
@media (max-width: 40rem) {
  .nojs-tabs:not(.nojs-accordion):has(.nojs-tabs-select) summary, .nojs-tabs:has(.nojs-tabs-select)::after { display: none; }
  .nojs-tabs:not(.nojs-accordion) .nojs-tabs-select { display: flex; }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn a_lazy_tab_renders_only_when_open() {
        let calls = Cell::new(0);
        let render = || {
            calls.set(calls.get() + 1);
            html! { p { "Changelog body" } }
        };
        let strip = |query: &str| {
            let ui = Ui::from_request("/docs", query, "");
            ui.tabs("docs")
                .tab("Install", html! { "cargo add" })
                .lazy("Changelog", render)
                .render()
                .into_string()
        };
        let closed = strip("");
        assert!(!closed.contains("Changelog body") && closed.contains("nojs-tabs-lazy"));
        assert_eq!(calls.get(), 0);
        let open = strip("tab.docs=1");
        assert!(open.contains("Changelog body") && !open.contains("nojs-tabs-lazy"));
        assert_eq!(calls.get(), 1);
    }
}
