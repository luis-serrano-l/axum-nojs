//! # Stat
//!
//! One number that matters, with its label and how it moved: the tiles at the top of a
//! dashboard. Several sit in a `div.nojs-stat-grid`, which wraps them to the width available.
//!
//! **Platform features:** a card of `<p>` elements that reads in order; the change
//! carries its direction in words for screen readers (`<span class="nojs-sr">`) as well as an
//! arrow and a colour; the grid is `repeat(auto-fit, minmax(12rem, 1fr))`, so no media query.
//!
//! **What it does not do without script:** update live; the number is as fresh as the page.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! assert!(ui.stat("Visitors", "12,480").render().into_string().contains("12,480"));
//! // The sign of the delta says the direction: `+` up, `-` down, zero flat.
//! let m = ui.stat("Error rate", "0.4%")
//!     .delta("-0.2 pt")
//!     .down_is_good()
//!     .note("last 7 days")
//!     .href("/errors");
//! let m = m.render().into_string();
//! assert!(m.contains("nojs-stat-good") && m.contains("down") && m.contains(r#"href="/errors""#));
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
/// switches: `.down_is_good()`.
#[derive(Clone, Debug, Default)]
pub struct Stat<'a> {
    label: &'a str,
    value: &'a str,
    delta: Option<&'a str>,
    trend: Option<Trend>,
    down_is_good: bool,
    note: Option<&'a str>,
    href: Option<&'a str>,
}

impl Stat<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props`] and kept in step with the setters by a test.
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
            p class="nojs-stat-label" { (self.label) }
            p class="nojs-stat-value" { (self.value) }
            @if let Some(text) = self.delta {
                @let trend = self.trend.unwrap_or_else(|| Trend::of(text));
                @let (arrow, word, good) = match trend {
                    Trend::Up => ("\u{25b2}", "up", !self.down_is_good),
                    Trend::Down => ("\u{25bc}", "down", self.down_is_good),
                    Trend::Flat => ("\u{25b6}", "unchanged", true),
                };
                @let tone = if trend == Trend::Flat { "nojs-stat-flat" } else if good { "nojs-stat-good" } else { "nojs-stat-bad" };
                p class={ "nojs-stat-delta " (tone) } {
                    span aria-hidden="true" { (arrow) " " } span class="nojs-sr" { (word) " " } (text)
                }
            }
            @if let Some(n) = self.note { p class="nojs-stat-note" { (n) } }
        };
        html! {
            @if let Some(href) = self.href {
                a class="nojs-stat nojs-stat-link" href=(href) { (inner) }
            } @else {
                div class="nojs-stat" { (inner) }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr)); gap: calc(var(--nojs-space) * 2); margin-block: calc(var(--nojs-space) * 2); }
.nojs-stat {
  display: block; padding: calc(var(--nojs-space) * 3); color: var(--nojs-fg); text-decoration: none;
  background: var(--nojs-card); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius-lg); box-shadow: var(--nojs-shadow-xs);
  transition: background-color 0.15s;
}
.nojs-stat p { margin: 0; max-width: none; }
.nojs-stat-link:hover { background: color-mix(in srgb, var(--nojs-accent) 50%, var(--nojs-card)); }
.nojs-stat-label { color: var(--nojs-muted); font-size: 0.875rem; font-weight: 500; }
.nojs-stat-value { font-size: 1.5rem; font-weight: 600; line-height: 2rem; margin-block: 0.25rem; font-variant-numeric: tabular-nums; }
.nojs-stat-delta { font-size: 0.75rem; font-weight: 500; }
.nojs-stat-good { color: var(--nojs-ok); }
.nojs-stat-bad { color: var(--nojs-danger); }
.nojs-stat-flat { color: var(--nojs-muted); }
.nojs-stat-note { color: var(--nojs-muted); font-size: 0.75rem; margin-top: 0.25rem; }
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
        assert!(inferred.contains("down") && inferred.contains("nojs-stat-bad"));
        let forced = ui
            .stat("Orders", "0")
            .delta("-3")
            .trend(Trend::Flat)
            .render()
            .into_string();
        assert!(forced.contains("unchanged"));
    }
}
