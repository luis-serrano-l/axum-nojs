//! # Flash
//!
//! A one-shot status message that survives a Post/Redirect/Get, no script.
//!
//! **Platform features:** a cookie with `Max-Age=60` set by [`crate::state::prg`] on the
//! redirect, read on the next request through `UiState`, and cleared by returning that
//! `UiState` with the response. `role="status"` announces it to assistive technology.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use webonsive::{Caps, flash};
//! let m = flash(&Caps::all(), Some("Saved."));
//! assert!(m.into_string().contains("Saved."));
//! assert_eq!(flash(&Caps::all(), None).into_string(), "");
//! ```

use maud::{Markup, html};

use crate::Caps;

/// Render `message` as a status banner; nothing when there is no message.
pub fn flash(_caps: &Caps, message: Option<&str>) -> Markup {
    html! {
        @if let Some(m) = message {
            p class="wo-flash" role="status" { (m) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-flash {
  padding: 0.6rem 1rem; border-radius: var(--wo-radius);
  background: color-mix(in srgb, var(--wo-accent) 15%, var(--wo-surface));
  border: 1px solid var(--wo-accent); color: var(--wo-fg);
}
"#;
