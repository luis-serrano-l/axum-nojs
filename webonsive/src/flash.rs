//! # Flash
//!
//! One-shot status messages that survive a Post/Redirect/Get, no script. Four levels
//! (`info`, `ok`, `warn`, `danger`), several at once stacked in arrival order, an optional
//! dismiss link, and an optional auto-hide for the calm levels.
//!
//! **Platform features:** a cookie with `Max-Age=60` set by [`crate::state::prg`] on the
//! redirect, read on the next request through `UiState`, and cleared by returning that
//! `UiState` with the response. `role="status"` announces info, ok and warn politely;
//! `role="alert"` announces danger at once. Auto-hide is a CSS animation
//! (`@keyframes`, `animation-fill-mode: forwards`) that `prefers-reduced-motion: reduce`
//! switches off, so the message then stays until the next page.
//!
//! The cookie value is plain text: one message per line, each optionally prefixed with its
//! level (`ok:Saved.`). A line without a known prefix is `info`, so `prg(to, Some("Saved."))`
//! keeps working. [`stack`] builds that text.
//!
//! **Dismiss:** a link back to the page (`FlashOptions::dismiss`). Reading the flash already
//! queued the cookie's deletion on that response, so following the link renders the page
//! without it; with `/wo/enhance.js` inside a swap root it updates in place.
//!
//! **What it does not do without script:** it cannot vanish in place when dismissed; the
//! dismiss link is a navigation.
//!
//! **Fallback:** without CSS animations the message stays; nothing else differs.
//!
//! ```rust
//! use webonsive::{Caps, flash, flash_with, flash::{FlashOptions, Level, stack}};
//! let caps = Caps::all();
//! let m = flash(&caps, Some("Saved.")).into_string();
//! assert!(m.contains("wo-flash-info") && m.contains("Saved."));
//! assert_eq!(flash(&caps, None).into_string(), "");
//!
//! // Two at once, one of them an error, with a dismiss link and auto-hide.
//! let text = stack(&[(Level::Ok, "Saved."), (Level::Danger, "Avatar too large.")]);
//! let m = flash_with(&caps, Some(&text), FlashOptions::default().dismiss("/settings").auto_hide(true)).into_string();
//! assert!(m.contains(r#"role="alert""#) && m.contains("wo-flash-auto"));
//! assert_eq!(m.matches("wo-flash-dismiss").count(), 2);
//! ```

use maud::{Markup, html};

use crate::Caps;

/// How much a message matters: sets its colour and how it is announced.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Level {
    /// Neutral news. `role="status"`.
    #[default]
    Info,
    /// Something worked. `role="status"`.
    Ok,
    /// Worked, but look at this. `role="status"`.
    Warn,
    /// Something failed. `role="alert"`, never auto-hidden.
    Danger,
}

impl Level {
    /// The prefix used in the cookie text and the class suffix (`wo-flash-ok`).
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Info => "info",
            Level::Ok => "ok",
            Level::Warn => "warn",
            Level::Danger => "danger",
        }
    }

    fn parse(s: &str) -> Option<Level> {
        [Level::Info, Level::Ok, Level::Warn, Level::Danger].into_iter().find(|l| l.as_str() == s)
    }
}

/// Split flash text into `(level, message)` pairs: one per non-empty line, a known
/// `level:` prefix picks the level, anything else is `info`.
pub fn parse(text: &str) -> Vec<(Level, &str)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|line| match line.split_once(':').and_then(|(p, m)| Level::parse(p).map(|l| (l, m.trim()))) {
            Some(pair) => pair,
            None => (Level::Info, line),
        })
        .collect()
}

/// Join messages into the text `prg` carries: `stack(&[(Level::Ok, "Saved.")])` is `"ok:Saved."`.
pub fn stack(messages: &[(Level, &str)]) -> String {
    messages.iter().map(|(l, m)| format!("{}:{m}", l.as_str())).collect::<Vec<_>>().join("\n")
}

