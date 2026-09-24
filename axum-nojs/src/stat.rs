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
//! use axum_nojs::{Caps, stat, stat_with, stat::StatOptions};
//! let m = stat(&Caps::all(), "Visitors", "12,480").into_string();
//! assert!(m.contains("12,480"));
//! // The sign of the delta says the direction: `+` up, `-` down, zero flat.
//! let m = stat_with(&Caps::all(), "Error rate", "0.4%", StatOptions::default()
//!     .delta("-0.2 pt")
//!     .down_is_good()
//!     .note("last 7 days")
//!     .href("/errors")).into_string();
//! assert!(m.contains("nojs-stat-good") && m.contains("down") && m.contains(r#"href="/errors""#));
//! ```

use maud::{Markup, html};

use crate::Caps;

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

/// Options for [`stat`].
#[derive(Clone, Debug, Default)]
pub struct StatOptions<'a> {
    /// The change, as text (`"+12%"`).
    pub delta: Option<&'a str>,
    /// Its direction; `None` reads it from the sign of `delta`.
    pub trend: Option<Trend>,
    /// Colour a fall as good (error rates, latency).
    pub down_is_good: bool,
    /// Small print under the value: the period, the source.
    pub note: Option<&'a str>,
    /// Make the whole card a link to the details.
    pub href: Option<&'a str>,
}

impl<'a> StatOptions<'a> {
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
    /// A fall is good news.
    pub fn down_is_good(mut self) -> Self {
        self.down_is_good = true;
        self
    }
    /// Small print under the value.
    pub fn note(mut self, note: &'a str) -> Self {
        self.note = Some(note);
        self
    }
    /// Link the card.
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
        if !t.chars().any(|c| c.is_ascii_digit() && c != '0') { return Trend::Flat; }
        match t.chars().next() {
            Some('+') => Trend::Up,
            Some('-' | '\u{2212}') => Trend::Down,
            _ => Trend::Flat,
        }
    }
}

/// A stat with no delta or note.
/// [`stat_with`] takes the options.
pub fn stat(caps: &Caps, label: &str, value: &str) -> Markup {
    stat_with(caps, label, value, Default::default())
}

/// A stat card: `label` above `value`.
pub fn stat_with(_caps: &Caps, label: &str, value: &str, options: StatOptions) -> Markup {
    let inner = html! {
        p class="nojs-stat-label" { (label) }
        p class="nojs-stat-value" { (value) }
        @if let Some(text) = options.delta {
            @let trend = options.trend.unwrap_or_else(|| Trend::of(text));
            @let (arrow, word, good) = match trend {
                Trend::Up => ("\u{25b2}", "up", !options.down_is_good),
                Trend::Down => ("\u{25bc}", "down", options.down_is_good),
                Trend::Flat => ("\u{25b6}", "unchanged", true),
            };
            @let tone = if trend == Trend::Flat { "nojs-stat-flat" } else if good { "nojs-stat-good" } else { "nojs-stat-bad" };
            p class={ "nojs-stat-delta " (tone) } {
                span aria-hidden="true" { (arrow) " " } span class="nojs-sr" { (word) " " } (text)
            }
        }
        @if let Some(n) = options.note { p class="nojs-stat-note" { (n) } }
    };
    html! {
        @if let Some(href) = options.href {
            a class="nojs-stat nojs-stat-link" href=(href) { (inner) }
        } @else {
            div class="nojs-stat" { (inner) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr)); gap: calc(var(--nojs-space) * 2); margin-block: calc(var(--nojs-space) * 2); }
.nojs-stat {
  display: block; padding: calc(var(--nojs-space) * 2); color: var(--nojs-fg); text-decoration: none;
  background: var(--nojs-surface); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
}
.nojs-stat p { margin: 0; max-width: none; }
.nojs-stat-link:hover { border-color: var(--nojs-accent); }
.nojs-stat-label { color: var(--nojs-muted); font-size: 0.875rem; }
.nojs-stat-value { font-size: 1.75rem; font-weight: 700; line-height: 1.2; margin-block: 0.25rem; font-variant-numeric: tabular-nums; }
.nojs-stat-delta { font-size: 0.875rem; font-weight: 600; }
.nojs-stat-good { color: var(--nojs-ok); }
.nojs-stat-bad { color: var(--nojs-danger); }
.nojs-stat-flat { color: var(--nojs-muted); }
.nojs-stat-note { color: var(--nojs-muted); font-size: 0.8rem; margin-top: 0.25rem; }
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
        let inferred = stat_with(&Caps::all(), "Orders", "0", StatOptions::default().delta("-3")).into_string();
        assert!(inferred.contains("down") && inferred.contains("nojs-stat-bad"));
        let forced = stat_with(&Caps::all(), "Orders", "0", StatOptions::default().delta("-3").trend(Trend::Flat)).into_string();
        assert!(forced.contains("unchanged"));
    }
}
