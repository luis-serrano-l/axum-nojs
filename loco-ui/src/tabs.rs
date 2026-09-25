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
//!   (an empty `.lui-tabs-mark`, never the title, so no text moves): the raised chip slides to
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
//! use loco_ui::prelude::*;
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
//! assert!(html.contains("view-transition-name: lui-tabs-docs"));
//! assert!(html.contains("<select name=\"tab.docs\""));
//! // The same in `lui!`:
//! let same = lui! { Tabs("docs") vertical select_below {
//!     tab "Install" { p { "cargo add" } }
//!     tab "Use" badge=3 { p { "html!" } }
//!     lazy "Changelog" || { p { "(long)" } }
//! } };
//! assert_eq!(same.into_string(), html);
//! ```

use std::rc::Rc;

use maud::{Markup, Render, html};

use crate::i18n::Text;
use crate::props::{Prop, PropKind};
use crate::{Cap, Ui};

/// One tab: a title, a panel (ready or rendered on demand), an optional badge count.
#[derive(Clone, Debug)]
struct Tab<'a> {
    title: &'a str,
    body: Body<'a>,
    badge: Option<usize>,
}

/// A panel: rendered already, or rendered on demand.
#[derive(Clone)]
enum Body<'a> {
    Ready(Markup),
    Lazy(Rc<dyn Fn() -> Markup + 'a>),
}

/// A lazy panel prints as `Lazy(<fn>)`.
impl std::fmt::Debug for Body<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Body::Ready(m) => f.debug_tuple("Ready").field(m).finish(),
            Body::Lazy(_) => f.write_str("Lazy(<fn>)"),
        }
    }
}

/// A tab strip, made by [`Ui::tabs`]: the open tab is `?tab.<name>=i` (or the cookie's
/// memory of it), and each title links to its own. Horizontal unless told otherwise.
///
/// **Setters.** Values and items: `.lazy(..)`, `.tab(..)`, `.badge(..)`; switches:
/// `.vertical()`, `.select_below()`.
#[derive(Clone, Debug)]
pub struct Tabs<'a> {
    ui: &'a Ui,
    name: &'a str,
    tabs: Vec<Tab<'a>>,
    vertical: bool,
    select_below: bool,
}

