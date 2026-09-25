//! # Flash
//!
//! One-shot status messages that survive a Post/Redirect/Get, no script. Four levels
//! (`info`, `ok`, `warn`, `danger`), several at once stacked in arrival order, an optional
//! dismiss link, and an optional auto-hide for the calm levels.
//!
//! **Platform features:** a cookie with `Max-Age=60` set by [`crate::Ui::redirect`] on the
//! redirect, read on the next request through `UiState`, and cleared by the next
//! [`crate::Ui::page`] that shows it. `role="status"` announces info, ok and warn politely;
//! `role="alert"` announces danger at once. Auto-hide is a CSS animation
//! (`@keyframes`, `animation-fill-mode: forwards`) that `prefers-reduced-motion: reduce`
//! switches off, so the message then stays until the next page.
//!
//! The cookie value is plain text: one message per line, each optionally prefixed with its
//! level (`ok:Saved.`). A line without a known prefix is `info`. `ui.redirect(to).ok(..)`
//! writes it; [`stack`] builds the same text by hand.
//!
//! **Dismiss:** a link back to the page (`.dismiss()`). Reading the flash already
//! queued the cookie's deletion on that response, so following the link renders the page
//! without it; with `/nojs/enhance.js` inside a swap root it updates in place.
//!
//! **What it does not do without script:** it cannot vanish in place when dismissed; the
//! dismiss link is a navigation.
//!
//! **Fallback:** without CSS animations the message stays; nothing else differs.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from_request("/settings", "", "nojs-flash=Saved.");
//! let m = ui.flash().render().into_string();
//! assert!(m.contains("nojs-flash-info") && m.contains("Saved."));
//! assert_eq!(Ui::default().flash().render().into_string(), "", "no message, no banner");
//!
//! // What `ui.redirect("/settings").ok("Saved.").danger("Avatar too large.")` sends, shown
//! // with a dismiss link and auto-hide.
//! let ui = Ui::from_request("/settings", "", "nojs-flash=ok%3ASaved.%0Adanger%3AAvatar%20too%20large.");
//! let m = ui.flash().dismiss().auto_hide().render().into_string();
//! assert!(m.contains(r#"role="alert""#) && m.contains("nojs-flash-auto"));
//! assert_eq!(m.matches(r#"href="/settings""#).count(), 2);
//!
//! // The same in `nojs!`:
//! let same = nojs! { Flash dismiss auto_hide; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

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
    /// The prefix used in the cookie text and the class suffix (`nojs-flash-ok`).
    pub fn as_str(self) -> &'static str {
        match self {
            Level::Info => "info",
            Level::Ok => "ok",
            Level::Warn => "warn",
            Level::Danger => "danger",
        }
    }

    fn parse(s: &str) -> Option<Level> {
        [Level::Info, Level::Ok, Level::Warn, Level::Danger]
            .into_iter()
            .find(|l| l.as_str() == s)
    }
}

/// Split flash text into `(level, message)` pairs: one per non-empty line, a known
/// `level:` prefix picks the level, anything else is `info`.
pub fn parse(text: &str) -> Vec<(Level, &str)> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(|line| {
            match line
                .split_once(':')
                .and_then(|(p, m)| Level::parse(p).map(|l| (l, m.trim())))
            {
                Some(pair) => pair,
                None => (Level::Info, line),
            }
        })
        .collect()
}

/// Join messages into the text the flash cookie carries: `stack(&[(Level::Ok, "Saved.")])` is `"ok:Saved."`.
pub fn stack(messages: &[(Level, &str)]) -> String {
    messages
        .iter()
        .map(|(l, m)| format!("{}:{m}", l.as_str()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// The request's flash messages as a stack of banners, made by [`Ui::flash`]; nothing when
/// there are none.
///
/// **Setters.** Switches: `.dismiss()`, `.auto_hide()`.
#[derive(Clone, Debug)]
pub struct Flash<'a> {
    ui: &'a Ui,
    dismiss: bool,
    auto_hide: bool,
}

impl Flash<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("dismiss", PropKind::Switch, "").doc(
            "A dismiss link on each message, back to this page (which no longer has the flash).",
        ),
        Prop::new("auto_hide", PropKind::Switch, "")
            .doc("Fade calm messages out after a few seconds."),
    ];
}

