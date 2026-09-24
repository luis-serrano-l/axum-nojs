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
//!   (an empty `.wo-tabs-mark`, never the title, so no text moves): the accent bar slides to
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
//! Because every switch is a request, a lazy tab ([`Tab::lazy`]) costs nothing until opened:
//! render its body only when `state.tab(name)` is its index.
//!
//! **Without script:** the narrow-screen `<select>` needs its "Go" button; the script submits
//! it on change. Without a state the tabs toggle natively and remember nothing.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, UiState, tabs, tabs::{Tab, TabsOptions}};
//! let m = tabs(&Caps::all(), "t", &[Tab::new("One", html! { p { "First." } }), Tab::new("Two", html! { p { "Second." } })], Default::default());
//!
//! let state = UiState::parse("/docs", "tab.docs=1", "");
//! let open = state.tab("docs");
//! let m = tabs(&Caps::all(), "docs", &[
//!     Tab::new("Install", html! { p { "cargo add" } }),
//!     Tab::new("Use", html! { p { "html!" } }).badge(3),
//!     if open == 2 { Tab::new("Changelog", html! { p { "(long)" } }) } else { Tab::lazy("Changelog") },
//! ], TabsOptions::default().state(&state).vertical(true).select_below(true));
//! let html = m.into_string();
//! assert!(html.contains("href=\"/docs?tab.docs=0\""));
//! assert!(html.contains("view-transition-name: wo-tabs-docs"));
//! assert!(html.contains("<select name=\"tab.docs\""));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps, UiState};

/// One tab: a title, a body (or none for a lazy tab), an optional badge count.
#[derive(Clone, Debug)]
pub struct Tab<'a> {
    title: &'a str,
    body: Option<Markup>,
    badge: Option<usize>,
}

impl<'a> Tab<'a> {
    /// A tab with its panel.
    pub const fn new(title: &'a str, body: Markup) -> Self {
        Tab { title, body: Some(body), badge: None }
    }
    /// A tab whose panel is not rendered: use it for the tabs that are not open, and render
    /// the open one with [`Tab::new`]. The panel is filled by the round trip that opens it.
    pub const fn lazy(title: &'a str) -> Self {
        Tab { title, body: None, badge: None }
    }
    /// A count shown after the title.
    pub const fn badge(mut self, count: usize) -> Self {
        self.badge = Some(count);
        self
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
    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }
    /// Collapse to a `<select>` on narrow screens.
    pub fn select_below(mut self, select: bool) -> Self {
        self.select_below = select;
        self
    }
}

/// Tab strip `name`. The open panel is `state.tab(name)`, or the first one without state.
pub fn tabs(caps: &Caps, name: &str, tabs: &[Tab], options: TabsOptions) -> Markup {
    let TabsOptions { state, vertical, select_below } = options;
    let strip = caps.has(Cap::DetailsContent);
    let active = state.map(|s| s.tab(name)).unwrap_or(0);
    let key = format!("tab.{name}");
    let class = match (strip, vertical) {
        (false, _) => "wo-tabs wo-accordion",
        (true, false) => "wo-tabs",
        (true, true) => "wo-tabs wo-tabs-vertical",
    };
    html! {
        div id=[state.map(|_| format!("wo-tabs-{name}"))] data-wo=[state.map(|_| "swap")]
            class=(class) style=[(strip && vertical).then(|| format!("--wo-tabs-n: {}", tabs.len()))] {
            @if let (Some(s), true) = (state, select_below) {
                form method="get" action=(s.path()) class="wo-tabs-select" {
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
                            span class="wo-tabs-mark" style=(format!("view-transition-name: wo-tabs-{name}")) {}
                        }
                    }
                    div class=(if strip { "wo-tabs-panel" } else { "wo-accordion-body" }) {
                        @if let Some(body) = &t.body { (body) } @else { span class="wo-tabs-lazy" {} }
                    }
                }
            }
        }
    }
}

