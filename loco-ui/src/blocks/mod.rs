//! # Blocks
//!
//! Whole pages built from the components: an app shell with a sidebar, a sign-in or sign-up
//! card, a settings page, a record's show page, a dashboard of numbers, and the 404 and 500
//! pages. Each is a builder reached from `ui` like a component, so a route fills one in and
//! wraps it in `ui.page(..)`; none adds a platform feature or a line of script of its own,
//! and each is one file to read and copy when yours needs to differ.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from_request("/reports", "", "");
//! let shell = ui.app_shell("Acme")
//!     .link("Dashboard", "/")
//!     .link("Reports", "/reports")
//!     .body(html! { h1 { "Reports" } });
//! let m = shell.render().into_string();
//! assert!(m.contains(r#"<a href="/reports" aria-current="page">Reports</a>"#));
//! // The same in `lui!`:
//! let same = lui! { AppShell("Acme") { link "Dashboard" "/"; link "Reports" "/reports"; body (html! { h1 { "Reports" } }); } };
//! assert_eq!(same.into_string(), m);
//! ```

pub mod app_shell;
pub mod auth_page;
pub mod dashboard_page;
pub mod error_page;
pub mod record_page;
pub mod settings_page;

#[cfg(feature = "axum")]
pub use error_page::not_found;

/// The blocks' styles, appended to the stylesheet after the components'.
pub(crate) const CSS: [&str; 6] = [
    app_shell::CSS,
    auth_page::CSS,
    settings_page::CSS,
    record_page::CSS,
    dashboard_page::CSS,
    error_page::CSS,
];
