//! Demo server: one route per component. Handlers only parse input and call `loco-ui`.
//! The binary in `main.rs` serves [`router`]; tests and `loco-ui-test` call it directly.

mod playground;
pub mod pricing;
pub mod snapshot;
pub mod spanish;

mod code;
mod routes;
mod site;

use axum::Router;
use std::sync::LazyLock;
use tower_http::compression::{
    CompressionLayer,
    predicate::{DefaultPredicate, Predicate},
};

/// Every demo path the no-script test and the screenshot test visit.
pub const PATHS: [&str; 49] = [
    "/",
    "/caps",
    "/button?loading=1",
    "/field?email=ada",
    "/card",
    "/layout",
    "/calendar?month.day=2026-09&day=2026-09-17",
    "/upload",
    "/kanban",
    "/pricing?billing=yearly",
    "/feedback",
    "/app/signin",
    "/app/notes",
    "/stream",
    "/settings",
    "/dialog?dialog=confirm",
    "/popover",
    "/tabs?tab.demo=1",
    "/accordion?open.faq=0,2&open.faq-more=0",
    "/combobox?q=r&sel=Zig",
    "/list?page=2",
    "/form",
    "/form?layout=inline",
    "/form?errors=1",
    "/chart",
    "/theme?brand=%2312a594&radius=12",
    "/sidebar",
    "/nav-menu",
    "/description-list",
    "/toggle-group?align=center&style=bold",
    "/otp",
    "/context-menu",
    "/blocks/shell",
    "/blocks/auth",
    "/blocks/settings",
    "/blocks/record",
    "/blocks/dashboard",
    "/blocks/error",
    "/counter",
    "/inputs",
    "/table?sort.files=size&dir.files=desc&q.files=a&per.files=5&page.files=2&cols.files=name,size",
    "/table?per.files=5&edit.files=src/build.rs",
    "/wizard?step.signup=1",
    "/swap?n=3",
    "/toast",
    "/nav",
    "/dashboard?orders=none",
    "/palette?q=ta",
    "/marquee",
];

/// The whole demo app.
pub fn router() -> Router {
    // Load syntect's grammars and highlight the snippets now, not on the first page view.
    std::thread::spawn(|| LazyLock::force(&code::CODE));
    loco_ui::i18n::languages(&spanish::LANGUAGES);
    Router::new()
        .merge(site::routes())
        .merge(routes::primitives::routes())
        .merge(routes::theme::routes())
        .merge(routes::blocks::routes())
        .merge(routes::overlays::routes())
        .merge(routes::disclosure::routes())
        .merge(routes::navigation::routes())
        .merge(routes::input::routes())
        .merge(routes::feedback::routes())
        .merge(routes::table::routes())
        .merge(routes::server_state::routes())
        .merge(routes::widgets::routes())
        .merge(routes::flows::routes())
        .merge(routes::own::routes())
        .merge(loco_ui::caps::router())
        .merge(loco_ui::enhance::router())
        .fallback(loco_ui::blocks::not_found)
        .layer(axum::middleware::from_fn(loco_ui::enhance::slim))
        .layer(axum::middleware::from_fn(loco_ui::enhance::csp))
        .layer(CompressionLayer::new().compress_when(DefaultPredicate::new().and(WholeBody)))
}

/// Compress only bodies whose size is known up front. A streamed page (`/stream`) has no
/// exact size, and gzip would hold its chunks until the buffer fills, so it is sent as is.
#[derive(Clone, Copy)]
struct WholeBody;

impl Predicate for WholeBody {
    fn should_compress<B: axum::body::HttpBody>(&self, response: &axum::http::Response<B>) -> bool {
        response.body().size_hint().exact().is_some()
    }
}

#[cfg(test)]
mod tests;
