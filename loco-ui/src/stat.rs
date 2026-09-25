//! # Stat
//!
//! One number that matters, with its label and how it moved: the tiles at the top of a
//! dashboard. Several sit in a `div.lui-stat-grid`, which wraps them to the width available.
//!
//! **Platform features:** a card of `<p>` elements that reads in order; the change
//! carries its direction in words for screen readers (`<span class="lui-sr">`) as well as an
//! arrow and a colour; the grid is `repeat(auto-fit, minmax(12rem, 1fr))`, so no media query.
//! `.reveal()` fades and rises the tile as it scrolls into view, as a card's does: a
//! scroll-driven animation with `animation-timeline: view()` (Chrome 115, Firefox no,
//! Safari 26), off under `prefers-reduced-motion: reduce`.
//!
//! **Accessibility:** a label and a value in text; trend arrows are `aria-hidden` and the
//! change is in words. Checked by axe-core in headless Firefox on every demo route, both
//! capability variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** update live; the number is as fresh as the page.
//!
//! **Fallback:** none needed. A `.reveal()` tile where `animation-timeline: view()` is missing,
//! or under `prefers-reduced-motion: reduce`, is shown in place from the start.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from(Caps::all());
//! assert!(ui.stat("Visitors", "12,480").render().into_string().contains("12,480"));
//! // The sign of the delta says the direction: `+` up, `-` down, zero flat.
//! let m = ui.stat("Error rate", "0.4%")
//!     .delta("-0.2 pt")
//!     .down_is_good()
//!     .note("last 7 days")
//!     .href("/errors");
//! let m = m.render().into_string();
//! assert!(m.contains("lui-stat-good") && m.contains("down") && m.contains(r#"href="/errors""#));
//! // The same in `lui!`:
//! let same = lui! {
//!     Stat("Error rate", "0.4%") delta="-0.2 pt" down_is_good note="last 7 days" href="/errors";
//! };
//! assert_eq!(same.into_string(), m);
//! // Opt-in motion: it fades in as it scrolls into view.
//! let up = ui.stat("Visitors", "12,480").reveal().render().into_string();
//! assert!(up.starts_with(r#"<div class="lui-stat lui-stat-reveal">"#));
//! assert_eq!(lui! { Stat("Visitors", "12,480") reveal; }.into_string(), up);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// Which way the number moved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Trend {
    /// Went up.
    Up,
    /// Went down.
    Down,
    /// No change.
    #[default]
    Flat,
}

/// A stat card, `label` above `value`, made by [`Ui::stat`].
///
/// **Setters.** Values and items: `.delta(..)`, `.trend(..)`, `.note(..)`, `.href(..)`;
/// switches: `.down_is_good()`, `.reveal()`.
#[derive(Clone, Debug, Default)]
pub struct Stat<'a> {
    label: &'a str,
    value: &'a str,
    delta: Option<&'a str>,
    trend: Option<Trend>,
    down_is_good: bool,
    note: Option<&'a str>,
    href: Option<&'a str>,
    reveal: bool,
}

impl Stat<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("delta", PropKind::Value, "text: &'a str").doc("The change."),
        Prop::new("trend", PropKind::Value, "trend: Trend")
            .doc("Override the direction read from the delta's sign."),
        Prop::new("down_is_good", PropKind::Switch, "")
            .doc("A fall is good news (error rates, latency)."),
        Prop::new("note", PropKind::Value, "note: &'a str").doc("Small print under the value."),
        Prop::new("href", PropKind::Value, "href: &'a str")
            .attr("href")
            .doc("Make the whole card a link to the details."),
        Prop::new("reveal", PropKind::Switch, "").doc("Fades and rises as it scrolls into view."),
    ];
}

impl Ui {
    /// A card showing `value` under `label`.
    pub fn stat<'a>(&self, label: &'a str, value: &'a str) -> Stat<'a> {
        Stat {
            label,
            value,
            ..Stat::default()
        }
    }
}

impl<'a> Stat<'a> {
    /// The change; a leading `+` is up, `-` (or `−`) is down, and a zero is flat.
    pub fn delta(mut self, text: &'a str) -> Self {
        self.delta = Some(text);
        self
    }

    /// Override the direction read from the delta's sign.
    pub fn trend(mut self, trend: Trend) -> Self {
        self.trend = Some(trend);
        self
    }

    /// A fall is good news (error rates, latency).
    pub fn down_is_good(mut self) -> Self {
        self.down_is_good = true;
        self
    }

    /// Small print under the value: the period, the source.
    pub fn note(mut self, note: &'a str) -> Self {
        self.note = Some(note);
        self
    }

