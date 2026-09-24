//! # Toast
//!
//! Short notices in a stack at the corner of the viewport, outside the page flow, that fade
//! on their own: "Copied", "Invite sent". The same one-shot cookie as [`crate::flash()`] carries
//! them across a Post/Redirect/Get, with the same `level:` lines ([`crate::flash::stack`]).
//!
//! **Platform features:** `position: fixed` in the bottom corner (top on narrow screens, clear
//! of the thumb); `role="status"` per notice and `role="alert"` for danger; a CSS fade with
//! `@keyframes` that pauses on `:hover` and `:focus-within` and that
//! `prefers-reduced-motion: reduce` switches off. Danger toasts never fade.
//!
//! **Fallback:** without CSS animations the toasts stay until the next page; the dismiss link
//! (`ToastOptions::dismiss`) clears them sooner. `/nojs/enhance.js` carries the list across a
//! swap like the flash.
//!
//! **What it does not do without script:** it cannot appear without a request (a toast is the
//! answer to a round trip), and dismissing one is a navigation, not an instant removal.
//!
//! ```rust
//! use axum_nojs::{Caps, toasts, toasts_with, toast::ToastOptions, flash::{Level, stack}};
//! let text = stack(&[(Level::Ok, "Invite sent."), (Level::Danger, "Mail server down.")]);
//! let m = toasts(&Caps::all(), Some(&text)).into_string();
//! assert!(m.contains("nojs-toast-ok") && m.contains(r#"role="alert""#));
//! let m = toasts_with(&Caps::all(), Some("Copied."), ToastOptions::default().dismiss("/toast")).into_string();
//! assert!(m.contains(r#"href="/toast""#));
//! assert_eq!(toasts(&Caps::all(), None).into_string(), "");
//! ```

use maud::{Markup, html};

use crate::Caps;
use crate::flash::{Level, parse};

/// Options for [`toasts`].
#[derive(Clone, Debug, Default)]
pub struct ToastOptions<'a> {
    /// Where each toast's close link goes, normally the page's own URL.
    pub dismiss: Option<&'a str>,
}

impl<'a> ToastOptions<'a> {
    /// A close link to `href` on every toast.
    pub fn dismiss(mut self, href: &'a str) -> Self {
        self.dismiss = Some(href);
        self
    }
}

/// A toast region with the default options.
/// [`toasts_with`] takes the options.
pub fn toasts(caps: &Caps, text: Option<&str>) -> Markup {
    toasts_with(caps, text, Default::default())
}

/// The messages in `text` as a stack of toasts; nothing when there are none.
pub fn toasts_with(_caps: &Caps, text: Option<&str>, options: ToastOptions) -> Markup {
    let messages = text.map(parse).unwrap_or_default();
    html! {
        @if !messages.is_empty() {
            ol class="nojs-toasts" {
                @for (level, message) in &messages {
                    li class={ "nojs-toast nojs-toast-" (level.as_str()) } role=(if *level == Level::Danger { "alert" } else { "status" }) {
                        span class="nojs-toast-text" { (message) }
                        @if let Some(href) = options.dismiss {
                            a class="nojs-toast-close" href=(href) aria-label={ "Dismiss: " (message) } { "\u{d7}" }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-toasts {
  position: fixed; z-index: 20; inset-inline-end: calc(var(--nojs-space) * 2); bottom: calc(var(--nojs-space) * 2);
  display: grid; gap: var(--nojs-space); width: min(22rem, calc(100vw - 2rem));
  list-style: none; margin: 0; padding: 0;
}
.nojs-toast {
  --nojs-toast-tone: var(--nojs-muted);
  display: flex; align-items: baseline; justify-content: space-between; gap: calc(var(--nojs-space) * 2);
  padding: 0.75rem 1rem; color: var(--nojs-fg); background: var(--nojs-surface);
  border: 1px solid var(--nojs-line); border-inline-start: 4px solid var(--nojs-toast-tone); border-radius: var(--nojs-radius);
  box-shadow: 0 8px 24px color-mix(in srgb, var(--nojs-fg) 14%, transparent);
  animation: nojs-toast-out 0.4s ease-in 5s forwards;
}
.nojs-toast:hover, .nojs-toast:focus-within { animation-play-state: paused; }
.nojs-toast-ok { --nojs-toast-tone: var(--nojs-ok); }
.nojs-toast-warn { --nojs-toast-tone: var(--nojs-warn); }
.nojs-toast-danger { --nojs-toast-tone: var(--nojs-danger); animation: none; }
.nojs-toast-close { color: var(--nojs-muted); text-decoration: none; font-size: 1.25rem; line-height: 1; }
.nojs-toast-close:hover { color: var(--nojs-fg); }
@keyframes nojs-toast-out { to { opacity: 0; visibility: hidden; transform: translateY(0.5rem); } }
@media (prefers-reduced-motion: reduce) { .nojs-toast { animation: none; } }
@media (max-width: 40rem) { .nojs-toasts { bottom: auto; top: calc(var(--nojs-space) * 2); } }
"#;
