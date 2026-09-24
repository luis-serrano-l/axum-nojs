//! # Stat
//!
//! One number that matters, with its label and how it moved: the tiles at the top of a
//! dashboard. Several sit in a `div.wo-stat-grid`, which wraps them to the width available.
//!
//! **Platform features:** a card of `<p>` elements that reads in order; the change
//! carries its direction in words for screen readers (`<span class="wo-sr">`) as well as an
//! arrow and a colour; the grid is `repeat(auto-fit, minmax(12rem, 1fr))`, so no media query.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use webonsive::{Caps, stat, stat::{StatOptions, Trend}};
//! let m = stat(&Caps::all(), "Visitors", "12,480", Default::default()).into_string();
//! assert!(m.contains("12,480"));
//! let m = stat(&Caps::all(), "Error rate", "0.4%", StatOptions::default()
//!     .delta("-0.2 pt", Trend::Down)
//!     .down_is_good(true)
//!     .note("last 7 days")
//!     .href("/errors")).into_string();
//! assert!(m.contains("wo-stat-good") && m.contains("down") && m.contains(r#"href="/errors""#));
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
    /// The change, as text (`"+12%"`), and its direction.
    pub delta: Option<(&'a str, Trend)>,
    /// Colour a fall as good (error rates, latency).
    pub down_is_good: bool,
    /// Small print under the value: the period, the source.
    pub note: Option<&'a str>,
    /// Make the whole card a link to the details.
    pub href: Option<&'a str>,
}

impl<'a> StatOptions<'a> {
    /// The change and its direction.
    pub fn delta(mut self, text: &'a str, trend: Trend) -> Self {
        self.delta = Some((text, trend));
        self
    }
    /// A fall is good news.
    pub fn down_is_good(mut self, on: bool) -> Self {
        self.down_is_good = on;
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

/// A stat card: `label` above `value`.
pub fn stat(_caps: &Caps, label: &str, value: &str, options: StatOptions) -> Markup {
    let inner = html! {
        p class="wo-stat-label" { (label) }
        p class="wo-stat-value" { (value) }
        @if let Some((text, trend)) = options.delta {
            @let (arrow, word, good) = match trend {
                Trend::Up => ("\u{25b2}", "up", !options.down_is_good),
                Trend::Down => ("\u{25bc}", "down", options.down_is_good),
                Trend::Flat => ("\u{25b6}", "unchanged", true),
            };
            @let tone = if trend == Trend::Flat { "wo-stat-flat" } else if good { "wo-stat-good" } else { "wo-stat-bad" };
            p class={ "wo-stat-delta " (tone) } {
                span aria-hidden="true" { (arrow) " " } span class="wo-sr" { (word) " " } (text)
            }
        }
        @if let Some(n) = options.note { p class="wo-stat-note" { (n) } }
    };
    html! {
        @if let Some(href) = options.href {
            a class="wo-stat wo-stat-link" href=(href) { (inner) }
        } @else {
            div class="wo-stat" { (inner) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-stat-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(12rem, 1fr)); gap: calc(var(--wo-space) * 2); margin-block: calc(var(--wo-space) * 2); }
.wo-stat {
  display: block; padding: calc(var(--wo-space) * 2); color: var(--wo-fg); text-decoration: none;
  background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius);
}
.wo-stat p { margin: 0; max-width: none; }
.wo-stat-link:hover { border-color: var(--wo-accent); }
.wo-stat-label { color: var(--wo-muted); font-size: 0.875rem; }
.wo-stat-value { font-size: 1.75rem; font-weight: 700; line-height: 1.2; margin-block: 0.25rem; font-variant-numeric: tabular-nums; }
.wo-stat-delta { font-size: 0.875rem; font-weight: 600; }
.wo-stat-good { color: var(--wo-ok); }
.wo-stat-bad { color: var(--wo-danger); }
.wo-stat-flat { color: var(--wo-muted); }
.wo-stat-note { color: var(--wo-muted); font-size: 0.8rem; margin-top: 0.25rem; }
"#;
