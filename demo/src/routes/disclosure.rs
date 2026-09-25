//! Disclosure: tabs and accordion.

use crate::site::page;
use axum::{Router, routing::get};
use axum_nojs::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/tabs", get(tabs_page))
        .route("/accordion", get(accordion_page))
}

async fn tabs_page(ui: Ui) -> Page {
    page(
        &ui,
        "Tabs",
        html! {
            // Hovering or focusing a tab title fetches it early; the click reuses the answer.
            // code: /tabs
            div data-nojs-prefetch { (ui.tabs("demo")
                .tab("Install", html! { p { code { "cargo add axum-nojs maud axum" } } })
                .tab("Use", html! { p { "Call a function, get " code { "Markup" } ", send it." } }).badge(3)
                // Lazy: the body is rendered only by the request that opens the tab.
                .lazy("Why", || html! { p { "Because the platform can do this without script now. (Rendered on demand.)" } })
                .select_below()) }
            // end code
            p class="nojs-note" { "Deep link: " a href="/tabs?tab.demo=2" { "?tab.demo=2" } ". Leave and come back: the tab is remembered. The third tab is lazy; under 40rem the strip becomes a select." }
            h2 { "Vertical" }
            (ui.tabs("side").vertical()
                .tab("General", html! { p { "Titles stack on the left; the open panel sits beside them." } })
                .tab("Members", html! { p { "Twelve members." } }).badge(12)
                .tab("Danger zone", html! { p { "Nothing here is destructive." } }))
        },
    )
}

async fn accordion_page(ui: Ui) -> Page {
    page(
        &ui,
        "Accordion",
        html! {
            // code: /accordion
            (ui.accordion("faq").multi().controls()
                .item("Does this need JavaScript?", html! { p { "No. Turn it off and reload: every control still works through links and form posts. The one script on the page only swaps the answer in place instead of reloading." } })
                    .icon("\u{1F50D}").summary("Every open and close is a link the server answers.")
                .item("Does it animate?", html! { p { "Yes, via ::details-content transitions where supported." } })
                    .icon("\u{1F3AC}").summary("Height animates to auto in Chrome; elsewhere it snaps.")
                .item("Can several be open?", html! {
                    p { "Yes: this group is " code { "multi" } ", so " code { "?open.faq=0,2" } " keeps two open. A body can hold another group:" }
                    (ui.accordion("faq-more")
                        .item("Nested", html! { p { "Its own key, " code { "open.faq-more" } "." } })
                        .item("Exclusive", html! { p { "This inner group opens one at a time." } }))
                }).icon("\u{1F4DA}").summary("Lists, links and a nested accordion."))
            // end code
            p class="nojs-note" { "Deep link: " a href="/accordion?open.faq=0,2" { "?open.faq=0,2" } ". Leave and come back: the open sections are remembered." }
        },
    )
}
