//! Primitives: the parts every component is built from.

use crate::site::page;
use axum::{Router, routing::get};
use axum_nojs::prelude::*;

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
        html! {
            (ui.stack(html! {
                // code: /button
                (ui.cluster(html! {
                    (ui.button("Save").primary().loading(loading))
                    (ui.button("Cancel"))
                    (ui.button("Delete").danger())
                    (ui.button("Skip").ghost())
                    (ui.button("Small").small())
                    (ui.button("\u{2026}").icon().ghost().label("More"))
                    (ui.link_button("Read the docs", "/"))
                }))
                (ui.cluster(html! {
                    (ui.badge("New")) (ui.badge("Draft").secondary()) (ui.badge("Failed").danger())
                    (ui.badge("rust").outline()) (ui.badge("Paid").ok()) (ui.badge("Pending").warn())
                }))
                (ui.cluster(html! { @for icon in Icon::ALL { (ui.icon(icon).label(icon.name())) } }).gap(3))
                // end code
                p class="nojs-note" { "The server decides a button is loading: " a href=(if loading { "/button" } else { "/button?loading=1" }) { @if loading { "stop" } @else { "start" } } "." }
            }))
        },
    )
}

async fn field_page(ui: Ui) -> Page {
    let email = ui.param("email").unwrap_or("");
    let bad = !email.is_empty() && !email.contains('@');
    page(
        &ui,
        "Fields",
        html! {
            form class="nojs-stack" method="get" action="/field" {
                // code: /field
                (ui.input("name", "Name").placeholder("Ada Lovelace").help("As it should appear on invoices."))
                (ui.input("email", "Email").email().required().value(email).error(if bad { "An email address needs an @." } else { "" }))
                (ui.checkbox("terms", "I accept the terms").required())
                (ui.switch("digest", "Weekly digest").checked(ui.param("digest").is_some()))
                (ui.radio_group("plan", "Plan").option("free", "Free").option("pro", "Pro").value(ui.param("plan").unwrap_or("free")))
                (ui.button("Check").primary())
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
        html! {
            // code: /card
            (ui.grid("16rem", html! {
                (ui.card().title("Team").description("3 people can edit this project.")
                    .header(html! { (ui.badge("Pro").secondary()) })
                    .body(html! { (ui.stack(html! { @for (name, role) in team {
                        (ui.cluster(html! { (ui.avatar(name)) span { (name) } (ui.badge(role).outline()) }))
                    } }).gap(3)) })
                    .footer(html! { (ui.button("Invite").primary()) (ui.button("Manage").ghost()) }))
                (ui.card().title("Storage").description("Resets on the 1st.")
                    .body(html! { p { "3.2 GB of 5 GB used." } })
                    .footer(html! { (ui.link_button("Upgrade", "/card")) }))
            }))
            // end code
        },
    )
}

async fn layout_page(ui: Ui) -> Page {
    let tile = |t: &str| html! { div class="nojs-layout-tile" { (t) } };
    page(
        &ui,
        "Layout",
        html! {
            // code: /layout
            (ui.stack(html! {
                (ui.cluster(html! { h3 { "Cluster" } (ui.cluster(html! { (ui.button("Export")) (ui.button("New").primary()) })) }).between())
                (ui.grid("8rem", html! { @for t in ["Grid", "fills", "the row", "then", "wraps"] { (tile(t)) } }).gap(2))
                (ui.split(html! { (tile("Split: side")) }, html! { (tile("main, stacks under the side when narrow")) }).side_width("12rem"))
            }).gap(6))
            // end code
        },
    )
}
