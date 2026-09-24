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
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! assert!(ui.empty_state("No files yet").render().into_string().contains("No files yet"));
//! let m = ui.empty_state("No results for \u{201c}zzz\u{201d}")
//!     .icon("\u{1f50d}")
//!     .text(html! { "Check the spelling or clear the filter." })
//!     .link("Clear the filter", "/table")
//!     .post("Create a file", "/files/new");
//! let m = m.render().into_string();
//! assert!(m.contains(r#"href="/table""#) && m.contains(r#"action="/files/new""#));
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::{Caps, Ui};

/// What a list shows when there is nothing in it, made by [`Ui::empty_state`].
///
/// **Setters.** Values and items: `.icon(..)`, `.text(..)`, `.link(..)`, `.post(..)`.
#[derive(Clone, Debug, Default)]
pub struct EmptyState<'a> {
    title: &'a str,
    icon: Option<&'a str>,
    text: Option<Markup>,
    link: Option<(&'a str, &'a str)>,
    post: Option<(&'a str, &'a str)>,
}

impl Ui {
    /// An empty state saying `title`: what is missing.
    pub fn empty_state<'a>(&self, title: &'a str) -> EmptyState<'a> {
        EmptyState {
            title,
            ..EmptyState::default()
        }
    }
}

impl<'a> EmptyState<'a> {
    /// A glyph or emoji above the title, hidden from screen readers.
    pub fn icon(mut self, icon: &'a str) -> Self {
        self.icon = Some(icon);
        self
    }

    /// One or two sentences: why it is empty.
    pub fn text(mut self, text: Markup) -> Self {
        self.text = Some(text);
        self
    }

    /// A link to follow: the secondary action.
    pub fn link(mut self, label: &'a str, href: &'a str) -> Self {
        self.link = Some((label, href));
        self
    }

    /// A button posting to `action`: the primary action.
    pub fn post(mut self, label: &'a str, action: &'a str) -> Self {
        self.post = Some((label, action));
        self
    }
}

impl Render for EmptyState<'_> {
    fn render(&self) -> Markup {
        html! {
            div class="nojs-empty" {
                @if let Some(i) = self.icon { span class="nojs-empty-icon" aria-hidden="true" { (i) } }
                p class="nojs-empty-title" { (self.title) }
                @if let Some(t) = &self.text { p class="nojs-empty-text" { (t) } }
                @if self.link.is_some() || self.post.is_some() {
                    div class="nojs-empty-actions" {
                        @if let Some((label, action)) = self.post {
                            form method="post" action=(action) { (Button::new(Caps::NONE, label).primary()) }
                        }
                        @if let Some((label, href)) = self.link { (Button::link(Caps::NONE, label, href)) }
                    }
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-empty {
  display: grid; justify-items: center; gap: var(--nojs-space); text-align: center;
  padding: calc(var(--nojs-space) * 6) calc(var(--nojs-space) * 3);
  border: 1px dashed var(--nojs-line); border-radius: var(--nojs-radius-lg);
}
.nojs-empty-icon { display: grid; place-items: center; width: 2.5rem; height: 2.5rem; font-size: 1.25rem; line-height: 1; border-radius: var(--nojs-radius-sm); background: var(--nojs-secondary); }
.nojs-empty-title { margin: 0; font-size: 1.125rem; font-weight: 500; letter-spacing: -0.0125em; }
.nojs-empty-text { margin: 0; color: var(--nojs-muted); max-width: 24rem; font-size: 0.875rem; }
.nojs-empty-actions { display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: calc(var(--nojs-space) * 2); margin-top: var(--nojs-space); }
.nojs-empty-actions form { margin: 0; }
"#;
