//! Disclosure: tabs and accordion.

use crate::site::page;
use axum::{Router, routing::get};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    super::pages(PAGES).route("/tabs", get(tabs_page))
}

/// The pages that are their component and a note (`super::pages`).
pub(crate) const PAGES: &[super::Simple] = &[(
    "/accordion",
    accordion,
    "Deep link: [?open.faq=0,2](/accordion?open.faq=0,2). Leave and come back: the open sections are remembered.",
)];

/// The other pages' live components, which the index shows too (`site::preview`).
pub(crate) const PREVIEWS: &[super::Preview] = &[("/tabs", tabs)];

fn tabs(ui: &Ui) -> Markup {
    lui! {
            // Hovering or focusing a tab title fetches it early; the click reuses the answer.
            // code: /tabs
            div data-lui-prefetch {
                Tabs("demo") select_below {
                    tab "Install" { p { code { "cargo add loco-ui maud axum" } } }
                    tab "Use" badge=3 { p { "Call a function, get " code { "Markup" } ", send it." } }
                    // Lazy: the body is rendered only by the request that opens the tab.
                    lazy "Why" || { p { "Because the platform can do this without script now. (Rendered on demand.)" } }
                }
            }
            // end code
    }
}

async fn tabs_page(ui: Ui) -> Page {
    let body = lui! {
        (tabs(&ui))
        p class="lui-note" { "Deep link: " a href="/tabs?tab.demo=2" { "?tab.demo=2" } ". Leave and come back: the tab is remembered. The third tab is lazy; under 40rem the strip becomes a select." }
        h2 { "Vertical" }
        Tabs("side") vertical {
            tab "General" { p { "Titles stack on the left; the open panel sits beside them." } }
            tab "Members" badge=12 { p { "Twelve members." } }
            tab "Danger zone" { p { "Nothing here is destructive." } }
        }
    };
    page(&ui, "Tabs", body)
}

fn accordion(ui: &Ui) -> Markup {
    lui! {
            // code: /accordion
            Accordion("faq") multiple controls {
                item "Does this need JavaScript?" icon="\u{1F50D}"
                    description="Every open and close is a link the server answers." {
                    p { "No. Turn it off and reload: every control still works through links and form posts. The one script on the page only swaps the answer in place instead of reloading." }
                }
                item "Does it animate?" icon="\u{1F3AC}"
                    description="Height animates to auto in Chrome; elsewhere it snaps." {
                    p { "Yes, via ::details-content transitions where supported." }
                }
                item "Can several be open?" icon="\u{1F4DA}"
                    description="Lists, links and a nested accordion." {
                    p { "Yes: this group is " code { "multi" } ", so " code { "?open.faq=0,2" } " keeps two open. A body can hold another group:" }
                    Accordion("faq-more") {
                        item "Nested" { p { "Its own key, " code { "open.faq-more" } "." } }
                        item "Exclusive" { p { "This inner group opens one at a time." } }
                    }
                }
            }
            // end code
    }
}
