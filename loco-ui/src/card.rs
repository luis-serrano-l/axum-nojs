//! # Card
//!
//! A bordered box with a header (title, description, and anything else such as an action),
//! a body and a footer: shadcn's Card. Settings panels, pricing tiers, sign-in boxes.
//!
//! **Platform features:** plain `<div>`s and a heading; the header is a grid, so an action
//! put in `.header(..)` sits at the top right beside the title. Four opt-in showpiece
//! effects, CSS only, each off under `prefers-reduced-motion: reduce` where it moves:
//! - `.beam()`: a light running round the border, a `conic-gradient` (Chrome 69, Firefox 83,
//!   Safari 12.1) whose start angle is an `@property` (Chrome 85, Firefox 128, Safari 16.4)
//!   animated by `@keyframes`, cut to a 1px ring with `mask-composite: exclude` (Chrome 120,
//!   Firefox 53, Safari 15.4), in `--lui-beam`, once round every `--lui-beam-duration`.
//! - `.glow()`: a spotlight at the top centre, `--lui-glow` in a `radial-gradient`, that
//!   brightens and grows on `:hover`; it does not follow the pointer. Behind
//!   `@supports` for `color-mix()` (Chrome 111, Firefox 113, Safari 16.2).
//! - `.gradient_border()`: the border drawn with `--lui-gradient-ring`, a padding-box layer
//!   over a border-box one, behind `@supports` for `linear-gradient(in oklch, ..)`
//!   (Chrome 111, Firefox 127, Safari 16.2).
//! - `.reveal()`: fades and rises as it scrolls into view, a scroll-driven animation with
//!   `animation-timeline: view()` (Chrome 115, Firefox no, Safari 26).
//!
//! **Accessibility:** a `<div>` of ordinary markup; its title is an `<h3>`, so it shows in the
//! outline. Checked by axe-core in headless Firefox on every
//! demo route, both capability variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed. Each effect sits inside `@supports` for the feature it needs,
//! and in `@media (prefers-reduced-motion: no-preference)` when it moves, so a browser without
//! it, or a visitor who asked for less motion, gets the card at rest: a plain border, no beam,
//! no glow, and shown in place from the start (a `.reveal()` card is never left hidden).
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.card().title("Team").body(html! { p { "3 members" } }).render().into_string();
//! assert!(m.contains(r#"<h3 class="lui-card-title">Team</h3>"#) && m.contains("3 members"));
//! // The same in `lui!`:
//! let same = lui! { Card title="Team" { p { "3 members" } } };
//! assert_eq!(same.into_string(), m);
//! // A description, an action in the header, and a footer of buttons.
//! let m = ui.card()
//!     .title("Plan")
//!     .description("Billed monthly.")
//!     .header(html! { (ui.badge("Current").secondary()) })
//!     .body(html! { p { "Pro, 12 seats" } })
//!     .footer(html! { (ui.button("Change plan").primary()) });
//! let m = m.render().into_string();
//! assert!(m.contains("lui-card-description") && m.contains("lui-card-action") && m.contains("lui-card-footer"));
//! // Opt-in showpieces, one class each on the root.
//! let m = ui.card().title("Pro").beam().glow().gradient_border().reveal().render().into_string();
//! assert!(m.starts_with(r#"<div class="lui-card lui-card-beam lui-card-glow lui-card-gradient-border lui-card-reveal">"#));
//! assert_eq!(lui! { Card title="Pro" beam glow gradient_border reveal; }.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// A card, made by [`Ui::card`].
///
/// **Setters.** Values and items: `.title(..)`, `.description(..)`, `.header(..)`, `.body(..)`,
/// `.footer(..)`, `.id(..)`; switches: `.beam()`, `.glow()`, `.gradient_border()`, `.reveal()`.
#[derive(Clone, Debug, Default)]
pub struct Card<'a> {
    title: Option<&'a str>,
    description: Option<&'a str>,
    header: Option<Markup>,
    body: Option<Markup>,
    footer: Option<Markup>,
    id: Option<&'a str>,
    beam: bool,
    glow: bool,
    gradient_border: bool,
    reveal: bool,
}

