//! # Loco
//!
//! [Loco](https://loco.rs) controllers are Axum handlers, so `Ui`, `Page`, `Redirect` and
//! `Saved<T>` work in them as they are. What Loco needs from this crate is the two routes every
//! page may ask for: the enhancement script at [`SCRIPT_PATH`](crate::enhance::SCRIPT_PATH)
//! and the capability beacon at `/lui/caps`. [`Initializer`] mounts both, plus the
//! [`slim`](crate::enhance::slim) layer that drops the stylesheet from enhanced responses, in
//! Loco's `after_routes`, so an app adds one line to `app.rs`:
//!
//! ```rust
//! # use loco_rs::{app::{AppContext, Initializer}, Result};
//! async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
//!     Ok(vec![Box::new(loco_ui::loco::Initializer)])
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
//! use loco_ui::prelude::*;
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
//! use loco_ui::{loco::FieldErrors, prelude::*};
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
//! [`Valid<T>`] does that in the extractor. Loco's `FormValidate` answers a bad form with an
//! error response; `Valid` gives the handler `Ok(T)` or `Err(Invalid)` (the messages by field
//! and what was posted), so a create or update is one `match`: redirect, or show the form
//! again. The scaffold templates use it.
//!
//! ```rust
//! use loco_ui::{loco::Valid, prelude::*};
//! use loco_rs::prelude::*;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize, Validate)]
//! struct NewNote {
//!     #[validate(length(max = 80, message = "At most 80 characters."))]
//!     title: String,
//!     stars: Option<u8>,
//!     #[serde(default, deserialize_with = "loco_ui::loco::checkbox")]
//!     done: bool,
//! }
//!
//! async fn create(ui: Ui, Valid(note): Valid<NewNote>) -> Response {
//!     match note {
//!         Ok(note) => ui.redirect("/notes").ok(&format!("Saved {}.", note.title)).into_response(),
//!         Err(bad) => {
//!             let body = html! {
//!                 (ui.form("/notes")
//!                     .text("title", "Title").required()
//!                     .number("stars", "Stars", 1, 5)
//!                     .checkbox("done", "Done")
//!                     .values(&bad.values)
//!                     .errors(&bad.errors.pairs()))
//!             };
//!             ui.page("New note", body).into_response()
//!         }
//!     }
//! }
//! # let posted = |s: &str| form_urlencoded::parse(s.as_bytes()).into_owned().collect::<Vec<_>>();
//! # let bad = Valid::<NewNote>::check(posted("title=&stars=many")).0.err().unwrap();
//! # assert_eq!(bad.errors.get("title"), Some("This field is required."));
//! # assert_eq!(bad.errors.get("stars"), Some("Check this field."));
//! # let ok = Valid::<NewNote>::check(posted("title=Plan&stars=&done=on")).0.ok().unwrap();
//! # assert!(ok.title == "Plan" && ok.stars.is_none() && ok.done);
//! ```
//!
//! Views are Rust, not Tera: a `views` module of functions taking `&Ui` and the data and
//! returning `Markup`, which a controller wraps in `ui.page(..)`. Tera views keep working
//! beside them. `docs/loco.md` says why there is no Tera function bridge.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! use loco_rs::prelude::*;
//! # pub struct Note { pub id: i32, pub title: String }
//!
//! mod views {
//!     pub mod notes {
//!         use loco_ui::{prelude::*, table::Row};
//!         use crate::Note; // `crate::models::_entities::notes::Model` in a Loco app
//!
//!         pub fn list(ui: &Ui, rows: &[Note], total: usize) -> Markup {
//!             let table = ui.table("notes", "/notes").column("title", "Title").sortable();
//!             let rows = rows.iter().map(|n| Row::new([html! { a href={ "/notes/" (n.id) } { (n.title) } }]));
//!             html! {
//!                 (ui.flash())
//!                 (table.rows(rows).paged(total))
//!                 (ui.link_button("New note", "/notes/new"))
//!             }
//!         }
//!     }
//! }
//!
//! async fn list(ui: Ui) -> Result<Page> {
//!     let rows = vec![Note { id: 1, title: "First".into() }]; // the paging query below
//!     Ok(ui.page("Notes", views::notes::list(&ui, &rows, 1)))
//! }
//! # fn main() {
//! # let html = views::notes::list(&Ui::default(), &[Note { id: 1, title: "First".into() }], 1).into_string();
//! # assert!(html.contains(r#"<a href="/notes/1">First</a>"#), "{html}");
//! # }
//! ```
//!
//! Paging with Loco's `query::fetch_page` (or `query::paginate`, which adds a condition): the
//! table reads its page (`?page.<id>=`), page size (`per.<id>`), sort and filter from the
//! request; Loco asks the database for that page and the count, never every row; and
//! `.paged_from(&meta)` draws the pager from the answer:
//!
//! ```rust
//! use loco_ui::{prelude::*, table::Row};
//! use loco_rs::{model::query::{self, PaginationQuery}, Result};
//! use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, QueryOrder};
//! # mod notes {
//! #     use sea_orm::entity::prelude::*;
//! #     #[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
//! #     #[sea_orm(table_name = "notes")]
//! #     pub struct Model { #[sea_orm(primary_key)] pub id: i32, pub title: String }
//! #     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! #     pub enum Relation {}
//! #     impl ActiveModelBehavior for ActiveModel {}
//! # }
//!
//! async fn notes_table(ui: &Ui, db: &DatabaseConnection) -> Result<Markup> {
//!     let table = ui.table("notes", "/notes").column("title", "Title").sortable();
//!     let select = notes::Entity::find().filter(notes::Column::Title.contains(table.filter()));
//!     let select = match table.sort() {
//!         Some(("title", true)) => select.order_by_desc(notes::Column::Title),
//!         _ => select.order_by_asc(notes::Column::Title),
//!     };
//!     let wanted = PaginationQuery { page: table.page() as u64, page_size: table.per_page() as u64 };
//!     let found = query::fetch_page(db, select, &wanted).await?;
//!     let rows = found.page.iter().map(|n| Row::new([html! { (n.title) }]));
//!     Ok(html! { (table.rows(rows).paged_from(&found.meta)) })
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
//! #     let ui = Ui::from_request("/notes", "page.notes=2", "");
//! #     let html = notes_table(&ui, &db).await.unwrap().into_string();
//! #     assert!(html.contains("11–20 of 42") && html.contains("Eleventh"), "{html}");
//! #     let log = format!("{:?}", db.into_transaction_log());
//! #     assert!(log.contains("LIMIT") && log.contains("OFFSET"), "{log}");
//! # }
//! ```
//!
//! `Table::paged_from` is `.paged(meta.total_items)`; SeaORM's own paginator (`paginate`,
//! `num_items`, `fetch_page(page - 1)`) with `.paged(total)` works the same without Loco.
//!
//! The "Load more" [`Pager`](crate::pager::Pager) shows every row up to `?page=`, so it
//! fetches the first `pager.shown()` rows: `query.limit(pager.shown() as u64).all(db)`.
//!
//! Languages: Loco's i18n is `fluent-templates` (`assets/i18n/<lang>/main.ftl`, a
//! `static_loader!`). Give each language the components' keys (`lui-next = Siguiente`,
//! `lui-load-more = Cargar más`, every [`Text::key`](crate::i18n::Text::key)) and build the
//! tables from the loader once, at start-up; a key the file lacks stays English:
//!
//! ```rust
//! use loco_ui::i18n::{self, Strings, Text};
//! # struct Loader;
//! # impl Loader { fn try_lookup(&self, lang: &str, key: &str) -> Option<String> {
//! #     (lang == "es" && key == "lui-next").then(|| "Siguiente".to_string()) } }
//! # static LOCALES: Loader = Loader;
//! // `LOCALES.try_lookup(&langid!("es"), key)` with fluent-templates.
//! let spanish = Strings::from_lookup("es", |key| LOCALES.try_lookup("es", key));
//! let tables: &'static [&'static Strings] = Box::leak(Box::new([&Strings::ENGLISH, spanish]));
//! i18n::languages(tables);
//! assert_eq!(spanish.get(Text::Next), "Siguiente");
//! ```
//!
//! Flash and Post/Redirect/Get need nothing from Loco. The flash and UI state are plain
//! `lui-*` cookies read and written by `Ui` and `Redirect`, so there is no signing key to take
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
//! (add `axum::middleware::from_fn(loco_ui::enhance::csp)` in `after_routes` to use ours).
//!
//! **Platform features:** none of its own; it serves what the components rely on.
//!
//! **What it does not do without script:** nothing is missing; the script is optional.
//!
//! **Fallback:** a page works without these routes too: the script 404s and the beacons stay
//! unset, so every visitor gets the baseline variant.

