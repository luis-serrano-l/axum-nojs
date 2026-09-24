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
//! **Server state:** `?dialog=<id>` in the URL renders the dialog already open (non-modal, no
//! backdrop), as does `.open(true)`. The confirm form goes back to the page it came from
//! unless `returns_to` says otherwise. `returns_to` is posted
//! with the confirm form as a hidden `returns_to` field, so the handler knows where to send
//! the browser back; check it is a local path before redirecting to it.
//!
//! **Without script:** `closedby` is honoured by the real `<dialog>` only; the `:target`
//! fallback closes through its links alone. Focus is not trapped in the fallback.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let hi = ui.dialog("Say hi").body(html! { p { "Hello." } });
//! assert!(hi.render().into_string().contains(r#"commandfor="say-hi""#), "the id is the trigger's slug");
//!
//! let m = ui.dialog("Delete account")
//!     .id("confirm")
//!     .title("Delete account?")
//!     .small()
//!     .danger()
//!     .confirm("Delete", "/account/delete")
//!     .returns_to("/settings")
//!     .cancel("Keep it")
//!     .closedby("closerequest")
//!     .open(true)
//!     .body(html! {
//!         p { "This cannot be undone." }
//!         label { "Reason " input name="reason"; }
//!     });
//! let html = m.render().into_string();
//! assert!(html.contains("<dialog id=\"confirm\" class=\"nojs-dialog-sm\" closedby=\"closerequest\" aria-labelledby=\"confirm-title\" open>"));
//! assert!(html.contains("<form method=\"post\" action=\"/account/delete\""));
//! assert!(html.contains("name=\"returns_to\" value=\"/settings\""));
//!
//! // From the request: open when the URL says `?dialog=confirm`, and the confirm form
//! // returns to this page.
//! let ui = Ui::from_request("/account", "dialog=confirm", "");
//! let html = ui.dialog("Delete account").id("confirm").confirm("Delete", "/account/delete").render().into_string();
//! assert!(html.contains(" open>") && html.contains("value=\"/account\""));
//! ```

use maud::{Markup, Render, html};

use crate::{Cap, Ui, slug};

/// Width of a dialog: `max-width` of 20, 28 or 40 rem.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum DialogSize {
    Sm,
    #[default]
    Md,
    Lg,
}

impl DialogSize {
    fn class(self) -> &'static str {
        match self {
            DialogSize::Sm => "nojs-dialog-sm",
            DialogSize::Md => "nojs-dialog-md",
            DialogSize::Lg => "nojs-dialog-lg",
        }
    }
}

/// A modal dialog behind a button, made by [`Ui::dialog`]: medium width, untitled, closed by
/// a "Close" button, unless told otherwise.
#[derive(Clone, Debug)]
pub struct Dialog<'a> {
    ui: &'a Ui,
    id: String,
    trigger: &'a str,
    body: Markup,
    open: bool,
    title: Option<&'a str>,
    size: DialogSize,
    danger: bool,
    confirm: Option<(&'a str, &'a str)>,
    returns_to: Option<&'a str>,
    close: &'a str,
    cancel: &'a str,
    closedby: &'a str,
}

impl Ui {
    /// A dialog opened by a button labelled `trigger`. Its id is the trigger's slug
    /// (`"Delete account"` is `delete-account`); it renders open when the URL says
    /// `?dialog=<id>`.
    pub fn dialog<'a>(&'a self, trigger: &'a str) -> Dialog<'a> {
        Dialog {
            ui: self,
            id: slug(trigger),
            trigger,
            body: Markup::default(),
            open: false,
            title: None,
            size: DialogSize::Md,
            danger: false,
            confirm: None,
            returns_to: None,
            close: "Close",
            cancel: "Cancel",
            closedby: "any",
        }
    }
}

impl<'a> Dialog<'a> {
    /// What the dialog says.
    pub fn body(mut self, body: Markup) -> Self {
        self.body = body;
        self
    }