    /// Make the whole card a link to the details.
    pub fn href(mut self, href: &'a str) -> Self {
        self.href = Some(href);
        self
    }

    /// Fades and rises as it scrolls into view, with `animation-timeline: view()`; shown in
    /// place where that is missing or under `prefers-reduced-motion: reduce`.
    pub fn reveal(mut self) -> Self {
        self.reveal = true;
        self
    }
}
impl Trend {
    /// The direction a delta's text says: `+` up, `-` or `−` down, flat when there is no
    /// sign or every digit is zero.
    pub fn of(delta: &str) -> Trend {
        let t = delta.trim_start();
        if !t.chars().any(|c| c.is_ascii_digit() && c != '0') {
            return Trend::Flat;
        }
        match t.chars().next() {
            Some('+') => Trend::Up,
            Some('-' | '\u{2212}') => Trend::Down,
            _ => Trend::Flat,
        }
    }
}

impl Render for Stat<'_> {
    fn render(&self) -> Markup {
        let inner = html! {
            p class="lui-stat-label" { (self.label) }
            p class="lui-stat-value" { (self.value) }
            @if let Some(text) = self.delta {
                @let trend = self.trend.unwrap_or_else(|| Trend::of(text));
                @let (arrow, word, good) = match trend {
                    Trend::Up => ("\u{25b2}", "up", !self.down_is_good),
                    Trend::Down => ("\u{25bc}", "down", self.down_is_good),
                    Trend::Flat => ("\u{25b6}", "unchanged", true),
                };
                @let tone = if trend == Trend::Flat { "lui-stat-flat" } else if good { "lui-stat-good" } else { "lui-stat-bad" };
                p class={ "lui-stat-delta " (tone) } {
                    span aria-hidden="true" { (arrow) " " } span class="lui-sr" { (word) " " } (text)
                }
            }
            @if let Some(n) = self.note { p class="lui-stat-note" { (n) } }
        };
        let reveal = if self.reveal { " lui-stat-reveal" } else { "" };
        html! {
            @if let Some(href) = self.href {
                a class={ "lui-stat lui-stat-link" (reveal) } href=(href) { (inner) }
            } @else {
                div class={ "lui-stat" (reveal) } { (inner) }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr)); gap: calc(var(--lui-space) * 2); margin-block: calc(var(--lui-space) * 2); }
.lui-stat {
  display: block; padding: calc(var(--lui-space) * 3); color: var(--lui-fg); text-decoration: none;
  background: var(--lui-card); border: 1px solid var(--lui-line); border-radius: var(--lui-radius-lg); box-shadow: var(--lui-shadow-xs);
  transition: background-color 0.15s;
}
.lui-stat p { margin: 0; max-width: none; }
.lui-stat-link:hover { background: color-mix(in srgb, var(--lui-accent) 50%, var(--lui-card)); }
.lui-stat-label { color: var(--lui-muted); font-size: 0.875rem; font-weight: 500; }
.lui-stat-value { font-size: 1.5rem; font-weight: 600; line-height: 2rem; margin-block: 0.25rem; font-variant-numeric: tabular-nums; }
.lui-stat-delta { font-size: 0.75rem; font-weight: 500; }
.lui-stat-good { color: var(--lui-ok); }
.lui-stat-bad { color: var(--lui-danger); }
.lui-stat-flat { color: var(--lui-muted); }
.lui-stat-note { color: var(--lui-muted); font-size: 0.75rem; margin-top: 0.25rem; }
/* .reveal(): @keyframes lui-reveal is the card's (card.rs). */
@media (prefers-reduced-motion: no-preference) {
  @supports (animation-timeline: view()) {
    .lui-stat-reveal { animation: lui-reveal linear both; animation-timeline: view(); animation-range: entry 0% cover 30%; }
  }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sign_of_the_delta_is_the_trend() {
        assert_eq!(Trend::of("+8.2%"), Trend::Up);
        assert_eq!(Trend::of("-3"), Trend::Down);
        assert_eq!(Trend::of("\u{2212}0.2 pt"), Trend::Down);
        assert_eq!(Trend::of("0"), Trend::Flat);
        assert_eq!(Trend::of("+0.0%"), Trend::Flat);
        assert_eq!(Trend::of("12"), Trend::Flat);
        let ui = Ui::default();
        let inferred = ui.stat("Orders", "0").delta("-3").render().into_string();
        assert!(inferred.contains("down") && inferred.contains("lui-stat-bad"));
        let forced = ui
            .stat("Orders", "0")
            .delta("-3")
            .trend(Trend::Flat)
            .render()
            .into_string();
        assert!(forced.contains("unchanged"));
    }
}
