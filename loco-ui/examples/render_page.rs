//! Render one page to stdout with no server at all: `cargo run -p axum-nojs --example render_page`.
//!
//! `Caps::all()` stands in for a modern browser; pass `--fallback` to see what an unknown
//! browser gets instead. Pipe into a file and open it, or `curl`-read it: no script anywhere.

use axum_nojs::prelude::*;

fn main() {
    let fallback = std::env::args().any(|a| a == "--fallback");
    let ui = Ui::from(if fallback { Caps::NONE } else { Caps::all() });
    let page = ui.page("axum-nojs example", html! {
        h1 { "Hello from axum-nojs" }
        (ui.dialog("Open a dialog").body(html! { p { "Closed by the platform, not by script." } }))
        " "
        (ui.menu("Menu").link("Docs", "/docs").link("Source", "/src"))
        h2 { "Tabs" }
        (ui.tabs("t").tab("One", html! { p { "First panel." } }).tab("Two", html! { p { "Second panel." } }))
        h2 { "Accordion" }
        (ui.accordion("faq").item("Why?", html! { p { "Because the platform can." } }))
    });
    print!("{}", page.into_string());
}
