//! # Error summary
//!
//! The list of what went wrong, at the top of a form the server sent back (GOV.UK's error
//! summary): a heading, then one link per field in error that jumps to the field. The page
//! opens with the focus on it, so a keyboard or screen reader user starts at the problems
//! instead of hunting for them. [`Form`](crate::form::Form) shows one by itself whenever it
//! has messages; `ui.error_summary(..)` is for a form built from loose inputs.
//!
//! **Platform features:**
//! - `role="alert"` (read out when the page loads) with `aria-labelledby` naming the heading.
//! - `autofocus` on the heading's link (global attribute on any focusable element: Chrome
//!   79, Firefox 110, Safari 15.4): the focus starts in the summary.
//! - Fragment links (`href="#f-email"`) to each field's `id`.
//!
//! **What it does not do without script:** move the focus into the field a link points to;
//! the page scrolls to it, and Tab goes on from there.
//!
//! **Fallback:** where `autofocus` only works on form controls, the summary is still the
//! first thing in the form and is read out as an alert.
//!
//! **Enhanced:** when the enhancement script swaps a form in place, it focuses the
//! `autofocus` element of the new markup too, as a page load would.
//!
//! ```rust
//! use loco_ui::prelude::*;
//! let ui = Ui::default();
//! let errors = [("email", "Enter an email address."), ("password", "At least 8 characters.")];
//! let m = ui.error_summary(&errors).render().into_string();
//! assert!(m.contains(r##"<a href="#f-email" autofocus>There is a problem</a>"##));
//! assert!(m.contains(r##"<li><a href="#f-password">At least 8 characters.</a></li>"##));
//! assert!(ui.error_summary(&[]).render().into_string().is_empty());
//!
//! let m = ui.error_summary(&errors).title("Check your details").render().into_string();
//! assert!(m.contains(r#"role="alert""#) && m.contains(">Check your details</a>"));
//! // The same in `lui!`:
//! let same = lui! { ErrorSummary(&errors) title="Check your details"; };
//! assert_eq!(same.into_string(), m);
//! ```

use maud::{Markup, Render, html};

use crate::Ui;
use crate::i18n::Text;
use crate::props::{Prop, PropKind};

/// A list of the fields in error, made by [`Ui::error_summary`] (and by a
/// [`Form`](crate::form::Form) with messages).
///
/// **Setters.** Values and items: `.title(..)`.
#[derive(Clone, Debug)]
pub struct ErrorSummary<'a> {
    title: &'a str,
    /// `(the field's id, its label, the message)`; no id for a message about no field.
    items: Vec<(Option<String>, Option<&'a str>, &'a str)>,
}

impl ErrorSummary<'_> {
    /// Every setter with its kind, arguments, default and the HTML attribute it sets; listed by
    /// [`crate::props()`] and kept in step with the setters by a test.
    pub const PROPS: &'static [Prop] = &[Prop::new("title", PropKind::Value, "text: &'a str")
        .default("There is a problem")
        .doc("The heading.")];
}

impl Ui {
    /// A summary of `errors`, `(field name, message)` pairs as [`Form::errors`](crate::form::Form::errors)
    /// takes them; each links to `#f-<name>`, the id a form field gets. Empty when there are
    /// none.
    pub fn error_summary<'a>(&self, errors: &'a [(&'a str, &'a str)]) -> ErrorSummary<'a> {
        let items = errors
            .iter()
            .map(|(name, message)| {
                let id = (!name.is_empty()).then(|| format!("f-{name}"));
                (id, None, *message)
            })
            .collect();
        ErrorSummary {
            title: self.text(Text::Problem),
            items,
        }
    }
}

impl<'a> ErrorSummary<'a> {
    /// The heading.
    pub fn title(mut self, text: &'a str) -> Self {
        self.title = text;
        self
    }

    /// A form's summary: every message with the id and label of its field.
    pub(crate) fn from_fields(items: Vec<(Option<String>, Option<&'a str>, &'a str)>) -> Self {
        ErrorSummary { title: "", items }
    }
}

impl Render for ErrorSummary<'_> {
    fn render(&self) -> Markup {
        if self.items.is_empty() {
            return html! {};
        }
        let first = self.items.iter().find_map(|(id, _, _)| id.as_deref());
        html! {
            div class="lui-error-summary" role="alert" aria-labelledby="lui-error-summary-title" {
                @if let Some(first) = first {
                    h2 class="lui-error-summary-title" id="lui-error-summary-title" {
                        a href={ "#" (first) } autofocus { (self.title) }
                    }
                } @else {
                    h2 class="lui-error-summary-title" id="lui-error-summary-title" tabindex="-1" autofocus { (self.title) }
                }
                ul class="lui-error-summary-list" {
                    @for (id, label, message) in &self.items {
                        li {
                            @if let Some(id) = id {
                                a href={ "#" (id) } { @if let Some(l) = label { (l) ": " } (message) }
                            } @else {
                                (message)
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. A danger-toned card, as
/// shadcn's destructive alert, with the links underlined.
pub const CSS: &str = r#"
.lui-error-summary {
  display: grid; gap: 0.5rem; padding: 0.75rem 1rem; font-size: 0.875rem;
  color: var(--lui-danger); background: var(--lui-card);
  border: 1px solid var(--lui-danger); border-radius: var(--lui-radius);
}
.lui-error-summary-title { margin: 0; font-size: 1rem; font-weight: 600; line-height: 1.5rem; }
.lui-error-summary-title a { color: inherit; text-decoration: none; }
.lui-error-summary-title a:focus-visible, .lui-error-summary-title:focus-visible { outline: 2px solid var(--lui-ring); outline-offset: 2px; border-radius: 2px; }
.lui-error-summary-list { margin: 0; padding-left: 1.25rem; display: grid; gap: 0.25rem; }
.lui-error-summary-list a { color: inherit; text-decoration: underline; text-underline-offset: 2px; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_message_about_no_field_is_listed_without_a_link() {
        let m = Ui::default()
            .error_summary(&[("", "The form expired. Send it again.")])
            .render()
            .into_string();
        assert!(
            m.contains(r#"tabindex="-1" autofocus>There is a problem</h2>"#),
            "{m}"
        );
        assert!(
            m.contains("<li>The form expired. Send it again.</li>"),
            "{m}"
        );
    }
}
