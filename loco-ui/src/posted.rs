//! # Posted
//!
//! What a form posted, read by name, whether it came urlencoded or as `multipart/form-data`
//! (a form with a file field): the handler asks for `posted.get("email")` instead of parsing
//! the body, and hands `posted.pairs()` back to [`crate::form::Form::values`] when the form
//! goes back with its messages.
//!
//! With the `axum` feature `Posted` is an extractor. Anywhere else, build it from the pairs a
//! form post parses into with [`Posted::from_pairs`].
//!
//! ```rust
//! use loco_ui::prelude::*;
//!
//! let posted = Posted::from_pairs(vec![
//!     ("email".into(), "ada@example.com".into()),
//!     ("tag".into(), "rust".into()),
//!     ("tag".into(), "html".into()),
//! ]);
//! assert_eq!(posted.get("email"), "ada@example.com");
//! assert_eq!(posted.get("missing"), "", "a missing field reads empty");
//! assert_eq!(posted.all("tag").collect::<Vec<_>>(), ["rust", "html"]);
//! assert_eq!(posted.pairs().len(), 3);
//! assert!(posted.files().is_empty());
//! ```
//!
//! In a handler, with the form sent back on a mistake:
//!
//! ```rust,no_run
//! use loco_ui::prelude::*;
//!
//! fn view(ui: &Ui, posted: &[(String, String)], errors: &[(&str, &str)]) -> Page {
//!     ui.page("Sign up", html! {
//!         (ui.form("/signup").email("email", "Email").required().values(posted).errors(errors))
//!     })
//! }
//!
//! async fn submit(ui: Ui, posted: Posted) -> Result<Redirect, Page> {
//!     if posted.get("email").ends_with("@example.com") {
//!         let errors = [("email", "example.com addresses are not accepted.")];
//!         return Err(view(&ui, posted.pairs(), &errors).invalid());
//!     }
//!     Ok(ui.redirect("/signup").ok("Signed up."))
//! }
//! ```

/// A file a `multipart/form-data` post carried: its field's name, the name it had on the
/// visitor's computer, its type as the browser said, and its bytes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostedFile {
    /// The file field's `name`.
    pub name: String,
    /// The file's name as sent (not a safe path: clean it before using it as one).
    pub file_name: String,
    /// The `Content-Type` the browser gave the file, if any.
    pub content_type: Option<String>,
    /// The file's contents.
    pub bytes: Vec<u8>,
}

/// What a form posted: its fields as `(name, value)` pairs in order, and its files; see the
/// [module docs](self).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Posted {
    pairs: Vec<(String, String)>,
    files: Vec<PostedFile>,
}

impl Posted {
    /// The fields of a post already parsed into pairs, and no files.
    pub fn from_pairs(pairs: Vec<(String, String)>) -> Posted {
        Posted {
            pairs,
            files: Vec::new(),
        }
    }

    /// The first value posted under `name`, as sent (trim it for text a person typed); empty
    /// when the form had no such field or an unticked checkbox.
    pub fn get(&self, name: &str) -> &str {
        self.pairs
            .iter()
            .find(|(n, _)| n == name)
            .map_or("", |(_, v)| v.as_str())
    }

