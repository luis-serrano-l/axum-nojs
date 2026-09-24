//! # Theme
//!
//! Light/dark theme chosen by the user and remembered by the server.
//!
//! **Platform features:** `prefers-color-scheme` (baseline 2020), `color-scheme` property,
//! CSS custom properties. The toggle is a `<form method="post">`; the server stores the choice
//! in a cookie and sets `data-theme` on `<html>`.
//!
//! **What it does not do without script:** follow a change of the OS preference while a cookie
//! choice is set; the cookie wins until reset to auto.
//!
//! **Fallback:** none needed. Without a cookie the OS preference wins. Colours are switched by
//! a media query and `data-theme`, so `light-dark()` support (`Caps::LightDark`) is only
//! reported, never required. The form is a swap root: the [`crate::enhance`] script applies
//! the new `data-theme` without a reload.
//!
//! ```rust
//! use axum_nojs::{Caps, Theme, theme_toggle};
//! let markup = theme_toggle(&Caps::all(), "/theme", Theme::Dark);
//! ```

use maud::{Markup, html};

use crate::Caps;

/// Name of the cookie that remembers the chosen theme.
pub const THEME_COOKIE: &str = "theme";

/// The theme the page should render with.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Theme {
    /// Follow the operating system preference.
    #[default]
    Auto,
    /// Always light.
    Light,
    /// Always dark.
    Dark,
}

impl Theme {
    /// Value used in the `data-theme` attribute and the cookie.
    pub fn as_str(self) -> &'static str {
        match self {
            Theme::Auto => "auto",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }

    /// Parse a cookie or form value. Unknown values become `Auto`.
    pub fn parse(value: &str) -> Theme {
        match value {
            "light" => Theme::Light,
            "dark" => Theme::Dark,
            _ => Theme::Auto,
        }
    }
}

/// A three-button form that posts the chosen theme to `action`.
pub fn theme_toggle(_caps: &Caps, action: &str, current: Theme) -> Markup {
    let choices = [Theme::Auto, Theme::Light, Theme::Dark];
    html! {
        form id="nojs-theme" data-nojs="swap" class="nojs-theme" method="post" action=(action) {
            @for choice in choices {
                button type="submit" name="theme" value=(choice.as_str())
                    aria-pressed=(if choice == current { "true" } else { "false" }) {
                    (choice.as_str())
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-theme { display: inline-flex; gap: 0; border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); overflow: hidden; }
.nojs-theme button { border: 0; border-radius: 0; background: transparent; padding: 0.4rem 0.8rem; text-transform: capitalize; }
.nojs-theme button[aria-pressed="true"] { background: var(--nojs-accent); color: var(--nojs-on-accent); }
"#;
