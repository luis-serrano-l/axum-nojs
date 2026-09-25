//! Demo server: one route per component. Handlers only parse input and call `axum-nojs`.
//! The binary in `main.rs` serves [`router`]; tests and `axum-nojs-test` call it directly.

pub mod pricing;
pub mod snapshot;

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
pub const PATHS: [&str; 33] = [
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
    "/counter",
    "/inputs",
    "/table?sort=size&dir=desc&q=a&per.files=5&page=2&cols=name,size",
    "/table?per.files=5&edit.files=src/build.rs",
    "/wizard?step.signup=1",
    "/swap?n=3",
    "/toast",
    "/nav",
    "/dashboard?orders=none",
    "/palette?q=ta",
];

/// The whole demo app.
pub fn router() -> Router {
    // Load syntect's grammars and highlight the snippets now, not on the first page view.
    std::thread::spawn(|| LazyLock::force(&code::CODE));
    Router::new()
        .merge(site::routes())
        .merge(routes::primitives::routes())
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
        .merge(axum_nojs::caps::router())
        .merge(axum_nojs::enhance::router())
        .layer(axum::middleware::from_fn(axum_nojs::enhance::slim))
        .layer(axum::middleware::from_fn(axum_nojs::enhance::csp))
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
