//! Feedback: alerts, progress and tooltips, toasts, stats and empty states.

use crate::site::page;
use axum::{Form, Router, routing::get};
use loco_ui::prelude::*;
use serde::Deserialize;

pub(crate) fn routes() -> Router {
    super::pages(PAGES)
        .route("/toast", axum::routing::post(toast_submit))
        .route("/dashboard", get(dashboard_page))
}

/// The pages that are their component and a note (`super::pages`).
pub(crate) const PAGES: &[super::Simple] = &[
    (
        "/feedback",
        feedback,
        "Hover or tab to the buttons for their tooltips. The meter turns amber past 60 and red past 80 because its best value is 0.",
    ),
    (
        "/toast",
        toasts,
        "Each button posts, the server redirects back, and the answer shows in the corner. Calm ones fade after five seconds (hover to keep them); errors stay until dismissed.",
    ),
    (
        "/chart",
        charts,
        "Hover a bar or a point for its value (the browser's own tooltip). The numbers are also a table that screen readers read and that stays when the picture cannot load.",
    ),
    ("/description-list", description_list, ""),
];

/// The other pages' live components, which the index shows too (`site::preview`).
pub(crate) const PREVIEWS: &[super::Preview] = &[("/dashboard", stats)];

fn feedback(ui: &Ui) -> Markup {
    lui! {
        Stack gap=6 {
            // code: /feedback
            Alert("Heads up") description="Deploys pause at 18:00 on Fridays.";
            Alert("Payment failed") danger description="The card was declined. Try another one.";
            Alert("Backups are complete") ok;
            Progress(62, 100) label="Uploading photos";
            Progress(0, 0) label="Waiting for the server";
            Meter(83, 0, 100) label="Disk used" low=60 high=80 optimum=0;
            Cluster {
                Tooltip("Copy the link", lui! { Button("") icon_only aria_label="Copy" body={ (Icon::Copy) }; });
                Separator vertical;
                Tooltip("Opens in a new tab", lui! { LinkButton("Docs", "/"); }) below;
            }
            Separator label="or";
            // end code
        }
    }
}

/// The buttons that post for a toast, and the toasts that came back.
fn toasts(ui: &Ui) -> Markup {
    lui! {
        // code: /toast
        form method="post" action="/toast" class="lui-cluster" {
            Button("Send invite") primary name="kind" value="ok";
            Button("Copy link") name="kind" value="warn";
            Button("Sync now") name="kind" value="danger";
            Button("All three") name="kind" value="all";
        }
        Toasts dismiss;
        // end code
    }
}

#[derive(Deserialize)]
struct ToastForm {
    kind: String,
}

/// Toasts come back from a post like a flash: the one-shot cookie, several at once.
async fn toast_submit(ui: Ui, Form(f): Form<ToastForm>) -> Redirect {
    let wants = |k: &str| f.kind == "all" || f.kind == k;
    // code: /toast
    let mut back = ui.redirect("/toast");
    if wants("ok") {
        back = back.ok("Invite sent to ada@example.org.");
    }
    if wants("warn") {
        back = back.warn("Link copied; it expires in an hour.");
    }
    if wants("danger") {
        back = back.danger("Sync failed: the server did not answer.");
    }
    // end code
    back
}

/// Stat cards; the orders count follows `?orders=none`.
fn stats(ui: &Ui) -> Markup {
    let none = ui.param("orders") == Some("none");
    lui! {
        div class="lui-stat-grid" {
            // code: /dashboard
            Stat("Visitors", "12,480") delta="+8.2%" description="last 7 days" reveal;
            Stat("Orders", if none { "0" } else { "3" }) delta=(if none { "-3" } else { "0" });
            Stat("Error rate", "0.4%") delta="-0.2 pt" down_is_good href="/table";
            Stat("p95 latency", "38 ms") delta="+6 ms" down_is_good;
            // end code
        }
    }
}

/// Stat cards over a list that may be empty (`?orders=none`).
async fn dashboard_page(ui: Ui) -> Page {
    let none = ui.param("orders") == Some("none");
    let body = lui! {
        (stats(&ui))
        h2 { "Recent orders" }
        @if none {
            // code: /dashboard
            EmptyState("No orders yet") icon="\u{1f4e6}" {
                body { "Orders show up here as soon as a customer checks out." }
                link "Show sample orders" "/dashboard";
            }
            // end code
        } @else {
            ul { li { "#1042, Ada Lovelace, 3 items" } li { "#1041, Grace Hopper, 1 item" } li { "#1040, Alan Turing, 2 items" } }
            p class="lui-note" { a href="/dashboard?orders=none" { "See the empty state" } }
        }
    };
    page(&ui, "Stats and empty states", body)
}

/// Bars, a line and a sparkline, drawn on the server as SVG with the data in a hidden table.
fn charts(ui: &Ui) -> Markup {
    lui! {
            // code: /chart
            Chart("Signups") description="New accounts per weekday, this week" {
                point "Mon" 12.0; point "Tue" 18.0; point "Wed" 9.0; point "Thu" 22.0; point "Fri" 15.0;
            }
            Chart("Latency") line unit=" ms" description="p50 response time per day" {
                point "Mon" 41.5; point "Tue" 38.0; point "Wed" 44.2; point "Thu" 36.9; point "Fri" 35.1;
            }
            p { "Revenue " strong { "$48,210" } " "
                Chart("Revenue, last 8 weeks") sparkline {
                    point "1" 30.0; point "2" 34.0; point "3" 31.0; point "4" 38.0;
                    point "5" 36.0; point "6" 41.0; point "7" 44.0; point "8" 48.0;
                }
            }
            // end code
    }
}

/// Terms beside their details, or stacked above them.
fn description_list(ui: &Ui) -> Markup {
    lui! {
        // code: /description-list
        DescriptionList {
            item "Plan" "Team"; item "Seats" "12 of 20";
            item "Renews" "1 October 2026"; item "Status" (ui.badge("Active").ok());
        }
        h3 { "Stacked" }
        DescriptionList stacked {
            item "Billing email" "ada@example.com"; item "Tax id" "ES-B12345678";
        }
        // end code
    }
}
