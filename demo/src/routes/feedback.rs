//! Feedback: alerts, progress and tooltips, toasts, stats and empty states.

use crate::site::page;
use axum::{Form, Router, routing::get};
use loco_ui::prelude::*;
use serde::Deserialize;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/feedback", get(feedback_page))
        .route("/toast", get(toast_page).post(toast_submit))
        .route("/dashboard", get(dashboard_page))
        .route("/chart", get(chart_page))
        .route("/description-list", get(description_list_page))
}

/// Each page's live component, which the index shows too (`site::preview`).
pub(crate) const PREVIEWS: &[super::Preview] = &[
    ("/feedback", feedback),
    ("/toast", toasts),
    ("/dashboard", stats),
    ("/chart", charts),
    ("/description-list", description_list),
];

fn feedback(ui: &Ui) -> Markup {
    lui! {
            Stack(lui! {
                // code: /feedback
                Alert("Heads up") description="Deploys pause at 18:00 on Fridays.";
                Alert("Payment failed") danger description="The card was declined. Try another one.";
                Alert("Backups are complete") ok;
                Progress(62, 100) label="Uploading photos";
                Progress(0, 0) label="Waiting for the server";
                Meter(83, 0, 100) label="Disk used" low=60 high=80 optimum=0;
                Cluster(lui! {
                    Tooltip("Copy the link", lui! { Button("") icon label="Copy" content=(html! { (Icon::Copy) }); });
                    Separator vertical;
                    Tooltip("Opens in a new tab", lui! { LinkButton("Docs", "/"); }) below;
                });
                Separator label="or";
                // end code
            }) gap=6;
    }
}

async fn feedback_page(ui: Ui) -> Page {
    page(
        &ui,
        "Alerts, progress and tooltips",
        lui! {
            Stack(lui! {
                (feedback(&ui))
                p class="lui-note" { "Hover or tab to the buttons for their tooltips. The meter turns amber past 60 and red past 80 because its best value is 0." }
            }) gap=6;
        },
    )
}

/// The buttons that post for a toast, and the toasts that came back.
fn toasts(ui: &Ui) -> Markup {
    lui! {
            // code: /toast
            form method="post" action="/toast" class="lui-cluster" {
                (ui.button("Send invite").primary().name("kind").value("ok"))
                (ui.button("Copy link").name("kind").value("warn"))
                (ui.button("Sync now").name("kind").value("danger"))
                (ui.button("All three").name("kind").value("all"))
            }
            Toasts dismiss;
            // end code
    }
}

/// Toasts come back from a post like a flash: the one-shot cookie, several at once.
async fn toast_page(ui: Ui) -> Page {
    page(
        &ui,
        "Toasts",
        lui! {
            p { "Each button posts, the server redirects back, and the answer shows in the corner. Calm ones fade after five seconds (hover to keep them); errors stay until dismissed." }
            (toasts(&ui))
        },
    )
}

#[derive(Deserialize)]
struct ToastForm {
    kind: String,
}

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
                Stat("Visitors", "12,480") delta="+8.2%" note="last 7 days";
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
    page(
        &ui,
        "Stats and empty states",
        lui! {
            (stats(&ui))
            h2 { "Recent orders" }
            @if none {
                // code: /dashboard
                EmptyState("No orders yet") icon="\u{1f4e6}" {
                    text() { "Orders show up here as soon as a customer checks out." }
                    link "Show sample orders" "/dashboard";
                }
                // end code
            } @else {
                ul { li { "#1042, Ada Lovelace, 3 items" } li { "#1041, Grace Hopper, 1 item" } li { "#1040, Alan Turing, 2 items" } }
                p class="lui-note" { a href="/dashboard?orders=none" { "See the empty state" } }
            }
        },
    )
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

async fn chart_page(ui: Ui) -> Page {
    page(
        &ui,
        "Charts",
        lui! {
            (charts(&ui))
            p class="lui-note" { "Hover a bar or a point for its value (the browser's own tooltip). The numbers are also a table that screen readers read and that stays when the picture cannot load." }
        },
    )
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

async fn description_list_page(ui: Ui) -> Page {
    page(&ui, "Description list", description_list(&ui))
}
