use loco_ui::prelude::*;

fn main() {
    let ui = Ui::from_request("/", "", "");
    let _ = lui! { Tabs() { tab "One" { "1" } } };
}