/// Options for [`flash`].
#[derive(Clone, Debug, Default)]
pub struct FlashOptions<'a> {
    /// Where the dismiss link goes, normally the page's own URL; `None` shows no link.
    pub dismiss: Option<&'a str>,
    /// Fade info and ok messages out after a few seconds (not warn or danger).
    pub auto_hide: bool,
}

impl<'a> FlashOptions<'a> {
    /// Show a dismiss link to `href`.
    pub fn dismiss(mut self, href: &'a str) -> Self {
        self.dismiss = Some(href);
        self
    }
    /// Fade calm messages out; reduced motion keeps them.
    pub fn auto_hide(mut self, on: bool) -> Self {
        self.auto_hide = on;
        self
    }
}

/// A flash banner with the default level.
/// [`flash_with`] takes the options.
pub fn flash(caps: &Caps, text: Option<&str>) -> Markup {
    flash_with(caps, text, Default::default())
}

/// Render the messages in `text` (see [`parse`]) as a stack of banners; nothing when there are none.
pub fn flash_with(_caps: &Caps, text: Option<&str>, options: FlashOptions) -> Markup {
    let messages = text.map(parse).unwrap_or_default();
    html! {
        @if !messages.is_empty() {
            div class="wo-flash" {
                @for (level, message) in &messages {
                    @let hide = options.auto_hide && matches!(level, Level::Info | Level::Ok);
                    p class={ "wo-flash-item wo-flash-" (level.as_str()) @if hide { " wo-flash-auto" } }
                        role=(if *level == Level::Danger { "alert" } else { "status" }) {
                        span class="wo-flash-text" { (message) }
                        @if let Some(href) = options.dismiss {
                            " " a class="wo-flash-dismiss" href=(href) aria-label={ "Dismiss: " (message) } { "Dismiss" }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-flash { display: grid; gap: calc(var(--wo-space) * 1); margin-block: calc(var(--wo-space) * 2); }
.wo-flash-item {
  --wo-flash-tone: var(--wo-accent);
  display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between;
  gap: calc(var(--wo-space) * 1) calc(var(--wo-space) * 2); margin: 0;
  padding: 0.6rem 1rem; border-radius: var(--wo-radius);
  background: color-mix(in srgb, var(--wo-flash-tone) 14%, var(--wo-surface));
  border: 1px solid var(--wo-flash-tone); border-inline-start-width: 4px; color: var(--wo-fg);
}
.wo-flash-info { --wo-flash-tone: var(--wo-muted); }
.wo-flash-ok { --wo-flash-tone: var(--wo-ok); }
.wo-flash-warn { --wo-flash-tone: var(--wo-warn); }
.wo-flash-danger { --wo-flash-tone: var(--wo-danger); }
.wo-flash-dismiss { color: var(--wo-fg); font-size: 0.875rem; }
.wo-flash-auto { animation: wo-flash-hide 0.4s ease-in 6s forwards; }
@keyframes wo-flash-hide {
  to { opacity: 0; visibility: hidden; height: 0; padding-block: 0; margin-block: -0.5rem 0; border-width: 0; }
}
@media (prefers-reduced-motion: reduce) { .wo-flash-auto { animation: none; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_levels_and_plain_lines() {
        assert_eq!(parse("Saved."), vec![(Level::Info, "Saved.")]);
        assert_eq!(parse("ok:Saved.\n\nwarn: Look.\nnote: kept"), vec![(Level::Ok, "Saved."), (Level::Warn, "Look."), (Level::Info, "note: kept")]);
        assert_eq!(parse(&stack(&[(Level::Danger, "No: really")])), vec![(Level::Danger, "No: really")]);
        assert!(parse("  \n").is_empty());
    }

    #[test]
    fn danger_is_an_alert_and_never_auto_hides() {
        let m = flash_with(&Caps::all(), Some("danger:Failed."), FlashOptions::default().auto_hide(true)).into_string();
        assert!(m.contains(r#"role="alert""#) && !m.contains("wo-flash-auto") && !m.contains("wo-flash-dismiss"));
    }
}
