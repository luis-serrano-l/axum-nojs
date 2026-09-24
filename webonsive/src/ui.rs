//! # Ui
//!
//! Everything a page handler reads from the request before it renders, in one value: the
//! browser's [`Caps`], the [`Theme`] from its cookie and the [`UiState`] (with the flash).
//! A route then reads as "parse input, render components":
//!
//! ```rust,ignore
//! async fn dialog_page(ui: Ui) -> (Ui, Markup) {
//!     let page = ui.layout("Dialog", html! { (ui.flash()) (dialog(&ui, "hi", "Say hi", body)) });
//!     (ui, page)
//! }
//! ```
//!
//! `Ui` dereferences to [`Caps`], so `&ui` goes wherever a component asks for `&Caps`.
//! Returning it beside the page (`(ui, markup)`) writes the changed state back and clears the
//! flash, exactly like returning the [`UiState`].
//!
//! **Any server.** [`Ui::from_request`] takes the path, the query string and the whole
//! `Cookie:` header; the `axum` feature adds the extractor and `IntoResponseParts`.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Theme, Ui, dialog};
//! let ui = Ui::from_request("/dialog", "dialog=hi", "theme=dark; wo-flash=Saved.; wo-cap-dialog=1; wo-cap-probed=1");
//! assert_eq!(ui.theme, Theme::Dark);
//! assert_eq!(ui.state.dialog(), Some("hi"));
//! let page = ui.layout("Dialog", html! { (ui.flash()) (dialog(&ui, "hi", "Say hi", html! { p { "Hello." } })) }).into_string();
//! assert!(page.contains(r#"data-theme="dark""#) && page.contains("Saved."));
//! assert_eq!(ui.set_cookies().len(), 2, "?dialog=hi is remembered and the shown flash cleared");
//! ```

use std::ops::Deref;

use maud::Markup;

use crate::{Caps, Theme, UiState, flash, layout, theme::THEME_COOKIE};

/// Caps, theme and UI state for one request.
#[derive(Clone, Debug)]
pub struct Ui {
    /// What the browser supports.
    pub caps: Caps,
    /// The theme from the `theme` cookie, `Auto` without one.
    pub theme: Theme,
    /// Query and `wo-ui` cookie state, and the flash.
    pub state: UiState,
}

impl Ui {
    /// Build from a raw request: path, query string and the whole `Cookie:` header value.
    /// Caps come from `?caps=` first, then the beacon cookies.
    pub fn from_request(path: &str, query: &str, cookie_header: &str) -> Ui {
        let theme = cookie_header
            .split(';')
            .filter_map(|pair| pair.trim().split_once('='))
            .find(|(k, _)| *k == THEME_COOKIE)
            .map_or(Theme::Auto, |(_, v)| Theme::parse(v));
        Ui {
            caps: Caps::from_query(query).unwrap_or_else(|| Caps::from_cookie_header(cookie_header)),
            theme,
            state: UiState::from_request(path, query, cookie_header),
        }
    }

    /// The flash banner for this request, or nothing when there is no message.
    pub fn flash(&self) -> Markup {
        flash(&self.caps, self.state.flash())
    }

    /// A whole page in this request's theme: [`layout()`] with the caps and theme filled in.
    pub fn layout(&self, title: &str, body: Markup) -> Markup {
        layout(&self.caps, title, self.theme, body)
    }

    /// The `Set-Cookie` values to send back; see [`UiState::set_cookies`].
    pub fn set_cookies(&self) -> Vec<String> {
        self.state.set_cookies()
    }
}

impl Deref for Ui {
    type Target = Caps;

    fn deref(&self) -> &Caps {
        &self.caps
    }
}

#[cfg(feature = "axum")]
mod axum_glue {
    use super::Ui;
    use axum::{
        extract::FromRequestParts,
        http::{header, request::Parts},
        response::{IntoResponseParts, ResponseParts},
    };

    /// A thin wrapper over [`Ui::from_request`].
    impl<S: Send + Sync> FromRequestParts<S> for Ui {
        type Rejection = std::convert::Infallible;

        async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Ui, Self::Rejection> {
            let cookies = parts
                .headers
                .get_all(header::COOKIE)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect::<Vec<_>>()
                .join("; ");
            Ok(Ui::from_request(parts.uri.path(), parts.uri.query().unwrap_or(""), &cookies))
        }
    }

    /// Returning `(ui, markup)` persists changed state and clears the flash.
    impl IntoResponseParts for Ui {
        type Error = std::convert::Infallible;

        fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
            self.state.into_response_parts(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cap;

    #[test]
    fn one_value_carries_caps_theme_state_and_flash() {
        let ui = Ui::from_request("/t", "tab.t=1&caps=popover", "theme=light; wo-ui=open.f=2");
        assert!(ui.has(Cap::Popover) && !ui.has(Cap::Invokers), "?caps= wins and Deref reaches Caps");
        assert_eq!((ui.theme, ui.state.tab("t"), ui.state.open("f")), (Theme::Light, 1, Some(2)));
        assert_eq!(ui.flash().into_string(), "", "no message, no banner");
        let bare = Ui::from_request("/", "", "");
        assert_eq!(bare.theme, Theme::Auto);
        assert_eq!(bare.caps, Caps::from_cookie_header(""));
    }
}
