//! The front page: what the app is and where to go.
use axum_nojs::prelude::*;
use loco_rs::prelude::*;

use crate::views;

#[debug_handler]
async fn index(ui: Ui) -> Result<Page> {
    Ok(ui.page("Notes", views::home::index(&ui)))
}

pub fn routes() -> Routes {
    Routes::new().add("/", get(index))
}
