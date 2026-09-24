//! The smallest Axum app that uses capability beacons and UI state:
//! `cargo run -p axum-nojs --example axum_server --features axum`, then open http://127.0.0.1:3001.
//!
//! First view: fallback markup, beacons fire. Reload: markup tailored to your browser.

use axum::{Router, routing::get};
use axum_nojs::{caps, enhance, prelude::*};

/// `Ui` is the browser's capabilities, the theme and the UI state in one extractor; the page
/// it returns remembers the open tab.
async fn index(ui: Ui) -> Page {
    ui.page(
        "axum-nojs",
        html! {
            h1 { "axum-nojs on Axum" }
            p { "This browser supports: " @for n in ui.caps.names() { code { (n) } " " } }
            (ui.tabs("demo")
                .tab("First", html! { p { "Tab state lives in the URL and a cookie." } })
                .tab("Second", html! { p { "Reload, leave, come back: still here." } }))
            (ui.dialog("Open dialog").body(html! { p { "Hello." } }))
        },
    )
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(index))
        .merge(caps::router())
        .merge(enhance::router());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .unwrap();
    println!("http://127.0.0.1:3001");
    axum::serve(listener, app).await.unwrap();
}
