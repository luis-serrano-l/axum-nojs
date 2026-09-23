//! # webonsive
//!
//! Interactive HTML components for Rust servers that need no JavaScript.
//!
//! Every component is a plain function that returns [`maud::Markup`]. Interactivity comes from
//! the HTML and CSS platform (dialog, popover, invokers, `<details name>`, datalist, view
//! transitions) and from ordinary form round trips. No page produced by this crate needs a
//! `<script>` tag; the one optional script in [`enhance`] only makes the same markup update
//! in place.
//!
//! Every component takes `&Caps` first and emits only the markup that browser needs: the
//! modern variant or the fallback, never both. See [`caps`] for how the server learns it.
//!
//! One component lives in one file. Each file starts with a doc header that lists the platform
//! features it relies on, the browser baseline, and the fallback for older browsers.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, layout, dialog, Theme};
//!
//! // `Caps` says what the browser supports; the demo reads it from a cookie set by beacons.
//! let caps = Caps::all();
//! let page = layout(&caps, "Hello", Theme::Auto, html! {
//!     (dialog(&caps, "hi", "Say hi", html! { p { "Hello from a <dialog>." } }, false))
//! });
//! // The only script is the optional enhancement tag; the page works without it.
//! assert_eq!(page.into_string().matches("<script").count(), 1);
//! ```

#![warn(missing_docs)]

pub mod accordion;
pub mod caps;
pub mod color;
pub mod combobox;
pub mod counter;
pub mod dialog;
pub mod enhance;
pub mod flash;
pub mod form;
pub mod layout;
pub mod pager;
pub mod popover;
pub mod range;
pub mod select;
pub mod spec;
pub mod state;
#[cfg(feature = "axum")]
pub mod stream;
pub mod tabs;
pub mod theme;

pub use accordion::accordion;
pub use caps::{Cap, Caps};
pub use color::color;
pub use combobox::combobox;
pub use counter::counter;
pub use dialog::dialog;
pub use flash::flash;
pub use form::{Field, FieldKind, form};
pub use layout::layout;
pub use pager::pager;
pub use popover::popover_menu;
pub use range::range;
pub use select::select;
#[cfg(feature = "axum")]
pub use state::prg;
pub use state::UiState;
#[cfg(feature = "axum")]
pub use stream::{Streamed, slot};
pub use tabs::tabs;
pub use theme::{Theme, theme_toggle};

/// All component stylesheets, concatenated. `layout` inlines this once per page.
pub fn stylesheet() -> String {
    let beacons = caps::beacon_css();
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
        flash::CSS,
        select::CSS,
        range::CSS,
        color::CSS,
        #[cfg(feature = "axum")]
        stream::CSS,
        beacons.as_str(),
    ]
    .join("\n")
}
