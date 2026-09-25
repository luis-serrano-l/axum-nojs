//! # Badge
//!
//! A short label beside something: a status, a count, a tag. shadcn's four looks (filled with
//! the primary colour, secondary, danger, outline) plus `ok` and `warn` tints for statuses.
//!
//! **Platform features:** a `<span>` (or an `<a>` with `.href()`); nothing interactive of its
//! own.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! assert_eq!(ui.badge("New").render().into_string(), r#"<span class="nojs-badge">New</span>"#);
//! // The same in `nojs!`:
//! let same = nojs! { Badge("New"); };
//! assert_eq!(same.into_string(), ui.badge("New").render().into_string());
//! let paid = ui.badge("Paid").ok().render().into_string();
//! assert!(paid.contains("nojs-badge nojs-badge-ok"));
//! let tag = ui.badge("rust").outline().href("/tags/rust").render().into_string();
//! assert!(tag.starts_with(r#"<a class="nojs-badge nojs-badge-outline" href="/tags/rust">"#));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::props::{Prop, PropKind};

/// A badge, made by [`Ui::badge`].
///
/// **Setters.** Values and items: `.href(..)`; switches: `.secondary()`, `.danger()`,
/// `.outline()`, `.ok()`, `.warn()`.
#[derive(Clone, Debug)]
pub struct Badge<'a> {
    text: &'a str,
    tone: Option<&'static str>,
    href: Option<&'a str>,
}

impl Badge<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("secondary", PropKind::Switch, "").doc("The quieter `--nojs-secondary` fill."),
        Prop::new("danger", PropKind::Switch, "").doc("Filled with `--nojs-danger`."),
        Prop::new("outline", PropKind::Switch, "").doc("A border and no fill."),
        Prop::new("ok", PropKind::Switch, "").doc("A tint of `--nojs-ok`."),
        Prop::new("warn", PropKind::Switch, "").doc("A tint of `--nojs-warn`."),
        Prop::new("href", PropKind::Value, "href: &'a str")
            .attr("href")
            .doc("Make the badge a link."),
    ];
}

impl Ui {
    /// A badge reading `text`, filled with `--nojs-primary`.
    pub fn badge<'a>(&self, text: &'a str) -> Badge<'a> {
        Badge {
            text,
            tone: None,
            href: None,
        }
    }
}

impl<'a> Badge<'a> {
    fn tone(mut self, tone: &'static str) -> Self {
        self.tone = Some(tone);
        self
    }

    /// The quieter `--nojs-secondary` fill.
    pub fn secondary(self) -> Self {
        self.tone("nojs-badge-secondary")
    }

    /// Filled with `--nojs-danger`.
    pub fn danger(self) -> Self {
        self.tone("nojs-badge-danger")
    }

    /// A border and no fill.
    pub fn outline(self) -> Self {
        self.tone("nojs-badge-outline")
    }

    /// A tint of `--nojs-ok`: done, paid, healthy.
    pub fn ok(self) -> Self {
        self.tone("nojs-badge-ok")
    }

    /// A tint of `--nojs-warn`: pending, degraded.
    pub fn warn(self) -> Self {
        self.tone("nojs-badge-warn")
    }

    /// Make the badge a link.
    pub fn href(mut self, href: &'a str) -> Self {
        self.href = Some(href);
        self
    }
}

impl Render for Badge<'_> {
    fn render(&self) -> Markup {
        let class = match self.tone {
            Some(t) => format!("nojs-badge {t}"),
            None => "nojs-badge".to_string(),
        };
        html! {
            @if let Some(href) = self.href {
                a class=(class) href=(href) { (self.text) }
            } @else {
                span class=(class) { (self.text) }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-badge {
  display: inline-flex; align-items: center; gap: 0.25rem; width: fit-content; white-space: nowrap;
  padding: 0.125rem 0.5rem; font-size: 0.75rem; line-height: 1rem; font-weight: 500; text-decoration: none;
  border: 1px solid transparent; border-radius: var(--nojs-radius-sm);
  background: var(--nojs-primary); color: var(--nojs-on-primary); transition: background-color 0.15s;
}
a.nojs-badge:hover { background: color-mix(in srgb, var(--nojs-primary) 90%, transparent); }
.nojs-badge.nojs-badge-secondary { background: var(--nojs-secondary); color: var(--nojs-fg); }
.nojs-badge.nojs-badge-danger { background: var(--nojs-danger); color: var(--nojs-on-primary); }
.nojs-badge.nojs-badge-outline { background: transparent; color: var(--nojs-fg); border-color: var(--nojs-line); }
a.nojs-badge:is(.nojs-badge-secondary, .nojs-badge-outline):hover { background: var(--nojs-accent); }
.nojs-badge.nojs-badge-ok { background: color-mix(in srgb, var(--nojs-ok) 15%, transparent); color: var(--nojs-ok); }
.nojs-badge.nojs-badge-warn { background: color-mix(in srgb, var(--nojs-warn) 15%, transparent); color: var(--nojs-warn); }
.nojs-badge .nojs-icon { width: 0.75rem; height: 0.75rem; }
"#;
