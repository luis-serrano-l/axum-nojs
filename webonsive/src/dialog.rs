//! # Dialog
//!
//! A modal opened by a button and closed by a button, with no script.
//!
//! **Platform features:**
//! - `<dialog>` element (baseline 2022) and its `::backdrop`.
//! - Invoker commands: `<button command="show-modal" commandfor="id">` (Chrome 135+,
//!   Firefox 144+, Safari 26+). Closing uses `command="close"`.
//! - `<form method="dialog">` (baseline 2022) closes the dialog on submit.
//!
//! **Fallback:** the dialog also has an `id`, so a plain link `href="#id"` opens it via a
//! `:target` CSS rule in browsers without invokers. CSS cannot feature-detect the `command`
//! attribute, so the fallback links are hidden behind `@supports (anchor-name: --x)`, which
//! shipped in the same Chrome release as invokers. That proxy is a documented finding.
//!
//! ```rust
//! use maud::html;
//! use webonsive::dialog;
//! let m = dialog("confirm", "Delete", html! { p { "Are you sure?" } });
//! ```

use maud::{Markup, html};

/// A modal dialog. `id` must be unique on the page; `trigger` is the opening button's label.
pub fn dialog(id: &str, trigger: &str, body: Markup) -> Markup {
    html! {
        div class="wo-dialog" {
            button type="button" command="show-modal" commandfor=(id) { (trigger) }
            a class="wo-dialog-fallback" href={ "#" (id) } { (trigger) " (fallback)" }
            dialog id=(id) closedby="any" {
                div class="wo-dialog-body" { (body) }
                form method="dialog" class="wo-dialog-actions" {
                    a href="#" class="wo-dialog-fallback" { "Close (fallback)" }
                    button type="submit" class="wo-primary" { "Close" }
                }
            }
        }
    }
}

pub const CSS: &str = r#"
.wo-dialog { display: inline-flex; gap: var(--wo-space); align-items: center; }
.wo-dialog dialog {
  background: var(--wo-surface); color: var(--wo-fg);
  border: 1px solid var(--wo-line); border-radius: var(--wo-radius);
  padding: calc(var(--wo-space) * 3); max-width: 28rem; width: calc(100% - 2rem);
}
.wo-dialog dialog::backdrop { background: rgb(0 0 0 / 0.45); }
.wo-dialog-actions { display: flex; justify-content: flex-end; gap: var(--wo-space); margin-top: var(--wo-space); }
.wo-dialog-fallback { font-size: 0.85rem; }

/* :target fallback: a dialog that is the URL fragment renders as a fixed overlay. */
.wo-dialog dialog:target {
  display: block; position: fixed; inset: 0; margin: auto; height: fit-content; z-index: 10;
  box-shadow: 0 0 0 100vmax rgb(0 0 0 / 0.45);
}

/* Hide the fallback links once the browser understands invoker commands. */
@supports (anchor-name: --x) { /* proxy: shipped alongside invokers in Chrome 135 */
  .wo-dialog-fallback { display: none; }
}
"#;