    /// Every value posted under `name`, in order (the ticked boxes of a group, the picks of
    /// a multiple select).
    pub fn all<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a str> + 'a {
        self.pairs
            .iter()
            .filter(move |(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// Every field as `(name, value)` in the order posted: what [`crate::form::Form::values`]
    /// and [`crate::wizard::Wizard::values`] take to show the form again as it was sent.
    pub fn pairs(&self) -> &[(String, String)] {
        &self.pairs
    }

    /// The files of a multipart post, in order; a file field left empty sends none.
    pub fn files(&self) -> &[PostedFile] {
        &self.files
    }

    /// The wizard step this post came from (its hidden `step` field), 0 when there is none.
    pub fn step(&self) -> usize {
        self.get("step").parse().unwrap_or(0)
    }

    /// Whether the visitor pressed a wizard's "Skip" (`skip=1`).
    pub fn skip(&self) -> bool {
        self.get("skip") == "1"
    }
}

#[cfg(feature = "axum")]
mod axum_glue {
    use super::{Posted, PostedFile};
    use axum::{
        body::Bytes,
        extract::{FromRequest, Multipart, Request},
        http::header,
        response::{IntoResponse, Response},
    };

    /// Reads an urlencoded or a `multipart/form-data` body; a body that is neither is empty.
    impl<S: Send + Sync> FromRequest<S> for Posted {
        type Rejection = Response;

        async fn from_request(req: Request, state: &S) -> Result<Posted, Response> {
            let multipart = req
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .is_some_and(|t| t.starts_with("multipart/form-data"));
            if !multipart {
                let body = Bytes::from_request(req, state)
                    .await
                    .map_err(IntoResponse::into_response)?;
                return Ok(Posted::from_pairs(
                    form_urlencoded::parse(&body).into_owned().collect(),
                ));
            }
            let mut parts = Multipart::from_request(req, state)
                .await
                .map_err(IntoResponse::into_response)?;
            let mut posted = Posted::default();
            while let Some(part) = parts
                .next_field()
                .await
                .map_err(IntoResponse::into_response)?
            {
                let name = part.name().unwrap_or_default().to_string();
                match part.file_name().map(str::to_string) {
                    Some(file_name) => {
                        let content_type = part.content_type().map(str::to_string);
                        let bytes = part.bytes().await.map_err(IntoResponse::into_response)?;
                        if !file_name.is_empty() || !bytes.is_empty() {
                            posted.files.push(PostedFile {
                                name,
                                file_name,
                                content_type,
                                bytes: bytes.to_vec(),
                            });
                        }
                    }
                    None => {
                        let value = part.text().await.map_err(IntoResponse::into_response)?;
                        posted.pairs.push((name, value));
                    }
                }
            }
            Ok(posted)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "axum")]
    async fn posted(content_type: &str, body: &str) -> Posted {
        use axum::extract::{FromRequest, Request};
        let req = Request::post("/")
            .header("content-type", content_type)
            .body(axum::body::Body::from(body.to_string()))
            .unwrap();
        Posted::from_request(req, &()).await.unwrap()
    }

    #[cfg(feature = "axum")]
    #[tokio::test]
    async fn urlencoded_and_multipart_read_the_same() {
        let url = posted(
            "application/x-www-form-urlencoded",
            "email=ada%40x.org&tag=a&tag=b+c",
        )
        .await;
        assert_eq!(url.get("email"), "ada@x.org");
        assert_eq!(url.all("tag").collect::<Vec<_>>(), ["a", "b c"]);
        let body = "--X\r\nContent-Disposition: form-data; name=\"email\"\r\n\r\nada@x.org\r\n\
                    --X\r\nContent-Disposition: form-data; name=\"avatar\"; filename=\"a.png\"\r\n\
                    Content-Type: image/png\r\n\r\nPNG\r\n\
                    --X\r\nContent-Disposition: form-data; name=\"cv\"; filename=\"\"\r\n\
                    Content-Type: application/octet-stream\r\n\r\n\r\n--X--\r\n";
        let multi = posted("multipart/form-data; boundary=X", body).await;
        assert_eq!(multi.get("email"), "ada@x.org");
        assert_eq!(multi.files().len(), 1, "an empty file field sends no file");
        let f = &multi.files()[0];
        assert_eq!(
            (
                f.name.as_str(),
                f.file_name.as_str(),
                f.content_type.as_deref(),
                &f.bytes[..]
            ),
            ("avatar", "a.png", Some("image/png"), &b"PNG"[..])
        );
    }

    #[test]
    fn a_wizard_post_says_its_step() {
        let p = Posted::from_pairs(vec![
            ("step".into(), "2".into()),
            ("skip".into(), "1".into()),
        ]);
        assert_eq!((p.step(), p.skip()), (2, true));
        assert_eq!(
            (Posted::default().step(), Posted::default().skip()),
            (0, false)
        );
    }
}
