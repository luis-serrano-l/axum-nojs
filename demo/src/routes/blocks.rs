//! Blocks: whole pages built from the components, one route each. Their posts land back on
//! the page with a flash; unknown paths get the 404 block through the router's fallback.

use axum::{Router, routing::post};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    super::pages(PAGES)
        .route("/blocks/settings", post(saved))
        .route("/blocks/record/delete", post(deleted))
}

/// Every block's page: the block, and a note (`super::pages`).
pub(crate) const PAGES: &[super::Simple] = &[
    ("/blocks/shell", app_shell, ""),
    ("/blocks/auth", auth, ""),
    ("/blocks/settings", settings, ""),
    ("/blocks/record", record, ""),
    ("/blocks/dashboard", dashboard, ""),
    (
        "/blocks/error",
        error,
        "Every unknown path answers with this page and a 404, [like this one](/no-such-page).",
    ),
];

fn app_shell(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/shell
        AppShell("Acme") user=("Ada Lovelace", "/app/signout") {
            link "Dashboard" "/blocks/dashboard"; link "App shell" "/blocks/shell"; link "Settings" "/blocks/settings";
            body { h2 { "Welcome back" } p { "The sidebar turns into a drawer on narrow screens." } }
        }
        // end code
    }
}

fn auth(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/auth
        AuthPage("Sign in") description="Use the email you signed up with."
            footer={ "No account? " a href="/app/signin" { "Sign up" } } {
            Form("/app/signin") submit="Sign in" { email "email" "Email" required; password "password" "Password" required; }
        }
        // end code
    }
}

fn settings(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/settings
        SettingsPage("Settings") {
            section "Profile" "How other people see you." { Form("/blocks/settings") submit="Save" { text "name" "Name" value="Ada"; } }
            section "Email" "Where we send receipts." { Form("/blocks/settings") submit="Save" { email "email" "Email" value="ada@example.com"; } }
        }
        // end code
    }
}

fn record(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/record
        RecordPage("Invoice 42") back="/blocks/dashboard" edit="/form" delete="/blocks/record/delete" {
            field "Customer" "Ada Lovelace"; field "Issued" "24 September 2026"; field "Total" "€1,280.00";
            field "Status" (ui.badge("Paid").ok());
        }
        // end code
    }
}

fn dashboard(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/dashboard
        DashboardPage("Overview") description="The last 30 days." {
            stat (ui.stat("Revenue", "$48,210").delta("+12%").description("vs last month"));
            stat (ui.stat("Orders", "1,284").delta("+4%"));
            stat (ui.stat("Refunds", "18").delta("-3").down_is_good());
            body { p class="lui-note" { "A table or a chart goes here." } }
        }
        // end code
    }
}

fn error(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/error
        ErrorPage(404) home="/";
        // end code
    }
}

/// The settings forms post here: back to the page with a note.
async fn saved(ui: Ui) -> Redirect {
    ui.redirect("/blocks/settings").ok("Saved.")
}

/// Deleting the demo invoice deletes nothing: back with a note.
async fn deleted(ui: Ui) -> Redirect {
    ui.redirect("/blocks/record")
        .ok("Deleted (not really: this is the demo).")
}