impl Tabs<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("tab", PropKind::Item, "title: &'a str, body: Markup")
            .doc("A tab titled `title` with its panel."),
        Prop::new(
            "lazy",
            PropKind::Item,
            "title: &'a str, body: impl Fn() -> Markup + 'a",
        )
        .doc("A tab whose panel is rendered only when it is the open one."),
        Prop::new("badge", PropKind::Modifier, "count: usize")
            .doc("A count after the title of the tab added last."),
        Prop::new("vertical", PropKind::Switch, "")
            .doc("Titles in a column on the left, the open panel beside them."),
        Prop::new("select_below", PropKind::Switch, "")
            .doc("On screens under 40rem the titles give way to a `<select>`."),
    ];
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
            body: Body::Lazy(Rc::new(body)),
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
            (false, _) => "lui-tabs lui-accordion",
            (true, false) => "lui-tabs",
            (true, true) => "lui-tabs lui-tabs-vertical",
        };
        html! {
            div id={ "lui-tabs-" (name) } data-lui="swap"
                class=(class) style=[(strip && vertical).then(|| format!("--lui-tabs-n: {}", tabs.len()))] {
                @if select_below {
                    form method="get" action=(s.path()) class="lui-tabs-select" {
                        @for (k, v) in s.entries() { @if k != key { input type="hidden" name=(k) value=(v); } }
                        select name=(key) aria-label=(ui.text(Text::Tab)) {
                            @for (i, t) in tabs.iter().enumerate() {
                                option value=(i) selected[i == active] { (t.title) @if let Some(n) = t.badge { " (" (n) ")" } }
                            }
                        }
                        (ui.button(ui.text(Text::Go)))
                    }
                }
                @for (i, t) in tabs.iter().enumerate() {
                    details name=(name) open[i == active] {
                        summary {
                            a href=(s.link(&key, &i.to_string())) { (t.title) (badge(t)) }
                            @if i == active && strip {
                                span class="lui-tabs-mark" style=(format!("view-transition-name: lui-tabs-{name}")) {}
                            }
                        }
                        div class=(if strip { "lui-tabs-panel" } else { "lui-accordion-body" }) {
                            @match (&t.body, i == active) {
                                (Body::Ready(body), _) => (body),
                                (Body::Lazy(render), true) => (render()),
                                (Body::Lazy(_), false) => span class="lui-tabs-lazy" {},
                            }
                        }
                    }
                }
            }
        }
    }
}
fn badge(t: &Tab) -> Markup {
    html! { @if let Some(n) = t.badge { " " span class="lui-tabs-badge" { (n) } } }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-tabs:not(.lui-accordion) { display: flex; flex-wrap: wrap; }
/* Fills the rest of the titles' row so the panels start on the next one. */
.lui-tabs:not(.lui-accordion):not(.lui-tabs-vertical)::after { content: ""; order: 0; flex: 1; }
/* shadcn Tabs: the titles sit in a muted pill (secondary, 3px padding, rounded-lg); the open
   one is a raised chip on the page background. Each summary paints its slice of the pill. */
.lui-tabs:not(.lui-accordion) summary {
  order: 0; position: relative; list-style: none; cursor: pointer; padding: 3px;
  background: var(--lui-secondary); color: var(--lui-muted);
}
.lui-tabs:not(.lui-accordion):not(.lui-tabs-vertical) > details:first-of-type > summary { border-radius: var(--lui-radius) 0 0 var(--lui-radius); }
.lui-tabs:not(.lui-accordion):not(.lui-tabs-vertical) > details:last-of-type > summary { border-radius: 0 var(--lui-radius) var(--lui-radius) 0; }
.lui-tabs:not(.lui-accordion):not(.lui-tabs-vertical) > details:only-of-type > summary { border-radius: var(--lui-radius); }
.lui-tabs summary::-webkit-details-marker { display: none; }
/* The link fills the summary, so every click goes through the server (a click on bare summary
   padding would toggle natively and be undone by the next render). */
.lui-tabs summary a { display: block; color: inherit; text-decoration: none; }
.lui-tabs:not(.lui-accordion) summary a {
  position: relative; z-index: 1; display: flex; align-items: center; gap: 0.375rem;
  padding: 0.25rem 0.75rem; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500; white-space: nowrap;
}
.lui-tabs:not(.lui-accordion) summary a:hover { color: var(--lui-fg); }
.lui-tabs:not(.lui-accordion) details[open] summary { color: var(--lui-fg); }
/* The chip is its own empty element so the view transition slides it, not the text. */
.lui-tabs-mark {
  position: absolute; inset: 3px; border-radius: var(--lui-radius-sm);
  background: var(--lui-bg); box-shadow: var(--lui-shadow-xs);
}
.lui-tabs-badge {
  display: inline-block; min-width: 1.25rem; padding: 0 0.3rem; border-radius: 1em; text-align: center;
  font-size: 0.75rem; font-weight: 500; line-height: 1.25rem; background: var(--lui-line); color: var(--lui-fg);
}
.lui-tabs details[open] .lui-tabs-badge { background: var(--lui-primary); color: var(--lui-on-primary); }
/* Push every panel to a full-width row under the strip. */
.lui-tabs details::details-content { order: 1; flex-basis: 100%; }
.lui-tabs .lui-tabs-panel { order: 1; flex-basis: 100%; padding: 1rem 0; }
.lui-tabs:not(.lui-accordion) details { display: contents; }
/* Vertical: a plain list with a rule, the open title marked by a bar on the rule. The titles
   are in the first column, the open panel spans every row of the second. The rule goes on
   ::details-content, the grid item, so it runs the full height; the padding stays on the
   panel (Blitz builds no ::details-content box, see FINDINGS). */
.lui-tabs.lui-tabs-vertical { display: grid; grid-template-columns: max-content 1fr; }
.lui-tabs.lui-tabs-vertical summary {
  grid-column: 1; padding: 0; background: none; border-right: 1px solid var(--lui-line); margin: 0 -1px 0 0;
}
.lui-tabs.lui-tabs-vertical summary a { padding: 0.5rem 1rem; }
.lui-tabs.lui-tabs-vertical .lui-tabs-mark { inset: 0 -1px 0 auto; width: 2px; border-radius: 0; background: var(--lui-fg); box-shadow: none; }
.lui-tabs.lui-tabs-vertical details::details-content {
  grid-column: 2; grid-row: 1 / span var(--lui-tabs-n, 1); border-left: 1px solid var(--lui-line);
}
.lui-tabs.lui-tabs-vertical .lui-tabs-panel {
  grid-column: 2; grid-row: 1 / span var(--lui-tabs-n, 1); padding: 0 0 0 calc(var(--lui-space) * 3);
}
.lui-tabs.lui-tabs-vertical .lui-tabs-select { grid-column: 1 / -1; }
/* Narrow screens: the titles give way to the select (needs the enhancement script for
   submit-on-change; the Go button is always there). */
.lui-tabs-select { display: none; gap: var(--lui-space); flex-basis: 100%; margin-bottom: var(--lui-space); }
@media (max-width: 40rem) {
  .lui-tabs:not(.lui-accordion):has(.lui-tabs-select) summary, .lui-tabs:has(.lui-tabs-select)::after { display: none; }
  .lui-tabs:not(.lui-accordion) .lui-tabs-select { display: flex; }
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
        assert!(!closed.contains("Changelog body") && closed.contains("lui-tabs-lazy"));
        assert_eq!(calls.get(), 0);
        let open = strip("tab.docs=1");
        assert!(open.contains("Changelog body") && !open.contains("lui-tabs-lazy"));
        assert_eq!(calls.get(), 1);
    }
}
