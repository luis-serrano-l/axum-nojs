//! # State
//!
//! Where UI state lives when there is no script: in the URL and in a cookie.
//!
//! [`UiState`] is a small string map with five kinds of keys: `tab.<name>` (open tab index),
//! `open.<group>` (open accordion indexes, a comma list), `step.<wizard>` (current wizard step),
//! `per.<table>` (rows per page of a paged table) and `dialog` (id of a dialog to render open). It is
//! read from the query string first and a `wo-ui` cookie second, so a link can change one key
//! while everything else is remembered. In Axum it is an extractor, and returning it as part
//! of the response writes the cookie back when the query changed something.
//!
//! **Platform features:** links, cookies, `303 See Other`. Nothing newer than 1997.
//!
//! **Fallback:** none needed. Without cookies, state still travels in links on the same page.
//!
//! **Post/Redirect/Get:** [`prg`] answers a form POST with a redirect and a one-shot
//! `wo-flash` cookie; the next page renders it with the `flash` component and, by returning
//! its `UiState`, clears it.
//!
//! **Any server.** The protocol is plain strings: [`UiState::from_request`] reads path, query
//! and the `Cookie:` header; [`UiState::set_cookies`] gives the `Set-Cookie` values to send
//! back; [`prg_parts`] gives the redirect's status, `Location` and `Set-Cookie`. The `http`
//! feature adds [`prg`] as an `http::Response`; the `axum` feature adds the extractor and the
//! `IntoResponseParts` impl on top.
//!
//! ```rust
//! use webonsive::UiState;
//! let state = UiState::parse("/settings", "tab.settings=1&page=3", "open.faq=2");
//! assert_eq!(state.tab("settings"), 1);
//! assert_eq!(state.open("faq"), Some(2));
//! assert_eq!(state.link("tab.settings", "0"), "/settings?open.faq=2&tab.settings=0");
//!
//! // By hand, from a raw request: the cookie header carries both state and flash.
//! let state = UiState::from_request("/settings", "tab.settings=1", "wo-ui=open.faq=2; wo-flash=Saved.");
//! assert_eq!(state.flash(), Some("Saved."));
//! assert_eq!(state.set_cookies().len(), 2); // remember tab.settings, clear the flash
//!
//! // A form POST answered with Post/Redirect/Get, for any server.
//! let (status, location, cookie) = webonsive::state::prg_parts("/settings", Some("Saved."));
//! assert_eq!((status, location), (303, "/settings"));
//! assert!(cookie.unwrap().starts_with("wo-flash=Saved."));
//! ```

use std::collections::BTreeMap;

/// Name of the cookie that remembers UI state between page views.
pub const UI_COOKIE: &str = "wo-ui";

/// Name of the one-shot cookie carrying a flash message across a redirect.
pub const FLASH_COOKIE: &str = "wo-flash";

/// UI state for one request: query string merged over the `wo-ui` cookie.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiState {
    path: String,
    from_query: BTreeMap<String, String>,
    from_cookie: BTreeMap<String, String>,
    flash: Option<String>,
}

fn is_state_key(key: &str) -> bool {
    key == "dialog" || ["tab.", "open.", "step.", "per."].iter().any(|p| key.starts_with(p))
}

/// Parse `a=b&c=d` pairs, keeping only state keys. Understands `%XX` and `+`.
fn parse_pairs(input: &str) -> BTreeMap<String, String> {
    input
        .split('&')
        .filter_map(|pair| pair.split_once('=').or(Some((pair, ""))))
        .map(|(k, v)| (decode(k), decode(v)))
        .filter(|(k, _)| is_state_key(k))
        .collect()
}

fn decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => {
                match u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    Ok(b) => {
                        out.push(b);
                        i += 2;
                    }
                    Err(_) => out.push(b'%'),
                }
            }
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