    /// The dialog's id instead of the trigger's slug; it must be unique on the page.
    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }

    /// Render the dialog already open (non-modal, no backdrop).
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// A title in a header with a close control; also the dialog's accessible name.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// 20 rem wide: a confirmation.
    pub fn small(mut self) -> Self {
        self.size = DialogSize::Sm;
        self
    }

    /// 40 rem wide: a long form or a table.
    pub fn large(mut self) -> Self {
        self.size = DialogSize::Lg;
        self
    }

    /// Red confirm button and title rule: for destructive actions.
    pub fn danger(mut self) -> Self {
        self.danger = true;
        self
    }

    /// A confirm button labelled `label`: the footer becomes `<form method="post">` to
    /// `action`, and the body's fields are posted with it.
    pub fn confirm(mut self, label: &'a str, action: &'a str) -> Self {
        self.confirm = Some((label, action));
        self
    }

    /// Where the server should send the browser after the confirm form, posted as a hidden
    /// `returns_to` field; this page by default. Check it is a local path before
    /// redirecting to it.
    pub fn returns_to(mut self, path: &'a str) -> Self {
        self.returns_to = Some(path);
        self
    }

    /// Label of the closing button when there is no confirm form.
    pub fn close(mut self, label: &'a str) -> Self {
        self.close = label;
        self
    }

    /// Label of the cancel button beside the confirm button.
    pub fn cancel(mut self, label: &'a str) -> Self {
        self.cancel = label;
        self
    }

    /// The `closedby` attribute: `any` (Escape and outside click, the default),
    /// `closerequest` (Escape only) or `none`.
    pub fn closedby(mut self, closedby: &'a str) -> Self {
        self.closedby = closedby;
        self
    }
}

