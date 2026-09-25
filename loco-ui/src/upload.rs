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
//! **Accessibility:** a native file input inside its label (Enter or Space opens the picker), a
//! labelled upload button, and remove buttons named with the file. Checked by axe-core in
//! headless Firefox on every demo route, both capability variants, light and dark (no serious
//! or critical violation).
//!
//! **What it does not do without script:** show the upload's progress (the enhancement script
//! sends the form through `XMLHttpRequest` and fills the `<progress data-lui-progress>` bar;
//! without it the browser shows its own loading indicator), preview a file before it is sent,
//! or accept a drop anywhere but on the input.
//!
//! **Fallback:** none needed: the form is a plain multipart post.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.upload("/files", "file").render().into_string();
//! assert!(m.contains(r#"enctype="multipart/form-data""#) && m.contains(r#"type="file""#));
//! // Images only, several at once, and what the server already has.
//! let m = ui.upload("/files", "file")
//!     .accept("image/*").multiple().help("PNG or JPEG, up to 200 KB.")
//!     .file("cat.png", 48_213).preview("/files/cat.png")
//!     .file("notes.txt", 1_024).href("/files/notes.txt")
//!     .remove("/files/remove");
//! let m = m.render().into_string();
//! assert!(m.contains(r#"accept="image/*" multiple"#) && m.contains(r#"src="/files/cat.png""#));
//! assert!(m.contains("47.1 KB") && m.contains(r#"name="file" value="notes.txt""#));
//!
//! // The same in `lui!`:
//! let same = lui! { Upload("/files", "file")
//!     accept="image/*" multiple help="PNG or JPEG, up to 200 KB." {
//!     file "cat.png" 48_213 preview="/files/cat.png";
//!     file "notes.txt" 1_024 href="/files/notes.txt";
//!     remove "/files/remove";
//! } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::i18n::Text;
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
/// **Setters.** Values and items: `.accept(..)`, `.help(..)`, `.file(..)`, `.preview(..)`,
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
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("accept", PropKind::Value, "types: &'a str")
            .attr("accept")
            .doc("`accept`."),
        Prop::new("multiple", PropKind::Switch, "")
            .attr("multiple")
            .doc("Several files at once."),
        Prop::new("help", PropKind::Value, "text: &'a str").doc("Small print in the drop zone."),
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
    pub fn help(mut self, text: &'a str) -> Self {
        self.hint = Some(text);
        self
    }

    /// The old name of [`Self::help`], kept for one release.
    #[deprecated(note = "use .help()")]
    pub fn hint(self, text: &'a str) -> Self {
        self.help(text)
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
        let root = enhance::swap_id("lui-upload", self.action);
        let input_id = format!("{root}-input");
        let caps = self.ui.caps;
        html! {
            div id=(root) data-lui="swap" class="lui-upload" {
                form method="post" action=(self.action) enctype="multipart/form-data" class="lui-upload-form" {
                    label class="lui-upload-drop" for=(input_id) {
                        (Icon::Upload)
                        span class="lui-upload-title" { (self.ui.text(if self.multiple { Text::ChooseFiles } else { Text::ChooseFile })) }
                        @if let Some(h) = self.hint { span class="lui-upload-hint" { (h) } }
                        input id=(input_id) class="lui-upload-input" type="file" name=(self.name)
                            accept=[self.accept] multiple[self.multiple] required;
                    }
                    progress class="lui-upload-progress" data-lui-progress hidden {}
                    (Button::new(caps, self.ui.text(Text::Upload)).primary())
                }
                @if !self.files.is_empty() {
                    ul class="lui-upload-files" aria-label=(self.ui.text(Text::UploadedFiles)) {
                        @for f in &self.files {
                            li class="lui-upload-file" {
                                @if let Some(src) = f.preview {
                                    img class="lui-upload-thumb" src=(src) alt="" loading="lazy";
                                } @else {
                                    span class="lui-upload-thumb" aria-hidden="true" { (Icon::File) }
                                }
                                span class="lui-upload-name" {
                                    @if let Some(h) = f.href { a href=(h) { (f.name) } } @else { (f.name) }
                                }
                                span class="lui-upload-size" { (size(f.size)) }
                                @if let Some(action) = self.remove {
                                    form method="post" action=(action) {
                                        @let label = self.ui.fill(Text::RemoveValue, &[&f.name]);
                                        (Button::new(caps, "").ghost().small().icon_only().aria_label(&label).name(self.name).value(f.name).body(html! { (Icon::Trash) }))
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
.lui-upload { display: grid; gap: calc(var(--lui-space) * 2); max-width: 32rem; }
.lui-upload-form { display: grid; gap: var(--lui-space); justify-items: start; }
.lui-upload-drop {
  display: grid; justify-items: center; gap: 0.5rem; width: 100%; box-sizing: border-box; padding: 1.5rem;
  text-align: center; cursor: pointer; font-weight: 400;
  border: 1px dashed var(--lui-input); border-radius: var(--lui-radius-lg); background: var(--lui-surface);
  transition: border-color 0.15s, background-color 0.15s;
}
.lui-upload-drop:hover, .lui-upload-drop:has(.lui-upload-input:focus-visible) { border-color: var(--lui-ring); }
.lui-upload-drop > .lui-icon { width: 1.5rem; height: 1.5rem; color: var(--lui-muted); }
.lui-upload-title { font-size: 0.875rem; font-weight: 500; }
.lui-upload-hint { font-size: 0.75rem; color: var(--lui-muted); }
.lui-upload-input { max-width: 100%; }
.lui-upload-progress { width: 100%; height: 0.5rem; accent-color: var(--lui-primary); }
.lui-upload-files { list-style: none; margin: 0; padding: 0; display: grid; gap: 0.5rem; }
.lui-upload-file {
  display: flex; align-items: center; gap: 0.75rem; padding: 0.5rem 0.75rem; font-size: 0.875rem;
  border: 1px solid var(--lui-line); border-radius: var(--lui-radius); background: var(--lui-card);
}
.lui-upload-file form { margin: 0 0 0 auto; }
.lui-upload-thumb { display: inline-grid; place-items: center; flex: none; width: 2.5rem; height: 2.5rem; object-fit: cover; border-radius: var(--lui-radius-sm); background: var(--lui-secondary); color: var(--lui-muted); }
.lui-upload-name { min-width: 0; overflow-wrap: anywhere; font-weight: 500; }
.lui-upload-size { color: var(--lui-muted); font-variant-numeric: tabular-nums; white-space: nowrap; }
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
