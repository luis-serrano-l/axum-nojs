//! # Empty state
//!
//! What a list, table or search shows when there is nothing in it: what is missing, why, and
//! the one thing to do next, as a real link or form button.
//!
//! **Platform features:** plain HTML; the action is an `<a>` or, for something that changes
//! data, a `<form method="post">` button, so it works with nothing else.
//!
//! **Accessibility:** a heading, a sentence and a real action; the illustration is
//! `aria-hidden`. Checked by axe-core in headless Firefox on every demo route, both capability
//! variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** nothing; it is static content.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from(Caps::all());
//! assert!(ui.empty_state("No files yet").render().into_string().contains("No files yet"));
//! let m = ui.empty_state("No results for \u{201c}zzz\u{201d}")
//!     .icon("\u{1f50d}")
//!     .body(html! { "Check the spelling or clear the filter." })
//!     .link("Clear the filter", "/table")
//!     .action("Create a file", "/files/new");
//! let m = m.render().into_string();
//! assert!(m.contains(r#"href="/table""#) && m.contains(r#"action="/files/new""#));
//! // The same in `lui!`:
//! let same = lui! {
//!     EmptyState("No results for \u{201c}zzz\u{201d}") icon="\u{1f50d}"
//!         body=(html! { "Check the spelling or clear the filter." }) {
//!         link "Clear the filter" "/table";
//!         action "Create a file" "/files/new";
//!     }
//! };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::icon::Glyph;
use crate::props::{Prop, PropKind};
use crate::{Caps, Ui};

/// What a list shows when there is nothing in it, made by [`Ui::empty_state`].
///
/// **Setters.** Values and items: `.icon(..)`, `.body(..)`, `.link(..)`, `.action(..)`.
#[derive(Clone, Debug, Default)]
pub struct EmptyState<'a> {
    title: &'a str,
    icon: Option<Glyph<'a>>,
    text: Option<Markup>,
    link: Option<(&'a str, &'a str)>,
    post: Option<(&'a str, &'a str)>,
}

impl EmptyState<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("icon", PropKind::Value, "icon: impl Into<Glyph<'a>>")
            .doc("An icon, or a glyph or emoji, above the title, hidden from screen readers."),
        Prop::new("body", PropKind::Value, "text: Markup").doc("One or two sentences."),
        Prop::new("link", PropKind::Value, "label: &'a str, href: &'a str")
            .doc("A link to follow."),
        Prop::new("action", PropKind::Value, "label: &'a str, action: &'a str")
            .doc("A button posting to `action`."),
    ];
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
    /// An icon, or a glyph or emoji, above the title, hidden from screen readers.
    pub fn icon(mut self, icon: impl Into<Glyph<'a>>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// One or two sentences: why it is empty.
    pub fn body(mut self, text: Markup) -> Self {
        self.text = Some(text);
        self
    }

    /// The old name of [`Self::body`], kept for one release.
    #[deprecated(note = "use .body()")]
    pub fn text(self, text: Markup) -> Self {
        self.body(text)
    }

    /// A link to follow: the secondary action.
    pub fn link(mut self, label: &'a str, href: &'a str) -> Self {
        self.link = Some((label, href));
        self
    }

    /// A button posting to `action`: the primary action.
    pub fn action(mut self, label: &'a str, action: &'a str) -> Self {
        self.post = Some((label, action));
        self
    }

    /// The old name of [`Self::action`], kept for one release.
    #[deprecated(note = "use .action()")]
    pub fn post(self, label: &'a str, action: &'a str) -> Self {
        self.action(label, action)
    }
}

impl Render for EmptyState<'_> {
    fn render(&self) -> Markup {
        html! {
            div class="lui-empty" {
                @if let Some(i) = self.icon { span class="lui-empty-icon" aria-hidden="true" { (i) } }
                p class="lui-empty-title" { (self.title) }
                @if let Some(t) = &self.text { p class="lui-empty-text" { (t) } }
                @if self.link.is_some() || self.post.is_some() {
                    div class="lui-empty-actions" {
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
.lui-empty {
  display: grid; justify-items: center; gap: var(--lui-space); text-align: center;
  padding: calc(var(--lui-space) * 6) calc(var(--lui-space) * 3);
  border: 1px dashed var(--lui-line); border-radius: var(--lui-radius-lg);
}
.lui-empty-icon { display: grid; place-items: center; width: 2.5rem; height: 2.5rem; font-size: 1.25rem; line-height: 1; border-radius: var(--lui-radius-sm); background: var(--lui-secondary); }
.lui-empty-title { margin: 0; font-size: 1.125rem; font-weight: 500; letter-spacing: -0.0125em; }
.lui-empty-text { margin: 0; color: var(--lui-muted); max-width: 24rem; font-size: 0.875rem; }
.lui-empty-actions { display: flex; flex-wrap: wrap; align-items: center; justify-content: center; gap: calc(var(--lui-space) * 2); margin-top: var(--lui-space); }
.lui-empty-actions form { margin: 0; }
"#;