fn badge(t: &Tab) -> Markup {
    html! { @if let Some(n) = t.badge { " " span class="wo-tabs-badge" { (n) } } }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-tabs:not(.wo-accordion) { display: flex; flex-wrap: wrap; }
/* The strip's rule is drawn by the titles and by a stretching pseudo-element that fills the
   rest of their row, so it sits under the titles and not under the panels. */
.wo-tabs:not(.wo-accordion):not(.wo-tabs-vertical)::after { content: ""; order: 0; flex: 1; border-bottom: 1px solid var(--wo-line); }
.wo-tabs:not(.wo-accordion) summary {
  order: 0; position: relative; list-style: none; cursor: pointer; padding: 0.5rem 1rem;
  border-bottom: 1px solid var(--wo-line); color: var(--wo-muted);
}
.wo-tabs summary::-webkit-details-marker { display: none; }
/* The link fills the summary, so every click goes through the server (a click on bare summary
   padding would toggle natively and be undone by the next render). */
.wo-tabs summary a { display: block; color: inherit; text-decoration: none; }
.wo-tabs:not(.wo-accordion) summary a { margin: -0.5rem -1rem; padding: 0.5rem 1rem; }
.wo-tabs:not(.wo-accordion) details[open] summary { color: var(--wo-fg); }
/* The underline is its own empty element so the view transition moves a 2px bar, not the text. */
.wo-tabs-mark { position: absolute; left: 0; right: 0; bottom: -1px; height: 2px; background: var(--wo-accent); }
.wo-tabs-badge {
  display: inline-block; min-width: 1.5em; padding: 0 0.4em; border-radius: 1em; text-align: center;
  font-size: 0.75em; font-weight: 600; line-height: 1.6; background: var(--wo-line); color: var(--wo-fg);
}
.wo-tabs details[open] .wo-tabs-badge { background: var(--wo-accent); color: var(--wo-on-accent); }
/* Push every panel to a full-width row under the strip. */
.wo-tabs details::details-content { order: 1; flex-basis: 100%; }
.wo-tabs .wo-tabs-panel { order: 1; flex-basis: 100%; padding: 1rem 0; }
.wo-tabs:not(.wo-accordion) details { display: contents; }
/* Vertical: titles in the first column, the open panel spans every row of the second. The rule
   goes on ::details-content, the grid item, so it runs the full height; the padding stays on the
   panel (Blitz builds no ::details-content box, see FINDINGS). */
.wo-tabs.wo-tabs-vertical { display: grid; grid-template-columns: max-content 1fr; }
.wo-tabs.wo-tabs-vertical summary {
  grid-column: 1; border-bottom: 0; border-right: 1px solid var(--wo-line); margin: 0 -1px 0 0;
}
.wo-tabs.wo-tabs-vertical .wo-tabs-mark { left: auto; right: -1px; top: 0; bottom: 0; width: 2px; height: auto; }
.wo-tabs.wo-tabs-vertical details::details-content {
  grid-column: 2; grid-row: 1 / span var(--wo-tabs-n, 1); border-left: 1px solid var(--wo-line);
}
.wo-tabs.wo-tabs-vertical .wo-tabs-panel {
  grid-column: 2; grid-row: 1 / span var(--wo-tabs-n, 1); padding: 0 0 0 calc(var(--wo-space) * 3);
}
.wo-tabs.wo-tabs-vertical .wo-tabs-select { grid-column: 1 / -1; }
/* Narrow screens: the titles give way to the select (needs the enhancement script for
   submit-on-change; the Go button is always there). */
.wo-tabs-select { display: none; gap: var(--wo-space); flex-basis: 100%; margin-bottom: var(--wo-space); }
@media (max-width: 40rem) {
  .wo-tabs:not(.wo-accordion):has(.wo-tabs-select) summary, .wo-tabs:has(.wo-tabs-select)::after { display: none; }
  .wo-tabs:not(.wo-accordion) .wo-tabs-select { display: flex; }
}
"#;
