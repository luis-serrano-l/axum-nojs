//! Render one page to stdout with no server at all: `cargo run -p webonsive --example render_page`.
//!
//! `Caps::all()` stands in for a modern browser; pass `--fallback` to see what an unknown
//! browser gets instead. Pipe into a file and open it, or `curl`-read it: no script anywhere.

use maud::html;
use webonsive::{AccordionItem, Caps, MenuItem, Tab, Theme, accordion, dialog, layout, popover_menu, tabs};

fn main() {
    let fallback = std::env::args().any(|a| a == "--fallback");
    let caps = if fallback { Caps::NONE } else { Caps::all() };
    let page = layout(&caps, "webonsive example", Theme::Auto, html! {
        h1 { "Hello from webonsive" }
        (dialog(&caps, "hi", "Open a dialog", html! { p { "Closed by the platform, not by script." } }))
        " "
        (popover_menu(&caps, "Menu", &[MenuItem::link("Docs", "/docs"), MenuItem::link("Source", "/src")]))
        h2 { "Tabs" }
        (tabs(&caps, "t", &[Tab::new("One", html! { p { "First panel." } }), Tab::new("Two", html! { p { "Second panel." } })]))
        h2 { "Accordion" }
        (accordion(&caps, "faq", &[AccordionItem::new("Why?", html! { p { "Because the platform can." } })]))
    });
    print!("{}", page.into_string());
}
