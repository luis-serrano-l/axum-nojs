//! # Stream
//!
//! Out-of-order streaming: the page is sent at once with placeholders, and each slow section
//! is appended to the response whenever it finishes, in any order. The browser moves each
//! chunk into its placeholder. No script.
//!
//! **Platform features:**
//! - Declarative shadow DOM, `<template shadowrootmode="open">` (Chrome 111, Firefox 123,
//!   Safari 16.4). `<body>` becomes a shadow host whose shadow tree is the page, with a
//!   `<slot name="id">placeholder</slot>` per section.
//! - Named slots: a light-DOM child `<div slot="id">` of `<body>` replaces the placeholder the
//!   moment the parser sees it, wherever it arrives in the byte stream.
//! - Chunked transfer: the response body is a stream of HTML chunks. [`Streamed::into_stream`]
//!   is that stream for any server (`http` feature); the `axum` feature turns it into a
//!   response with `IntoResponse`.
//!
//! **Fallback:** without `Caps::StreamingDsd`, `slot` leaves an HTML comment marker and the
//! response is streamed *in document order*: the bytes up to the first marker go out at once,
//! then each section as soon as it and everything before it are ready. Every browser renders
//! that progressively; only the out-of-order part is lost.
//!
//! **Finding:** shadow trees do not see document stylesheets, so the stylesheet is inlined
//! twice in DSD mode, once in `<head>` for the slotted chunks and once in the shadow tree.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, Streamed, Theme, slot};
//! let caps = Caps::all();
//! let page = Streamed::page(&caps, "Feed", Theme::Auto, html! {
//!     h1 { "Feed" }
//!     (slot(&caps, "news", html! { p { "Loading news…" } }))
//! })
//! .fill("news", async { html! { p { "Fresh news." } } });
//! ```

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

use futures_util::stream::{self, FuturesUnordered, Stream, StreamExt};
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{Cap, Caps, Theme, caps, enhance, layout, stylesheet};

type Fill = Pin<Box<dyn Future<Output = Markup> + Send + 'static>>;

/// Placeholder for the section `id`. In DSD mode it is a named `<slot>` showing `placeholder`
/// until the chunk arrives; otherwise a comment marker where the chunk is spliced in order.
pub fn slot(caps: &Caps, id: &str, placeholder: Markup) -> Markup {
    if caps.has(Cap::StreamingDsd) {
        html! { slot name=(id) { (placeholder) } }
    } else {
        html! { (PreEscaped(marker(id))) }
    }
}

fn marker(id: &str) -> String {
    format!("<!--wo-slot:{id}-->")
}

/// A page whose slow sections arrive later. Build with [`Streamed::page`], add sections with
/// [`Streamed::fill`], then return it from an Axum handler or send [`Streamed::into_stream`]
/// as a chunked `text/html; charset=utf-8` body from any server.
pub struct Streamed {
    dsd: bool,
    /// Whole page in fallback mode; everything up to `</template>` in DSD mode.
    prefix: String,
    suffix: String,
    fills: Vec<(String, Fill)>,
}

impl Streamed {
    /// The page shell with `body` inside `<main>`, like `layout`, ready for fills.
    pub fn page(caps: &Caps, title: &str, theme: Theme, body: Markup) -> Streamed {
        let dsd = caps.has(Cap::StreamingDsd);
        let inner = html! {
            (layout::header())
            main { (body) }
            (caps::beacons(caps))
        };
        let open = html! {
            (DOCTYPE)
            html lang="en" data-theme=(theme.as_str()) { (layout::head(title)) }
        };
        // `html!` closes every element it opens, so the tags after `<head>` are written by hand.
        let open = open.into_string().trim_end_matches("</html>").to_string();
        let prefix = if dsd {
            let shadow_css = format!("<style>{}</style>", stylesheet());
            format!(
                "{open}<body><template shadowrootmode=\"open\">{shadow_css}{}</template>",
                inner.into_string()
            )
        } else {
            format!("{open}<body>{}", inner.into_string())
        };
        let suffix = format!("{}</body></html>", enhance::script_tag().into_string());
        Streamed { dsd, prefix, suffix, fills: Vec::new() }
    }

    /// Register the content for slot `id`. It is awaited while the response streams.
    pub fn fill<F>(mut self, id: &str, future: F) -> Streamed
    where
        F: Future<Output = Markup> + Send + 'static,
    {
        self.fills.push((id.to_string(), Box::pin(future)));
        self
    }

    /// Whether the response will use declarative shadow DOM (out of order) or in-order splicing.
    pub fn out_of_order(&self) -> bool {
        self.dsd
    }

    /// The response body as HTML chunks, each ready to send as soon as it is yielded. Send them
    /// with `Content-Type: text/html; charset=utf-8` and chunked transfer.
    pub fn into_stream(self) -> Pin<Box<dyn Stream<Item = String> + Send + 'static>> {
        if self.dsd {
            let fills: FuturesUnordered<_> = self
                .fills
                .into_iter()
                .map(|(id, fut)| async move { html! { div slot=(id) { (fut.await) } }.into_string() })
                .collect();
            let chunks = stream::once(async { self.prefix })
                .chain(fills)
                .chain(stream::once(async { self.suffix }));
            return Box::pin(chunks);
        }
        // Fallback: walk the page in order, splicing each fill at its marker.
        let mut fills: HashMap<String, Fill> = self.fills.into_iter().collect();
        let mut pieces: Vec<Piece> = Vec::new();
        let mut rest = self.prefix.as_str();
        while let Some(start) = rest.find("<!--wo-slot:") {
            let end = rest[start..].find("-->").map(|e| start + e + 3).unwrap_or(rest.len());
            pieces.push(Piece::Text(rest[..start].to_string()));
            let id = &rest[start + "<!--wo-slot:".len()..end - 3];
            if let Some(fut) = fills.remove(id) {
                pieces.push(Piece::Fill(fut));
            }
            rest = &rest[end..];
        }
        pieces.push(Piece::Text(format!("{rest}{}", self.suffix)));
        let chunks = stream::iter(pieces).then(|piece| async move {
            match piece {
                Piece::Text(s) => s,
                Piece::Fill(fut) => fut.await.into_string(),
            }
        });
        Box::pin(chunks)
    }
}

enum Piece {
    Text(String),
    Fill(Fill),
}

#[cfg(feature = "axum")]
impl axum::response::IntoResponse for Streamed {
    fn into_response(self) -> axum::response::Response {
        let chunks = self.into_stream().map(|s| Ok::<bytes::Bytes, std::convert::Infallible>(s.into()));
        (
            [(http::header::CONTENT_TYPE, "text/html; charset=utf-8")],
            axum::body::Body::from_stream(chunks),
        )
            .into_response()
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-stream-pending { color: var(--wo-muted); font-style: italic; }
.wo-stream-section { border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 1rem; margin-bottom: var(--wo-space); background: var(--wo-surface); }
"#;
