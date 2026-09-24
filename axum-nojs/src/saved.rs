//! # Saved
//!
//! A value the server keeps for a visitor between requests, in a cookie, with no database:
//! the counter's number, the settings a form saved, what a wizard has collected so far.
//!
//! `Saved<T>` is an extractor: it reads the cookie named after the type (`Settings` lives in
//! `nojs-settings`) and falls back to `T::default()` when the cookie is missing or no longer
//! parses. [`Redirect::save`] writes it back with the answer to a form post, and
//! [`Redirect::forget`] removes it. The value is form-encoded, so it holds plain fields
//! (strings, numbers, booleans, options), or a newtype over a list of `(key, value)` pairs
//! (`struct Notes(Vec<(String, String)>)`, a repeated field or what a form posted).
//!
//! ```rust,no_run
//! use axum::Form;
//! use axum_nojs::prelude::*;
//! use serde::{Deserialize, Serialize};
//!
//! #[derive(Default, Deserialize, Serialize)]
//! struct Settings { name: String, notify: bool }
//!
//! async fn show(ui: Ui, Saved(s): Saved<Settings>) -> Page {
//!     ui.page("Settings", html! { p { "Hello, " (s.name) } })
//! }
//!
//! async fn save(ui: Ui, Form(s): Form<Settings>) -> Redirect {
//!     ui.redirect("/settings").ok("Settings saved.").save(&s)
//! }
//! ```
//!
//! **Platform features:** a cookie, `Path=/`, kept for a year, `SameSite=Lax`, `HttpOnly`
//! (only the server reads it).
//!
//! **Not for secrets:** the visitor can read and change the cookie. Keep ids and preferences
//! here, check them like any other input, and keep anything that must not be forged on the
//! server.

use axum::{extract::FromRequestParts, http::{header, request::Parts}};
use serde::{Deserializer, Serialize, de::{DeserializeOwned, Visitor}};

use crate::state::{decode, encode};
use crate::ui::Redirect;

/// A value kept in a cookie named after its type; see the [module docs](self).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Saved<T>(pub T);

/// `nojs-` and the type's name in kebab case: `Settings` is `nojs-settings`, `WizardData`
/// is `nojs-wizard-data`.
pub fn cookie_name<T>() -> String {
    let full = std::any::type_name::<T>();
    let name = full.split('<').next().unwrap_or(full).rsplit("::").next().unwrap_or(full);
    let mut out = String::from("nojs");
    for c in name.chars() {
        if c.is_ascii_uppercase() || !out.ends_with(|p: char| p.is_ascii_alphanumeric()) {
            out.push('-');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}

impl<S: Send + Sync, T: DeserializeOwned + Default> FromRequestParts<S> for Saved<T> {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let name = cookie_name::<T>();
        let value = parts
            .headers
            .get_all(header::COOKIE)
            .iter()
            .filter_map(|v| v.to_str().ok())
            .flat_map(|h| h.split(';'))
            .filter_map(|pair| pair.trim().split_once('='))
            .find(|(k, _)| *k == name)
            .and_then(|(_, v)| T::deserialize(Form(serde_urlencoded::Deserializer::new(form_urlencoded::parse(decode(v).as_bytes())))).ok());
        Ok(Saved(value.unwrap_or_default()))
    }
}

/// `serde_urlencoded`'s deserializer, but a newtype (`struct Notes(Vec<(String, String)>)`)
/// reads as what it wraps: `serde_urlencoded` alone refuses newtypes, though it writes them.
struct Form<'de>(serde_urlencoded::Deserializer<'de>);

impl<'de> Deserializer<'de> for Form<'de> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_any(visitor)
    }

    fn deserialize_seq<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        self.0.deserialize_seq(visitor)
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(self, _: &'static str, visitor: V) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string bytes byte_buf
        option unit unit_struct tuple tuple_struct map struct enum identifier ignored_any
    }
}

impl Redirect {
    /// Keep `value` for this visitor; the next `Saved<T>` reads it. A value that cannot be
    /// form-encoded (a nested struct, a list inside a field) is not saved.
    pub fn save<T: Serialize>(self, value: &T) -> Self {
        match serde_urlencoded::to_string(value) {
            Ok(v) => self.cookie(format!("{}={}; Path=/; Max-Age=31536000; SameSite=Lax; HttpOnly", cookie_name::<T>(), encode(&v))),
            Err(_) => self,
        }
    }

    /// Remove what [`Redirect::save`] kept for `T`.
    pub fn forget<T>(self) -> Self {
        self.cookie(format!("{}=; Path=/; Max-Age=0; SameSite=Lax; HttpOnly", cookie_name::<T>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Ui;
    use axum::http::Request;
    use serde::Deserialize;

    #[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
    struct WizardData { name: String, notify: bool, age: Option<u8> }

    #[tokio::test]
    async fn a_saved_value_comes_back_on_the_next_request() {
        assert_eq!(cookie_name::<WizardData>(), "nojs-wizard-data");
        let value = WizardData { name: "Ada L".into(), notify: true, age: None };
        let set = Ui::default().redirect("/").save(&value).set_cookies().remove(0);
        let pair = set.split(';').next().unwrap().to_string();
        let (mut parts, ()) = Request::builder().header("cookie", format!("a=b; {pair}")).body(()).unwrap().into_parts();
        let Saved(back) = Saved::<WizardData>::from_request_parts(&mut parts, &()).await.unwrap();
        assert_eq!(back, value);
        let (mut bare, ()) = Request::builder().header("cookie", "nojs-wizard-data=%%%").body(()).unwrap().into_parts();
        assert_eq!(Saved::<WizardData>::from_request_parts(&mut bare, &()).await.unwrap().0, WizardData::default(), "garbage is the default");
        let pairs: Vec<(String, String)> = vec![("topics".into(), "rust, html".into())];
        assert!(Ui::default().redirect("/").save(&pairs).set_cookies()[0].starts_with("nojs-vec="), "a list of pairs saves");
        #[derive(Debug, Default, Deserialize, Serialize, PartialEq)]
        struct Notes(Vec<(String, String)>);
        let notes = Notes(vec![("note".into(), "one".into()), ("note".into(), "two & 3".into())]);
        let set = Ui::default().redirect("/").save(&notes).set_cookies().remove(0);
        assert!(set.starts_with("nojs-notes="), "{set}");
        let (mut parts, ()) = Request::builder().header("cookie", set.split(';').next().unwrap()).body(()).unwrap().into_parts();
        assert_eq!(Saved::<Notes>::from_request_parts(&mut parts, &()).await.unwrap().0, notes, "a newtype over pairs round-trips");
        assert!(Ui::default().redirect("/").forget::<WizardData>().set_cookies()[0].contains("Max-Age=0"));
    }
}
