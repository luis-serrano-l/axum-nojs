//! # Loco
//!
//! [Loco](https://loco.rs) controllers are Axum handlers, so `Ui`, `Page`, `Redirect` and
//! `Saved<T>` work in them as they are. What Loco needs from this crate is the two routes every
//! page may ask for: the enhancement script at [`SCRIPT_PATH`](crate::enhance::SCRIPT_PATH)
//! and the capability beacon at `/nojs/caps`. [`Initializer`] mounts both, plus the
//! [`slim`](crate::enhance::slim) layer that drops the stylesheet from enhanced responses, in
//! Loco's `after_routes`, so an app adds one line to `app.rs`:
//!
//! ```rust
//! # use loco_rs::{app::{AppContext, Initializer}, Result};
//! async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
//!     Ok(vec![Box::new(axum_nojs::loco::Initializer)])
//! }
//! ```
//!
//! A controller takes `ui: Ui` beside Loco's extractors and returns Loco's `Result`. `Page`,
//! `Redirect` and `Streamed` implement `IntoResponse`, and so does Loco's `Error`, so a handler
//! returns `Result<Page>` (or `Result<Redirect>`) and uses `?` on the model call; or it
//! returns `Result<Response>` with `.into_response()` when branches answer differently.
//! No wrapper type:
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! use loco_rs::prelude::*;
//!
//! async fn show(ui: Ui, Path(id): Path<u32>) -> Result<Page> {
//!     let title = find(id)?; // a model call; Loco's `Error` becomes its own response
//!     Ok(ui.page(&title, html! { h1 { (title) } }))
//! }
//!
//! async fn remove(ui: Ui, Path(id): Path<u32>) -> Result<Response> {
//!     if id == 0 {
//!         return Ok(ui.redirect("/notes").danger("Nothing to delete.").into_response());
//!     }
//!     Ok(ui.redirect("/notes").ok("Deleted.").into_response())
//! }
//!
//! pub fn routes() -> Routes {
//!     Routes::new()
//!         .prefix("notes")
//!         .add("/{id}", get(show))
//!         .add("/{id}/delete", post(remove))
//! }
//! # fn find(id: u32) -> Result<String> {
//! #     if id == 0 { Err(Error::NotFound) } else { Ok(format!("Note {id}")) }
//! # }
//! ```
//!
//! The strict [`csp`](crate::enhance::csp) layer is not added: an app states its own policy
//! (add `axum::middleware::from_fn(axum_nojs::enhance::csp)` in `after_routes` to use ours).
//!
//! **Platform features:** none of its own; it serves what the components rely on.
//!
//! **What it does not do without script:** nothing is missing; the script is optional.
//!
//! **Fallback:** a page works without these routes too: the script 404s and the beacons stay
//! unset, so every visitor gets the baseline variant.

use async_trait::async_trait;
use axum::Router;
use loco_rs::{Result, app::AppContext};

/// The Loco initializer: `Box::new(axum_nojs::loco::Initializer)` in `App::initializers`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Initializer;

#[async_trait]
impl loco_rs::app::Initializer for Initializer {
    fn name(&self) -> String {
        "axum-nojs".to_string()
    }

    async fn after_routes(&self, router: Router, _ctx: &AppContext) -> Result<Router> {
        Ok(mount(router))
    }
}

/// What [`Initializer`] does, for a test or an app that builds its router by hand.
pub fn mount(router: Router) -> Router {
    router
        .merge(crate::caps::router())
        .merge(crate::enhance::router())
        .layer(axum::middleware::from_fn(crate::enhance::slim))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get};
    use tower::ServiceExt;

    async fn show(
        ui: crate::Ui,
        axum::extract::Path(id): axum::extract::Path<u32>,
    ) -> Result<crate::Page> {
        if id == 0 {
            return Err(loco_rs::Error::NotFound);
        }
        Ok(ui.page("Note", maud::html! { h1 { "Note " (id) } }))
    }

    #[tokio::test]
    async fn a_result_of_page_answers_with_the_page_or_locos_error() {
        let app = Router::new().route("/notes/{id}", get(show));
        for (path, status) in [("/notes/1", 200), ("/notes/0", 404)] {
            let req = Request::get(path).body(Body::empty()).unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), status, "{path}");
        }
    }

    #[tokio::test]
    async fn mounts_the_script_and_the_beacon_beside_the_app() {
        let app = mount(Router::new().route("/", get(|| async { "home" })));
        for (path, status) in [
            ("/", 200),
            (crate::enhance::SCRIPT_PATH, 200),
            ("/nojs/caps?flag=popover", 204),
        ] {
            let req = Request::get(path).body(Body::empty()).unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), status, "{path}");
        }
    }
}
