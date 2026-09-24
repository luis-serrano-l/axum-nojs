//! # Accordion
//!
//! Stacked disclosure sections, no script. Exclusive by default (one open at a time), or
//! `multi` so several stay open, with "Expand all" and "Collapse all" links. An item can
//! carry a summary line under its title and an icon before it, and a body can hold another
//! accordion.
//!
//! **Platform features:** `<details name="group">` (baseline 2024) for exclusivity.
//! `::details-content` (Chrome 131, Firefox 143, Safari 18.4) plus
//! `interpolate-size: allow-keywords` (Chrome 129 only) animate the height between `0` and
//! `auto`; without `interpolate-size` the panel snaps.
//!
//! **What it does not do without script:** arrow keys between summaries (the WAI-ARIA accordion
//! pattern); remembering which section was open across reloads needs `UiState` or `?open=`.
//!
//! **Fallback:** `<details>` alone (baseline 2020) still toggles; only the exclusivity and the
//! animation are lost. No `Caps` branch is needed; the markup is the same everywhere.
//!
//! **Server persistence:** with a `UiState`, the open sections come from `state.opens(group)`
//! (`?open.<group>=0,2`, a comma list) and each title is a link that toggles its own index in
//! that list, so the choice survives navigation. "Expand all" links to every index, "Collapse
//! all" to `open.<group>=` (which removes the key). The group is then a swap root for the
//! [`crate::enhance`] script. A nested accordion is its own group with its own key.
//!
//! **Without script:** nothing is lost. Without a state the sections toggle natively and
//! remember nothing, and the expand/collapse links are not rendered (there is no link target).
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, UiState, accordion, accordion_with, accordion::{AccordionItem, AccordionOptions}};
//! let m = accordion(&Caps::all(), "faq", &[
//!     AccordionItem::new("What?", html! { p { "A" } }),
//!     AccordionItem::new("Why?", html! { p { "B" } }),
//! ]);
//!
//! let state = UiState::parse("/help", "open.faq=0,2", "");
//! let m = accordion_with(&Caps::all(), "faq", &[
//!     AccordionItem::new("Install", html! { p { "cargo add" } }).icon("\u{1F4E6}").summary("One line."),
//!     AccordionItem::new("Use", html! { p { "html!" } }),
//!     AccordionItem::new("More", accordion_with(&Caps::all(), "faq-more", &[AccordionItem::new("Nested", html! { p { "Own group." } })], AccordionOptions::default().state(&state))),
//! ], AccordionOptions::default().state(&state).multi(true).controls(true));
//! let html = m.into_string();
//! assert!(html.contains("href=\"/help?open.faq=2\">Install"), "open item's link removes itself from the list");
//! assert!(html.contains("href=\"/help?open.faq=0%2C1%2C2\">Expand all"));
//! assert!(html.contains("class=\"wo-accordion-summary\">One line."));
//! ```

use maud::{Markup, html};

use crate::{Caps, UiState};

/// One section: a title, a body, an optional icon before the title and summary line under it.
#[derive(Clone, Debug)]
pub struct AccordionItem<'a> {
    title: &'a str,
    body: Markup,
    icon: Option<&'a str>,
    summary: Option<&'a str>,
}

impl<'a> AccordionItem<'a> {
    /// A section with its title and body.
    pub const fn new(title: &'a str, body: Markup) -> Self {
        AccordionItem { title, body, icon: None, summary: None }
    }
    /// Text (an emoji or a glyph) shown before the title, hidden from assistive tech.
    pub const fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    /// A muted line under the title, visible while the section is closed.
    pub const fn summary(mut self, summary: &'a str) -> Self {
        self.summary = Some(summary);
        self
    }
}

/// Options for [`accordion`]; `Default::default()` is exclusive, unpersisted, without links.
#[derive(Clone, Copy, Debug, Default)]
pub struct AccordionOptions<'a> {
    /// Server-held open sections and links to change them.
    pub state: Option<&'a UiState>,
    /// Several sections may be open at once (no `name` attribute; `open.<group>` is a list).
    pub multi: bool,
    /// "Expand all" and "Collapse all" links above the sections (needs `state` and `multi`).
    pub controls: bool,
}

impl<'a> AccordionOptions<'a> {
    /// Server-held open sections and links to change them.
    pub fn state(mut self, state: &'a UiState) -> Self {
        self.state = Some(state);
        self
    }
    /// Let several sections stay open.
    pub fn multi(mut self, multi: bool) -> Self {
        self.multi = multi;
        self
    }
    /// Show the expand/collapse links.
    pub fn controls(mut self, controls: bool) -> Self {
        self.controls = controls;
        self
    }
}

