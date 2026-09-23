//! # Color
//!
//! A colour picker whose value the server remembers, no script.
//!
//! **Platform features:** `<input type="color">` (Chrome 20, Firefox 29, Safari 12.1). The
//! swatch beside it is a plain `<span>` painted with the server's current value through an
//! inline `background`, so the page shows what was saved, not what is being picked.
//!
//! **Fallback:** none needed; a browser without a colour picker shows a text field that
//! accepts `#rrggbb`.
//!
//! ```rust
//! use webonsive::{Caps, color};
//! let m = color(&Caps::all(), "accent", "#2f5bea");
//! assert!(m.into_string().contains("type=\"color\""));
//! ```

use maud::{Markup, html};

use crate::Caps;

/// A colour input named `name` with the current `#rrggbb` value and a swatch of it.
pub fn color(_caps: &Caps, name: &str, value: &str) -> Markup {
    let id = format!("f-{name}");
    html! {
        div class="wo-color" {
            input type="color" id=(id) name=(name) value=(value);
            span class="wo-color-swatch" style={ "background: " (value) } aria-hidden="true" {}
            code { (value) }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-color { display: inline-flex; align-items: center; gap: var(--wo-space); }
.wo-color input { width: 3rem; height: 2.25rem; padding: 2px; }
.wo-color-swatch { width: 2rem; height: 2rem; border-radius: var(--wo-radius); border: 1px solid var(--wo-line); }
"#;
