//! Navigation: command palette, drawer and breadcrumbs.

use crate::site::{COMPONENTS, page};
use axum::{
    Router,
    response::{IntoResponse, Response},
    routing::get,
};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    super::pages(PAGES).route("/palette", get(palette_page))
}

/// The pages that are their component and a note (`super::pages`).
pub(crate) const PAGES: &[super::Simple] = &[
    ("/nav", drawer, ""),
    (
        "/sidebar",
        sidebar,
        "Put it in a drawer's `.sidebar()` (or the app shell block) and it becomes a drawer on narrow screens.",
    ),
    (
        "/nav-menu",
        nav_menu,
        "A panel opens on click and closes on a click outside or Escape; the link to this page is marked current.",
    ),
];

/// The other pages' live components, which the index shows too (`site::preview`).
pub(crate) const PREVIEWS: &[super::Preview] = &[("/palette", |ui| palette(ui).render())];

/// Every demo page as a command, plus a few deep links.
fn palette(ui: &Ui) -> loco_ui::palette::Palette<'_> {
    // code: /palette
    ui.palette("/palette")
        .id("cmd")
        .group("Components")
        .links(COMPONENTS.iter().map(|c| (c.1, c.0)))
        .group("Shortcuts")
        .link("Notification settings", "/settings?tab.settings=1")
        .keywords("email releases")
        .link("Largest files", "/table?sort=size&dir=desc")
        .keywords("sort size big")
        .link("Open the delete dialog", "/dialog?dialog=confirm")
        .keywords("account remove")
    // end code
}

/// An exact command name redirects; anything else lists the matches.
async fn palette_page(ui: Ui) -> Response {
    let palette = palette(&ui);
    // code: /palette
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
fn drawer(ui: &Ui) -> Markup {
    lui! {
            // code: /nav
            Drawer("Menu") id="site" title="loco-ui" sidebar
                nav={ ul {
                    li { a href="/nav" aria-current="page" { "Overview" } }
                    li { a href="/table" { "Files" } } li { a href="/dashboard" { "Reports" } } li { a href="/settings" { "Settings" } }
                } } {
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
    }
}

/// A navigation column with groups, icons and counts; the link to this page is current.
fn sidebar(ui: &Ui) -> Markup {
    lui! {
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
    }
}

/// Top navigation: plain links and buttons that open a panel of links.
fn nav_menu(ui: &Ui) -> Markup {
    lui! {
        // code: /nav-menu
        NavMenu("Main") {
            panel "Products" ([("Mail", "/sidebar"), ("Calendar", "/calendar"), ("Files", "/table")]);
            panel "Resources" ([("Docs", "/"), ("Theming", "/?palette=linen")]);
            link "Pricing" "/pricing";
            link "Navigation menu" "/nav-menu";
        }
        // end code
    }
}
