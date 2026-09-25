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
