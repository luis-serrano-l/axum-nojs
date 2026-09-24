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
//! (`ToastOptions::dismiss`) clears them sooner. `/wo/enhance.js` carries the list across a
//! swap like the flash.
//!
//! **What it does not do without script:** it cannot appear without a request (a toast is the
//! answer to a round trip), and dismissing one is a navigation, not an instant removal.
//!
//! ```rust
//! use webonsive::{Caps, toasts, toast::ToastOptions, flash::{Level, stack}};
//! let text = stack(&[(Level::Ok, "Invite sent."), (Level::Danger, "Mail server down.")]);
//! let m = toasts(&Caps::all(), Some(&text), Default::default()).into_string();
//! assert!(m.contains("wo-toast-ok") && m.contains(r#"role="alert""#));
//! let m = toasts(&Caps::all(), Some("Copied."), ToastOptions::default().dismiss("/toast")).into_string();
//! assert!(m.contains(r#"href="/toast""#));
//! assert_eq!(toasts(&Caps::all(), None, Default::default()).into_string(), "");
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

/// The messages in `text` as a stack of toasts; nothing when there are none.
pub fn toasts(_caps: &Caps, text: Option<&str>, options: ToastOptions) -> Markup {
    let messages = text.map(parse).unwrap_or_default();
    html! {
        @if !messages.is_empty() {
            ol class="wo-toasts" {
                @for (level, message) in &messages {
                    li class={ "wo-toast wo-toast-" (level.as_str()) } role=(if *level == Level::Danger { "alert" } else { "status" }) {
                        span class="wo-toast-text" { (message) }
                        @if let Some(href) = options.dismiss {
                            a class="wo-toast-close" href=(href) aria-label={ "Dismiss: " (message) } { "\u{d7}" }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-toasts {
  position: fixed; z-index: 20; inset-inline-end: calc(var(--wo-space) * 2); bottom: calc(var(--wo-space) * 2);
  display: grid; gap: var(--wo-space); width: min(22rem, calc(100vw - 2rem));
  list-style: none; margin: 0; padding: 0;
}
.wo-toast {
  --wo-toast-tone: var(--wo-muted);
  display: flex; align-items: baseline; justify-content: space-between; gap: calc(var(--wo-space) * 2);
  padding: 0.75rem 1rem; color: var(--wo-fg); background: var(--wo-surface);
  border: 1px solid var(--wo-line); border-inline-start: 4px solid var(--wo-toast-tone); border-radius: var(--wo-radius);
  box-shadow: 0 8px 24px color-mix(in srgb, var(--wo-fg) 14%, transparent);
  animation: wo-toast-out 0.4s ease-in 5s forwards;
}
.wo-toast:hover, .wo-toast:focus-within { animation-play-state: paused; }
.wo-toast-ok { --wo-toast-tone: var(--wo-ok); }
.wo-toast-warn { --wo-toast-tone: var(--wo-warn); }
.wo-toast-danger { --wo-toast-tone: var(--wo-danger); animation: none; }
.wo-toast-close { color: var(--wo-muted); text-decoration: none; font-size: 1.25rem; line-height: 1; }
.wo-toast-close:hover { color: var(--wo-fg); }
@keyframes wo-toast-out { to { opacity: 0; visibility: hidden; transform: translateY(0.5rem); } }
@media (prefers-reduced-motion: reduce) { .wo-toast { animation: none; } }
@media (max-width: 40rem) { .wo-toasts { bottom: auto; top: calc(var(--wo-space) * 2); } }
"#;
