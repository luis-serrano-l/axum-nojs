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
        nojs! {
            // Hovering or focusing a tab title fetches it early; the click reuses the answer.
            // code: /tabs
            div data-nojs-prefetch {
                Tabs("demo") select_below {
                    tab "Install" { p { code { "cargo add axum-nojs maud axum" } } }
                    tab "Use" badge=3 { p { "Call a function, get " code { "Markup" } ", send it." } }
                    // Lazy: the body is rendered only by the request that opens the tab.
                    lazy "Why" || { p { "Because the platform can do this without script now. (Rendered on demand.)" } }
                }
            }
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
        nojs! {
            // code: /accordion
            Accordion("faq") multi controls {
                item "Does this need JavaScript?" icon="\u{1F50D}"
                    summary="Every open and close is a link the server answers." {
                    p { "No. Turn it off and reload: every control still works through links and form posts. The one script on the page only swaps the answer in place instead of reloading." }
                }
                item "Does it animate?" icon="\u{1F3AC}"
                    summary="Height animates to auto in Chrome; elsewhere it snaps." {
                    p { "Yes, via ::details-content transitions where supported." }
                }
                item "Can several be open?" icon="\u{1F4DA}"
                    summary="Lists, links and a nested accordion." {
                    p { "Yes: this group is " code { "multi" } ", so " code { "?open.faq=0,2" } " keeps two open. A body can hold another group:" }
                    Accordion("faq-more") {
                        item "Nested" { p { "Its own key, " code { "open.faq-more" } "." } }
                        item "Exclusive" { p { "This inner group opens one at a time." } }
                    }
                }
            }
            // end code
            p class="nojs-note" { "Deep link: " a href="/accordion?open.faq=0,2" { "?open.faq=0,2" } ". Leave and come back: the open sections are remembered." }
        },
    )
}
