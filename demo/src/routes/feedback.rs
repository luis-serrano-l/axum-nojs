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
}

async fn feedback_page(ui: Ui) -> Page {
    page(
        &ui,
        "Alerts, progress and tooltips",
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
                p class="lui-note" { "Hover or tab to the buttons for their tooltips. The meter turns amber past 60 and red past 80 because its best value is 0." }
            }) gap=6;
        },
    )
}

/// Toasts come back from a post like a flash: the one-shot cookie, several at once.
async fn toast_page(ui: Ui) -> Page {
    page(
        &ui,
        "Toasts",
        lui! {
            p { "Each button posts, the server redirects back, and the answer shows in the corner. Calm ones fade after five seconds (hover to keep them); errors stay until dismissed." }
            form method="post" action="/toast" class="lui-cluster" {
                (ui.button("Send invite").primary().name("kind").value("ok"))
                (ui.button("Copy link").name("kind").value("warn"))
                (ui.button("Sync now").name("kind").value("danger"))
                (ui.button("All three").name("kind").value("all"))
            }
            // code: /toast
            Toasts dismiss;
            // end code
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

/// Stat cards over a list that may be empty (`?orders=none`).
async fn dashboard_page(ui: Ui) -> Page {
    let none = ui.param("orders") == Some("none");
    page(
        &ui,
        "Stats and empty states",
        lui! {
            div class="lui-stat-grid" {
                // code: /dashboard
                Stat("Visitors", "12,480") delta="+8.2%" note="last 7 days";
                Stat("Orders", if none { "0" } else { "3" }) delta=(if none { "-3" } else { "0" });
                Stat("Error rate", "0.4%") delta="-0.2 pt" down_is_good href="/table";
                Stat("p95 latency", "38 ms") delta="+6 ms" down_is_good;
                // end code
            }
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
