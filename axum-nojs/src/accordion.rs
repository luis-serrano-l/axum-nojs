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
//! pattern).
//!
//! **Fallback:** `<details>` alone (baseline 2020) still toggles; only the exclusivity and the
//! animation are lost. No `Caps` branch is needed; the markup is the same everywhere.
//!
//! **Server persistence:** the open sections come from the request's `?open.<group>=0,2` (a
//! comma list, see [`crate::UiState::opens`]) and each title is a link that toggles its own index in
//! that list, so the choice survives navigation. "Expand all" links to every index, "Collapse
//! all" to `open.<group>=` (which removes the key). The group is a swap root for the
//! [`crate::enhance`] script. A nested accordion is its own group with its own key.
//!
//! **Without script:** nothing is lost.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! // Items 0 and 2 of "faq" were left open, as the URL records it.
//! let ui = Ui::from_request("/help", "open.faq=0,2", "");
//! // `icon` and `summary` apply to the item added last.
//! let m = ui.accordion("faq")
//!     .item("Install", html! { p { "cargo add" } }).icon("\u{1F4E6}").summary("One line.")
//!     .item("Use", html! { p { "html!" } })
//!     .item("More", html! { (ui.accordion("faq-more").item("Nested", html! { p { "Own group." } })) })
//!     .multi()
//!     .controls();
//! let html = m.render().into_string();
//! assert!(html.contains("href=\"/help?open.faq=2\">Install"), "open item's link removes itself from the list");
//! assert!(html.contains("href=\"/help?open.faq=0%2C1%2C2\">Expand all"));
//! assert!(html.contains("class=\"nojs-accordion-summary\">One line."));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;

/// One section: a title, a body, an optional icon before the title and summary line under it.
#[derive(Clone, Debug)]
struct Item<'a> {
    title: &'a str,
    body: Markup,
    icon: Option<&'a str>,
    summary: Option<&'a str>,
}

/// Stacked sections, made by [`Ui::accordion`]: the open ones are `?open.<group>=` (or the
/// cookie's memory of it), and each title links to toggle its own. One open at a time unless
/// [`Accordion::multi`].
#[derive(Clone, Debug)]
pub struct Accordion<'a> {
    ui: &'a Ui,
    group: &'a str,
    items: Vec<Item<'a>>,
    multi: bool,
    controls: bool,
}

impl Ui {
    /// An empty accordion named `group` (the key in `?open.<group>=`); add sections with
    /// [`Accordion::item`].
    pub fn accordion<'a>(&'a self, group: &'a str) -> Accordion<'a> {
        Accordion {
            ui: self,
            group,
            items: Vec::new(),
            multi: false,
            controls: false,
        }
    }
}

impl<'a> Accordion<'a> {
    /// A section titled `title` with its body.
    pub fn item(mut self, title: &'a str, body: Markup) -> Self {
        self.items.push(Item {
            title,
            body,
            icon: None,
            summary: None,
        });
        self
    }

    /// Text (an emoji or a glyph) before the title of the section added last, hidden from
    /// assistive tech.
    pub fn icon(mut self, icon: &'a str) -> Self {
        if let Some(it) = self.items.last_mut() {
            it.icon = Some(icon);
        }
        self
    }

    /// A muted line under the title of the section added last, visible while it is closed.
    pub fn summary(mut self, summary: &'a str) -> Self {
        if let Some(it) = self.items.last_mut() {
            it.summary = Some(summary);
        }
        self
    }

    /// Several sections may be open at once (`?open.<group>=0,2`).
    pub fn multi(mut self) -> Self {
        self.multi = true;
        self
    }

    /// "Expand all" and "Collapse all" links above the sections (with `multi`).
    pub fn controls(mut self) -> Self {
        self.controls = true;
        self
    }
}

