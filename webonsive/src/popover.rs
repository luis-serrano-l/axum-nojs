//! # Popover menu
//!
//! A dropdown menu that opens on click, closes on outside click or Escape, no script.
//!
//! **Platform features:**
//! - `popover` attribute + `popovertarget` button (baseline 2024). Light dismiss and
//!   top-layer stacking come for free.
//! - CSS anchor positioning `anchor-name` / `position-anchor` / `position-area`
//!   (Chrome 125+, Safari 26+; Firefox behind flag) to place the menu under its button.
//!
//! **Fallback:** without anchor positioning the popover renders centred by the UA stylesheet,
//! which is still usable.
//!
//! ```rust
//! use webonsive::popover_menu;
//! let m = popover_menu("acct", "Account", &[("Profile", "/profile"), ("Sign out", "/logout")]);
//! ```

use maud::{Markup, html};

/// A button labelled `label` that toggles a menu of `(text, href)` links.
pub fn popover_menu(id: &str, label: &str, items: &[(&str, &str)]) -> Markup {
    html! {
        div class="wo-popover" style={ "anchor-name: --" (id) } {
            button type="button" popovertarget=(id) { (label) " ▾" }
            nav id=(id) popover style={ "position-anchor: --" (id) } {
                ul {
                    @for (text, href) in items {
                        li { a href=(href) { (text) } }
                    }
                }
            }
        }
    }
}

pub const CSS: &str = r#"
.wo-popover { display: inline-block; position: relative; }
.wo-popover [popover] {
  margin: 0; padding: var(--wo-space) 0; min-width: 12rem;
  background: var(--wo-surface); color: var(--wo-fg);
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius);
  box-shadow: 0 8px 24px rgb(0 0 0 / 0.15);
  position-area: bottom span-right; margin-top: 4px;
}
.wo-popover ul { list-style: none; margin: 0; padding: 0; }
.wo-popover li a { display: block; padding: 0.4rem 1rem; color: inherit; text-decoration: none; }
.wo-popover li a:hover { background: var(--wo-bg); }
"#;
