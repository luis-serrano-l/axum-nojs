//! # Upload
//!
//! A file upload: a drop zone around a file input, an Upload button, and under it the files the
//! server already holds, with a thumbnail for images and a Remove button each. The list is what
//! the server says after the round trip, so it is always true.
//!
//! **Platform features:** `<input type="file" accept multiple>` (baseline 2015) in a
//! `<form method="post" enctype="multipart/form-data">`; a file dropped on the input is picked
//! like a chosen one, with no script; `<progress>` (baseline 2013) for the upload bar;
//! `loading="lazy"` thumbnails; Post/Redirect/Get after the upload and after each removal.
//!
//! **What it does not do without script:** show the upload's progress (the enhancement script
//! sends the form through `XMLHttpRequest` and fills the `<progress data-nojs-progress>` bar;
//! without it the browser shows its own loading indicator), preview a file before it is sent,
//! or accept a drop anywhere but on the input.
//!
//! **Fallback:** none needed: the form is a plain multipart post.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.upload("/files", "file").render().into_string();
//! assert!(m.contains(r#"enctype="multipart/form-data""#) && m.contains(r#"type="file""#));
//! // Images only, several at once, and what the server already has.
//! let m = ui.upload("/files", "file")
//!     .accept("image/*").multiple().hint("PNG or JPEG, up to 200 KB.")
//!     .file("cat.png", 48_213).preview("/files/cat.png")
//!     .file("notes.txt", 1_024).href("/files/notes.txt")
//!     .remove("/files/remove");
//! let m = m.render().into_string();
//! assert!(m.contains(r#"accept="image/*" multiple"#) && m.contains(r#"src="/files/cat.png""#));
//! assert!(m.contains("47.1 KB") && m.contains(r#"name="file" value="notes.txt""#));
//!
//! // The same in `nojs!`:
//! let same = nojs! { Upload("/files", "file")
//!     accept="image/*" multiple hint="PNG or JPEG, up to 200 KB." {
//!     file "cat.png" 48_213 preview="/files/cat.png";
//!     file "notes.txt" 1_024 href="/files/notes.txt";
//!     remove "/files/remove";
//! } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::props::{Prop, PropKind};
use crate::{Icon, Ui, enhance};

/// A file the server holds: its name, size, a thumbnail and a link.
#[derive(Clone, Debug)]
struct Held<'a> {
    name: &'a str,
    size: u64,
    preview: Option<&'a str>,
    href: Option<&'a str>,
}

/// An upload form and its list of files, made by [`Ui::upload`].
///
/// **Setters.** Values and items: `.accept(..)`, `.hint(..)`, `.file(..)`, `.preview(..)`,
/// `.href(..)`, `.remove(..)`; switches: `.multiple()`.
#[derive(Clone, Debug)]
pub struct Upload<'a> {
    ui: &'a Ui,
    action: &'a str,
    name: &'a str,
    accept: Option<&'a str>,
    multiple: bool,
    hint: Option<&'a str>,
    files: Vec<Held<'a>>,
    remove: Option<&'a str>,
}

impl Upload<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("accept", PropKind::Value, "types: &'a str")
            .attr("accept")
            .doc("`accept`."),
        Prop::new("multiple", PropKind::Switch, "")
            .attr("multiple")
            .doc("Several files at once."),
        Prop::new("hint", PropKind::Value, "text: &'a str").doc("Small print in the drop zone."),
        Prop::new("file", PropKind::Item, "name: &'a str, size: u64")
            .doc("A file the server already holds, `size` in bytes."),
        Prop::new("preview", PropKind::Modifier, "src: &'a str")
            .doc("A thumbnail for the file added last (an image URL)."),
        Prop::new("href", PropKind::Modifier, "href: &'a str")
            .attr("href")
            .doc("A link to the file added last, on its name."),
        Prop::new("remove", PropKind::Value, "action: &'a str")
            .doc("A Remove button per file, posting `<name>=<file name>` to `action`."),
    ];
}

impl Ui {
    /// A form posting files as the multipart field `name` to `action`.
    pub fn upload<'a>(&'a self, action: &'a str, name: &'a str) -> Upload<'a> {
        Upload {
            ui: self,
            action,
            name,
            accept: None,
            multiple: false,
            hint: None,
            files: Vec::new(),
            remove: None,
        }
    }
}

impl<'a> Upload<'a> {
    /// `accept`: MIME types or extensions the picker offers (`image/*,.pdf`).
    pub fn accept(mut self, types: &'a str) -> Self {
        self.accept = Some(types);
        self
    }

    /// Several files at once.
    pub fn multiple(mut self) -> Self {
        self.multiple = true;
        self
    }

    /// Small print in the drop zone: the types and sizes the server takes.
    pub fn hint(mut self, text: &'a str) -> Self {
        self.hint = Some(text);
        self
    }

    /// A file the server already holds, `size` in bytes.
    pub fn file(mut self, name: &'a str, size: u64) -> Self {
        self.files.push(Held {
            name,
            size,
            preview: None,
            href: None,
        });
        self
    }

