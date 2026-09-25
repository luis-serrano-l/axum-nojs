//! Primitives: the parts every component is built from.

use crate::site::page;
use axum::{Router, routing::get};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    super::pages(PAGES).route("/button", get(button_page))
}

/// The pages that are their component alone (`super::pages`).
pub(crate) const PAGES: &[super::Simple] = &[
    ("/field", fields, ""),
    ("/card", cards, ""),
    ("/layout", layouts, ""),
];

/// The other pages' live components, which the index shows too (`site::preview`).
pub(crate) const PREVIEWS: &[super::Preview] = &[("/button", buttons)];

fn buttons(ui: &Ui) -> Markup {
    let loading = ui.param("loading") == Some("1");
    lui! {
        Stack {
            // code: /button
            Cluster {
                Button("Save") primary loading=(loading);
                Button("Cancel");
                Button("Delete") danger;
                Button("Skip") ghost;
                Button("Small") small;
                Button("\u{2026}") icon_only ghost aria_label="More";
                LinkButton("Read the docs", "/");
            }
            Cluster {
                Badge("New"); Badge("Draft") secondary; Badge("Failed") danger;
                Badge("rust") outline; Badge("Paid") ok; Badge("Pending") warn;
            }
            Cluster {
                Button("Upgrade") primary shimmer; Button("What's new") shimmer;
                Badge("New") shimmer; Badge("Beta") outline shimmer;
            }
            Cluster gap=3 { @for icon in Icon::ALL { Icon(icon) label=(icon.name()); } }
            // end code
        }
    }
}

async fn button_page(ui: Ui) -> Page {
    let loading = ui.param("loading") == Some("1");
    let body = lui! { Stack {
        (buttons(&ui))
        p class="lui-note" { "The server decides a button is loading: " a href=(if loading { "/button" } else { "/button?loading=1" }) { @if loading { "stop" } @else { "start" } } "." }
    } };
    page(&ui, "Buttons and badges", body)
}

fn fields(ui: &Ui) -> Markup {
    let email = ui.param("email").unwrap_or("");
    let bad = !email.is_empty() && !email.contains('@');
    lui! {
        // code: /field
        Form("/field") get submit="Check" {
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
        }
        // end code
    }
}

fn cards(ui: &Ui) -> Markup {
    let team = [
        ("Ada Lovelace", "Owner"),
        ("Grace Hopper", "Admin"),
        ("Alan Turing", "Member"),
    ];
    lui! {
        // code: /card
        Grid("16rem") {
            Card title="Team" description="3 people can edit this project."
                header={ Badge("Pro") secondary; }
                footer={ Button("Invite") primary; Button("Manage") ghost; } {
                Stack gap=3 { @for (name, role) in team {
                    Cluster { Avatar(name); span { (name) } Badge(role) outline; }
                } }
            }
            Card title="Storage" description="Resets on the 1st." footer={ LinkButton("Upgrade", "/card"); } {
                p { "3.2 GB of 5 GB used." }
            }
            Card title="Pro" description="A beam runs round the border." beam glow
                footer={ Button("Start trial") primary shimmer; } {
                p { "The light at the top grows when you point at it." }
            }
            Card title="Changelog" description="A gradient border." gradient_border reveal {
                p { "Fades in as it scrolls into view." }
            }
        }
        // end code
    }
}

fn layouts(ui: &Ui) -> Markup {
    let tile = |t: &str| html! { div class="lui-layout-tile" { (t) } };
    lui! {
        // code: /layout
        Stack gap=6 {
            Cluster between { h3 { "Cluster" } Cluster { Button("Export"); Button("New") primary; } }
            Grid("8rem") gap=2 { @for t in ["Grid", "fills", "the row", "then", "wraps"] { (tile(t)) } }
            Split(html! { (tile("Split: side")) }, html! { (tile("main, stacks under the side when narrow")) })
                side_width="12rem";
        }
        // end code
    }
}
