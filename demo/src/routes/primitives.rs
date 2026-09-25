//! Primitives: the parts every component is built from.

use crate::site::page;
use axum::{Router, routing::get};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/button", get(button_page))
        .route("/field", get(field_page))
        .route("/card", get(card_page))
        .route("/layout", get(layout_page))
}

async fn button_page(ui: Ui) -> Page {
    let loading = ui.param("loading") == Some("1");
    page(
        &ui,
        "Buttons and badges",
        lui! {
            Stack(lui! {
                // code: /button
                Cluster(lui! {
                    Button("Save") primary loading=(loading);
                    Button("Cancel");
                    Button("Delete") danger;
                    Button("Skip") ghost;
                    Button("Small") small;
                    Button("\u{2026}") icon ghost label="More";
                    LinkButton("Read the docs", "/");
                });
                Cluster(lui! {
                    Badge("New"); Badge("Draft") secondary; Badge("Failed") danger;
                    Badge("rust") outline; Badge("Paid") ok; Badge("Pending") warn;
                });
                Cluster(lui! {
                    Button("Upgrade") primary shimmer; Button("What's new") shimmer;
                    Badge("New") shimmer; Badge("Beta") outline shimmer;
                });
                Cluster(lui! { @for icon in Icon::ALL { Icon(icon) label=(icon.name()); } }) gap=3;
                // end code
                p class="lui-note" { "The server decides a button is loading: " a href=(if loading { "/button" } else { "/button?loading=1" }) { @if loading { "stop" } @else { "start" } } "." }
            });
        },
    )
}

async fn field_page(ui: Ui) -> Page {
    let email = ui.param("email").unwrap_or("");
    let bad = !email.is_empty() && !email.contains('@');
    page(
        &ui,
        "Fields",
        lui! {
            form class="lui-stack" method="get" action="/field" {
                // code: /field
                Input("name", "Name") placeholder="Ada Lovelace" help="As it should appear on invoices.";
                Input("email", "Email") email required value=(email)
                    error=(if bad { "An email address needs an @." } else { "" });
                Input("key", "API key") gradient_border placeholder="sk-live-...";
                Checkbox("terms", "I accept the terms") required;
                Switch("digest", "Weekly digest") checked=(ui.param("digest").is_some());
                RadioGroup("plan", "Plan") value=(ui.param("plan").unwrap_or("free")) {
                    option "free" "Free";
                    option "pro" "Pro";
                }
                Button("Check") primary;
                // end code
            }
        },
    )
}

async fn card_page(ui: Ui) -> Page {
    let team = [
        ("Ada Lovelace", "Owner"),
        ("Grace Hopper", "Admin"),
        ("Alan Turing", "Member"),
    ];
    page(
        &ui,
        "Cards and avatars",
        lui! {
            // code: /card
            Grid("16rem", lui! {
                Card title="Team" description="3 people can edit this project."
                    header=(lui! { Badge("Pro") secondary; })
                    body=(lui! { Stack(lui! { @for (name, role) in team {
                        Cluster(lui! { Avatar(name); span { (name) } Badge(role) outline; });
                    } }) gap=3; })
                    footer=(lui! { Button("Invite") primary; Button("Manage") ghost; });
                Card title="Storage" description="Resets on the 1st."
                    footer=(lui! { LinkButton("Upgrade", "/card"); }) {
                    p { "3.2 GB of 5 GB used." }
                }
                Card title="Pro" description="A beam runs round the border." beam glow
                    footer=(lui! { Button("Start trial") primary shimmer; }) {
                    p { "The light at the top grows when you point at it." }
                }
                Card title="Changelog" description="A gradient border." gradient_border reveal {
                    p { "Fades in as it scrolls into view." }
                }
            });
            // end code
        },
    )
}

async fn layout_page(ui: Ui) -> Page {
    let tile = |t: &str| html! { div class="lui-layout-tile" { (t) } };
    page(
        &ui,
        "Layout",
        lui! {
            // code: /layout
            Stack(lui! {
                Cluster(lui! { h3 { "Cluster" } Cluster(lui! { Button("Export"); Button("New") primary; }); }) between;
                Grid("8rem", html! { @for t in ["Grid", "fills", "the row", "then", "wraps"] { (tile(t)) } }) gap=2;
                Split(html! { (tile("Split: side")) }, html! { (tile("main, stacks under the side when narrow")) })
                    side_width="12rem";
            }) gap=6;
            // end code
        },
    )
}
