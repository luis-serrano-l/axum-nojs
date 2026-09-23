//! # Popover menu
//!
//! A dropdown menu that opens on click, closes on outside click or Escape, no script.
//!
//! **Platform features:**
//! - `popover` attribute + `popovertarget` button (Chrome 114, Firefox 125, Safari 17). Light
//!   dismiss and top-layer stacking come for free.
//! - CSS anchor positioning `anchor-name` / `position-anchor` / `position-area`
//!   (Chrome 125, Firefox 147, Safari 26) to place the menu under its button.
//!
//! **Fallback:** without `Caps::Anchor` the popover is UA-centred, which is still usable.
//! Without `Caps::Popover` the menu is a `<details>` dropdown: it opens and closes on click but
//! has no light dismiss.
//!
//! ```rust
//! use webonsive::{Caps, popover_menu};
//! let m = popover_menu(&Caps::all(), "acct", "Account", &[("Profile", "/profile"), ("Sign out", "/logout")]);
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// A button labelled `label` that toggles a menu of `(text, href)` links.
pub fn popover_menu(caps: &Caps, id: &str, label: &str, items: &[(&str, &str)]) -> Markup {
    let list = html! {
        ul {
            @for (text, href) in items {
                li { a href=(href) { (text) } }
            }
        }
    };
    html! {
        @if !caps.has(Cap::Popover) {
            details class="wo-popover wo-popover-details" id=(id) {
                summary { (label) " ▾" }
                nav { (list) }
            }
        } @else if caps.has(Cap::Anchor) {
            div class="wo-popover wo-popover-anchored" style={ "anchor-name: --" (id) } {
                button type="button" popovertarget=(id) { (label) " ▾" }
                nav id=(id) popover style={ "position-anchor: --" (id) } { (list) }
            }
        } @else {
            div class="wo-popover" {
                button type="button" popovertarget=(id) { (label) " ▾" }
                nav id=(id) popover { (list) }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-popover { display: inline-block; position: relative; }
.wo-popover nav {
  margin: 0; padding: var(--wo-space) 0; min-width: 12rem;
  background: var(--wo-surface); color: var(--wo-fg);
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius);
  box-shadow: 0 8px 24px rgb(0 0 0 / 0.15);
}
.wo-popover-anchored nav { position-area: bottom span-right; margin-top: 4px; }
.wo-popover ul { list-style: none; margin: 0; padding: 0; }
.wo-popover li a { display: block; padding: 0.4rem 1rem; color: inherit; text-decoration: none; }
.wo-popover li a:hover { background: var(--wo-bg); }
/* <details> fallback: summary styled as the button, menu absolutely positioned below it. */
.wo-popover-details summary {
  list-style: none; cursor: pointer; background: var(--wo-surface);
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 0.5rem 1rem;
}
.wo-popover-details summary::-webkit-details-marker { display: none; }
.wo-popover-details nav { position: absolute; top: 100%; left: 0; margin-top: 4px; z-index: 10; }
"#;
