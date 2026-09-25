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
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let m = ui.alert("Heads up").description("You can add components to your app.").render().into_string();
//! assert!(m.contains(r#"role="status""#) && m.contains("Heads up"));
//! let m = ui.alert("Payment failed").danger().description("Your card was declined.").render().into_string();
//! assert!(m.contains(r#"class="lui-alert lui-alert-danger" role="alert""#));
//! // The same in `lui!`:
//! let same = lui! { Alert("Payment failed") danger description="Your card was declined."; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::props::{Prop, PropKind};
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

impl Alert<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[
        Prop::new("description", PropKind::Value, "text: &'a str")
            .doc("A line of text under the title."),
        Prop::new("body", PropKind::Value, "markup: Markup")
            .doc("Markup under the title (a list, a link), after the description."),
        Prop::new("icon", PropKind::Value, "icon: Icon").doc("Another icon than the tone's own."),
        Prop::new("danger", PropKind::Switch, "").doc("Something went wrong."),
        Prop::new("warn", PropKind::Switch, "").doc("Something to watch."),
        Prop::new("ok", PropKind::Switch, "").doc("Something worked."),
    ];
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

    /// Something went wrong: `--lui-danger`, and `role="alert"`.
    pub fn danger(mut self) -> Self {
        self.tone = Some("danger");
        self
    }

    /// Something to watch: `--lui-warn`.
    pub fn warn(mut self) -> Self {
        self.tone = Some("warn");
        self
    }

    /// Something worked: `--lui-ok`.
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
            div class={ "lui-alert" @if let Some(t) = self.tone { " lui-alert-" (t) } }
                role=(if self.tone == Some("danger") { "alert" } else { "status" }) {
                (icon)
                p class="lui-alert-title" { (self.title) }
                @if let Some(d) = self.description { p class="lui-alert-description" { (d) } }
                @if let Some(b) = &self.body { div class="lui-alert-description" { (b) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn Alert: a bordered
/// card, the icon in the first column, title and description beside it.
pub const CSS: &str = r#"
.lui-alert {
  display: grid; grid-template-columns: 1rem 1fr; column-gap: 0.75rem; row-gap: 0.125rem; align-items: start;
  padding: 0.75rem 1rem; font-size: 0.875rem; color: var(--lui-fg);
  background: var(--lui-card); border: 1px solid var(--lui-line); border-radius: var(--lui-radius);
}
.lui-alert > .lui-icon { grid-row: 1 / span 2; margin-top: 0.125rem; }
.lui-alert > :not(.lui-icon) { grid-column: 2; margin: 0; }
.lui-alert-title { font-weight: 500; line-height: 1.25rem; }
.lui-alert-description { color: var(--lui-muted); line-height: 1.25rem; }
.lui-alert-description p { margin: 0; }
.lui-alert-danger { color: var(--lui-danger); }
.lui-alert-danger .lui-alert-description { color: color-mix(in srgb, var(--lui-danger) 90%, var(--lui-fg)); }
.lui-alert-warn > .lui-icon { color: var(--lui-warn); }
.lui-alert-ok > .lui-icon { color: var(--lui-ok); }
"#;
