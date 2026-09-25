//! # Ui
//!
//! The one value a handler needs: the browser's [`Caps`], the [`Theme`] from its cookie and
//! the [`UiState`] (with the flash). Every component starts from it, so a route reads like the
//! page it describes:
//!
//! ```rust
//! use loco_ui::prelude::*;
//!
//! let ui = Ui::from_request("/account", "dialog=delete-account", "lui-flash=Saved.");
//! let page = ui.page("Account", html! {
//!     (ui.dialog("Delete account")
//!         .title("Delete account?")
//!         .danger()
//!         .confirm("Delete", "/account/delete")
//!         .body(html! { p { "This cannot be undone." } }))
//! });
//! let html = page.into_string();
//! assert!(html.contains("Saved.") && html.contains(" open>"), "the flash shows and ?dialog= opens it");
//! ```
//!
//! The page shows a pending flash at the top of the body by itself; place `(ui.flash())`
//! (or [`Ui::toasts`]) in the body to show it somewhere else, or with its setters.
//!
//! A component is a builder that renders where `html!` splices it. The id, the caps, the open
//! state and where a form returns to all come from `ui`; a setter is needed only for what the
//! page says differently.
//!
//! **Responses.** [`Ui::page`] is the whole document; as an Axum response it also writes back
//! the UI state the query changed and clears a flash it showed. [`Ui::redirect`] answers a
//! form post (Post/Redirect/Get) with a flash and, with the `axum` feature, values kept in a
//! cookie ([`crate::Saved`]).
//!
//! **Any server.** [`Ui::from_request`] takes the path, the query string and the whole
//! `Cookie:` header, and `Ui::from(caps)` works where there is no request at all. The `axum`
//! feature adds the extractor and the `IntoResponse` impls.

use std::ops::Deref;

use maud::{Markup, Render};

use crate::{
    Caps, Theme, UiState,
    flash::{Level, stack},
    i18n::{self, Strings, Text},
    layout::{self, Tokens},
    state::{FLASH_COOKIE, decode, encode},
    theme::THEME_COOKIE,
};

/// Caps, theme and UI state for one request.
#[derive(Clone, Debug, Default)]
pub struct Ui {
    /// What the browser supports.
    pub caps: Caps,
    /// The theme from the `theme` cookie, `Auto` without one.
    pub theme: Theme,
    /// Query and `lui-ui` cookie state, and the flash.
    pub state: UiState,
    /// The visitor's language: its texts and tag ([`crate::i18n`]).
    pub strings: &'static Strings,
    /// Whether the `lui-lang` cookie chose the language (then `Accept-Language` does not).
    lang_from_cookie: bool,
    /// Every query parameter, decoded, in order: what components read their own input from.
    params: Vec<(String, String)>,
}

