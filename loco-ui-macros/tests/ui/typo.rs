use loco_ui::prelude::*;

fn main() {
    let ui = Ui::from_request("/", "", "");
    let _ = lui! { Tabs("demo") vertcal { tab "One" { "1" } } };
}
