//! # Dialog
//!
//! A modal opened by a button and closed by a button, with no script.
//!
//! **Platform features:**
//! - `<dialog>` element (baseline 2022) and its `::backdrop`.
//! - Invoker commands: `<button command="show-modal" commandfor="id">` (Chrome 135+,
//!   Firefox 144+, Safari 26.2+). Closing uses `<form method="dialog">` (baseline 2022).
//!
//! **Fallback:** when `Caps` lacks `Invokers`, the opener is a link to `#id` and a `:target`
//! rule shows the dialog as a fixed overlay; a link to `#` closes it. Only one variant is ever
//! in the markup.
//!
//! **Server state:** `open` renders the dialog already open (non-modal, no backdrop), for
//! example from `UiState::dialog()` after a redirect to `?dialog=<id>`.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, dialog, dialog::DialogOptions};
//! let m = dialog(&Caps::all(), "confirm", "Delete", html! { p { "Are you sure?" } }, Default::default());
//! let m = dialog(&Caps::all(), "confirm", "Delete", html! { p { "Are you sure?" } },
//!                DialogOptions::default().open(true).close_label("Keep it"));
//! assert!(m.into_string().contains("<dialog id=\"confirm\" closedby=\"any\" open>"));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps};

/// Options for [`dialog`]; `Default::default()` is a closed dialog with a "Close" button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogOptions<'a> {
    /// Render the dialog already open (non-modal, no backdrop), for example after a
    /// redirect to `?dialog=<id>` read through `UiState::dialog()`.
    pub open: bool,
    /// Label of the closing button.
    pub close_label: &'a str,
}

impl Default for DialogOptions<'_> {
    fn default() -> Self {
        DialogOptions { open: false, close_label: "Close" }
    }
}

impl<'a> DialogOptions<'a> {
    /// Render the dialog already open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Label of the closing button.
    pub fn close_label(mut self, label: &'a str) -> Self {
        self.close_label = label;
        self
    }
}

/// A modal dialog. `id` must be unique on the page; `trigger` is the opening button's label.
pub fn dialog(caps: &Caps, id: &str, trigger: &str, body: Markup, options: DialogOptions) -> Markup {
    let DialogOptions { open, close_label } = options;
    let invokers = caps.has(Cap::Invokers);
    html! {
        div class="wo-dialog" {
            @if invokers {
                button type="button" command="show-modal" commandfor=(id) { (trigger) }
            } @else {
                a class="wo-dialog-open" role="button" href={ "#" (id) } { (trigger) }
            }
            dialog id=(id) closedby="any" open[open] {
                div class="wo-dialog-body" { (body) }
                @if invokers {
                    form method="dialog" class="wo-dialog-actions" {
                        button type="submit" class="wo-primary" { (close_label) }
                    }
                } @else {
                    p class="wo-dialog-actions" { a href="#" role="button" { (close_label) } }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-dialog { display: inline-flex; gap: var(--wo-space); align-items: center; }
.wo-dialog dialog {
  background: var(--wo-surface); color: var(--wo-fg);
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius);
  padding: calc(var(--wo-space) * 3); max-width: 28rem; width: calc(100% - 2rem);
}
.wo-dialog dialog::backdrop { background: color-mix(in srgb, var(--wo-fg) 45%, transparent); }
.wo-dialog dialog h2 { margin-top: 0; }
.wo-dialog-actions { display: flex; justify-content: flex-end; gap: var(--wo-space); margin: var(--wo-space) 0 0; }
.wo-dialog-open, .wo-dialog-actions a[role="button"] {
  display: inline-block; padding: 0.5rem 1rem; border: 1px solid var(--wo-line);
  border-radius: var(--wo-radius); background: var(--wo-surface); color: inherit; text-decoration: none;
}
.wo-dialog-actions a[role="button"] { background: var(--wo-accent); color: var(--wo-on-accent); border-color: transparent; }

/* :target fallback: a dialog that is the URL fragment renders as a fixed overlay. */
.wo-dialog dialog:target {
  display: block; position: fixed; inset: 0; margin: auto; height: fit-content; z-index: 10;
  box-shadow: 0 0 0 100vmax rgb(0 0 0 / 0.45);
}
"#;
