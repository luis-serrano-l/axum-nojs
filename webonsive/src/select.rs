//! # Select
//!
//! A `<select>` whose closed state shows the chosen option's full content, no script.
//!
//! **Platform features:**
//! - `<select>` with a `<button>` first child holding `<selectedcontent>` (Chrome 135, Safari
//!   27; Firefox behind flags): the button is the closed control and `<selectedcontent>`
//!   mirrors the picked option's markup into it, so options can carry a swatch or an icon.
//! - `appearance: base-select` on the select and its `::picker(select)` pseudo-element hands
//!   both to author CSS.
//!
//! **Fallback:** without `Caps::BaseSelect` a plain `<select>` with plain options. Older
//! parsers also drop a `<button>` inside `<select>`, so the enhanced markup is only emitted
//! when the browser is known to want it.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, select};
//! let m = select(&Caps::all(), "size", &[("s", html! { "Small" }), ("l", html! { "Large" })], "l");
//! assert!(m.into_string().contains("<selectedcontent>"));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// `options` are `(value, label)`; `selected` is the current value from the server.
pub fn select(caps: &Caps, name: &str, options: &[(&str, Markup)], selected: &str) -> Markup {
    let rich = caps.has(Cap::BaseSelect);
    html! {
        select class="wo-select" name=(name) {
            @if rich {
                button type="button" { selectedcontent {} }
            }
            @for (value, label) in options {
                option value=(value) selected[*value == selected] { (label) }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-select, .wo-select::picker(select) { appearance: base-select; }
.wo-select { min-width: 12rem; }
.wo-select::picker(select) {
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: var(--wo-space) 0;
  background: var(--wo-surface); color: var(--wo-fg); box-shadow: 0 8px 24px color-mix(in srgb, var(--wo-fg) 14%, transparent);
}
.wo-select option { padding: 0.4rem 1rem; }
.wo-select option:hover, .wo-select option:checked { background: var(--wo-bg); }
.wo-select option::checkmark { order: 1; margin-left: auto; }
.wo-swatch { display: inline-block; width: 1em; height: 1em; border-radius: 50%; vertical-align: -0.15em; margin-right: 0.4em; border: 1px solid var(--wo-line); }
"#;
