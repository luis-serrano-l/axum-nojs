//! Blocks: whole pages built from the components, one route each. Their posts land back on
//! the page with a flash; unknown paths get the 404 block through the router's fallback.

use crate::site::page;
use axum::{Router, routing::get};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/blocks/shell", get(shell_page))
        .route("/blocks/auth", get(auth_page))
        .route("/blocks/settings", get(settings_page).post(saved))
        .route("/blocks/record", get(record_page))
        .route("/blocks/record/delete", axum::routing::post(deleted))
        .route("/blocks/dashboard", get(dashboard_page))
        .route("/blocks/error", get(error_page))
}

/// Each page's block, which the index shows too (`site::preview`).
pub(crate) const PREVIEWS: &[super::Preview] = &[
    ("/blocks/shell", app_shell),
    ("/blocks/auth", auth),
    ("/blocks/settings", settings),
    ("/blocks/record", record),
    ("/blocks/dashboard", dashboard),
    ("/blocks/error", error),
];

fn app_shell(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/shell
        AppShell("Acme") user=("Ada Lovelace", "/app/signout") {
            link "Dashboard" "/blocks/dashboard"; link "App shell" "/blocks/shell"; link "Settings" "/blocks/settings";
            body (html! { h2 { "Welcome back" } p { "The sidebar turns into a drawer on narrow screens." } });
        }
        // end code
    }
}

async fn shell_page(ui: Ui) -> Page {
    page(&ui, "App shell", app_shell(&ui))
}

fn auth(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/auth
        AuthPage("Sign in") description="Use the email you signed up with."
            body=(ui.form("/app/signin").email("email", "Email").required().password("password", "Password").required().submit("Sign in").render())
            footer=(html! { "No account? " a href="/app/signin" { "Sign up" } });
        // end code
    }
}

async fn auth_page(ui: Ui) -> Page {
    page(&ui, "Auth page", auth(&ui))
}

fn settings(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/settings
        SettingsPage("Settings") {
            section "Profile" "How other people see you." (ui.form("/blocks/settings").text("name", "Name").value("Ada").submit("Save").render());
            section "Email" "Where we send receipts." (ui.form("/blocks/settings").email("email", "Email").value("ada@example.com").submit("Save").render());
        }
        // end code
    }
}

async fn settings_page(ui: Ui) -> Page {
    page(&ui, "Settings page", html! { (ui.flash()) (settings(&ui)) })
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

async fn record_page(ui: Ui) -> Page {
    page(&ui, "Record page", html! { (ui.flash()) (record(&ui)) })
}

fn dashboard(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/dashboard
        DashboardPage("Overview") description="The last 30 days." {
            stat (ui.stat("Revenue", "$48,210").delta("+12%").description("vs last month"));
            stat (ui.stat("Orders", "1,284").delta("+4%"));
            stat (ui.stat("Refunds", "18").delta("-3").down_is_good());
            body (html! { p class="lui-note" { "A table or a chart goes here." } });
        }
        // end code
    }
}

async fn dashboard_page(ui: Ui) -> Page {
    page(&ui, "Dashboard page", dashboard(&ui))
}

fn error(ui: &Ui) -> Markup {
    lui! {
        // code: /blocks/error
        ErrorPage(404) home="/";
        // end code
    }
}

async fn error_page(ui: Ui) -> Page {
    let body = lui! {
        (error(&ui))
        p class="lui-note" { "Every unknown path answers with this page and a 404, " a href="/no-such-page" { "like this one" } "." }
    };
    page(&ui, "Error page", body)
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
