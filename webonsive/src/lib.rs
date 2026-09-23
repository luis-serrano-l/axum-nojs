//! # webonsive
//!
//! Zero-JavaScript interactive HTML components for Rust servers.
//!
//! Every component is a plain function that returns [`maud::Markup`]. Interactivity comes from
//! the HTML and CSS platform (dialog, popover, invokers, `<details name>`, datalist, view
//! transitions) and from ordinary form round trips. No page produced by this crate needs a
//! `<script>` tag.
//!
//! One component lives in one file. Each file starts with a doc header that lists the platform
//! features it relies on, the browser baseline, and the fallback for older browsers.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{layout, dialog, Theme};
//!
//! let page = layout("Hello", Theme::Auto, html! {
//!     (dialog("hi", "Say hi", html! { p { "Hello from a <dialog>." } }))
//! });
//! assert!(!page.into_string().contains("<script"));
//! ```

pub mod accordion;
pub mod combobox;
pub mod counter;
pub mod dialog;
pub mod form;
pub mod layout;
pub mod pager;
pub mod popover;
pub mod tabs;
pub mod theme;

pub use accordion::accordion;
pub use combobox::combobox;
pub use counter::counter;
pub use dialog::dialog;
pub use form::{Field, FieldKind, form};
pub use layout::layout;
pub use pager::pager;
pub use popover::popover_menu;
pub use tabs::tabs;
pub use theme::{Theme, theme_toggle};

/// All component stylesheets, concatenated. `layout` inlines this once per page.
pub fn stylesheet() -> String {
    [
        layout::CSS,
        dialog::CSS,
        popover::CSS,
        tabs::CSS,
        accordion::CSS,
        combobox::CSS,
        pager::CSS,
        form::CSS,
        counter::CSS,
        theme::CSS,
    ]
    .join("\n")
}
