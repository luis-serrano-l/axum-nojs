//! # Marquee
//!
//! A row that scrolls sideways on its own and loops: customer logos, a line of testimonials,
//! the tags of a changelog. It stops while the pointer is over it or focus is inside it, so a
//! link in it can be read and followed.
//!
//! **Platform features:** a `<ul>` of the items, then the same list again with
//! `aria-hidden="true"` and `inert` (Chrome 102, Firefox 112, Safari 15.5), so the copy is
//! neither read nor focused. A `@keyframes` animation moves the pair with the `translate`
//! property (Chrome 104, Firefox 72, Safari 14.1) by exactly one list and a gap, so the copy
//! lands where the first list began and the loop has no seam; `animation-play-state` pauses it
//! on `:hover` and `:focus-within`. The edges fade out with `mask-image` (Chrome 120, Firefox
//! 53, Safari 15.4). The pace is `--lui-marquee-duration` (40s), or `.duration(..)`.
//!
//! **Accessibility:** a `role="region"` named by the label; the items are read once, in order,
//! since the copy is `aria-hidden` and `inert`; the motion stops on hover and focus, and does
//! not start under `prefers-reduced-motion: reduce`. Checked by axe-core in headless Firefox
//! on every demo route, both capability variants, light and dark (no serious or critical
//! violation).
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** the motion sits inside `@supports (translate: 100%)` and
//! `@media (prefers-reduced-motion: no-preference)`. Without either, the row is at rest: the
//! items wrap onto as many lines as they need and the copy is not shown.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.marquee("Customers").text("Acme").text("Globex").render().into_string();
//! assert!(m.starts_with(r#"<div class="lui-marquee" role="region" aria-label="Customers">"#));
//! assert!(m.contains(r#"<ul class="lui-marquee-group" aria-hidden="true" inert>"#));
//! assert_eq!(m.matches("<li class=\"lui-marquee-item\">Acme</li>").count(), 2);
//! // Markup items, the other way round, at its own pace.
//! let quotes = ui.marquee("What people say")
//!     .item(html! { blockquote { "Fast." } })
//!     .item(html! { blockquote { "No script!" } })
//!     .reverse()
//!     .duration(20);
//! let quotes = quotes.render().into_string();
//! assert!(quotes.contains("lui-marquee lui-marquee-reverse") && quotes.contains("--lui-marquee-duration: 20s"));
//! // The same in `lui!`:
//! let same = lui! {
//!     Marquee("What people say") reverse duration=20 {
//!         item() { blockquote { "Fast." } }
//!         item() { blockquote { "No script!" } }
//!     }
//! };
//! assert_eq!(same.into_string(), quotes);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// A looping row, made by [`Ui::marquee`].
///
/// **Setters.** Values and items: `.text(..)`, `.item(..)`, `.duration(..)`; switches:
/// `.reverse()`.
#[derive(Clone, Debug)]
pub struct Marquee<'a> {
    label: &'a str,
    items: Vec<Markup>,
    reverse: bool,
    duration: Option<u32>,
}

impl Marquee<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("text", PropKind::Item, "text: &'a str").doc("One item of plain text."),
        Prop::new("item", PropKind::Item, "markup: Markup")
            .doc("One item of markup: a logo, a quote, a card."),
        Prop::new("reverse", PropKind::Switch, "")
            .doc("Scroll towards the end of the line instead of the start."),
        Prop::new("duration", PropKind::Number, "seconds: u32")
            .default("40")
            .attr("style")
            .doc("Seconds for one full loop (`--lui-marquee-duration`)."),
    ];
}

impl Ui {
    /// An empty looping row named `label` for screen readers; add items with `.text(..)` or
    /// `.item(..)`.
    pub fn marquee<'a>(&self, label: &'a str) -> Marquee<'a> {
        Marquee {
            label,
            items: Vec::new(),
            reverse: false,
            duration: None,
        }
    }
}

impl<'a> Marquee<'a> {
    /// One item of plain text.
    pub fn text(mut self, text: &'a str) -> Self {
        self.items.push(html! { (text) });
        self
    }

    /// One item of markup: a logo, a quote, a card.
    pub fn item(mut self, markup: Markup) -> Self {
        self.items.push(markup);
        self
    }

    /// Scroll towards the end of the line instead of the start.
    pub fn reverse(mut self) -> Self {
        self.reverse = true;
        self
    }

    /// Seconds for one full loop (`--lui-marquee-duration`, 40 by default): fewer is faster.
    pub fn duration(mut self, seconds: u32) -> Self {
        self.duration = Some(seconds);
        self
    }
}

impl Render for Marquee<'_> {
    fn render(&self) -> Markup {
        let group = |copy: bool| {
            html! {
                ul class="lui-marquee-group" aria-hidden=[copy.then_some("true")] inert[copy] {
                    @for item in &self.items { li class="lui-marquee-item" { (item) } }
                }
            }
        };
        let style = self
            .duration
            .map(|s| format!("--lui-marquee-duration: {s}s"));
        html! {
            div class={ "lui-marquee" @if self.reverse { " lui-marquee-reverse" } } role="region"
                aria-label=(self.label) style=[style] {
                div class="lui-marquee-track" { (group(false)) (group(true)) }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. At rest by default (the items
/// wrap, the copy hidden); the loop is switched on only where it can run and is wanted.
pub const CSS: &str = r#"
.lui-marquee { overflow: hidden; padding-block: var(--lui-space-1); }
.lui-marquee-track { display: flex; gap: var(--lui-space-4); }
.lui-marquee-group { display: flex; flex: 1; flex-wrap: wrap; align-items: center; gap: var(--lui-space-4); margin: 0; padding: 0; list-style: none; }
.lui-marquee-group[aria-hidden=true] { display: none; }
.lui-marquee-item { flex: none; }
.lui-marquee-item > :first-child { margin-top: 0; }
.lui-marquee-item > :last-child { margin-bottom: 0; }
@media (prefers-reduced-motion: no-preference) {
  @supports (translate: 100%) {
    /* One list and one gap is half the track less half a gap: the copy lands where the list began. */
    .lui-marquee-track { width: max-content; animation: lui-marquee var(--lui-marquee-duration) linear infinite; }
    .lui-marquee-group { flex: none; flex-wrap: nowrap; }
    .lui-marquee-group[aria-hidden=true] { display: flex; }
    .lui-marquee:is(:hover, :focus-within) .lui-marquee-track { animation-play-state: paused; }
    .lui-marquee-reverse .lui-marquee-track { animation-direction: reverse; }
    @supports (mask-image: linear-gradient(transparent, transparent)) {
      .lui-marquee { mask-image: linear-gradient(to right, transparent, var(--lui-fg) 3rem, var(--lui-fg) calc(100% - 3rem), transparent); }
    }
  }
}
@keyframes lui-marquee { to { translate: calc(-50% - var(--lui-space-4) / 2) 0; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_copy_is_hidden_and_inert_and_holds_the_same_items() {
        let ui = Ui::default();
        let m = ui
            .marquee("Logos")
            .text("A")
            .text("B")
            .render()
            .into_string();
        let (first, copy) = m.split_once("</ul>").unwrap();
        assert!(!first.contains("aria-hidden") && !first.contains("inert"));
        assert!(copy.contains(r#"aria-hidden="true" inert"#));
        let items = |s: &str| s.matches("lui-marquee-item").count();
        assert_eq!((items(first), items(copy)), (2, 2));
        assert!(!m.contains("style="), "no duration, no inline style");
    }
}