use async_trait::async_trait;
use axum::{
    Router,
    body::Bytes,
    extract::{FromRequest, Request, rejection::BytesRejection},
};
use loco_rs::{Result, app::AppContext, validation::ModelValidationErrors, validator::Validate};
use serde::{Deserialize, Deserializer, de::DeserializeOwned};

/// The Loco initializer: `Box::new(loco_ui::loco::Initializer)` in `App::initializers`.
#[derive(Clone, Copy, Debug, Default)]
pub struct Initializer;

#[async_trait]
impl loco_rs::app::Initializer for Initializer {
    fn name(&self) -> String {
        "loco-ui".to_string()
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

/// A posted form read field by field, collecting a message for each field that is missing
/// or does not parse, so the form can be shown again with the values and the messages. The
/// scaffold templates (`loco-templates/`) use it; it works for any handler taking
/// `Form<Vec<(String, String)>>`.
///
/// ```rust
/// use loco_ui::loco::Submitted;
/// let mut form = Submitted::new(vec![("title".into(), "".into()), ("stars".into(), "x".into())]);
/// let title: Option<String> = form.required("title");
/// let stars: Option<u8> = form.optional("stars");
/// let done = form.checkbox("done");
/// assert!((title, stars, done) == (None, None, false) && !form.is_ok());
/// assert_eq!(form.errors().get("title"), Some("This field is required."));
/// assert_eq!(form.errors().get("stars"), Some("Check this field."));
/// ```
#[derive(Clone, Debug, Default)]
pub struct Submitted {
    values: Vec<(String, String)>,
    errors: Vec<(String, String)>,
}

impl Submitted {
    /// The posted `(name, value)` pairs.
    pub fn new(values: Vec<(String, String)>) -> Self {
        Self {
            values,
            errors: Vec::new(),
        }
    }

    fn raw(&self, name: &str) -> &str {
        self.values
            .iter()
            .find(|(n, _)| n == name)
            .map_or("", |(_, v)| v.trim())
    }

    fn fail(&mut self, name: &str, message: &str) {
        if !self.errors.iter().any(|(n, _)| n == name) {
            self.errors.push((name.to_string(), message.to_string()));
        }
    }

    /// The value of `name`; a message if it is empty or does not parse as `T`.
    pub fn required<T: std::str::FromStr>(&mut self, name: &str) -> Option<T> {
        if self.raw(name).is_empty() {
            self.fail(name, "This field is required.");
            return None;
        }
        self.optional(name)
    }

    /// The value of `name`, `None` when empty; a message if it does not parse as `T`.
    pub fn optional<T: std::str::FromStr>(&mut self, name: &str) -> Option<T> {
        let raw = self.raw(name);
        if raw.is_empty() {
            return None;
        }
        let parsed = raw.parse().ok();
        if parsed.is_none() {
            self.fail(name, "Check this field.");
        }
        parsed
    }

    /// Whether the checkbox `name` was ticked (a form's checkbox posts `true`).
    pub fn checkbox(&self, name: &str) -> bool {
        matches!(self.raw(name), "true" | "on")
    }

    /// No field failed.
    pub fn is_ok(&self) -> bool {
        self.errors.is_empty()
    }

    /// What was posted, for [`Form::values`](crate::form::Form::values).
    pub fn values(&self) -> &[(String, String)] {
        &self.values
    }

    /// The messages so far, sorted by field.
    pub fn errors(&self) -> FieldErrors {
        let mut errors = self.errors.clone();
        errors.sort();
        FieldErrors(errors)
    }
}

/// A posted form, deserialized and validated: `Ok(T)`, or `Err(`[`Invalid`]`)` with a message
/// for every field in error and the values to show again. See the module docs.
///
/// How a posted form becomes a `T`: values are trimmed and empty ones left out, so an empty
/// field is `None` for an `Option`, and "This field is required." for anything else. A value
/// that does not parse gets "Check this field.". Every field is checked, not only the first
/// bad one: a bad field is stood in for (by `""`, `0`, `false`, a date, a date-time or a
/// nil UUID) so the rest can be read, and then `T`'s `#[validate(..)]` rules run; the first
/// message per field is kept. A field of a type none of those stand-ins reads stops there,
/// so the rules wait until it is fixed. A checkbox posts `on` (or `true`, from [`Form`](crate::form::Form)), which serde
/// does not read as a `bool`: mark it `#[serde(default, deserialize_with =
/// "loco_ui::loco::checkbox")]`.
#[derive(Clone, Debug)]
pub struct Valid<T>(pub std::result::Result<T, Invalid>);

/// Why a [`Valid`] form was refused: the messages by field and what was posted.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Invalid {
    /// For [`Form::errors`](crate::form::Form::errors), via `.pairs()`.
    pub errors: FieldErrors,
    /// For [`Form::values`](crate::form::Form::values), as posted.
    pub values: Vec<(String, String)>,
}

/// Values tried in place of a missing or unreadable field, so the fields after it are still
/// read (the form is refused anyway, so none of them reaches the handler).
const STAND_INS: [&str; 7] = [
    "",
    "0",
    "false",
    "1970-01-01",
    "1970-01-01T00:00:00",
    "00:00",
    "00000000-0000-0000-0000-000000000000",
];

impl<T: DeserializeOwned + Validate> Valid<T> {
    /// What the extractor does with the posted `(name, value)` pairs.
    pub fn check(values: Vec<(String, String)>) -> Self {
        let mut input: Vec<(String, String)> = values
            .iter()
            .map(|(n, v)| (n.clone(), v.trim().to_string()))
            .filter(|(_, v)| !v.is_empty())
            .collect();
        let mut errors: Vec<(String, String)> = Vec::new();
        // Each round either succeeds or records one more field, so this ends.
        let parsed = loop {
            let pairs = input.iter().map(|(n, v)| (n.as_str(), v.as_str()));
            let encoded = form_urlencoded::Serializer::new(String::new())
                .extend_pairs(pairs)
                .finish();
            let de =
                serde_urlencoded::Deserializer::new(form_urlencoded::parse(encoded.as_bytes()));
            let err = match serde_path_to_error::deserialize::<_, T>(de) {
                Ok(value) => break Some(value),
                Err(err) => err,
            };
            let Some((field, message)) = failed_field(&err) else {
                break None;
            };
            let tried = input.iter().position(|(n, _)| *n == field);
            let next = match tried {
                None => Some(STAND_INS[0]),
                Some(i) => STAND_INS
                    .iter()
                    .position(|s| *s == input[i].1)
                    .map_or(Some(STAND_INS[0]), |k| STAND_INS.get(k + 1).copied()),
            };
            if !errors.iter().any(|(f, _)| *f == field) {
                errors.push((field.clone(), message.to_string()));
            }
            let Some(next) = next else { break None };
            match tried {
                Some(i) => input[i].1 = next.to_string(),
                None => input.push((field, next.to_string())),
            }
        };
        let Some(parsed) = parsed else {
            if errors.is_empty() {
                errors.push((String::new(), "Check the form.".to_string()));
            }
            return Self::refused(errors, values);
        };
        if let Err(e) = Validate::validate(&parsed) {
            for (field, message) in FieldErrors::from(&e).0 {
                if !errors.iter().any(|(f, _)| *f == field) {
                    errors.push((field, message));
                }
            }
        }
        if errors.is_empty() {
            Self(Ok(parsed))
        } else {
            Self::refused(errors, values)
        }
    }

