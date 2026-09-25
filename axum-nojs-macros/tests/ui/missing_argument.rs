use axum_nojs::prelude::*;

fn main() {
    let ui = Ui::from_request("/", "", "");
    let _ = nojs! { Tabs() { tab "One" { "1" } } };
}