impl Card<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("title", PropKind::Value, "title: &'a str").doc("The heading at the top (`h3`)."),
        Prop::new("description", PropKind::Value, "text: &'a str")
            .doc("Muted text under the title."),
        Prop::new("header", PropKind::Value, "markup: Markup")
            .doc("More header content, placed at the top right beside the title."),
        Prop::new("body", PropKind::Value, "markup: Markup").doc("The main content."),
        Prop::new("footer", PropKind::Value, "markup: Markup")
            .doc("A row at the bottom, usually buttons."),
        Prop::new("id", PropKind::Value, "id: &'a str")
            .attr("id")
            .doc("The root's id, for a link to the card or a swap target."),
        Prop::new("beam", PropKind::Switch, "")
            .doc("A light runs round the border, drawn with a conic gradient."),
        Prop::new("glow", PropKind::Switch, "")
            .doc("A spotlight at the top centre that brightens and grows on hover."),
        Prop::new("gradient_border", PropKind::Switch, "")
            .doc("The border drawn with `--lui-gradient-ring`."),
        Prop::new("reveal", PropKind::Switch, "").doc("Fades and rises as it scrolls into view."),
    ];
}

impl Ui {
    /// An empty card; give it a `.title()`, a `.body()` and so on.
    pub fn card<'a>(&self) -> Card<'a> {
        Card::default()
    }
}

impl<'a> Card<'a> {
    /// The heading at the top (`h3`).
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Muted text under the title.
    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }

    /// More header content, placed at the top right beside the title: a badge, a menu, a
    /// link.
    pub fn header(mut self, markup: Markup) -> Self {
        self.header = Some(markup);
        self
    }

    /// The main content.
    pub fn body(mut self, markup: Markup) -> Self {
        self.body = Some(markup);
        self
    }

    /// A row at the bottom, usually buttons.
    pub fn footer(mut self, markup: Markup) -> Self {
        self.footer = Some(markup);
        self
    }

    /// The root's id, for a link to the card or a swap target.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// A light runs round the border, drawn with a conic gradient whose angle is an
    /// `@property`: the one card a page wants noticed. At rest without `mask-composite` or
    /// under `prefers-reduced-motion: reduce`.
    pub fn beam(mut self) -> Self {
        self.beam = true;
        self
    }

    /// A spotlight at the top centre that brightens and grows on hover. It stays where it is:
    /// following the pointer would need script.
    pub fn glow(mut self) -> Self {
        self.glow = true;
        self
    }

    /// The border drawn with `--lui-gradient-ring`; a plain border without `in oklch`
    /// gradients.
    pub fn gradient_border(mut self) -> Self {
        self.gradient_border = true;
        self
    }

    /// Fades and rises as it scrolls into view, with `animation-timeline: view()`; shown in
    /// place where that is missing or under `prefers-reduced-motion: reduce`.
    pub fn reveal(mut self) -> Self {
        self.reveal = true;
        self
    }
}