impl Render for Accordion<'_> {
    fn render(&self) -> Markup {
        let Accordion {
            ui,
            group,
            ref items,
            multi,
            controls,
        } = *self;
        let s = &ui.state;
        let open: Vec<usize> = s.opens(group);
        let key = format!("open.{group}");
        let list = |ix: &[usize]| {
            ix.iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
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
            div id={ "nojs-accordion-" (group) } data-nojs="swap" class="nojs-accordion" {
                @if multi && controls {
                    p class="nojs-accordion-controls" {
                        a href=(s.link(&key, &list(&(0..items.len()).collect::<Vec<_>>()))) { "Expand all" }
                        a href=(s.link(&key, "")) { "Collapse all" }
                    }
                }
                @for (i, item) in items.iter().enumerate() {
                    details name=[(!multi).then_some(group)] open[open.contains(&i)] {
                        summary {
                            @if let Some(icon) = item.icon { span class="nojs-accordion-icon" aria-hidden="true" { (icon) } }
                            a href=(s.link(&key, &toggled(i))) {
                                (item.title)
                                @if let Some(line) = item.summary { span class="nojs-accordion-summary" { (line) } }
                            }
                        }
                        div class="nojs-accordion-body" { (item.body) }
                    }
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
/* shadcn Accordion: items divided by a bottom rule, text-sm font-medium triggers that
   underline on hover, a chevron on the right that turns when open. interpolate-size
   (Chrome 129) lets height animate to auto; elsewhere it snaps. */
.nojs-accordion { interpolate-size: allow-keywords; }
.nojs-accordion details { border-bottom: 1px solid var(--nojs-line); }
.nojs-accordion summary {
  display: flex; align-items: flex-start; gap: 0.5rem; list-style: none; cursor: pointer;
  padding: 1rem 0; font-size: 0.875rem; line-height: 1.25rem; font-weight: 500;
}
.nojs-accordion summary::-webkit-details-marker { display: none; }
/* The chevron is two borders of a rotated square in the muted colour. */
.nojs-accordion summary::after {
  content: ""; flex: none; order: 2; width: 0.45rem; height: 0.45rem; margin: 0.3rem 0.25rem 0 auto;
  border-right: 1.5px solid var(--nojs-muted); border-bottom: 1.5px solid var(--nojs-muted);
  rotate: 45deg; transition: rotate 0.2s;
}
.nojs-accordion details[open] > summary::after { rotate: 225deg; margin-top: 0.5rem; }
/* The link fills the rest of the summary so a click never toggles natively without the server. */
.nojs-accordion summary a, .nojs-accordion-title { flex: 1; margin: -1rem 0; padding: 1rem 0; color: inherit; text-decoration: none; }
.nojs-accordion summary a:hover { text-decoration: underline; }
.nojs-accordion-icon { font-size: 1.1em; line-height: 1; }
.nojs-accordion-summary { display: block; font-weight: 400; font-size: 0.875rem; color: var(--nojs-muted); margin-top: 0.15rem; }
.nojs-accordion details[open] > summary .nojs-accordion-summary { display: none; }
.nojs-accordion-body { padding: 0 0 1rem; font-size: 0.875rem; }
.nojs-accordion details::details-content { transition: height 0.2s, content-visibility 0.2s allow-discrete; height: 0; overflow: hidden; }
.nojs-accordion details[open]::details-content { height: auto; }
.nojs-accordion-controls { display: flex; gap: calc(var(--nojs-space) * 2); margin: 0; max-width: none; padding: 0 0 0.5rem; font-size: 0.875rem; border-bottom: 1px solid var(--nojs-line); }
/* A nested accordion sits inside a body, indented. */
.nojs-accordion .nojs-accordion { margin: 0.5rem 0 0 1rem; }
.nojs-accordion .nojs-accordion details:last-child { border-bottom: 0; }
.nojs-accordion .nojs-accordion summary { padding: 0.5rem 0; }
.nojs-accordion .nojs-accordion summary a, .nojs-accordion .nojs-accordion .nojs-accordion-title { margin: -0.5rem 0; padding: 0.5rem 0; }
.nojs-accordion .nojs-accordion .nojs-accordion-body { padding: 0 0 0.75rem; }
"#;
