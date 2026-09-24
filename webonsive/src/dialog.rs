//! # Dialog
//!
//! A modal opened by a button and closed by a button, with no script. It can carry a title
//! with a close control, a size, a `danger` variant, and a footer that is a real form: a
//! confirm button posting to a URL beside a cancel button, so "Delete account?" is one round
//! trip and the server redirects back to the page that opened it.
//!
//! **Platform features:**
//! - `<dialog>` element (baseline 2022), its `::backdrop`, and `closedby` (Chrome 134+,
//!   Firefox 141+, Safari 26+) for Escape and light dismiss.
//! - Invoker commands: `<button command="show-modal" commandfor="id">` and `command="close"`
//!   (Chrome 135+, Firefox 144+, Safari 26.2+). Closing without a confirm form uses
//!   `<form method="dialog">` (baseline 2022).
//! - Focus: the close control sits last in the markup, so the dialog's own focusing steps land
//!   on the first field in the body, then on the confirm button.
//!
//! **What it does not do without script:** return focus to the opener in the `:target`
//! fallback, trap focus there, or post a form and close without reloading.
//!
//! **Fallback:** when `Caps` lacks `Invokers`, the opener is a link to `#id` and a `:target`
//! rule shows the dialog as a fixed overlay; links to `#` close and cancel it. Only one variant
//! is ever in the markup.
//!
//! **Server state:** `open` renders the dialog already open (non-modal, no backdrop).
//! `state(&ui_state)` does that from `?dialog=<id>` itself, and sends the confirm form back
//! to the page it came from unless `returns_to` says otherwise. `returns_to` is posted
//! with the confirm form as a hidden `returns_to` field, so the handler knows where to send
//! the browser back; check it is a local path before redirecting to it.
//!
//! **Without script:** `closedby` is honoured by the real `<dialog>` only; the `:target`
//! fallback closes through its links alone. Focus is not trapped in the fallback.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, UiState, dialog, dialog_with, dialog::{DialogOptions, DialogSize}};
//! let m = dialog(&Caps::all(), "hi", "Say hi", html! { p { "Hello." } });
//! let m = dialog_with(&Caps::all(), "confirm", "Delete account", html! {
//!         p { "This cannot be undone." }
//!         label { "Reason " input name="reason"; }
//!     },
//!     DialogOptions::default()
//!         .title("Delete account?")
//!         .size(DialogSize::Sm)
//!         .danger()
//!         .confirm("Delete", "/account/delete")
//!         .returns_to("/settings")
//!         .cancel_label("Keep it")
//!         .closedby("closerequest")
//!         .open(true));
//! let html = m.into_string();
//! assert!(html.contains("<dialog id=\"confirm\" class=\"wo-dialog-sm\" closedby=\"closerequest\" aria-labelledby=\"confirm-title\" open>"));
//! assert!(html.contains("<form method=\"post\" action=\"/account/delete\""));
//! assert!(html.contains("name=\"returns_to\" value=\"/settings\""));
//!
//! // With the request's state: open when the URL says `?dialog=confirm`, and the
//! // confirm form returns to this page.
//! let state = UiState::parse("/account", "dialog=confirm", "");
//! let html = dialog_with(&Caps::all(), "confirm", "Delete account", html! {},
//!     DialogOptions::default().confirm("Delete", "/account/delete").state(&state)).into_string();
//! assert!(html.contains(" open>") && html.contains("value=\"/account\""));
//! ```

use maud::{Markup, html};

use crate::{Cap, Caps, UiState};

/// Width of a [`dialog`]: `max-width` of 20, 28 or 40 rem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DialogSize {
    /// 20 rem: a confirmation.
    Sm,
    /// 28 rem: a short form.
    #[default]
    Md,
    /// 40 rem: a long form or a table.
    Lg,
}

impl DialogSize {
    fn class(self) -> &'static str {
        match self {
            DialogSize::Sm => "wo-dialog-sm",
            DialogSize::Md => "wo-dialog-md",
            DialogSize::Lg => "wo-dialog-lg",
        }
    }
}

