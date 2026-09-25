//! Navigation: command palette, drawer and breadcrumbs.

use crate::site::{COMPONENTS, page};
use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::get,
};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/palette", get(palette_page))
        .route("/nav", get(nav_page))
        .route("/sidebar", get(sidebar_page))
        .route("/nav-menu", get(nav_menu_page))
}

/// Every demo page as a command, plus a few deep links. An exact command name redirects;
/// anything else lists the matches.
async fn palette_page(ui: Ui) -> Response {
    // code: /palette
    let palette = ui
        .palette("/palette")
        .id("cmd")
        .group("Components")
        .commands(COMPONENTS.iter().map(|c| (c.1, c.0)))
        .group("Shortcuts")
        .command("Notification settings", "/settings?tab.settings=1")
        .keywords("email releases")
        .command("Largest files", "/table?sort=size&dir=desc")
        .keywords("sort size big")
        .command("Open the delete dialog", "/dialog?dialog=confirm")
        .keywords("account remove");
    if let Some(href) = palette.exact() {
        return ui.redirect(href).into_response();
    }
    // end code
    page(&ui, "Command palette", html! {
        p { "Open it with the button or the access key, type, pick a suggestion and press Enter. An exact name goes straight to the page; anything else lists what matches." }
        (palette)
    }).into_response()
}

/// A sidebar on wide screens, a drawer on narrow ones, and breadcrumbs above the content.
async fn nav_page(ui: Ui) -> Page {
    page(
        &ui,
        "Drawer and breadcrumbs",
        lui! {
            // code: /nav
            Drawer("Menu") id="site" title="loco-ui" sidebar
                nav=(html! { ul {
                    li { a href="/nav" aria-current="page" { "Overview" } }
                    li { a href="/table" { "Files" } } li { a href="/dashboard" { "Reports" } } li { a href="/settings" { "Settings" } }
                } }) {
                    Breadcrumbs { link "Home" "/"; link "Projects" "/nav"; here "loco-ui"; }
                    p { "Wider than 60rem the navigation is a sidebar; narrower, the menu button opens it as a drawer. Escape or a click outside closes it." }
                    p { "A long trail folds its middle so both ends stay readable:" }
                    Breadcrumbs {
                        link "Home" "/"; link "Projects" "/nav"; link "loco-ui" "/nav";
                        link "Components" "/"; link "Navigation" "/nav"; here "Breadcrumbs";
                    }
            // end code
                    p class="lui-note" { "Server-opened: " a href="/nav?dialog=site" { "?dialog=site" } }
                }
        },
    )
}

/// A navigation column with groups, icons and counts; the link to this page is current.
async fn sidebar_page(ui: Ui) -> Page {
    let body = lui! {
        div style="max-width: 16rem" {
            // code: /sidebar
            Sidebar("Mail") {
                group "Mail";
                link "Inbox" "/sidebar" icon=(Icon::Mail) badge="12";
                link "Drafts" "/sidebar?box=drafts" icon=(Icon::Pencil);
                link "Sent" "/sidebar?box=sent" icon=(Icon::ArrowRight);
                group "Labels";
                link "Work" "/nav"; link "Personal" "/nav";
            }
            // end code
        }
        p class="lui-note" { "Put it in a drawer's " code { ".sidebar()" } " (or the app shell block) and it becomes a drawer on narrow screens." }
    };
    page(&ui, "Sidebar", body)
}

/// Top navigation: plain links and buttons that open a panel of links.
async fn nav_menu_page(ui: Ui) -> Page {
    let body = lui! {
        // code: /nav-menu
        NavMenu("Main") {
            panel "Products" ([("Mail", "/sidebar"), ("Calendar", "/calendar"), ("Files", "/table")]);
            panel "Resources" ([("Docs", "/"), ("Theming", "/?palette=linen")]);
            link "Pricing" "/pricing";
            link "Navigation menu" "/nav-menu";
        }
        // end code
        p class="lui-note" { "A panel opens on click and closes on a click outside or Escape; the link to this page is marked current." }
    };
    page(&ui, "Navigation menu", body)
}
