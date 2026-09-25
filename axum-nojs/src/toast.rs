//! # Toast
//!
//! Short notices in a stack at the corner of the viewport, outside the page flow, that fade
//! on their own: "Copied", "Invite sent". The same one-shot cookie as [`crate::Ui::flash`] carries
//! them across a Post/Redirect/Get, with the same `level:` lines ([`crate::flash::stack`]).
//!
//! **Platform features:** `position: fixed` in the bottom corner (top on narrow screens, clear
//! of the thumb); `role="status"` per notice and `role="alert"` for danger; a CSS fade with
//! `@keyframes` that pauses on `:hover` and `:focus-within` and that
//! `prefers-reduced-motion: reduce` switches off. Danger toasts never fade.
//!
//! **Fallback:** without CSS animations the toasts stay until the next page; the dismiss link
//! (`.dismiss()`) clears them sooner. `/nojs/enhance.js` carries the list across a
//! swap like the flash.
//!
//! **What it does not do without script:** it cannot appear without a request (a toast is the
//! answer to a round trip), and dismissing one is a navigation, not an instant removal.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! // What `ui.redirect("/toast").ok("Invite sent.").danger("Mail server down.")` sends.
//! let ui = Ui::from_request("/toast", "", "nojs-flash=ok%3AInvite%20sent.%0Adanger%3AMail%20server%20down.");
//! let m = ui.toasts().render().into_string();
//! assert!(m.contains("nojs-toast-ok") && m.contains(r#"role="alert""#));
//! assert!(ui.toasts().dismiss().render().into_string().contains(r#"href="/toast""#));
//! assert_eq!(Ui::default().toasts().render().into_string(), "");
//! // The same in `nojs!`:
//! let same = nojs! { Toasts; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::flash::{Level, parse};
use crate::props::{Prop, PropKind};

/// The request's flash messages as toasts in the corner, made by [`Ui::toasts`]; nothing when
/// there are none.
///
/// **Setters.** Switches: `.dismiss()`.
#[derive(Clone, Debug)]
pub struct Toasts<'a> {
    ui: &'a Ui,
    dismiss: bool,
}

impl Toasts<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[Prop::new("dismiss", PropKind::Switch, "")
        .doc("A close link on each toast, back to this page.")];
}

impl Ui {
    /// The flash messages a [`Ui::redirect`] left for this page, as toasts.
    pub fn toasts(&self) -> Toasts<'_> {
        Toasts {
            ui: self,
            dismiss: false,
        }
    }
}

impl Toasts<'_> {
    /// A close link on each toast, back to this page.
    pub fn dismiss(mut self) -> Self {
        self.dismiss = true;
        self
    }
}

impl Render for Toasts<'_> {
    fn render(&self) -> Markup {
        let messages = self.ui.state.flash().map(parse).unwrap_or_default();
        let dismiss = self.dismiss.then(|| self.ui.state.path());
        html! {
            @if !messages.is_empty() {
                ol class="nojs-toasts" {
                    @for (level, message) in &messages {
                        li class={ "nojs-toast nojs-toast-" (level.as_str()) } role=(if *level == Level::Danger { "alert" } else { "status" }) {
                            span class="nojs-toast-text" { (message) }
                            @if let Some(href) = dismiss {
                                a class="nojs-toast-close" href=(href) aria-label={ "Dismiss: " (message) } { "\u{d7}" }
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
/* Sonner-style cards: bottom-right stack, popover surface, rounded, shadow-lg, text-sm; the
   level shows as a small dot in its colour rather than a tinted card. */
.nojs-toasts {
  position: fixed; z-index: 20; inset-inline-end: calc(var(--nojs-space) * 3); bottom: calc(var(--nojs-space) * 3);
  display: grid; gap: 0.875rem; width: min(22.25rem, calc(100vw - 2rem));
  list-style: none; margin: 0; padding: 0;
}
.nojs-toast {
  --nojs-toast-tone: var(--nojs-muted);
  display: flex; align-items: center; gap: 0.625rem;
  padding: 1rem; color: var(--nojs-fg); background: var(--nojs-popover);
  font-size: 0.875rem; line-height: 1.25rem;
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
  box-shadow: var(--nojs-shadow-lg);
  animation: nojs-toast-out 0.4s ease-in 5s forwards;
}
.nojs-toast::before { content: ""; flex: none; width: 0.5rem; height: 0.5rem; border-radius: 50%; background: var(--nojs-toast-tone); }
.nojs-toast-text { flex: 1; font-weight: 500; }
.nojs-toast:hover, .nojs-toast:focus-within { animation-play-state: paused; }
.nojs-toast-ok { --nojs-toast-tone: var(--nojs-ok); }
.nojs-toast-warn { --nojs-toast-tone: var(--nojs-warn); }
.nojs-toast-danger { --nojs-toast-tone: var(--nojs-danger); animation: none; }
.nojs-toast-close {
  display: inline-flex; align-items: center; justify-content: center; width: 1.25rem; height: 1.25rem;
  color: var(--nojs-muted); text-decoration: none; font-size: 1rem; line-height: 1; border-radius: 50%;
}
.nojs-toast-close:hover { color: var(--nojs-fg); background: var(--nojs-accent); }
@keyframes nojs-toast-out { to { opacity: 0; visibility: hidden; transform: translateY(0.5rem); } }
@media (prefers-reduced-motion: reduce) { .nojs-toast { animation: none; } }
@media (max-width: 40rem) { .nojs-toasts { bottom: auto; top: calc(var(--nojs-space) * 2); inset-inline: 1rem; width: auto; } }
"#;
