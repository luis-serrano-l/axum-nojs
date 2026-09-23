//! The smallest Axum app that uses capability beacons and UI state:
//! `cargo run -p webonsive --example axum_server --features axum`, then open http://127.0.0.1:3001.
//!
//! First view: fallback markup, beacons fire. Reload: markup tailored to your browser.

use axum::{Router, routing::get};
use maud::{Markup, html};
use webonsive::{Caps, Theme, UiState, caps, dialog, dialog::DialogOptions, layout, tabs};

async fn index(caps: Caps, state: UiState) -> (UiState, Markup) {
    let page = layout(&caps, "webonsive", Theme::Auto, html! {
        h1 { "webonsive on Axum" }
        p { "This browser supports: " @for n in caps.names() { code { (n) } " " } }
        (tabs(&caps, "demo", &[("First", html! { p { "Tab state lives in the URL and a cookie." } }),
                              ("Second", html! { p { "Reload, leave, come back: still here." } })], Some(&state)))
        (dialog(&caps, "d", "Open dialog", html! { p { "Hello." } }, DialogOptions::default().open(state.dialog() == Some("d"))))
    });
    (state, page)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index)).merge(caps::router());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001").await.unwrap();
    println!("http://127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}