/// A request with no state: only what the browser supports.
impl From<Caps> for Ui {
    fn from(caps: Caps) -> Ui {
        Ui {
            caps,
            ..Ui::default()
        }
    }
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
        let lang = cookie_header
            .split(';')
            .filter_map(|pair| pair.trim().split_once('='))
            .find(|(k, _)| *k == i18n::LANG_COOKIE)
            .map(|(_, v)| v);
        let strings = i18n::choose(lang, "");
        Ui {
            strings,
            lang_from_cookie: lang.is_some_and(|l| strings.lang().eq_ignore_ascii_case(l.trim())),
            caps: Caps::from_query(query)
                .unwrap_or_else(|| Caps::from_cookie_header(cookie_header)),
            theme,
            state: UiState::from_request(path, query, cookie_header),
            params: query
                .split('&')
                .filter(|p| !p.is_empty())
                .map(|p| p.split_once('=').unwrap_or((p, "")))
                .map(|(k, v)| (decode(k).into_owned(), decode(v).into_owned()))
                .collect(),
        }
    }

    /// Pick the language from an `Accept-Language` header, unless the `lui-lang` cookie
    /// already did. The Axum extractor calls it; other servers pass the header themselves.
    pub fn accept_language(mut self, header: &str) -> Ui {
        if !self.lang_from_cookie {
            self.strings = i18n::choose(None, header);
        }
        self
    }

    /// The visitor's language tag, as `<html lang>` says it.
    pub fn lang(&self) -> &'static str {
        self.strings.lang()
    }

    /// A component text in the visitor's language.
    pub fn text(&self, text: Text) -> &'static str {
        self.strings.get(text)
    }

    /// A component text in the visitor's language, its placeholders filled.
    pub fn fill(&self, text: Text, args: &[&dyn std::fmt::Display]) -> String {
        self.strings.fill(text, args)
    }

    /// The first value of the query parameter `key`.
    pub fn param(&self, key: &str) -> Option<&str> {
        self.params
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// Every value of the query parameter `key`, in order (`?sel=a&sel=b`).
    pub fn params<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.params
            .iter()
            .filter(move |(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// This page's URL with the query parameter `key` set to `value` (added last when absent)
    /// and every other parameter kept in order: a link that changes one thing, the way the
    /// calendar's month links and the table's "Edit" links are built.
    ///
    /// ```rust
    /// use loco_ui::prelude::*;
    /// let ui = Ui::from_request("/orders", "sort=date&page=2", "");
    /// assert_eq!(ui.link_with("page", "3"), "/orders?sort=date&page=3");
    /// assert_eq!(ui.link_with("q", "late fee"), "/orders?sort=date&page=2&q=late%20fee");
    /// assert_eq!(ui.link_without("page"), "/orders?sort=date");
    /// ```
    pub fn link_with(&self, key: &str, value: &str) -> String {
        self.link_changing(key, Some(value))
    }

    /// This page's URL without the query parameter `key` (the path alone when nothing is left).
    pub fn link_without(&self, key: &str) -> String {
        self.link_changing(key, None)
    }

    fn link_changing(&self, key: &str, value: Option<&str>) -> String {
        use crate::state::encode;
        let mut pairs: Vec<(&str, &str)> = self
            .params
            .iter()
            .filter(|(k, _)| k != key)
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();
        pairs.extend(value.map(|v| (key, v)));
        let query: Vec<String> = pairs
            .iter()
            .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
            .collect();
        if query.is_empty() {
            self.state.path().to_string()
        } else {
            format!("{}?{}", self.state.path(), query.join("&"))
        }
    }

    /// A whole page titled `title` in this request's theme. A flash a [`Ui::redirect`] left
    /// shows at the top of `body` unless `body` already shows it (`ui.flash()` placed
    /// elsewhere, with its setters, or as [`Ui::toasts`]).
    pub fn page(&self, title: &str, body: Markup) -> Page {
        let shown = |b: &Markup| {
            let b = b.0.as_str();
            b.contains(r#"class="lui-flash""#) || b.contains(r#"class="lui-toasts""#)
        };
        let body = if self.state.flash().is_some() && !shown(&body) {
            maud::html! { (self.flash()) (body) }
        } else {
            body
        };
        Page {
            caps: self.caps,
            theme: self.theme,
            lang: self.strings.lang(),
            title: title.to_string(),
            tokens: None,
            body,
            cookies: self.state.set_cookies(),
            css: Vec::new(),
            script: true,
            status: 200,
        }
    }

    /// Post/Redirect/Get: a `303 See Other` to `to`. Add messages with [`Redirect::flash`]
    /// and friends; they show on the next page through [`Ui::flash`] or [`Ui::toasts`].
    pub fn redirect(&self, to: &str) -> Redirect {
        Redirect {
            to: to.to_string(),
            messages: Vec::new(),
            cookies: Vec::new(),
        }
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

/// A whole HTML document, made by [`Ui::page`]. Renders the layout (head, stylesheet, header,
/// `<main>`, beacons, the optional script); as an Axum response it also carries the
/// `Set-Cookie` values of the request's state.
#[derive(Clone, Debug)]
pub struct Page {
    caps: Caps,
    lang: &'static str,
    theme: Theme,
    title: String,
    tokens: Option<Tokens>,
    body: Markup,
    cookies: Vec<String>,
    css: Vec<&'static str>,
    script: bool,
    status: u16,
}

impl Page {
    /// Answer `422 Unprocessable Content`: the form comes back with its messages, so a POST
    /// handler returns `Result<Redirect, Page>`, `Ok` for Post/Redirect/Get and
    /// `Err(page.invalid())` for the form again.
    ///
    /// ```rust
    /// use loco_ui::prelude::*;
    /// let ui = Ui::default();
    /// let page = ui.page("Sign up", html! { p { "Check the fields." } }).invalid();
    /// assert_eq!(page.status(), 422);
    /// assert_eq!(ui.page("Sign up", html! {}).status(), 200);
    /// ```
    pub fn invalid(mut self) -> Self {
        self.status = 422;
        self
    }

    /// The HTTP status this page answers with: 200, or 422 after [`Page::invalid`].
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Leave out the enhancement script: the page is exactly what Blitz renders, and
    /// [`crate::enhance::csp`] answers it with `script-src 'none'`
    /// ([`crate::enhance::CSP_NO_SCRIPT`]).
    pub fn without_script(mut self) -> Self {
        self.script = false;
        self
    }

    /// Whether the page carries the enhancement script tag.
    pub fn has_script(&self) -> bool {
        self.script
    }

    /// Add a stylesheet to this page's `<head>`, after the library's: a component of your own
    /// ships its `CSS` const this way (see `docs/components.md`). Call it once per component;
    /// each is minified and inlined once.
    pub fn css(mut self, css: &'static str) -> Self {
        if !self.css.contains(&css) {
            self.css.push(css);
        }
        self
    }

    /// Render under other [`Tokens`] (a palette is a value, see `docs/theming.md`).
    pub fn tokens(mut self, tokens: &Tokens) -> Self {
        self.tokens = Some(*tokens);
        self
    }

    /// The `Set-Cookie` values this page sends: changed UI state, and the shown flash cleared.
    pub fn set_cookies(&self) -> &[String] {
        &self.cookies
    }

    /// The document as a string, for any server.
    pub fn into_string(self) -> String {
        self.render().into_string()
    }
}

impl Render for Page {
    fn render(&self) -> Markup {
        layout::page(
            &self.caps,
            self.lang,
            &self.title,
            self.theme,
            self.tokens.as_ref(),
            &self.css,
            self.script,
            self.body.clone(),
        )
    }
}

/// A `303 See Other` answering a form post, made by [`Ui::redirect`]. Messages ride along in
/// the one-shot `lui-flash` cookie; each call adds one, and they show stacked in call order.
#[derive(Clone, Debug)]
pub struct Redirect {
    to: String,
    messages: Vec<(Level, String)>,
    cookies: Vec<String>,
}

impl Redirect {
    /// A neutral message.
    pub fn flash(self, message: &str) -> Self {
        self.say(Level::Info, message)
    }

    /// Something worked.
    pub fn ok(self, message: &str) -> Self {
        self.say(Level::Ok, message)
    }

    /// Worked, but look at this.
    pub fn warn(self, message: &str) -> Self {
        self.say(Level::Warn, message)
    }

    /// Something failed: announced at once and never faded.
    pub fn danger(self, message: &str) -> Self {
        self.say(Level::Danger, message)
    }

    fn say(mut self, level: Level, message: &str) -> Self {
        self.messages.push((level, message.to_string()));
        self
    }

    /// Send one more `Set-Cookie` value with the redirect.
    pub fn cookie(mut self, set_cookie: String) -> Self {
        self.cookies.push(set_cookie);
        self
    }

    /// Where the browser goes next.
    pub fn location(&self) -> &str {
        &self.to
    }

    /// Every `Set-Cookie` value: the flash first, then the rest in call order.
    pub fn set_cookies(&self) -> Vec<String> {
        let flash = (!self.messages.is_empty()).then(|| {
            let pairs: Vec<(Level, &str)> = self
                .messages
                .iter()
                .map(|(l, m)| (*l, m.as_str()))
                .collect();
            // A lone info message stays plain text.
            let text = match pairs.as_slice() {
                [(Level::Info, m)] => m.to_string(),
                _ => stack(&pairs),
            };
            format!(
                "{FLASH_COOKIE}={}; Path=/; Max-Age=60; SameSite=Lax",
                encode(&text)
            )
        });
        flash
            .into_iter()
            .chain(self.cookies.iter().cloned())
            .collect()
    }

    /// As an `http::Response` with an empty body, for any server built on the `http` crate.
    #[cfg(feature = "http")]
    pub fn into_http<B: From<String>>(self) -> http::Response<B> {
        let mut res = http::Response::builder()
            .status(303)
            .header(http::header::LOCATION, &self.to);
        for c in self.set_cookies() {
            res = res.header(http::header::SET_COOKIE, c);
        }
        res.body(B::from(String::new()))
            .expect("valid redirect headers")
    }
}

#[cfg(feature = "axum")]
mod axum_glue {
    use super::{Page, Redirect, Ui};
    use axum::{
        extract::FromRequestParts,
        http::{HeaderValue, header, request::Parts},
        response::{Html, IntoResponse, IntoResponseParts, Response, ResponseParts},
    };
    use maud::Render;

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
            let accept = parts
                .headers
                .get(header::ACCEPT_LANGUAGE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("");
            Ok(
                Ui::from_request(parts.uri.path(), parts.uri.query().unwrap_or(""), &cookies)
                    .accept_language(accept),
            )
        }
    }

    /// Returning `(ui, response)` persists changed state and clears the flash.
    impl IntoResponseParts for Ui {
        type Error = std::convert::Infallible;

        fn into_response_parts(self, res: ResponseParts) -> Result<ResponseParts, Self::Error> {
            self.state.into_response_parts(res)
        }
    }

    fn with_cookies(mut res: Response, cookies: impl IntoIterator<Item = String>) -> Response {
        for c in cookies {
            res.headers_mut().append(
                header::SET_COOKIE,
                HeaderValue::from_str(&c).expect("cookie is ASCII"),
            );
        }
        res
    }

    impl IntoResponse for Page {
        fn into_response(self) -> Response {
            let mut res = Html(self.render().into_string()).into_response();
            *res.status_mut() =
                axum::http::StatusCode::from_u16(self.status).unwrap_or(axum::http::StatusCode::OK);
            if !self.script {
                // For `enhance::csp`: this page may be served under `script-src 'none'`.
                res.extensions_mut().insert(crate::enhance::NoScript);
            }
            with_cookies(res, self.cookies)
        }
    }

    impl IntoResponse for Redirect {
        fn into_response(self) -> Response {
            self.into_http()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Cap;

    #[test]
    fn one_value_carries_caps_theme_state_and_flash() {
        let ui = Ui::from_request("/t", "tab.t=1&caps=popover", "theme=light; lui-ui=open.f=2");
        assert!(
            ui.has(Cap::Popover) && !ui.has(Cap::Invokers),
            "?caps= wins and Deref reaches Caps"
        );
        assert_eq!(
            (ui.theme, ui.state.tab("t"), ui.state.open("f")),
            (Theme::Light, 1, Some(2))
        );
        let q = Ui::from_request("/", "q=a+b&sel=x&sel=y%2Cz", "");
        assert_eq!(
            (q.param("q"), q.params("sel").collect::<Vec<_>>()),
            (Some("a b"), vec!["x", "y,z"])
        );
        let bare = Ui::from_request("/", "", "");
        assert_eq!(bare.theme, Theme::Auto);
        assert_eq!(bare.caps, Caps::from_cookie_header(""));
    }

    #[test]
    fn a_page_carries_the_state_it_changed() {
        let ui = Ui::from_request("/t", "tab.t=1", "theme=dark; lui-flash=Saved.");
        let page = ui.page("T", maud::html! { p { "body" } });
        assert_eq!(
            page.set_cookies().len(),
            2,
            "tab.t remembered, the flash cleared"
        );
        let html = page.tokens(&Tokens::default()).into_string();
        assert!(
            html.contains(r#"data-theme="dark""#)
                && html.contains("<title>T</title>")
                && html.contains("lui-tokens")
        );
    }

    #[test]
    fn a_page_shows_a_pending_flash_once() {
        let ui = Ui::from_request("/t", "", "lui-flash=Saved.");
        let count = |body: Markup| {
            let html = ui.page("T", body).into_string();
            html.matches(r#"class="lui-flash""#).count()
                + html.matches(r#"class="lui-toasts""#).count()
        };
        assert_eq!(count(maud::html! { p { "body" } }), 1, "shown by the page");
        assert_eq!(
            count(maud::html! { p { "x" } (ui.flash().dismiss()) }),
            1,
            "the body's own"
        );
        assert_eq!(count(maud::html! { (ui.toasts()) }), 1, "as toasts");
        let none = Ui::default()
            .page("T", maud::html! { p { "body" } })
            .into_string();
        assert!(!none.contains("lui-flash\""), "no message, no banner");
    }

    #[test]
    fn redirect_messages_stack_in_call_order() {
        let ui = Ui::default();
        let plain = ui.redirect("/s").flash("Saved.");
        assert_eq!(
            plain.set_cookies(),
            ["lui-flash=Saved.; Path=/; Max-Age=60; SameSite=Lax"]
        );
        let two = ui
            .redirect("/s")
            .ok("Saved.")
            .warn("Look.")
            .cookie("x=1".into());
        let cookies = two.set_cookies();
        assert!(
            cookies[0].starts_with("lui-flash=ok%3ASaved.%0Awarn%3ALook.") && cookies[1] == "x=1"
        );
        assert!(ui.redirect("/s").set_cookies().is_empty());
    }
}