impl Render for Dialog<'_> {
    fn render(&self) -> Markup {
        let Dialog { ui, ref id, trigger, ref body, open, title, size, danger, confirm, returns_to, close, cancel, closedby } = *self;
        let open = open || ui.state.dialog() == Some(id);
        let returns_to = returns_to.unwrap_or(ui.state.path());
        let returns_to = (!returns_to.is_empty()).then_some(returns_to);
        let invokers = ui.has(Cap::Invokers);
        let title_id = format!("{id}-title");
        html! {
            div class={ "nojs-dialog" @if danger { " nojs-dialog-danger" } } {
                @if invokers {
                    button type="button" command="show-modal" commandfor=(id) { (trigger) }
                } @else {
                    a class="nojs-dialog-open" role="button" href={ "#" (id) } { (trigger) }
                }
                dialog id=(id) class=(size.class()) closedby=(closedby) aria-labelledby=[title.map(|_| &title_id)] open[open] {
                    @if let Some(t) = title { h2 id=(title_id) class="nojs-dialog-title" { (t) } }
                    @if let Some((label, action)) = confirm {
                        form method="post" action=(action) class="nojs-dialog-form" {
                            div class="nojs-dialog-body" { (body) }
                            @if let Some(to) = returns_to { input type="hidden" name="returns_to" value=(to); }
                            div class="nojs-dialog-actions" {
                                @if invokers {
                                    button type="button" command="close" commandfor=(id) { (cancel) }
                                } @else {
                                    a href="#" role="button" class="nojs-dialog-cancel" { (cancel) }
                                }
                                button type="submit" class={ @if danger { "nojs-danger" } @else { "nojs-primary" } } { (label) }
                            }
                        }
                    } @else {
                        div class="nojs-dialog-body" { (body) }
                        @if invokers {
                            form method="dialog" class="nojs-dialog-actions" {
                                button type="submit" class="nojs-primary" { (close) }
                            }
                        } @else {
                            p class="nojs-dialog-actions" { a href="#" role="button" { (close) } }
                        }
                    }
                    // Last in the markup so the dialog's focusing steps skip it for the first field.
                    @if title.is_some() {
                        @if invokers {
                            button type="button" command="close" commandfor=(id) class="nojs-dialog-close" aria-label="Close" { "\u{d7}" }
                        } @else {
                            a href="#" class="nojs-dialog-close" aria-label="Close" { "\u{d7}" }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-dialog { display: inline-flex; gap: var(--nojs-space); align-items: center; }
.nojs-dialog dialog {
  background: var(--nojs-surface); color: var(--nojs-fg);
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
  padding: calc(var(--nojs-space) * 3); width: calc(100% - 2rem);
}
/* Server-opened (non-modal) dialogs sit in the flow; positioned so the close control anchors. */
.nojs-dialog dialog:not(:modal):not(:target) { position: relative; }
.nojs-dialog-sm { max-width: 20rem; }
.nojs-dialog-md { max-width: 28rem; }
.nojs-dialog-lg { max-width: 40rem; }
.nojs-dialog dialog::backdrop { background: color-mix(in srgb, var(--nojs-fg) 45%, transparent); }
.nojs-dialog dialog h2 { margin-top: 0; }
.nojs-dialog-title {
  font-size: 1.25rem; margin: 0 2rem calc(var(--nojs-space) * 2) 0; padding-bottom: var(--nojs-space);
  border-bottom: 1px solid var(--nojs-line);
}
.nojs-dialog-danger .nojs-dialog-title { border-bottom-color: var(--nojs-danger); }
.nojs-dialog-body > :last-child { margin-bottom: 0; }
.nojs-dialog-body label { display: block; margin: var(--nojs-space) 0; }
.nojs-dialog-body input:not([type=hidden]), .nojs-dialog-body textarea { display: block; width: 100%; box-sizing: border-box; margin-top: 0.25rem; }
.nojs-dialog-actions { display: flex; justify-content: flex-end; gap: var(--nojs-space); margin: calc(var(--nojs-space) * 2) 0 0; }
.nojs-dialog-open, .nojs-dialog-actions a[role="button"] {
  display: inline-block; padding: 0.5rem 1rem; border: 1px solid var(--nojs-line);
  border-radius: var(--nojs-radius); background: var(--nojs-surface); color: inherit; text-decoration: none;
}
.nojs-dialog-actions a[role="button"]:not(.nojs-dialog-cancel) { background: var(--nojs-accent); color: var(--nojs-on-accent); border-color: transparent; }
.nojs-dialog-close {
  position: absolute; top: calc(var(--nojs-space) * 2); right: calc(var(--nojs-space) * 2);
  width: 2rem; height: 2rem; padding: 0; line-height: 1; font-size: 1.25rem;
  display: inline-flex; align-items: center; justify-content: center;
  color: var(--nojs-muted); background: none; border: 1px solid transparent; border-radius: var(--nojs-radius); text-decoration: none;
}
.nojs-dialog-close:hover { color: var(--nojs-fg); border-color: var(--nojs-line); }

/* :target fallback: a dialog that is the URL fragment renders as a fixed overlay. */
.nojs-dialog dialog:target {
  display: block; position: fixed; inset: 0; margin: auto; height: fit-content; z-index: 10;
  box-shadow: 0 0 0 100vmax color-mix(in srgb, var(--nojs-fg) 45%, transparent);
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_opens_the_named_dialog_and_returns_to_its_page() {
        let ui = Ui::from_request("/dialog", "dialog=confirm", "");
        let with = |id: &str, to: Option<&str>| {
            let d = ui.dialog("Open").id(id).confirm("Go", "/go");
            to.map_or(d.clone(), |to| d.returns_to(to)).render().into_string()
        };
        let named = with("confirm", None);
        assert!(named.contains(" open>") && named.contains(r#"name="returns_to" value="/dialog""#));
        assert!(!with("other", None).contains(" open>"));
        let explicit = with("confirm", Some("/home"));
        assert!(explicit.contains(r#"value="/home""#) && !explicit.contains(r#"value="/dialog""#));
        assert!(!Ui::default().dialog("Open").confirm("Go", "/go").render().into_string().contains("returns_to"), "no request, no return path");
    }
}
