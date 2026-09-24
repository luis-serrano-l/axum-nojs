//! # Empty state
//!
//! What a list, table or search shows when there is nothing in it: what is missing, why, and
//! the one thing to do next, as a real link or form button.
//!
//! **Platform features:** plain HTML; the action is an `<a>` or, for something that changes
//! data, a `<form method="post">` button, so it works with nothing else.
//!
//! **What it does not do without script:** nothing; it is static content.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use maud::html;
//! use axum_nojs::{Caps, empty_state, empty_state_with, empty_state::EmptyOptions};
//! let m = empty_state(&Caps::all(), "No files yet").into_string();
//! assert!(m.contains("No files yet"));
//! let m = empty_state_with(&Caps::all(), "No results for \u{201c}zzz\u{201d}", EmptyOptions::default()
//!     .icon("\u{1f50d}")
//!     .text(html! { "Check the spelling or clear the filter." })
//!     .link("Clear the filter", "/table")
//!     .post("Create a file", "/files/new")).into_string();
//! assert!(m.contains(r#"href="/table""#) && m.contains(r#"action="/files/new""#));
//! ```

use maud::{Markup, html};

use crate::Caps;

/// Options for [`empty_state`].
#[derive(Clone, Debug, Default)]
pub struct EmptyOptions<'a> {
    /// A glyph or emoji above the title, hidden from screen readers.
    pub icon: Option<&'a str>,
    /// One or two sentences: why it is empty.
    pub text: Option<Markup>,
    /// A secondary action: `(label, href)`.
    pub link: Option<(&'a str, &'a str)>,
    /// The primary action, posted: `(label, action)`.
    pub post: Option<(&'a str, &'a str)>,
}

impl<'a> EmptyOptions<'a> {
    /// A glyph above the title.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Why it is empty.
    pub fn text(mut self, text: Markup) -> Self {
        self.text = Some(text);
        self
    }
    /// A link to follow.
    pub fn link(mut self, label: &'a str, href: &'a str) -> Self {
        self.link = Some((label, href));
        self
    }
    /// A button posting to `action`.
    pub fn post(mut self, label: &'a str, action: &'a str) -> Self {
        self.post = Some((label, action));
        self
    }
}

/// An empty state with only a title.
/// [`empty_state_with`] takes the options.
pub fn empty_state(caps: &Caps, title: &str) -> Markup {
    empty_state_with(caps, title, Default::default())
}

/// An empty state titled `title`.
pub fn empty_state_with(_caps: &Caps, title: &str, options: EmptyOptions) -> Markup {
    html! {
        div class="nojs-empty" {
            @if let Some(i) = options.icon { span class="nojs-empty-icon" aria-hidden="true" { (i) } }
            p class="nojs-empty-title" { (title) }
            @if let Some(t) = &options.text { p class="nojs-empty-text" { (t) } }
            @if options.link.is_some() || options.post.is_some() {
                div class="nojs-empty-actions" {
                    @if let Some((label, action)) = options.post {
                        form method="post" action=(action) { button type="submit" class="nojs-primary" { (label) } }
                    }
                    @if let Some((label, href)) = options.link { a href=(href) { (label) } }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-empty {
  display: grid; justify-items: center; gap: var(--nojs-space); text-align: center;
  padding: calc(var(--nojs-space) * 5) calc(var(--nojs-space) * 2);
  border: 1px dashed var(--nojs-line); border-radius: var(--nojs-radius);
}
.nojs-empty-icon { font-size: 2rem; line-height: 1; }
.nojs-empty-title { margin: 0; font-size: 1.125rem; font-weight: 600; }
.nojs-empty-text { margin: 0; color: var(--nojs-muted); max-width: 32rem; }
.nojs-empty-actions { display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: calc(var(--nojs-space) * 2); margin-top: var(--nojs-space); }
.nojs-empty-actions form { margin: 0; }
"#;
