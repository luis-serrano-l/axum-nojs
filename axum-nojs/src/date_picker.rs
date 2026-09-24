//! # Date picker
//!
//! A form field for a date: a button showing the date, which opens a calendar in a popover;
//! picking a day checks a radio named after the field, so the form posts `name=YYYY-MM-DD`.
//! `.native()` gives the browser's own `<input type="date">` instead.
//!
//! **Platform features:** `popover` with `popovertarget` (Chrome 114, Firefox 125, Safari 17);
//! CSS anchor positioning (`anchor-name`, `position-anchor`, Chrome 125, Firefox 147, Safari 26)
//! places it under the button; inside, the [`crate::calendar`] in radio mode. `.native()` is
//! `<input type="date">` with `min`/`max` (Chrome 20, Firefox 57, Safari 14.1).
//!
//! **What it does not do without script:** write the picked day onto the button before the form
//! is sent (the button shows the saved value; the picked day is filled in the calendar), or
//! change month without a page load. A month link comes back with the calendar laid out in the
//! page instead of in the popover, because a popover cannot arrive open.
//!
//! **Fallback:** without `popover` the calendar is laid out in the form under the label.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let m = ui.date_picker("due", "Due date").value("2026-09-24").render().into_string();
//! assert!(m.contains(r#"popovertarget="f-due-calendar""#) && m.contains("24 September 2026"));
//! assert!(m.contains(r#"type="radio" name="due" value="2026-09-24" checked"#));
//! // No popover support: the same calendar, in the page.
//! let old = Ui::from(Caps::default());
//! assert!(!old.date_picker("due", "Due date").render().into_string().contains("popover"));
//! // The browser's own control, with bounds.
//! let m = ui.date_picker("due", "Due date").min("2026-01-01").max("2026-12-31").native();
//! assert!(m.render().into_string().contains(r#"type="date" value="" min="2026-01-01" max="2026-12-31""#));
//! ```

use maud::{Markup, Render, html};

use crate::button::Button;
use crate::calendar::Date;
use crate::{Cap, Icon, Ui};

/// A date field, made by [`Ui::date_picker`].
#[derive(Clone, Debug)]
pub struct DatePicker<'a> {
    ui: &'a Ui,
    name: &'a str,
    label: &'a str,
    value: Option<&'a str>,
    min: Option<&'a str>,
    max: Option<&'a str>,
    disabled: Option<fn(Date) -> bool>,
    required: bool,
    native: bool,
}

impl Ui {
    /// A date field named `name` under `label`; its value is `?<name>=` unless `.value()` says
    /// otherwise.
    pub fn date_picker<'a>(&'a self, name: &'a str, label: &'a str) -> DatePicker<'a> {
        DatePicker {
            ui: self,
            name,
            label,
            value: None,
            min: None,
            max: None,
            disabled: None,
            required: false,
            native: false,
        }
    }
}

impl<'a> DatePicker<'a> {
    /// The current value, `YYYY-MM-DD` (a saved date).
    pub fn value(mut self, date: &'a str) -> Self {
        self.value = Some(date);
        self
    }

    /// The first day that can be picked, `YYYY-MM-DD`.
    pub fn min(mut self, date: &'a str) -> Self {
        self.min = Some(date);
        self
    }

    /// The last day that can be picked, `YYYY-MM-DD`.
    pub fn max(mut self, date: &'a str) -> Self {
        self.max = Some(date);
        self
    }

    /// Days for which `off` returns true cannot be picked (the native control ignores this).
    pub fn disabled(mut self, off: fn(Date) -> bool) -> Self {
        self.disabled = Some(off);
        self
    }

    /// A day must be picked before the form submits.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// The browser's own `<input type="date">` instead of the calendar.
    pub fn native(mut self) -> Self {
        self.native = true;
        self
    }
}

impl Render for DatePicker<'_> {
    fn render(&self) -> Markup {
        let ui = self.ui;
        let value = self.value.or_else(|| ui.param(self.name)).unwrap_or("");
        if self.native {
            let input = ui
                .input(self.name, self.label)
                .date(self.min.unwrap_or(""), self.max.unwrap_or(""));
            let input = if self.required {
                input.required()
            } else {
                input
            };
            return input.value(value).render();
        }
        let mut calendar = ui.calendar(self.name).radio().value(value);
        if let Some(m) = self.min {
            calendar = calendar.min(m);
        }
        if let Some(m) = self.max {
            calendar = calendar.max(m);
        }
        if let Some(off) = self.disabled {
            calendar = calendar.disabled(off);
        }
        if self.required {
            calendar = calendar.required();
        }
        let id = format!("f-{}", self.name);
        let panel = format!("{id}-calendar");
        // A month link comes back with the calendar in the page: a popover cannot arrive open.
        let browsing = ui.param(&format!("month.{}", self.name)).is_some();
        let popover = ui.has(Cap::Popover) && !browsing;
        let anchor = ui.has(Cap::Anchor);
        let shown = Date::parse(value).map(|d| d.long());
        let face = html! {
            (Icon::Calendar)
            span class=[shown.is_none().then_some("nojs-date-picker-empty")] {
                @if let Some(s) = &shown { (s) } @else { "Pick a date" }
            }
        };
        let legend = format!("{id}-label");
        let text = html! { (self.label) @if self.required { " *" } };
        html! {
            div class="nojs-field nojs-date-picker" role=[(!popover).then_some("group")]
                aria-labelledby=[(!popover).then_some(&legend)] {
                @if popover {
                    label for=(id) { (text) }
                    div class="nojs-date-picker-anchor" style=[anchor.then(|| format!("anchor-name: --{panel}"))] {
                        (Button::new(ui.caps, self.label).id(&id).class("nojs-date-picker-trigger").popovertarget(&panel).aria_haspopup("dialog").content(face))
                        div id=(panel) popover class="nojs-date-picker-panel"
                            style=[anchor.then(|| format!("position-anchor: --{panel}; position-area: bottom span-right"))] {
                            (calendar)
                        }
                    }
                } @else {
                    span id=(legend) class="nojs-date-picker-label" { (text) }
                    (calendar)
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`]. shadcn DatePicker: an outline
/// button with a calendar icon, 15rem wide, the Calendar in a PopoverContent (p-0, no border).
pub const CSS: &str = r#"
.nojs-date-picker-anchor { position: relative; }
.nojs-date-picker-label { font-size: 0.875rem; line-height: 1; font-weight: 500; }
.nojs-date-picker > .nojs-calendar { justify-self: start; }
.nojs-date-picker-trigger { width: 15rem; justify-content: flex-start; font-weight: 400; }
.nojs-date-picker-empty { color: var(--nojs-muted); }
.nojs-date-picker-panel { margin: 0; margin-top: 4px; padding: 0; border: 0; background: none; overflow: visible; }
.nojs-date-picker-panel > .nojs-calendar { box-shadow: var(--nojs-shadow-lg); }
"#;
