use loco_ui::prelude::*;

fn main() {
    let ui = Ui::from_request("/", "", "");
    let _ = lui! { Counter("/counter", 3) step="two"; };
}
