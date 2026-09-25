//! Overlays: dialog and popover menu.

use crate::site::page;
use axum::{
    Form, Router,
    routing::{get, post},
};
use axum_nojs::prelude::*;
use serde::Deserialize;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/dialog", get(dialog_page))
        .route("/dialog/delete", post(dialog_delete))
        .route("/popover", get(popover_page))
        .route("/popover/signout", post(popover_signout))
}

async fn dialog_page(ui: Ui) -> Page {
    page(
        &ui,
        "Dialog",
        nojs! {
            (ui.flash())
            // code: /dialog
            Dialog("Delete account") id="confirm" title="Delete account?" small danger
                confirm=("Delete account", "/dialog/delete") cancel="Keep it" {
                p { "This cannot be undone. Everything you wrote goes with it." }
                Input("reason", "Tell us why (optional)") placeholder="Moving on";
            }
            // end code
            p class="nojs-note" { "Opened by an invoker button; the footer is a real form posting to " code { "/dialog/delete" } " with a hidden " code { "returns_to" } " so the server comes back here. Server-opened: " a href="/dialog?dialog=confirm" { "?dialog=confirm" } }
        },
    )
}

#[derive(Deserialize)]
struct DeleteForm {
    #[serde(default)]
    reason: String,
    #[serde(default)]
    returns_to: String,
}

/// The confirm form's target: only ever redirects to a local path from `returns_to`.
async fn dialog_delete(ui: Ui, Form(f): Form<DeleteForm>) -> Redirect {
    let local = f.returns_to.starts_with('/') && !f.returns_to.starts_with("//");
    let msg = if f.reason.is_empty() {
        "Account deleted (not really)".to_string()
    } else {
        format!("Account deleted (not really). Reason: {}", f.reason)
    };
    ui.redirect(if local { &f.returns_to } else { "/dialog" })
        .flash(&msg)
}

async fn popover_page(ui: Ui) -> Page {
    page(
        &ui,
        "Popover menu",
        nojs! {
            (ui.flash())
            div class="nojs-popover-row" {
                // code: /popover
                Menu("Account") {
                    heading "Signed in as Ada";
                    link "Profile" "/popover" icon="@" shortcut="g p";
                    link "Settings" "/settings" icon="\u{2699}" shortcut="g s";
                    link "Billing" "/popover" icon="$" disabled;
                    separator();
                    submenu "Theme" ([("Light", "/popover?theme=light"), ("Dark", "/popover?theme=dark")]) icon="\u{25d0}";
                    separator();
                    action "Sign out" "/popover/signout" icon="\u{2192}" danger;
                }
                Menu("More") align_end {
                    link "Documentation" "/" icon="?";
                    action "Clear cache" "/popover/signout";
                }
                // end code
            }
            p class="nojs-note" { "Links, a heading, a disabled item, a submenu that is another popover, and a " code { "<form method=\"post\">" } " action. Click outside or press Escape to close; the second menu opens end-aligned." }
        },
    )
}

/// A menu action: Post/Redirect/Get back to the menu page with a flash.
async fn popover_signout(ui: Ui) -> Redirect {
    ui.redirect("/popover").flash("Signed out (not really)")
}
