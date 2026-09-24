//! The smallest Axum app that uses capability beacons and UI state:
//! `cargo run -p axum-nojs --example axum_server --features axum`, then open http://127.0.0.1:3001.
//!
//! First view: fallback markup, beacons fire. Reload: markup tailored to your browser.

use axum::{Router, routing::get};
use maud::{Markup, html};
use axum_nojs::{Ui, caps, dialog_with, dialog::DialogOptions, enhance, tabs_with, tabs::{Tab, TabsOptions}};

/// `Ui` is the browser's capabilities, the theme and the UI state in one extractor; returning
/// it beside the page remembers the open tab.
async fn index(ui: Ui) -> (Ui, Markup) {
    let page = ui.layout("axum-nojs", html! {
        h1 { "axum-nojs on Axum" }
        p { "This browser supports: " @for n in ui.caps.names() { code { (n) } " " } }
        (tabs_with(&ui, "demo", &[Tab::new("First", html! { p { "Tab state lives in the URL and a cookie." } }),
                                  Tab::new("Second", html! { p { "Reload, leave, come back: still here." } })], TabsOptions::default().state(&ui.state)))
        (dialog_with(&ui, "d", "Open dialog", html! { p { "Hello." } }, DialogOptions::default().state(&ui.state)))
    });
    (ui, page)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index)).merge(caps::router()).merge(enhance::router());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    println!("http://127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}
