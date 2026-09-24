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
//! - `view-transition-name` (Chrome 111, Firefox 144, Safari 18) on the open tab's underline
//!   (an empty `.nojs-tabs-mark`, never the title, so no text moves): the accent bar slides to
//!   the new tab, across documents through the layout's `@view-transition` rule and in place
//!   with the enhancement script.
//!
//! **What it does not do without script:** arrow keys between tabs (the WAI-ARIA tabs pattern);
//! each tab is a `<summary>` reached by Tab.
//!
//! **Fallback:** without `Caps::DetailsContent` the same `<details>` render as a stacked
//! accordion, reusing the accordion styles. Without `name` support exclusivity is lost.
//!
//! **Server persistence:** with a `UiState` in the options, the open tab comes from
//! `state.tab(name)` and each title is a link to `?tab.<name>=i`, so the choice survives
//! navigation (query first, cookie after). The link fills the summary, so every click is a
//! round trip: a native toggle would be undone by the next render. With a state the strip is a
//! swap root, so the [`crate::enhance`] script replaces just the strip instead of the page.
//! Because every switch is a request, a lazy tab ([`Tab::lazy_with`]) costs nothing until
//! opened: the component calls its closure only when it is the open tab.
//!
//! **Without script:** the narrow-screen `<select>` needs its "Go" button; the script submits
//! it on change. Without a state the tabs toggle natively and remember nothing.
//!
//! ```rust
//! use maud::html;
//! use axum_nojs::{Caps, UiState, tabs, tabs_with, tabs::{Tab, TabsOptions}};
//! let m = tabs(&Caps::all(), "t", &[Tab::new("One", html! { p { "First." } }), Tab::new("Two", html! { p { "Second." } })]);
//!
//! let state = UiState::parse("/docs", "tab.docs=1", "");
//! let m = tabs_with(&Caps::all(), "docs", &[
//!     Tab::new("Install", html! { p { "cargo add" } }),
//!     Tab::new("Use", html! { p { "html!" } }).badge(3),
//!     Tab::lazy_with("Changelog", &|| html! { p { "(long)" } }),
//! ], TabsOptions::default().state(&state).vertical().select_below());
//! let html = m.into_string();
//! assert!(html.contains("href=\"/docs?tab.docs=0\""));
//! assert!(html.contains("view-transition-name: nojs-tabs-docs"));
//! assert!(html.contains("<select name=\"tab.docs\""));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps, UiState};

/// One tab: a title, a body (or none for a lazy tab), an optional badge count.
#[derive(Clone)]
pub struct Tab<'a> {
    title: &'a str,
    body: Body<'a>,
    badge: Option<usize>,
}

/// A panel: rendered already, or rendered on demand.
#[derive(Clone)]
enum Body<'a> {
    Ready(Markup),
    Lazy(&'a dyn Fn() -> Markup),
}

impl std::fmt::Debug for Tab<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let body = match &self.body { Body::Ready(_) => "ready", Body::Lazy(_) => "lazy" };
        f.debug_struct("Tab").field("title", &self.title).field("body", &body).field("badge", &self.badge).finish()
    }
}

impl<'a> Tab<'a> {
    /// A tab with its panel.
    pub const fn new(title: &'a str, body: Markup) -> Self {
        Tab { title, body: Body::Ready(body), badge: None }
    }
    /// A tab whose panel is rendered only when it is the open one: the component calls
    /// `render` for the open tab and never for the others, so an expensive panel costs
    /// nothing until the round trip that opens it.
    pub const fn lazy_with(title: &'a str, render: &'a dyn Fn() -> Markup) -> Self {
        Tab { title, body: Body::Lazy(render), badge: None }
    }
    /// A count shown after the title.
    pub const fn badge(mut self, count: usize) -> Self {
        self.badge = Some(count);
        self
    }
}

/// `("Install", html! { … })`: a tab, its title and its panel.
impl<'a> From<(&'a str, Markup)> for Tab<'a> {
    fn from((title, body): (&'a str, Markup)) -> Self {
        Tab::new(title, body)
    }
}

/// Options for [`tabs`]; `Default::default()` is a horizontal strip with no server state.
#[derive(Clone, Copy, Debug, Default)]
pub struct TabsOptions<'a> {
    /// Server-held open tab and links to change it.
    pub state: Option<&'a UiState>,
    /// Titles in a column on the left, the open panel beside them.
    pub vertical: bool,
    /// On screens under 40rem the titles give way to a `<select>` (needs `state`).
    pub select_below: bool,
}

impl<'a> TabsOptions<'a> {
    /// Server-held open tab and links to change it.
    pub fn state(mut self, state: &'a UiState) -> Self {
        self.state = Some(state);
        self
    }
    /// Titles in a column on the left.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
    /// Collapse to a `<select>` on narrow screens.
    pub fn select_below(mut self) -> Self {
        self.select_below = true;
        self
    }
}

/// Tabs with the first one open and no remembered state.
/// [`tabs_with`] takes the options.
pub fn tabs(caps: &Caps, name: &str, tabs: &[Tab]) -> Markup {
    tabs_with(caps, name, tabs, Default::default())
}

