//! # Alert
//!
//! A callout in the page: a title, a line or two of explanation and an icon, in a neutral,
//! danger, warning or success tone. For a message that stays where it is, unlike a flash or a
//! toast.
//!
//! **Platform features:** a `role="alert"` box for the danger tone (read out when the page
//! loads) and `role="status"` for the others; the icon is decorative.
//!
//! **What it does not do without script:** nothing is missing.
//!
//! **Fallback:** none needed.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let m = ui.alert("Heads up").description("You can add components to your app.").render().into_string();
//! assert!(m.contains(r#"role="status""#) && m.contains("Heads up"));
//! let m = ui.alert("Payment failed").danger().description("Your card was declined.").render().into_string();
//! assert!(m.contains(r#"class="nojs-alert nojs-alert-danger" role="alert""#));
//! ```

use maud::{Markup, Render, html};

use crate::{Icon, Ui};

/// A callout, made by [`Ui::alert`].
///
/// **Setters.** Values and items: `.description(..)`, `.body(..)`, `.icon(..)`; switches:
/// `.danger()`, `.warn()`, `.ok()`.
#[derive(Clone, Debug)]
pub struct Alert<'a> {
    title: &'a str,
    description: Option<&'a str>,
    body: Option<Markup>,
    tone: Option<&'static str>,
    icon: Option<Icon>,
}

impl Ui {
    /// A callout headed `title`.
    pub fn alert<'a>(&self, title: &'a str) -> Alert<'a> {
        Alert {
            title,
            description: None,
            body: None,
            tone: None,
            icon: None,
        }
    }
}

impl<'a> Alert<'a> {
    /// A line of text under the title.
    pub fn description(mut self, text: &'a str) -> Self {
        self.description = Some(text);
        self
    }

    /// Markup under the title (a list, a link), after the description.
    pub fn body(mut self, markup: Markup) -> Self {
        self.body = Some(markup);
        self
    }

    /// Another icon than the tone's own.
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Something went wrong: `--nojs-danger`, and `role="alert"`.
    pub fn danger(mut self) -> Self {
        self.tone = Some("danger");
        self
    }

    /// Something to watch: `--nojs-warn`.
    pub fn warn(mut self) -> Self {
        self.tone = Some("warn");
        self
    }

    /// Something worked: `--nojs-ok`.
    pub fn ok(mut self) -> Self {
        self.tone = Some("ok");
        self
    }
}

impl Render for Alert<'_> {
    fn render(&self) -> Markup {
        let icon = self.icon.unwrap_or(match self.tone {
            Some("danger") | Some("warn") => Icon::TriangleAlert,
            Some("ok") => Icon::CircleCheck,
            _ => Icon::Info,
        });
        html! {
            div class={ "nojs-alert" @if let Some(t) = self.tone { " nojs-alert-" (t) } }
                role=(if self.tone == Some("danger") { "alert" } else { "status" }) {
                (icon)
                p class="nojs-alert-title" { (self.title) }
                @if let Some(d) = self.description { p class="nojs-alert-description" { (d) } }
                @if let Some(b) = &self.body { div class="nojs-alert-description" { (b) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn Alert: a bordered
/// card, the icon in the first column, title and description beside it.
pub const CSS: &str = r#"
.nojs-alert {
  display: grid; grid-template-columns: 1rem 1fr; column-gap: 0.75rem; row-gap: 0.125rem; align-items: start;
  padding: 0.75rem 1rem; font-size: 0.875rem; color: var(--nojs-fg);
  background: var(--nojs-card); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius);
}
.nojs-alert > .nojs-icon { grid-row: 1 / span 2; margin-top: 0.125rem; }
.nojs-alert > :not(.nojs-icon) { grid-column: 2; margin: 0; }
.nojs-alert-title { font-weight: 500; line-height: 1.25rem; }
.nojs-alert-description { color: var(--nojs-muted); line-height: 1.25rem; }
.nojs-alert-description p { margin: 0; }
.nojs-alert-danger { color: var(--nojs-danger); }
.nojs-alert-danger .nojs-alert-description { color: color-mix(in srgb, var(--nojs-danger) 90%, var(--nojs-fg)); }
.nojs-alert-warn > .nojs-icon { color: var(--nojs-warn); }
.nojs-alert-ok > .nojs-icon { color: var(--nojs-ok); }
"#;
