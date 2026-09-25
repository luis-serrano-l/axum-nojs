//! # Breadcrumbs
//!
//! Where this page sits: a trail of links ending in the current page. A long trail folds its
//! middle into a disclosure so the ends stay readable on a phone.
//!
//! **Platform features:** `<nav aria-label="Breadcrumb">` around an ordered list, the last
//! item marked `aria-current="page"`; separators drawn by CSS `::before` so screen readers do
//! not read them; the folded middle is a `<details>` element.
//!
//! **Accessibility:** a `<nav>` named "Breadcrumb" with an ordered list; the current page has
//! `aria-current="page"`; folded steps sit in a named `<details>`. Checked by axe-core in
//! headless Firefox on every demo route, both capability variants, light and dark (no serious
//! or critical violation).
//!
//! **What it does not do without script:** collapse to fit the width as it changes; the fold is
//! decided by item count on the server.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let m = ui.breadcrumbs().link("Home", "/").link("Projects", "/projects").here("loco-ui").render().into_string();
//! assert!(m.contains(r#"<a href="/projects">Projects</a>"#));
//! assert!(m.contains(r#"aria-current="page">loco-ui"#));
//! // More than four: the middle folds into <details>.
//! let long = ui.breadcrumbs().link("Home", "/").link("A", "/a").link("B", "/a/b").link("C", "/a/b/c").here("Here");
//! assert!(long.render().into_string().contains("<details"));
//! // The same in `lui!`:
//! let same = lui! { Breadcrumbs {
//!     link "Home" "/"; link "Projects" "/projects"; here "loco-ui";
//! } };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::i18n::{Strings, Text};
use crate::props::{Prop, PropKind};

/// A trail of links ending in the current page, made by [`Ui::breadcrumbs`].
///
/// **Setters.** Values and items: `.link(..)`, `.here(..)`.
#[derive(Clone, Debug, Default)]
pub struct Breadcrumbs<'a> {
    trail: Vec<(&'a str, &'a str)>,
    here: &'a str,
    strings: &'static Strings,
}

impl Breadcrumbs<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("link", PropKind::Item, "label: &'a str, href: &'a str")
            .doc("One step from the root towards this page."),
        Prop::new("here", PropKind::Item, "label: &'a str")
            .doc("The current page, last in the trail and not a link."),
    ];
}

impl Ui {
    /// An empty trail: add the way down with [`Breadcrumbs::link`], then the page with
    /// [`Breadcrumbs::here`].
    pub fn breadcrumbs(&self) -> Breadcrumbs<'_> {
        Breadcrumbs {
            strings: self.strings,
            ..Breadcrumbs::default()
        }
    }
}

impl<'a> Breadcrumbs<'a> {
    /// One step from the root towards this page.
    pub fn link(mut self, label: &'a str, href: &'a str) -> Self {
        self.trail.push((label, href));
        self
    }

    /// The current page, last in the trail and not a link.
    pub fn here(mut self, label: &'a str) -> Self {
        self.here = label;
        self
    }
}

impl Render for Breadcrumbs<'_> {
    fn render(&self) -> Markup {
        let before = &self.trail[..];
        let fold = before.len() > 3;
        let (head, middle, tail) = if fold {
            (
                &before[..1],
                &before[1..before.len() - 1],
                &before[before.len() - 1..],
            )
        } else {
            (before, &before[..0], &before[..0])
        };
        html! {
            nav class="lui-breadcrumbs" aria-label=(self.strings.get(Text::Breadcrumb)) {
                ol {
                    @for (label, href) in head { li { a href=(href) { (label) } } }
                    @if fold {
                        li class="lui-breadcrumbs-fold" {
                            details {
                                summary aria-label=(self.strings.fill(Text::ShowMore, &[&middle.len()])) { "\u{2026}" }
                                ol { @for (label, href) in middle { li { a href=(href) { (label) } } } }
                            }
                        }
                    }
                    @for (label, href) in tail { li { a href=(href) { (label) } } }
                    li { span aria-current="page" { (self.here) } }
                }
            }
        }
    }
}
/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.lui-breadcrumbs { font-size: 0.875rem; color: var(--lui-muted); margin-bottom: calc(var(--lui-space) * 2); }
.lui-breadcrumbs > ol { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 0; list-style: none; margin: 0; padding: 0; }
.lui-breadcrumbs > ol > li + li::before { content: "/"; margin-inline: 0.625rem; color: var(--lui-muted); }
.lui-breadcrumbs a { color: var(--lui-muted); text-decoration: none; transition: color 0.15s; }
.lui-breadcrumbs a:hover { color: var(--lui-fg); }
.lui-breadcrumbs [aria-current] { color: var(--lui-fg); font-weight: 400; }
.lui-breadcrumbs-fold { position: relative; }
.lui-breadcrumbs-fold details { display: inline-block; }
.lui-breadcrumbs-fold summary { display: inline; list-style: none; cursor: pointer; padding-inline: 0.25rem; border-radius: var(--lui-radius-sm); }
.lui-breadcrumbs-fold summary::-webkit-details-marker { display: none; }
.lui-breadcrumbs-fold summary:hover { background: var(--lui-accent); color: var(--lui-on-accent); }
.lui-breadcrumbs-fold ol {
  position: absolute; z-index: 5; top: 100%; left: 0; margin: 0.25rem 0 0; padding: 0.25rem;
  list-style: none; min-width: 10rem; background: var(--lui-popover);
  border: 1px solid var(--lui-line); border-radius: var(--lui-radius); box-shadow: var(--lui-shadow-md), var(--lui-highlight);
}
.lui-breadcrumbs-fold ol a { display: block; padding: 0.375rem 0.5rem; border-radius: var(--lui-radius-sm); color: var(--lui-fg); }
.lui-breadcrumbs-fold ol a:hover { background: var(--lui-accent); color: var(--lui-on-accent); }
"#;
