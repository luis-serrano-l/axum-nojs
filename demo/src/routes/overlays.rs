//! Overlays: dialog and popover menu.

use axum::{Form, Router, routing::post};
use loco_ui::prelude::*;
use serde::Deserialize;

pub(crate) fn routes() -> Router {
    super::pages(PAGES)
        .route("/dialog/delete", post(dialog_delete))
        .route("/popover/signout", post(popover_signout))
}

/// The pages that are their component and a note (`super::pages`).
pub(crate) const PAGES: &[super::Simple] = &[
    (
        "/dialog",
        dialog,
        "Opened by an invoker button; the footer is a real form posting to `/dialog/delete` with a hidden `returns_to` so the server comes back here. Server-opened: [?dialog=confirm](/dialog?dialog=confirm)",
    ),
    (
        "/popover",
        menus,
        "Links, a heading, a disabled item, a submenu that is another popover, and a `<form method=\"post\">` action. Click outside or press Escape to close; the second menu opens end-aligned.",
    ),
    (
        "/context-menu",
        context_menu,
        "A right-click cannot be caught without script, so the menu hangs on a button in the corner.",
    ),
];

fn dialog(ui: &Ui) -> Markup {
    lui! {
            // code: /dialog
            Dialog("Delete account") id="confirm" title="Delete account?" small danger
                confirm=("Delete account", "/dialog/delete") cancel="Keep it" {
                p { "This cannot be undone. Everything you wrote goes with it." }
                Input("reason", "Tell us why (optional)") placeholder="Moving on";
            }
            // end code
    }
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

fn menus(ui: &Ui) -> Markup {
    lui! {
            div class="lui-popover-row" {
                // code: /popover
                Menu("Account") {
                    group "Signed in as Ada";
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
    }
}

/// A menu action: Post/Redirect/Get back to the menu page with a flash.
async fn popover_signout(ui: Ui) -> Redirect {
    ui.redirect("/popover").flash("Signed out (not really)")
}

/// Actions on one thing, behind a "more" button in its corner.
fn context_menu(ui: &Ui) -> Markup {
    lui! {
        div style="max-width: 24rem" {
            // code: /context-menu
            ContextMenu("report.pdf") {
                link "Open" "/table";
                link "Download" "/table.csv";
                separator();
                action "Delete" "/blocks/record/delete" danger;
                body { p { strong { "report.pdf" } } p class="lui-note" { "2.4 MB · edited yesterday" } }
            }
            // end code
        }
    }
}