impl Ui {
    /// The flash messages a [`Ui::redirect`] left for this page.
    pub fn flash(&self) -> Flash<'_> {
        Flash {
            ui: self,
            dismiss: false,
            auto_hide: false,
        }
    }
}

impl Flash<'_> {
    /// A dismiss link on each message, back to this page (which no longer has the flash).
    pub fn dismiss(mut self) -> Self {
        self.dismiss = true;
        self
    }

    /// Fade calm messages out after a few seconds; reduced motion keeps them.
    pub fn auto_hide(mut self) -> Self {
        self.auto_hide = true;
        self
    }
}

impl Render for Flash<'_> {
    fn render(&self) -> Markup {
        let messages = self.ui.state.flash().map(parse).unwrap_or_default();
        let dismiss = self.dismiss.then(|| self.ui.state.path());
        html! {
            @if !messages.is_empty() {
                div class="nojs-flash" {
                    @for (level, message) in &messages {
                        @let hide = self.auto_hide && matches!(level, Level::Info | Level::Ok);
                        p class={ "nojs-flash-item nojs-flash-" (level.as_str()) @if hide { " nojs-flash-auto" } }
                            role=(if *level == Level::Danger { "alert" } else { "status" }) {
                            span class="nojs-flash-text" { (message) }
                            @if let Some(href) = dismiss {
                                " " a class="nojs-flash-dismiss" href=(href) aria-label={ "Dismiss: " (message) } { "Dismiss" }
                            }
                        }
                    }
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-flash { display: grid; gap: calc(var(--nojs-space) * 1); margin-block: calc(var(--nojs-space) * 2); }
.nojs-flash-item {
  --nojs-flash-tone: var(--nojs-primary);
  display: flex; flex-wrap: wrap; align-items: baseline; justify-content: space-between;
  gap: calc(var(--nojs-space) * 1) calc(var(--nojs-space) * 2); margin: 0;
  padding: 0.75rem 1rem; border-radius: var(--nojs-radius); font-size: 0.875rem;
  background: var(--nojs-card); border: 1px solid var(--nojs-line); color: var(--nojs-fg);
}
/* shadcn Alert: a neutral card; the level is a dot in its colour, and danger colours the text. */
.nojs-flash-item::before { content: ""; flex: none; align-self: center; width: 0.5rem; height: 0.5rem; margin-right: -0.5rem; border-radius: 50%; background: var(--nojs-flash-tone); }
.nojs-flash-item > :first-child { flex: 1; }
.nojs-flash-info { --nojs-flash-tone: var(--nojs-muted); }
.nojs-flash-ok { --nojs-flash-tone: var(--nojs-ok); }
.nojs-flash-warn { --nojs-flash-tone: var(--nojs-warn); }
.nojs-flash-danger { --nojs-flash-tone: var(--nojs-danger); color: var(--nojs-danger); }
.nojs-flash-dismiss { color: var(--nojs-muted); font-size: 0.875rem; }
.nojs-flash-dismiss:hover { color: var(--nojs-fg); }
.nojs-flash-auto { animation: nojs-flash-hide 0.4s ease-in 6s forwards; }
@keyframes nojs-flash-hide {
  to { opacity: 0; visibility: hidden; height: 0; padding-block: 0; margin-block: -0.5rem 0; border-width: 0; }
}
@media (prefers-reduced-motion: reduce) { .nojs-flash-auto { animation: none; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_levels_and_plain_lines() {
        assert_eq!(parse("Saved."), vec![(Level::Info, "Saved.")]);
        assert_eq!(
            parse("ok:Saved.\n\nwarn: Look.\nnote: kept"),
            vec![
                (Level::Ok, "Saved."),
                (Level::Warn, "Look."),
                (Level::Info, "note: kept")
            ]
        );
        assert_eq!(
            parse(&stack(&[(Level::Danger, "No: really")])),
            vec![(Level::Danger, "No: really")]
        );
        assert!(parse("  \n").is_empty());
    }

    #[test]
    fn danger_is_an_alert_and_never_auto_hides() {
        let ui = Ui::from_request("/", "", "nojs-flash=danger:Failed.");
        let m = ui.flash().auto_hide().render().into_string();
        assert!(
            m.contains(r#"role="alert""#)
                && !m.contains("nojs-flash-auto")
                && !m.contains("nojs-flash-dismiss")
        );
    }
}