/// Options for [`dialog`]; `Default::default()` is a closed, medium, untitled dialog with a
/// "Close" button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DialogOptions<'a> {
    /// Render the dialog already open (non-modal, no backdrop), for example after a
    /// redirect to `?dialog=<id>` read through `UiState::dialog()`.
    pub open: bool,
    /// Title shown in a header with a close control; also the dialog's accessible name.
    pub title: Option<&'a str>,
    /// Width.
    pub size: DialogSize,
    /// Red confirm button and a red title rule: for destructive actions.
    pub danger: bool,
    /// A confirm button `(label, action)`: the footer becomes `<form method="post">` to that
    /// URL and the body's fields are posted with it.
    pub confirm: Option<(&'a str, &'a str)>,
    /// Posted as a hidden `returns_to` field with the confirm form.
    pub returns_to: Option<&'a str>,
    /// Label of the closing button when there is no confirm form.
    pub close_label: &'a str,
    /// Label of the cancel button beside the confirm button.
    pub cancel_label: &'a str,
    /// The `closedby` attribute: `any` (Escape and outside click), `closerequest` (Escape
    /// only) or `none`.
    pub closedby: &'a str,
    /// The request's state: opens the dialog when `?dialog=<id>` names it, and is where the
    /// confirm form returns when `returns_to` is not set.
    pub state: Option<&'a UiState>,
}

impl Default for DialogOptions<'_> {
    fn default() -> Self {
        DialogOptions {
            open: false,
            title: None,
            size: DialogSize::Md,
            danger: false,
            confirm: None,
            returns_to: None,
            close_label: "Close",
            cancel_label: "Cancel",
            closedby: "any",
            state: None,
        }
    }
}

impl<'a> DialogOptions<'a> {
    /// Render the dialog already open.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Title in a header with a close control.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Width.
    pub fn size(mut self, size: DialogSize) -> Self {
        self.size = size;
        self
    }

    /// Destructive styling.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// A confirm button with this label, posting the body's fields to `action`.
    pub fn confirm(mut self, label: &'a str, action: &'a str) -> Self {
        self.confirm = Some((label, action));
        self
    }

    /// Where the server should redirect after the confirm form; posted as `returns_to`.
    pub fn returns_to(mut self, path: &'a str) -> Self {
        self.returns_to = Some(path);
        self
    }

    /// Label of the closing button.
    pub fn close_label(mut self, label: &'a str) -> Self {
        self.close_label = label;
        self
    }

    /// Label of the cancel button beside a confirm button.
    pub fn cancel_label(mut self, label: &'a str) -> Self {
        self.cancel_label = label;
        self
    }

    /// `any`, `closerequest` or `none`.
    pub fn closedby(mut self, closedby: &'a str) -> Self {
        self.closedby = closedby;
        self
    }

    /// Read `?dialog=<id>` to open it, and return the confirm form to this page.
    pub fn state(mut self, state: &'a UiState) -> Self {
        self.state = Some(state);
        self
    }
}

/// A dialog opened by a button labelled `trigger`, with the default options.
/// [`dialog_with`] takes the options.
pub fn dialog(caps: &Caps, id: &str, trigger: &str, body: Markup) -> Markup {
    dialog_with(caps, id, trigger, body, Default::default())
}