pub(crate) fn encode(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => out.push(b as char),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

impl UiState {
    /// Build from the request path, its raw query string and the raw `wo-ui` cookie value.
    pub fn parse(path: &str, query: &str, cookie: &str) -> UiState {
        UiState {
            path: path.to_string(),
            from_query: parse_pairs(query),
            from_cookie: parse_pairs(cookie),
            flash: None,
        }
    }

    /// Build from a raw request: path, query string and the whole `Cookie:` header value
    /// (several headers joined with `; `). Reads both the `wo-ui` and the `wo-flash` cookie.
    pub fn from_request(path: &str, query: &str, cookie_header: &str) -> UiState {
        let cookie = |name: &str| {
            cookie_header
                .split(';')
                .filter_map(|pair| pair.trim().split_once('='))
                .find(|(k, _)| *k == name)
                .map(|(_, v)| v)
        };
        UiState::parse(path, query, cookie(UI_COOKIE).unwrap_or(""))
            .with_flash(cookie(FLASH_COOKIE).map(decode))
    }

    /// The `Set-Cookie` values a response should carry: the merged state when the query
    /// changed something, and a deletion of the flash cookie once it has been read.
    pub fn set_cookies(&self) -> Vec<String> {
        let mut out = Vec::new();
        if let Some(value) = self.cookie_value() {
            out.push(format!("{UI_COOKIE}={value}; Path=/; Max-Age=2592000; SameSite=Lax"));
        }
        if self.flash.is_some() {
            out.push(format!("{FLASH_COOKIE}=; Path=/; Max-Age=0; SameSite=Lax"));
        }
        out
    }

    /// Attach the flash message read from the `wo-flash` cookie.
    pub fn with_flash(mut self, flash: Option<String>) -> UiState {
        self.flash = flash.filter(|f| !f.is_empty());
        self
    }

    /// The pending flash message, if any.
    pub fn flash(&self) -> Option<&str> {
        self.flash.as_deref()
    }

    /// The request path the links are built on.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Merged value for `key`: query wins over cookie.
    pub fn get(&self, key: &str) -> Option<&str> {
        self.from_query.get(key).or_else(|| self.from_cookie.get(key)).map(String::as_str)
    }

    /// Open tab index for the tab group `name`; `0` when unknown.
    pub fn tab(&self, name: &str) -> usize {
        self.get(&format!("tab.{name}")).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    /// First open section index for the accordion `group`; `None` when unknown or closed.
    pub fn open(&self, group: &str) -> Option<usize> {
        self.opens(group).first().copied()
    }

    /// Every open section index for the accordion `group`: `open.<group>` is a comma list
    /// (`0,2`), so a `multi` accordion can keep several sections open. Empty when unknown.
    pub fn opens(&self, group: &str) -> Vec<usize> {
        self.get(&format!("open.{group}"))
            .map(|v| v.split(',').filter_map(|i| i.trim().parse().ok()).collect())
            .unwrap_or_default()
    }

    /// Current step (0-based) of the wizard `id`; `0` when unknown.
    pub fn step(&self, id: &str) -> usize {
        self.get(&format!("step.{id}")).and_then(|v| v.parse().ok()).unwrap_or(0)
    }

    /// Whether `key` comes from the `wo-ui` cookie alone, not from this request's query: the
    /// visitor came back without a link naming it (a new tab, a bookmark of the bare path).
    pub fn remembered(&self, key: &str) -> bool {
        !self.from_query.contains_key(key) && self.from_cookie.contains_key(key)
    }

    /// Rows per page remembered for the paged table `id` (`per.<id>`); `None` when unknown.
    pub fn per_page(&self, id: &str) -> Option<usize> {
        self.get(&format!("per.{id}")).and_then(|v| v.parse().ok()).filter(|&n| n > 0)
    }

    /// Id of the dialog to render open, if any.
    pub fn dialog(&self) -> Option<&str> {
        self.get("dialog").filter(|d| !d.is_empty())
    }

    /// Merged state as key/value pairs.
    pub fn entries(&self) -> BTreeMap<&str, &str> {
        self.from_cookie
            .iter()
            .chain(self.from_query.iter())
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect()
    }

    /// A link to the current path with `key` set to `value` and every other state key kept.
    /// An empty `value` stays in the link as `key=`: an explicit "nothing" that beats the
    /// cookie's memory, which is how "Collapse all" and closing the open section work.
    pub fn link(&self, key: &str, value: &str) -> String {
        let mut entries = self.entries();
        entries.insert(key, value);
        let query: Vec<String> = entries.iter().map(|(k, v)| format!("{}={}", encode(k), encode(v))).collect();
        if query.is_empty() { self.path.clone() } else { format!("{}?{}", self.path, query.join("&")) }
    }

    /// Whether the query changed something the cookie should now remember.
    pub fn changed(&self) -> bool {
        self.from_query.iter().any(|(k, v)| self.from_cookie.get(k) != Some(v))
    }

    /// Value for the `wo-ui` cookie: the merged state, or `None` when nothing changed.
    pub fn cookie_value(&self) -> Option<String> {
        self.changed().then(|| {
            self.entries().iter().map(|(k, v)| format!("{}={}", encode(k), encode(v))).collect::<Vec<_>>().join("&")
        })
    }
}

/// Post/Redirect/Get for any server: the status (`303`), the `Location` value, and the
/// `Set-Cookie` value carrying `flash` for one minute, if there is a message.
pub fn prg_parts<'a>(to: &'a str, flash: Option<&str>) -> (u16, &'a str, Option<String>) {
    let cookie =
        flash.map(|msg| format!("{FLASH_COOKIE}={}; Path=/; Max-Age=60; SameSite=Lax", encode(msg)));
    (303, to, cookie)
}

/// Post/Redirect/Get: `303 See Other` to `to`, carrying `flash` in a one-shot cookie. The
/// body is empty and generic over anything built from a `String`, so an Axum handler can
/// return it as `axum::response::Response` and a hyper one as `Response<Full<Bytes>>`.
#[cfg(feature = "http")]
pub fn prg<B: From<String>>(to: &str, flash: Option<&str>) -> http::Response<B> {
    let (status, location, cookie) = prg_parts(to, flash);
    let mut res = http::Response::builder().status(status).header(http::header::LOCATION, location);
    if let Some(c) = cookie {
        res = res.header(http::header::SET_COOKIE, c);
    }
    res.body(B::from(String::new())).expect("valid redirect headers")
}

#[cfg(feature = "axum")]
mod axum_glue {
    use super::UiState;
    use axum::{
        extract::FromRequestParts,
        http::{HeaderValue, header, request::Parts},
        response::{IntoResponseParts, ResponseParts},
    };

    /// A thin wrapper over [`UiState::from_request`].
    impl<S: Send + Sync> FromRequestParts<S> for UiState {
        type Rejection = std::convert::Infallible;

        async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<UiState, Self::Rejection> {
            let cookies = parts
                .headers
                .get_all(header::COOKIE)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect::<Vec<_>>()
                .join("; ");
            Ok(UiState::from_request(parts.uri.path(), parts.uri.query().unwrap_or(""), &cookies))
        }
    }

    /// Returning `(state, markup)` from a handler persists changed state and clears the flash.
    impl IntoResponseParts for UiState {
        type Error = std::convert::Infallible;

        fn into_response_parts(self, mut res: ResponseParts) -> Result<ResponseParts, Self::Error> {
            for c in self.set_cookies() {
                res.headers_mut().append(header::SET_COOKIE, HeaderValue::from_str(&c).unwrap());
            }
            Ok(res)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_wins_over_cookie_and_links_keep_the_rest() {
        let s = UiState::parse("/p", "tab.a=2&q=x", "tab.a=1&open.faq=0&dialog=confirm");
        assert_eq!(s.tab("a"), 2);
        assert_eq!(s.open("faq"), Some(0));
        assert_eq!(UiState::parse("/p", "open.faq=2,0", "").opens("faq"), vec![2, 0]);
        assert_eq!(UiState::parse("/p", "open.faq=", "").opens("faq"), Vec::<usize>::new());
        assert_eq!(s.dialog(), Some("confirm"));
        assert_eq!(s.link("open.faq", ""), "/p?dialog=confirm&open.faq=&tab.a=2", "an empty value stays explicit so it beats the cookie");
        let closed = UiState::parse("/p", "open.faq=", "open.faq=0,2");
        assert!(closed.opens("faq").is_empty() && closed.changed(), "the explicit empty wins and is remembered");
        assert!(s.changed());
        assert_eq!(s.cookie_value().as_deref(), Some("dialog=confirm&open.faq=0&tab.a=2"));
        let same = UiState::parse("/p", "tab.a=1", "tab.a=1");
        assert!(!same.changed() && same.cookie_value().is_none());
    }

    #[test]
    fn request_and_response_by_hand() {
        let s = UiState::from_request("/p", "tab.a=2", "theme=dark; wo-ui=tab.a=1; wo-flash=Saved%20it");
        assert_eq!(s.flash(), Some("Saved it"));
        let cookies = s.set_cookies();
        assert_eq!(cookies[0], "wo-ui=tab.a=2; Path=/; Max-Age=2592000; SameSite=Lax");
        assert!(cookies[1].starts_with("wo-flash=; ") && cookies[1].contains("Max-Age=0"));
        assert!(UiState::from_request("/p", "", "wo-ui=tab.a=1").set_cookies().is_empty());
        assert_eq!(prg_parts("/p", None), (303, "/p", None));
    }

    #[test]
    fn encoding_round_trips() {
        let s = UiState::parse("/p", "dialog=a%20b+c", "");
        assert_eq!(s.dialog(), Some("a b c"));
        assert_eq!(s.link("dialog", "a b"), "/p?dialog=a%20b");
    }
}