    fn refused(mut errors: Vec<(String, String)>, values: Vec<(String, String)>) -> Self {
        errors.sort();
        Self(Err(Invalid {
            errors: FieldErrors(errors),
            values,
        }))
    }
}

/// The field a deserialization error is about, and the message it gets.
fn failed_field(
    err: &serde_path_to_error::Error<serde::de::value::Error>,
) -> Option<(String, &'static str)> {
    if let Some(serde_path_to_error::Segment::Map { key }) = err.path().iter().next() {
        return Some((key.clone(), "Check this field."));
    }
    // Serde's own wording: "missing field `title`".
    let text = err.inner().to_string();
    let field = text.strip_prefix("missing field `")?.strip_suffix('`')?;
    Some((field.to_string(), "This field is required."))
}

impl<S: Send + Sync, T: DeserializeOwned + Validate> FromRequest<S> for Valid<T> {
    type Rejection = BytesRejection;

    async fn from_request(req: Request, state: &S) -> std::result::Result<Self, Self::Rejection> {
        let body = Bytes::from_request(req, state).await?;
        Ok(Self::check(
            form_urlencoded::parse(&body).into_owned().collect(),
        ))
    }
}

/// A checkbox as a `bool`: `on` (a bare checkbox), `true` or `1` is ticked. For
/// `#[serde(default, deserialize_with = "loco_ui::loco::checkbox")]`; the `default` makes an
/// unticked box, which posts nothing, `false`.
pub fn checkbox<'de, D: Deserializer<'de>>(d: D) -> std::result::Result<bool, D::Error> {
    let value = String::deserialize(d)?;
    Ok(matches!(value.as_str(), "on" | "true" | "1"))
}

/// A date-time or time as a browser posts it, for `#[serde(default, deserialize_with =
/// "loco_ui::loco::local")]` on a `DateTime`, `DateTimeWithTimeZone` or `Time` field (or an
/// `Option` of one). `<input type="datetime-local">` posts `2026-01-31T09:00` and
/// `type="time"` posts `09:00`, without seconds, which chrono's own parsing refuses. Seconds
/// are read when present; a date-time with no offset is taken as UTC.
///
/// ```rust
/// use chrono::{DateTime, FixedOffset, NaiveDateTime, NaiveTime};
/// use loco_ui::loco::Local;
/// assert_eq!(NaiveDateTime::read("2026-01-31T09:00").unwrap().to_string(), "2026-01-31 09:00:00");
/// assert_eq!(DateTime::<FixedOffset>::read("2026-01-31T09:00").unwrap().to_rfc3339(), "2026-01-31T09:00:00+00:00");
/// assert_eq!(NaiveTime::read("09:30").unwrap().to_string(), "09:30:00");
/// assert!(NaiveTime::read("half past nine").is_none());
/// ```
pub fn local<'de, D: Deserializer<'de>, T: Local>(d: D) -> std::result::Result<T, D::Error> {
    let text = String::deserialize(d)?;
    T::read(&text).ok_or_else(|| serde::de::Error::custom("not a date or time"))
}

/// What [`local`] reads: a date-time or time from a form field's text.
pub trait Local: Sized {
    /// The value, or `None` when the text is not one.
    fn read(text: &str) -> Option<Self>;
}

impl Local for chrono::NaiveDateTime {
    fn read(text: &str) -> Option<Self> {
        let text = text.trim();
        Self::parse_from_str(text, "%Y-%m-%dT%H:%M:%S%.f")
            .or_else(|_| Self::parse_from_str(text, "%Y-%m-%dT%H:%M"))
            .ok()
    }
}

impl Local for chrono::DateTime<chrono::FixedOffset> {
    fn read(text: &str) -> Option<Self> {
        Self::parse_from_rfc3339(text.trim())
            .ok()
            .or_else(|| Some(chrono::NaiveDateTime::read(text)?.and_utc().fixed_offset()))
    }
}

impl Local for chrono::NaiveTime {
    fn read(text: &str) -> Option<Self> {
        let text = text.trim();
        Self::parse_from_str(text, "%H:%M:%S%.f")
            .or_else(|_| Self::parse_from_str(text, "%H:%M"))
            .ok()
    }
}

impl<T: Local> Local for Option<T> {
    fn read(text: &str) -> Option<Self> {
        T::read(text).map(Some)
    }
}

/// What a row is called in a list of choices (a select of parent rows, say): its `name`,
/// `title`, `label` or `email`, the first that is a non-empty string, else `#<id>`. Loco's
/// entities derive `Serialize`, so any model works.
///
/// ```rust
/// #[derive(serde::Serialize)]
/// struct User { id: i32, email: String, name: String }
/// let ada = User { id: 7, email: "ada@example.com".into(), name: "Ada".into() };
/// assert_eq!(loco_ui::loco::label(&ada), "Ada");
/// #[derive(serde::Serialize)]
/// struct Tag { id: i32 }
/// assert_eq!(loco_ui::loco::label(&Tag { id: 3 }), "#3");
/// ```
pub fn label<T: serde::Serialize>(row: &T) -> String {
    let value = serde_json::to_value(row).unwrap_or_default();
    ["name", "title", "label", "email"]
        .iter()
        .find_map(|k| value[k].as_str().filter(|s| !s.is_empty()))
        .map(str::to_string)
        .unwrap_or_else(|| format!("#{}", value["id"]))
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

    #[derive(Debug, serde::Deserialize, Validate)]
    struct Signup {
        #[validate(length(min = 2, message = "At least 2 characters."))]
        name: String,
        #[validate(email(message = "Enter an email address."))]
        email: String,
        age: u32,
        seats: Option<u8>,
        host: std::net::Ipv4Addr,
        #[serde(default, deserialize_with = "checkbox")]
        terms: bool,
    }

    fn posted(s: &str) -> Vec<(String, String)> {
        form_urlencoded::parse(s.as_bytes()).into_owned().collect()
    }

    #[test]
    fn valid_reports_every_bad_field_at_once() {
        let form = "name=A&email=&age=old&seats=&host=10.0.0.1";
        let bad = Valid::<Signup>::check(posted(form)).0.unwrap_err();
        assert_eq!(
            bad.errors.pairs(),
            [
                ("age", "Check this field."),
                ("email", "This field is required."),
                ("name", "At least 2 characters."),
            ]
        );
        assert_eq!(bad.values, posted(form));
        // No stand-in reads as an address, so the rules cannot run; the field still says why.
        let bad = Valid::<Signup>::check(posted("name=A&email=&age=1"))
            .0
            .unwrap_err();
        assert_eq!(
            bad.errors.pairs(),
            [
                ("email", "This field is required."),
                ("host", "This field is required.")
            ]
        );
    }

    #[test]
    fn valid_trims_leaves_out_empty_options_and_reads_checkboxes() {
        let ok = Valid::<Signup>::check(posted(
            "name=+Ada+&email=ada%40example.com&age=36&seats=&host=10.0.0.1&terms=on",
        ));
        let ok = ok.0.unwrap();
        assert_eq!((ok.name.as_str(), ok.age, ok.seats), ("Ada", 36, None));
        assert!(ok.terms && ok.host.is_private());
        let unticked =
            Valid::<Signup>::check(posted("name=Ada&email=a%40b.co&age=1&host=10.0.0.1"));
        assert!(!unticked.0.unwrap().terms);
    }

    async fn signup(Valid(form): Valid<Signup>) -> String {
        match form {
            Ok(s) => format!("ok {}", s.name),
            Err(bad) => format!("{:?}", bad.errors.get("email")),
        }
    }

    #[tokio::test]
    async fn valid_is_an_extractor() {
        let app = Router::new().route("/signup", axum::routing::post(signup));
        for (body, answer) in [
            (
                "name=Ada&email=ada%40example.com&age=36&host=10.0.0.1",
                "ok Ada",
            ),
            (
                "name=Ada&email=nope&age=36&host=10.0.0.1",
                r#"Some("Enter an email address.")"#,
            ),
        ] {
            let req = Request::post("/signup").body(Body::from(body)).unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap();
            assert_eq!(bytes, answer.as_bytes());
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
            ("/lui/caps?flag=popover", 204),
        ] {
            let req = Request::get(path).body(Body::empty()).unwrap();
            let res = app.clone().oneshot(req).await.unwrap();
            assert_eq!(res.status(), status, "{path}");
        }
    }
}
