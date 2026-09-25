use loco_ui::prelude::*;

fn main() {
    let ui = Ui::from_request("/", "", "");
    let _ = lui! { DashboardPage("Overview") { stat Stat("Revenue", "$1") delta="+8%"; } };
}