/// A modal dialog. `id` must be unique on the page; `trigger` is the opening button's label.
pub fn dialog_with(caps: &Caps, id: &str, trigger: &str, body: Markup, options: DialogOptions) -> Markup {
    let DialogOptions { open, title, size, danger, confirm, returns_to, close_label, cancel_label, closedby, state } = options;
    let open = open || state.is_some_and(|s| s.dialog() == Some(id));
    let returns_to = returns_to.or(state.map(UiState::path));
    let invokers = caps.has(Cap::Invokers);
    let title_id = format!("{id}-title");
    html! {
        div class={ "wo-dialog" @if danger { " wo-dialog-danger" } } {
            @if invokers {
                button type="button" command="show-modal" commandfor=(id) { (trigger) }
            } @else {
                a class="wo-dialog-open" role="button" href={ "#" (id) } { (trigger) }
            }
            dialog id=(id) class=(size.class()) closedby=(closedby) aria-labelledby=[title.map(|_| &title_id)] open[open] {
                @if let Some(t) = title { h2 id=(title_id) class="wo-dialog-title" { (t) } }
                @if let Some((label, action)) = confirm {
                    form method="post" action=(action) class="wo-dialog-form" {
                        div class="wo-dialog-body" { (body) }
                        @if let Some(to) = returns_to { input type="hidden" name="returns_to" value=(to); }
                        div class="wo-dialog-actions" {
                            @if invokers {
                                button type="button" command="close" commandfor=(id) { (cancel_label) }
                            } @else {
                                a href="#" role="button" class="wo-dialog-cancel" { (cancel_label) }
                            }
                            button type="submit" class={ @if danger { "wo-danger" } @else { "wo-primary" } } { (label) }
                        }
                    }
                } @else {
                    div class="wo-dialog-body" { (body) }
                    @if invokers {
                        form method="dialog" class="wo-dialog-actions" {
                            button type="submit" class="wo-primary" { (close_label) }
                        }
                    } @else {
                        p class="wo-dialog-actions" { a href="#" role="button" { (close_label) } }
                    }
                }
                // Last in the markup so the dialog's focusing steps skip it for the first field.
                @if title.is_some() {
                    @if invokers {
                        button type="button" command="close" commandfor=(id) class="wo-dialog-close" aria-label="Close" { "\u{d7}" }
                    } @else {
                        a href="#" class="wo-dialog-close" aria-label="Close" { "\u{d7}" }
                    }
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
  padding: calc(var(--wo-space) * 3); width: calc(100% - 2rem);
}
/* Server-opened (non-modal) dialogs sit in the flow; positioned so the close control anchors. */
.wo-dialog dialog:not(:modal):not(:target) { position: relative; }
.wo-dialog-sm { max-width: 20rem; }
.wo-dialog-md { max-width: 28rem; }
.wo-dialog-lg { max-width: 40rem; }
.wo-dialog dialog::backdrop { background: color-mix(in srgb, var(--wo-fg) 45%, transparent); }
.wo-dialog dialog h2 { margin-top: 0; }
.wo-dialog-title {
  font-size: 1.25rem; margin: 0 2rem calc(var(--wo-space) * 2) 0; padding-bottom: var(--wo-space);
  border-bottom: 1px solid var(--wo-line);
}
.wo-dialog-danger .wo-dialog-title { border-bottom-color: var(--wo-danger); }
.wo-dialog-body > :last-child { margin-bottom: 0; }
.wo-dialog-body label { display: block; margin: var(--wo-space) 0; }
.wo-dialog-body input:not([type=hidden]), .wo-dialog-body textarea { display: block; width: 100%; box-sizing: border-box; margin-top: 0.25rem; }
.wo-dialog-actions { display: flex; justify-content: flex-end; gap: var(--wo-space); margin: calc(var(--wo-space) * 2) 0 0; }
.wo-dialog-open, .wo-dialog-actions a[role="button"] {
  display: inline-block; padding: 0.5rem 1rem; border: 1px solid var(--wo-line);
  border-radius: var(--wo-radius); background: var(--wo-surface); color: inherit; text-decoration: none;
}
.wo-dialog-actions a[role="button"]:not(.wo-dialog-cancel) { background: var(--wo-accent); color: var(--wo-on-accent); border-color: transparent; }
.wo-dialog-close {
  position: absolute; top: calc(var(--wo-space) * 2); right: calc(var(--wo-space) * 2);
  width: 2rem; height: 2rem; padding: 0; line-height: 1; font-size: 1.25rem;
  display: inline-flex; align-items: center; justify-content: center;
  color: var(--wo-muted); background: none; border: 1px solid transparent; border-radius: var(--wo-radius); text-decoration: none;
}
.wo-dialog-close:hover { color: var(--wo-fg); border-color: var(--wo-line); }

/* :target fallback: a dialog that is the URL fragment renders as a fixed overlay. */
.wo-dialog dialog:target {
  display: block; position: fixed; inset: 0; margin: auto; height: fit-content; z-index: 10;
  box-shadow: 0 0 0 100vmax color-mix(in srgb, var(--wo-fg) 45%, transparent);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_opens_the_named_dialog_and_returns_to_its_page() {
        let state = UiState::parse("/dialog", "dialog=confirm", "");
        let with = |id: &str, options: DialogOptions| dialog_with(&Caps::all(), id, "Open", html! {}, options.confirm("Go", "/go").state(&state)).into_string();
        let named = with("confirm", DialogOptions::default());
        assert!(named.contains(" open>") && named.contains(r#"name="returns_to" value="/dialog""#));
        let other = with("other", DialogOptions::default());
        assert!(!other.contains(" open>"));
        let explicit = with("confirm", DialogOptions::default().returns_to("/home"));
        assert!(explicit.contains(r#"value="/home""#) && !explicit.contains(r#"value="/dialog""#));
    }
}