/// Tab strip `name`. The open panel is `state.tab(name)`, or the first one without state.
pub fn tabs_with(caps: &Caps, name: &str, tabs: &[Tab], options: TabsOptions) -> Markup {
    let TabsOptions { state, vertical, select_below } = options;
    let strip = caps.has(Cap::DetailsContent);
    let active = state.map(|s| s.tab(name)).unwrap_or(0);
    let key = format!("tab.{name}");
    let class = match (strip, vertical) {
        (false, _) => "nojs-tabs nojs-accordion",
        (true, false) => "nojs-tabs",
        (true, true) => "nojs-tabs nojs-tabs-vertical",
    };
    html! {
        div id=[state.map(|_| format!("nojs-tabs-{name}"))] data-nojs=[state.map(|_| "swap")]
            class=(class) style=[(strip && vertical).then(|| format!("--nojs-tabs-n: {}", tabs.len()))] {
            @if let (Some(s), true) = (state, select_below) {
                form method="get" action=(s.path()) class="nojs-tabs-select" {
                    @for (k, v) in s.entries() { @if k != key { input type="hidden" name=(k) value=(v); } }
                    select name=(key) aria-label="Tab" {
                        @for (i, t) in tabs.iter().enumerate() {
                            option value=(i) selected[i == active] { (t.title) @if let Some(n) = t.badge { " (" (n) ")" } }
                        }
                    }
                    button type="submit" { "Go" }
                }
            }
            @for (i, t) in tabs.iter().enumerate() {
                details name=(name) open[i == active] {
                    summary {
                        @match state {
                            Some(s) => a href=(s.link(&key, &i.to_string())) { (t.title) (badge(t)) },
                            None => { (t.title) (badge(t)) },
                        }
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

fn badge(t: &Tab) -> Markup {
    html! { @if let Some(n) = t.badge { " " span class="nojs-tabs-badge" { (n) } } }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-tabs:not(.nojs-accordion) { display: flex; flex-wrap: wrap; }
/* The strip's rule is drawn by the titles and by a stretching pseudo-element that fills the
   rest of their row, so it sits under the titles and not under the panels. */
.nojs-tabs:not(.nojs-accordion):not(.nojs-tabs-vertical)::after { content: ""; order: 0; flex: 1; border-bottom: 1px solid var(--nojs-line); }
.nojs-tabs:not(.nojs-accordion) summary {
  order: 0; position: relative; list-style: none; cursor: pointer; padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--nojs-line); color: var(--nojs-muted);
}
.nojs-tabs summary::-webkit-details-marker { display: none; }
/* The link fills the summary, so every click goes through the server (a click on bare summary
   padding would toggle natively and be undone by the next render). */
.nojs-tabs summary a { display: block; color: inherit; text-decoration: none; }
.nojs-tabs:not(.nojs-accordion) summary a { margin: -0.5rem -1rem; padding: 0.5rem 1rem; }
.nojs-tabs:not(.nojs-accordion) details[open] summary { color: var(--nojs-fg); }
/* The underline is its own empty element so the view transition moves a 2px bar, not the text. */
.nojs-tabs-mark { position: absolute; left: 0; right: 0; bottom: -1px; height: 2px; background: var(--nojs-accent); }
.nojs-tabs-badge {
  display: inline-block; min-width: 1.5em; padding: 0 0.4em; border-radius: 1em; text-align: center;
  font-size: 0.75em; font-weight: 600; line-height: 1.6; background: var(--nojs-line); color: var(--nojs-fg);
}
.nojs-tabs details[open] .nojs-tabs-badge { background: var(--nojs-accent); color: var(--nojs-on-accent); }
/* Push every panel to a full-width row under the strip. */
.nojs-tabs details::details-content { order: 1; flex-basis: 100%; }
.nojs-tabs .nojs-tabs-panel { order: 1; flex-basis: 100%; padding: 1rem 0; }
.nojs-tabs:not(.nojs-accordion) details { display: contents; }
/* Vertical: titles in the first column, the open panel spans every row of the second. The rule
   goes on ::details-content, the grid item, so it runs the full height; the padding stays on the
   panel (Blitz builds no ::details-content box, see FINDINGS). */
.nojs-tabs.nojs-tabs-vertical { display: grid; grid-template-columns: max-content 1fr; }
.nojs-tabs.nojs-tabs-vertical summary {
  grid-column: 1; border-bottom: 0; border-right: 1px solid var(--nojs-line); margin: 0 -1px 0 0;
}
.nojs-tabs.nojs-tabs-vertical .nojs-tabs-mark { left: auto; right: -1px; top: 0; bottom: 0; width: 2px; height: auto; }
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
        let render = || { calls.set(calls.get() + 1); html! { p { "Changelog body" } } };
        let strip = |query: &str| {
            let state = UiState::parse("/docs", query, "");
            tabs_with(&Caps::all(), "docs", &[Tab::new("Install", html! { "cargo add" }), Tab::lazy_with("Changelog", &render)],
                TabsOptions::default().state(&state)).into_string()
        };
        let closed = strip("");
        assert!(!closed.contains("Changelog body") && closed.contains("nojs-tabs-lazy"));
        assert_eq!(calls.get(), 0);
        let open = strip("tab.docs=1");
        assert!(open.contains("Changelog body") && !open.contains("nojs-tabs-lazy"));
        assert_eq!(calls.get(), 1);
    }
}
