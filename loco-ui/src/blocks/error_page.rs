//! # Error page
//!
//! The page for a 404 or a 500: the status, what happened in a sentence, and a way home, in
//! the site's look instead of the framework's plain text. [`not_found`] is an Axum handler
//! for `Router::fallback`, so every unknown path gets it; on Loco, add it in `after_routes`.
//!
//! **Platform features:** none of its own; the HTTP status is set on the response.
//!
//! **Accessibility:** the title is the page's `<h1>`; the status number is decorative and
//! hidden from assistive tech (the title says it). Checked by axe-core with the demo routes.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.error_page(404).render().into_string();
//! assert!(m.contains("<h1>Page not found</h1>") && m.contains(r#"href="/""#));
//! let m = ui.error_page(500).home("/dashboard").render().into_string();
//! assert!(m.contains("Something went wrong") && m.contains(r#"href="/dashboard""#));
//! // The same in `lui!`:
//! let same = lui! { ErrorPage(500) home="/dashboard"; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::i18n::Text;
use crate::props::{Prop, PropKind};
use crate::{Page, Ui};

/// A 404 or 500 page, made by [`Ui::error_page`].
///
/// **Setters.** Values and items: `.title(..)`, `.message(..)`, `.home(..)`.
#[derive(Clone, Debug)]
pub struct ErrorPage<'a> {
    ui: &'a Ui,
    status: u16,
    title: &'a str,
    message: &'a str,
    home: &'a str,
}

impl ErrorPage<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("title", PropKind::Value, "text: &'a str")
            .default("Page not found / Something went wrong")
            .doc("The heading."),
        Prop::new("message", PropKind::Value, "text: &'a str").doc("The sentence under it."),
        Prop::new("home", PropKind::Value, "href: &'a str")
            .default("/")
            .attr("href")
            .doc("Where the button goes."),
    ];
}

impl Ui {
    /// The page for HTTP `status`: 404 says the page is not here, anything else that
    /// something went wrong.
    pub fn error_page(&self, status: u16) -> ErrorPage<'_> {
        let (title, message) = if status == 404 {
            (Text::NotFound, Text::NotFoundMessage)
        } else {
            (Text::ServerError, Text::ServerErrorMessage)
        };
        ErrorPage {
            ui: self,
            status,
            title: self.text(title),
            message: self.text(message),
            home: "/",
        }
    }
}

impl<'a> ErrorPage<'a> {
    /// The heading.
    pub fn title(mut self, text: &'a str) -> Self {
        self.title = text;
        self
    }

    /// The sentence under it.
    pub fn message(mut self, text: &'a str) -> Self {
        self.message = text;
        self
    }

    /// Where the button goes.
    pub fn home(mut self, href: &'a str) -> Self {
        self.home = href;
        self
    }

    /// The whole page, titled by the heading.
    pub fn page(&self) -> Page {
        self.ui.page(self.title, self.render())
    }
}

impl Render for ErrorPage<'_> {
    fn render(&self) -> Markup {
        let ui = self.ui;
        html! {
            div class="lui-error-page" {
                p class="lui-error-page-status" aria-hidden="true" { (self.status) }
                h1 { (self.title) }
                p { (self.message) }
                (ui.link_button(ui.text(Text::GoHome), self.home).primary())
            }
        }
    }
}

#[cfg(feature = "axum")]
mod axum_glue {
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};

    use super::ErrorPage;
    use crate::Ui;

    /// The page with its status: `ui.error_page(404).into_response()` answers 404.
    impl IntoResponse for ErrorPage<'_> {
        fn into_response(self) -> Response {
            let status =
                StatusCode::from_u16(self.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
            (status, self.page()).into_response()
        }
    }

    /// The 404 page for `Router::fallback(loco_ui::blocks::not_found)`: every path no route
    /// answers gets it, in the visitor's theme and language.
    pub async fn not_found(ui: Ui) -> Response {
        ui.error_page(404).into_response()
    }
}

#[cfg(feature = "axum")]
pub use axum_glue::not_found;

/// Styles for this block; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-error-page { display: grid; justify-items: center; gap: var(--lui-space); padding-block: calc(var(--lui-space) * 8); text-align: center; }
.lui-error-page h1 { margin: 0; }
.lui-error-page p { margin: 0; color: var(--lui-muted); max-width: 32rem; }
.lui-error-page .lui-error-page-status { font-size: 3rem; line-height: 1; font-weight: 700; color: var(--lui-fg); letter-spacing: -0.02em; }
.lui-error-page .lui-button { margin-top: calc(var(--lui-space) * 2); }
"#;
