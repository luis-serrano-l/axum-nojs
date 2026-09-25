//! # Badge
//!
//! A short label beside something: a status, a count, a tag. shadcn's four looks (filled with
//! the primary colour, secondary, danger, outline) plus `ok` and `warn` tints for statuses.
//!
//! **Platform features:** a `<span>` (or an `<a>` with `.href()`); nothing interactive of its
//! own.
//!
//! **Accessibility:** plain text in a `<span>`; the tone colours are mixed with the text colour
//! so they pass AA on their tint. Checked by axe-core in headless Firefox on every demo route,
//! both capability variants, light and dark (no serious or critical violation).
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! assert_eq!(ui.badge("New").render().into_string(), r#"<span class="lui-badge">New</span>"#);
//! // The same in `lui!`:
//! let same = lui! { Badge("New"); };
//! assert_eq!(same.into_string(), ui.badge("New").render().into_string());
//! let paid = ui.badge("Paid").ok().render().into_string();
//! assert!(paid.contains("lui-badge lui-badge-ok"));
//! let tag = ui.badge("rust").outline().href("/tags/rust").render().into_string();
//! assert!(tag.starts_with(r#"<a class="lui-badge lui-badge-outline" href="/tags/rust">"#));
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
        Prop::new("secondary", PropKind::Switch, "").doc("The quieter `--lui-secondary` fill."),
        Prop::new("danger", PropKind::Switch, "").doc("Filled with `--lui-danger`."),
        Prop::new("outline", PropKind::Switch, "").doc("A border and no fill."),
        Prop::new("ok", PropKind::Switch, "").doc("A tint of `--lui-ok`."),
        Prop::new("warn", PropKind::Switch, "").doc("A tint of `--lui-warn`."),
        Prop::new("href", PropKind::Value, "href: &'a str")
            .attr("href")
            .doc("Make the badge a link."),
    ];
}

impl Ui {
    /// A badge reading `text`, filled with `--lui-primary`.
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

    /// The quieter `--lui-secondary` fill.
    pub fn secondary(self) -> Self {
        self.tone("lui-badge-secondary")
    }

    /// Filled with `--lui-danger`.
    pub fn danger(self) -> Self {
        self.tone("lui-badge-danger")
    }

    /// A border and no fill.
    pub fn outline(self) -> Self {
        self.tone("lui-badge-outline")
    }

    /// A tint of `--lui-ok`: done, paid, healthy.
    pub fn ok(self) -> Self {
        self.tone("lui-badge-ok")
    }

    /// A tint of `--lui-warn`: pending, degraded.
    pub fn warn(self) -> Self {
        self.tone("lui-badge-warn")
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
            Some(t) => format!("lui-badge {t}"),
            None => "lui-badge".to_string(),
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
.lui-badge {
  display: inline-flex; align-items: center; gap: 0.25rem; width: fit-content; white-space: nowrap;
  padding: 0.125rem 0.5rem; font-size: 0.75rem; line-height: 1rem; font-weight: 500; text-decoration: none;
  border: 1px solid transparent; border-radius: var(--lui-radius-sm);
  background: var(--lui-primary); color: var(--lui-on-primary); transition: background-color 0.15s;
}
a.lui-badge:hover { background: color-mix(in srgb, var(--lui-primary) 90%, transparent); }
.lui-badge.lui-badge-secondary { background: var(--lui-secondary); color: var(--lui-fg); }
.lui-badge.lui-badge-danger { background: var(--lui-danger); color: var(--lui-on-primary); }
.lui-badge.lui-badge-outline { background: transparent; color: var(--lui-fg); border-color: var(--lui-line); }
a.lui-badge:is(.lui-badge-secondary, .lui-badge-outline):hover { background: var(--lui-accent); }
/* The tone mixed with the text colour: darker on light, lighter on dark, AA on its tint. */
.lui-badge.lui-badge-ok { background: color-mix(in srgb, var(--lui-ok) 15%, transparent); color: color-mix(in srgb, var(--lui-ok) 75%, var(--lui-fg)); }
.lui-badge.lui-badge-warn { background: color-mix(in srgb, var(--lui-warn) 15%, transparent); color: color-mix(in srgb, var(--lui-warn) 70%, var(--lui-fg)); }
.lui-badge .lui-icon { width: 0.75rem; height: 0.75rem; }
"#;
