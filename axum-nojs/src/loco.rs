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
//! Loco validates with the `validator` crate. [`FieldErrors`] turns its `ValidationErrors`, or
//! Loco's `ModelValidationErrors` (what `Error::Validation` and `ModelError::Validation`
//! carry), into `(field, message)` pairs for [`Form::errors`](crate::form::Form::errors), so
//! each message lands on the field it is about when the form is shown again:
//!
//! ```rust
//! use axum_nojs::{loco::FieldErrors, prelude::*};
//! use loco_rs::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Validate)]
//! struct NewNote {
//!     #[validate(length(min = 1, message = "Give the note a title."))]
//!     title: String,
//! }
//!
//! async fn create(ui: Ui, Form(note): Form<NewNote>) -> Result<Response> {
//!     // `Validate::validate`: Loco's prelude also brings `Validatable::validate` into scope.
//!     if let Err(e) = Validate::validate(&note) {
//!         let errors = FieldErrors::from(&e);
//!         // Built inside `html!`, so the borrowed pairs live as long as the render.
//!         let body = html! {
//!             (ui.form("/notes").text("title", "Title").value(&note.title).errors(&errors.pairs()))
//!         };
//!         return Ok(ui.page("New note", body).into_response());
//!     }
//!     Ok(ui.redirect("/notes").ok("Saved.").into_response())
//! }
//! # let e = Validate::validate(&NewNote { title: String::new() }).unwrap_err();
//! # assert_eq!(FieldErrors::from(&e).get("title"), Some("Give the note a title."));
//! ```
//!
//! Paging with SeaORM (Loco's ORM): the table reads `?page=`, the page size (`per.<id>`), the
//! sort and the filter from the request, and SeaORM's paginator asks the database for that
//! page and the count, never every row. `.paged(total)` then draws the pager:
//!
//! ```rust
//! use axum_nojs::{prelude::*, table::Row};
//! use sea_orm::{DatabaseConnection, DbErr, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder};
//! # mod notes {
//! #     use sea_orm::entity::prelude::*;
//! #     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//! #     #[sea_orm(table_name = "notes")]
//! #     pub struct Model { #[sea_orm(primary_key)] pub id: i32, pub title: String }
//! #     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! #     pub enum Relation {}
//! #     impl ActiveModelBehavior for ActiveModel {}
//! # }
//! use sea_orm::ColumnTrait;
//!
//! async fn notes_table(ui: &Ui, db: &DatabaseConnection) -> Result<Markup, DbErr> {
//!     let table = ui.table("notes", "/notes").column("title", "Title").sortable();
//!     let query = notes::Entity::find().filter(notes::Column::Title.contains(table.filter()));
//!     let query = match table.sort() {
//!         Some(("title", true)) => query.order_by_desc(notes::Column::Title),
//!         _ => query.order_by_asc(notes::Column::Title),
//!     };
//!     let pages = query.paginate(db, table.per_page() as u64);
//!     let total = pages.num_items().await? as usize;
//!     let rows = pages.fetch_page(table.page() as u64 - 1).await?; // SeaORM counts from 0
//!     let rows = rows.into_iter().map(|n| Row::new([html! { (n.title) }]));
//!     Ok(html! { (table.rows(rows).paged(total)) })
//! }
//! # use sea_orm::{DatabaseBackend, MockDatabase, Value};
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() {
//! #     let count = std::collections::BTreeMap::from([("num_items", Value::BigInt(Some(42)))]);
//! #     let note = notes::Model { id: 11, title: "Eleventh".into() };
//! #     let db = MockDatabase::new(DatabaseBackend::Sqlite)
//! #         .append_query_results([[count]])
//! #         .append_query_results([[note]])
//! #         .into_connection();
//! #     let ui = Ui::from_request("/notes", "page=2", "");
//! #     let html = notes_table(&ui, &db).await.unwrap().into_string();
//! #     assert!(html.contains("11–20 of 42") && html.contains("Eleventh"), "{html}");
//! #     let log = format!("{:?}", db.into_transaction_log());
//! #     assert!(log.contains("LIMIT") && log.contains("OFFSET"), "{log}");
//! # }
//! ```
//!
//! The "Load more" [`Pager`](crate::pager::Pager) shows every row up to `?page=`, so it
//! fetches the first `pager.shown()` rows: `query.limit(pager.shown() as u64).all(db)`.
//!
//! Flash and Post/Redirect/Get need nothing from Loco. The flash and UI state are plain
//! `nojs-*` cookies read and written by `Ui` and `Redirect`, so there is no signing key to take
//! from `config/*.yaml`, and they never collide with the JWT cookie `auth.jwt.location` names.
//! Loco 1.2 has no session middleware in its core. A test runs a post, the redirect and the
//! page showing the flash through Loco's default middleware stack. Two settings of Loco's
//! `secure_headers` middleware matter:
//! - The default `github` preset sends `script-src https:`, so over plain `http://` in
//!   development the enhancement script is blocked and pages run as they do without it.
//! - The `owasp` preset sends `Clear-Site-Data: "cache","cookies","storage"` on every
//!   response, which wipes the flash, the theme and the sign-in cookie on each page. Use
//!   `github`, or `owasp` with that header overridden.
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
use loco_rs::{Result, app::AppContext, validation::ModelValidationErrors};

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

/// Validation messages by field, from `validator` or Loco; see the module docs.
///
/// A rule without a `message` gets one from its code (`length` → "Check the length.",
/// `email` → "Enter a valid email address."). Only the first message per field is kept, and
/// errors on nested structs are left out: a form field has one name and shows one message.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FieldErrors(Vec<(String, String)>);

impl FieldErrors {
    /// The pairs [`Form::errors`](crate::form::Form::errors) takes, sorted by field name.
    pub fn pairs(&self) -> Vec<(&str, &str)> {
        self.0
            .iter()
            .map(|(f, m)| (f.as_str(), m.as_str()))
            .collect()
    }

    /// The message for `field`, for an [`Input::error`](crate::input::Input::error) outside a form.
    pub fn get(&self, field: &str) -> Option<&str> {
        self.0
            .iter()
            .find(|(f, _)| f == field)
            .map(|(_, m)| m.as_str())
    }

    /// No field has a message.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The messages in a Loco error, if it is `Error::Validation`. (`ModelError::Validation`,
    /// with Loco's `with-db`, carries the same `ModelValidationErrors`: use `From` on it.)
    pub fn from_error(error: &loco_rs::Error) -> Option<Self> {
        match error {
            loco_rs::Error::Validation(e) => Some(Self::from(e)),
            _ => None,
        }
    }
}

impl From<&ModelValidationErrors> for FieldErrors {
    fn from(errors: &ModelValidationErrors) -> Self {
        // A `BTreeMap`, so already sorted by field.
        Self(
            errors
                .errors
                .iter()
                .filter_map(|(field, list)| {
                    let first = list.first()?;
                    Some((
                        field.clone(),
                        message(&first.code, first.message.as_deref()),
                    ))
                })
                .collect(),
        )
    }
}

impl From<&loco_rs::validator::ValidationErrors> for FieldErrors {
    fn from(errors: &loco_rs::validator::ValidationErrors) -> Self {
        let mut pairs: Vec<(String, String)> = errors
            .field_errors()
            .into_iter()
            .filter_map(|(field, list)| {
                let first = list.first()?;
                Some((
                    field.to_string(),
                    message(&first.code, first.message.as_deref()),
                ))
            })
            .collect();
        pairs.sort();
        Self(pairs)
    }
}

/// The rule's own message, or a sentence for the `validator` built-in codes.
fn message(code: &str, message: Option<&str>) -> String {
    if let Some(m) = message {
        return m.to_string();
    }
    match code {
        "required" => "This field is required.",
        "email" => "Enter a valid email address.",
        "url" => "Enter a valid URL.",
        "length" => "Check the length.",
        "range" => "Enter a value in range.",
        "must_match" => "The values do not match.",
        "contains" | "does_not_contain" | "regex" => "Check the format.",
        _ => "Check this field.",
    }
    .to_string()
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
    use maud::Render;
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

    #[test]
    fn field_errors_keep_the_first_message_per_field_and_fill_in_missing_ones() {
        use loco_rs::validator::{ValidationError, ValidationErrors};
        let mut e = ValidationErrors::new();
        e.add(
            "title",
            ValidationError::new("length").with_message("Too short.".into()),
        );
        e.add("title", ValidationError::new("regex"));
        e.add("email", ValidationError::new("email"));
        let errors = FieldErrors::from(&e);
        assert_eq!(
            errors.pairs(),
            [
                ("email", "Enter a valid email address."),
                ("title", "Too short.")
            ]
        );
        let loco = loco_rs::Error::Validation(ModelValidationErrors::from(e));
        assert_eq!(FieldErrors::from_error(&loco), Some(errors.clone()));
        assert_eq!(FieldErrors::from_error(&loco_rs::Error::NotFound), None);

        let html = crate::Ui::default()
            .form("/notes")
            .text("title", "Title")
            .errors(&errors.pairs())
            .render()
            .into_string();
        assert!(html.contains("Too short."), "{html}");
    }

    /// Post/Redirect/Get with a flash, through Loco's default middleware stack (secure headers,
    /// ETag, compression, request id, ...) built the way Loco's boot does it: routes, then the
    /// enabled middleware, then `with_state`, then initializers' `after_routes`.
    #[tokio::test]
    async fn flash_survives_locos_default_middleware() {
        use axum::http::header;
        use loco_rs::controller::middleware::default_middleware_stack;

        async fn list(ui: crate::Ui) -> crate::Page {
            ui.page("Notes", maud::html! { (ui.flash()) h1 { "Notes" } })
        }
        async fn create(ui: crate::Ui) -> crate::Redirect {
            ui.redirect("/notes").ok("Note saved.")
        }

        let ctx = loco_rs::tests_cfg::app::get_app_context().await;
        let mut app = Router::<AppContext>::new().route("/notes", get(list).post(create));
        for layer in default_middleware_stack(&ctx) {
            if layer.is_enabled() {
                app = layer.apply(app).unwrap();
            }
        }
        let app = loco_rs::app::Initializer::after_routes(
            &Initializer,
            app.with_state(ctx.clone()),
            &ctx,
        )
        .await
        .unwrap();

        let post = Request::post("/notes").body(Body::empty()).unwrap();
        let res = app.clone().oneshot(post).await.unwrap();
        assert_eq!(res.status(), 303);
        assert_eq!(res.headers()[header::LOCATION], "/notes");
        let cookies: Vec<&str> = res
            .headers()
            .get_all(header::SET_COOKIE)
            .iter()
            .map(|v| v.to_str().unwrap().split(';').next().unwrap())
            .collect();
        assert!(!cookies.is_empty(), "the flash cookie is set");

        let get = Request::get("/notes")
            .header(header::COOKIE, cookies.join("; "))
            .body(Body::empty())
            .unwrap();
        let res = app.clone().oneshot(get).await.unwrap();
        assert_eq!(res.status(), 200);
        let cleared = res.headers().get_all(header::SET_COOKIE).iter().count();
        assert!(cleared > 0, "the shown flash is cleared");
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&body).contains("Note saved."));

        let res = app
            .oneshot(
                Request::get(crate::enhance::script_url())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            res.status(),
            200,
            "the script is served beside Loco's routes"
        );
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
