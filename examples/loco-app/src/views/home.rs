use loco_ui::prelude::*;

pub fn index(ui: &Ui) -> Markup {
    html! {
        (ui.flash())
        h1 { "Notes" }
        p { "A Loco app on loco-ui. Every page works with script off." }
        (ui.cluster().body(html! {
            (ui.link_button("Sign in", "/signin").primary())
            (ui.link_button("Sign up", "/signup"))
            (ui.link_button("Notes", "/notes").ghost())
        }))
    }
}