impl Render for Card<'_> {
    fn render(&self) -> Markup {
        let has_header =
            self.title.is_some() || self.description.is_some() || self.header.is_some();
        let mut class = String::from("lui-card");
        for (on, name) in [
            (self.beam, " lui-card-beam"),
            (self.glow, " lui-card-glow"),
            (self.gradient_border, " lui-card-gradient-border"),
            (self.reveal, " lui-card-reveal"),
        ] {
            if on {
                class.push_str(name);
            }
        }
        html! {
            div class=(class) id=[self.id] {
                @if has_header {
                    div class="lui-card-header" {
                        @if let Some(t) = self.title { h3 class="lui-card-title" { (t) } }
                        @if let Some(d) = self.description { p class="lui-card-description" { (d) } }
                        @if let Some(h) = &self.header { div class="lui-card-action" { (h) } }
                    }
                }
                @if let Some(b) = &self.body { div class="lui-card-body" { (b) } }
                @if let Some(f) = &self.footer { div class="lui-card-footer" { (f) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn: py-6, gap-6, px-6.
pub const CSS: &str = r#"
.lui-card {
  display: flex; flex-direction: column; gap: 1.5rem; padding-block: 1.5rem;
  background: var(--lui-card); color: var(--lui-fg);
  border: 1px solid var(--lui-line); border-radius: var(--lui-radius-lg); box-shadow: var(--lui-shadow-xs);
}
.lui-card-header { display: grid; grid-template-columns: 1fr auto; row-gap: 0.375rem; column-gap: 1rem; padding-inline: 1.5rem; }
.lui-card-header > :not(.lui-card-action) { grid-column: 1; }
.lui-card-title { margin: 0; font-size: 1rem; line-height: 1.25; font-weight: 600; }
.lui-card-description { margin: 0; color: var(--lui-muted); font-size: 0.875rem; }
.lui-card-action { grid-column: 2; grid-row: 1 / span 2; align-self: start; justify-self: end; }
.lui-card-body { padding-inline: 1.5rem; }
.lui-card-body > :first-child { margin-top: 0; }
.lui-card-body > :last-child { margin-bottom: 0; }
.lui-card-footer { display: flex; align-items: center; flex-wrap: wrap; gap: 0.5rem; padding-inline: 1.5rem; }
/* Showpieces, each opt-in by its setter and each at rest without its feature. */
.lui-card:is(.lui-card-beam, .lui-card-glow) { position: relative; isolation: isolate; }
/* .beam(): a conic gradient on ::before, cut to the 1px border ring by the mask, its start
   angle a registered property so the keyframes can turn it. */
@property --lui-beam-angle { syntax: "<angle>"; inherits: false; initial-value: 0deg; }
@media (prefers-reduced-motion: no-preference) {
  @supports (mask-composite: exclude) {
    .lui-card-beam::before {
      content: ""; position: absolute; inset: -1px; z-index: 1; pointer-events: none;
      padding: 1px; border-radius: inherit;
      background: conic-gradient(from var(--lui-beam-angle, 0deg), transparent 0 70%, var(--lui-beam) 88%, transparent 96%);
      mask: linear-gradient(var(--lui-fg) 0 0) content-box, linear-gradient(var(--lui-fg) 0 0);
      mask-composite: exclude;
      animation: lui-beam var(--lui-beam-duration) linear infinite;
    }
  }
}
@keyframes lui-beam { to { --lui-beam-angle: 1turn; } }
/* .glow(): a radial light behind the content, centred on the top edge; hover makes it brighter
   and larger. It does not follow the pointer. */
@supports (color: color-mix(in srgb, currentColor, transparent)) {
  .lui-card-glow::after {
    content: ""; position: absolute; inset: 0; z-index: -1; pointer-events: none; border-radius: inherit;
    background: radial-gradient(closest-side, var(--lui-glow), transparent) no-repeat center top -6rem / 70% 12rem;
    opacity: 0.6;
  }
  .lui-card-glow:hover::after { opacity: 1; background-size: 100% 20rem; background-position: center top -10rem; }
  @media (prefers-reduced-motion: no-preference) {
    .lui-card-glow::after { transition: opacity 0.4s, background-size 0.4s, background-position 0.4s; }
  }
}
/* .gradient_border(): the card's fill on the padding box over the ring gradient on the border box. */
@supports (background: linear-gradient(in oklch, currentColor, transparent)) {
  .lui-card.lui-card-gradient-border {
    border-color: transparent;
    background: linear-gradient(var(--lui-card), var(--lui-card)) padding-box, var(--lui-gradient-ring) border-box;
  }
}
/* .reveal(): fade and rise over the first part of the card's way into the viewport. The
   keyframes are shared with the stat tile's .reveal(). */
@media (prefers-reduced-motion: no-preference) {
  @supports (animation-timeline: view()) {
    .lui-card-reveal { animation: lui-reveal linear both; animation-timeline: view(); animation-range: entry 0% cover 30%; }
  }
}
@keyframes lui-reveal { from { opacity: 0; translate: 0 1.5rem; } }
"#;
