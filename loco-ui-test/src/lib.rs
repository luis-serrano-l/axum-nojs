//! # loco-ui-test
//!
//! Render an Axum route, load the HTML into Blitz (Stylo styles + Taffy layout, no browser),
//! and assert on what a user would see: elements exist, are visible, have a bounding box, a
//! computed style. `screenshot` paints the same document to a PNG through vello_cpu.
//!
//! Blitz has no JavaScript engine, which is exactly the point: whatever renders here renders
//! with zero script. Anything Blitz cannot render is a finding, not a test failure, and is
//! listed in `FINDINGS.md`.
//!
//! ```no_run
//! # async fn demo() {
//! use loco_ui_test::Page;
//! let mut page = Page::render(demo::router(), "/dialog", "").await;
//! assert!(page.is_visible(".lui-dialog-open"));
//! page.screenshot("target/shots/dialog.png").unwrap();
//! # }
//! ```

use std::path::Path;

use anyrender::render_to_buffer;
use anyrender_vello_cpu::VelloCpuImageRenderer;
use axum::{Router, body::Body, http::Request};
use blitz_dom::{BaseDocument, DocumentConfig, NodeId};
use blitz_html::HtmlDocument;
use blitz_traits::shell::{ColorScheme, Viewport};
use tower::ServiceExt;

/// Viewport used for every page, in CSS pixels.
pub const WIDTH: u32 = 1000;
pub const HEIGHT: u32 = 700;

/// A rendered page: parsed, styled and laid out by Blitz.
pub struct Page {
    doc: HtmlDocument,
    /// The HTML as the server sent it, for string assertions.
    pub html: String,
}

/// An element's box in CSS pixels, relative to the page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Page {
    /// GET `path` from `router` with a raw `Cookie` header and render the response.
    pub async fn render(router: Router, path: &str, cookie: &str) -> Page {
        Page::render_expecting(router, path, cookie, 200).await
    }

    /// [`Page::render`] for a response with another status (a 404 page, say).
    pub async fn render_expecting(router: Router, path: &str, cookie: &str, status: u16) -> Page {
        let req = Request::get(path)
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap();
        let res = router.oneshot(req).await.unwrap();
        assert_eq!(res.status(), status, "{path}");
        let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        Page::from_html(String::from_utf8(bytes.to_vec()).unwrap())
    }

    /// Render a standalone HTML string.
    pub fn from_html(html: String) -> Page {
        let config = DocumentConfig {
            viewport: Some(Viewport::new(WIDTH, HEIGHT, 1.0, ColorScheme::Light)),
            base_url: Some("http://localhost/".into()),
            ua_stylesheets: Some(Vec::new()),
            ..Default::default()
        };
        let mut doc = HtmlDocument::from_html(&html, config);
        doc.resolve(0.0);
        Page { doc, html }
    }

    /// The underlying Blitz document, for anything not wrapped here.
    pub fn doc(&self) -> &BaseDocument {
        &self.doc
    }

    /// First node matching a CSS selector. Panics on a selector syntax error.
    pub fn node(&self, selector: &str) -> Option<NodeId> {
        self.doc
            .query_selector(selector)
            .unwrap_or_else(|e| panic!("bad selector {selector}: {e:?}"))
    }

    /// Whether any element matches `selector`.
    pub fn exists(&self, selector: &str) -> bool {
        self.node(selector).is_some()
    }

    /// Number of elements matching `selector`.
    pub fn count(&self, selector: &str) -> usize {
        self.doc
            .query_selector_all(selector)
            .map(|v| v.len())
            .unwrap_or(0)
    }

    /// Text content of the first match, whitespace collapsed.
    pub fn text(&self, selector: &str) -> Option<String> {
        let id = self.node(selector)?;
        let raw = self.doc.get_node(id)?.text_content();
        Some(raw.split_whitespace().collect::<Vec<_>>().join(" "))
    }

    /// Bounding box of the first match after layout. `None` when the element has no box
    /// (missing, `display: none`, or inside a closed `<details>`).
    pub fn bbox(&self, selector: &str) -> Option<Rect> {
        let id = self.node(selector)?;
        let r = self.doc.get_client_bounding_rect(id)?;
        Some(Rect {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        })
    }

    /// Visible means: has a box with non-zero area, is not `visibility: hidden`, and has no
    /// `opacity: 0` on itself.
    pub fn is_visible(&self, selector: &str) -> bool {
        let Some(id) = self.node(selector) else {
            return false;
        };
        let Some(rect) = self.bbox(selector) else {
            return false;
        };
        if rect.width <= 0.0 || rect.height <= 0.0 {
            return false;
        }
        let Some(node) = self.doc.get_node(id) else {
            return false;
        };
        match node.primary_styles() {
            Some(style) => {
                let visibility = format!("{:?}", style.clone_visibility());
                style.clone_opacity() > 0.0 && visibility == "Visible"
            }
            None => false,
        }
    }

    /// Computed `display` of the first match: `none`, `contents`, or `<outside>/<inside>`
    /// such as `block/flow`, `inline/flow`, `block/flex`, `block/table`.
    pub fn display(&self, selector: &str) -> Option<String> {
        let id = self.node(selector)?;
        let style = self.doc.get_node(id)?.primary_styles()?;
        let d = style.clone_display();
        Some(if d.is_none() {
            "none".to_string()
        } else if d.is_contents() {
            "contents".to_string()
        } else {
            format!("{:?}/{:?}", d.outside(), d.inside()).to_lowercase()
        })
    }

    /// Paint the page to `path` as an RGBA PNG at the viewport size.
    pub fn screenshot(&mut self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let mut buf = render_to_buffer::<VelloCpuImageRenderer, _>(
            |scene| blitz_paint::paint_scene(scene, &mut self.doc, 1.0, WIDTH, HEIGHT, 0, 0),
            WIDTH,
            HEIGHT,
        );
        // vello_cpu hands back premultiplied RGBA; PNG wants straight alpha.
        for px in buf.as_chunks_mut::<4>().0 {
            let a = px[3] as u32;
            if a != 0 && a != 255 {
                for c in &mut px[..3] {
                    *c = (u32::from(*c) * 255 / a).min(255) as u8;
                }
            }
        }
        let path = path.as_ref();
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let file = std::fs::File::create(path)?;
        let mut enc = png::Encoder::new(std::io::BufWriter::new(file), WIDTH, HEIGHT);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut writer = enc.write_header()?;
        writer.write_image_data(&buf)?;
        Ok(())
    }
}