/// An exclusive accordion with no state and the default options.
/// [`accordion_with`] takes the options.
pub fn accordion(caps: &Caps, group: &str, items: &[AccordionItem]) -> Markup {
    accordion_with(caps, group, items, Default::default())
}

/// Accordion `group`. Open sections are `state.opens(group)` (none without a state).
pub fn accordion_with(_caps: &Caps, group: &str, items: &[AccordionItem], options: AccordionOptions) -> Markup {
    let AccordionOptions { state, multi, controls } = options;
    let open: Vec<usize> = state.map(|s| s.opens(group)).unwrap_or_default();
    let key = format!("open.{group}");
    let list = |ix: &[usize]| ix.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
    let toggled = |i: usize| -> String {
        if open.contains(&i) {
            list(&open.iter().copied().filter(|&o| o != i).collect::<Vec<_>>())
        } else if multi {
            let mut all: Vec<usize> = open.iter().copied().chain([i]).collect();
            all.sort_unstable();
            list(&all)
        } else {
            i.to_string()
        }
    };
    html! {
        div id=[state.map(|_| format!("wo-accordion-{group}"))] data-wo=[state.map(|_| "swap")] class="wo-accordion" {
            @if let (Some(s), true, true) = (state, multi, controls) {
                p class="wo-accordion-controls" {
                    a href=(s.link(&key, &list(&(0..items.len()).collect::<Vec<_>>()))) { "Expand all" }
                    a href=(s.link(&key, "")) { "Collapse all" }
                }
            }
            @for (i, item) in items.iter().enumerate() {
                details name=[(!multi).then_some(group)] open[open.contains(&i)] {
                    summary {
                        @if let Some(icon) = item.icon { span class="wo-accordion-icon" aria-hidden="true" { (icon) } }
                        @let inner = html! {
                            (item.title)
                            @if let Some(line) = item.summary { span class="wo-accordion-summary" { (line) } }
                        };
                        @match state {
                            Some(s) => a href=(s.link(&key, &toggled(i))) { (inner) },
                            None => span class="wo-accordion-title" { (inner) },
                        }
                    }
                    div class="wo-accordion-body" { (item.body) }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
/* interpolate-size (Chrome 129) lets height animate to auto; elsewhere it snaps. */
.wo-accordion { border: 1px solid var(--wo-line); border-radius: var(--wo-radius); overflow: hidden; interpolate-size: allow-keywords; }
.wo-accordion details + details { border-top: 1px solid var(--wo-line); }
.wo-accordion summary { display: flex; align-items: center; gap: 0.5rem; list-style: none; cursor: pointer; padding: 0.75rem 1rem; font-weight: 600; }
.wo-accordion summary::-webkit-details-marker { display: none; }
.wo-accordion summary::before { content: "\25B8"; color: var(--wo-muted); }
.wo-accordion details[open] > summary::before { content: "\25BE"; }
.wo-accordion summary:hover { background: var(--wo-surface); }
/* The link fills the rest of the summary so a click never toggles natively without the server. */
.wo-accordion summary a, .wo-accordion-title { flex: 1; margin: -0.75rem -1rem -0.75rem 0; padding: 0.75rem 1rem 0.75rem 0; color: inherit; text-decoration: none; }
.wo-accordion-icon { font-size: 1.1em; line-height: 1; }
.wo-accordion-summary { display: block; font-weight: 400; font-size: 0.875rem; color: var(--wo-muted); margin-top: 0.15rem; }
.wo-accordion details[open] > summary .wo-accordion-summary { display: none; }
.wo-accordion-body { padding: 0 1rem 1rem; }
.wo-accordion details::details-content { transition: height 0.2s, content-visibility 0.2s allow-discrete; height: 0; overflow: hidden; }
.wo-accordion details[open]::details-content { height: auto; }
.wo-accordion-controls { display: flex; gap: calc(var(--wo-space) * 2); margin: 0; max-width: none; padding: 0.5rem 1rem; font-size: 0.875rem; border-bottom: 1px solid var(--wo-line); background: var(--wo-surface); }
/* A nested accordion sits inside a body: lighter, indented by the body's own padding. */
.wo-accordion .wo-accordion { margin-top: 0.5rem; }
.wo-accordion .wo-accordion summary { padding: 0.5rem 0.75rem; font-weight: 500; }
.wo-accordion .wo-accordion summary a, .wo-accordion .wo-accordion .wo-accordion-title { margin: -0.5rem -0.75rem -0.5rem 0; padding: 0.5rem 0.75rem 0.5rem 0; }
.wo-accordion .wo-accordion .wo-accordion-body { padding: 0 0.75rem 0.75rem; }
"#;