    /// A thumbnail for the file added last (an image URL).
    pub fn preview(mut self, src: &'a str) -> Self {
        if let Some(f) = self.files.last_mut() {
            f.preview = Some(src);
        }
        self
    }

    /// A link to the file added last, on its name.
    pub fn href(mut self, href: &'a str) -> Self {
        if let Some(f) = self.files.last_mut() {
            f.href = Some(href);
        }
        self
    }

    /// A Remove button per file, posting `<name>=<file name>` to `action`.
    pub fn remove(mut self, action: &'a str) -> Self {
        self.remove = Some(action);
        self
    }
}

/// `48213` → `47.1 KB`: bytes in the largest unit that keeps a whole number in front.
fn size(bytes: u64) -> String {
    match bytes {
        b if b < 1024 => format!("{b} B"),
        b if b < 1024 * 1024 => format!("{:.1} KB", b as f64 / 1024.0),
        b => format!("{:.1} MB", b as f64 / (1024.0 * 1024.0)),
    }
}

impl Render for Upload<'_> {
    fn render(&self) -> Markup {
        let root = enhance::swap_id("nojs-upload", self.action);
        let input_id = format!("{root}-input");
        let caps = self.ui.caps;
        html! {
            div id=(root) data-nojs="swap" class="nojs-upload" {
                form method="post" action=(self.action) enctype="multipart/form-data" class="nojs-upload-form" {
                    label class="nojs-upload-drop" for=(input_id) {
                        (Icon::Upload)
                        span class="nojs-upload-title" { @if self.multiple { "Choose files or drop them on the button" } @else { "Choose a file or drop it on the button" } }
                        @if let Some(h) = self.hint { span class="nojs-upload-hint" { (h) } }
                        input id=(input_id) class="nojs-upload-input" type="file" name=(self.name)
                            accept=[self.accept] multiple[self.multiple] required;
                    }
                    progress class="nojs-upload-progress" data-nojs-progress hidden {}
                    (Button::new(caps, "Upload").primary())
                }
                @if !self.files.is_empty() {
                    ul class="nojs-upload-files" aria-label="Uploaded files" {
                        @for f in &self.files {
                            li class="nojs-upload-file" {
                                @if let Some(src) = f.preview {
                                    img class="nojs-upload-thumb" src=(src) alt="" loading="lazy";
                                } @else {
                                    span class="nojs-upload-thumb" aria-hidden="true" { (Icon::File) }
                                }
                                span class="nojs-upload-name" {
                                    @if let Some(h) = f.href { a href=(h) { (f.name) } } @else { (f.name) }
                                }
                                span class="nojs-upload-size" { (size(f.size)) }
                                @if let Some(action) = self.remove {
                                    form method="post" action=(action) {
                                        @let label = format!("Remove {}", f.name);
                                        (Button::new(caps, "").ghost().small().icon().label(&label).name(self.name).value(f.name).content(html! { (Icon::Trash) }))
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-upload { display: grid; gap: calc(var(--nojs-space) * 2); max-width: 32rem; }
.nojs-upload-form { display: grid; gap: var(--nojs-space); justify-items: start; }
.nojs-upload-drop {
  display: grid; justify-items: center; gap: 0.5rem; width: 100%; box-sizing: border-box; padding: 1.5rem;
  text-align: center; cursor: pointer; font-weight: 400;
  border: 1px dashed var(--nojs-input); border-radius: var(--nojs-radius-lg); background: var(--nojs-surface);
  transition: border-color 0.15s, background-color 0.15s;
}
.nojs-upload-drop:hover, .nojs-upload-drop:has(.nojs-upload-input:focus-visible) { border-color: var(--nojs-ring); }
.nojs-upload-drop > .nojs-icon { width: 1.5rem; height: 1.5rem; color: var(--nojs-muted); }
.nojs-upload-title { font-size: 0.875rem; font-weight: 500; }
.nojs-upload-hint { font-size: 0.75rem; color: var(--nojs-muted); }
.nojs-upload-input { max-width: 100%; }
.nojs-upload-progress { width: 100%; height: 0.5rem; accent-color: var(--nojs-primary); }
.nojs-upload-files { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.5rem; }
.nojs-upload-file {
  display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; font-size: 0.875rem;
  border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); background: var(--nojs-card);
}
.nojs-upload-file form { margin: 0 0 0 auto; }
.nojs-upload-thumb { display: inline-grid; place-items: center; flex: none; width: 2.5rem; height: 2.5rem; object-fit: cover; border-radius: var(--nojs-radius-sm); background: var(--nojs-secondary); color: var(--nojs-muted); }
.nojs-upload-name { min-width: 0; overflow-wrap: anywhere; font-weight: 500; }
.nojs-upload-size { color: var(--nojs-muted); font-variant-numeric: tabular-nums; white-space: nowrap; }
"#;

#[cfg(test)]
mod tests {
    use super::size;

    #[test]
    fn sizes_read_like_a_file_manager() {
        assert_eq!(size(512), "512 B");
        assert_eq!(size(48_213), "47.1 KB");
        assert_eq!(size(3 * 1024 * 1024 + 200_000), "3.2 MB");
    }
}
