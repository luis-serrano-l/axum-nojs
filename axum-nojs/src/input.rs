//! # Input
//!
//! One labelled field on its own: the label, the control, help text, a character counter and
//! the server's error, tied together with `aria-describedby`. `ui.checkbox`, `ui.switch` and
//! `ui.radio_group` are the same for choices. [`crate::form`] renders its fields with this
//! file, so a field looks and behaves the same inside a form built with `ui.form` and in a
//! form a route writes by hand.
//!
//! **Platform features:** constraint validation (`required`, `pattern`, `min`, `max`,
//! `maxlength`, `type=email`, baseline 2015; `type=date`/`time`, Chrome 20, Firefox 57,
//! Safari 14.1); `:user-invalid` (baseline 2023) so a field is not red before it is touched;
//! `<output for>` counts characters; `role="switch"` on a checkbox (ARIA 1.2) drawn as a
//! track and thumb with `appearance: none`; `<fieldset>` + `<legend>` for a radio group.
//!
//! **What it does not do without script:** keep the counter live while typing (the
//! enhancement script does; without it the counter shows the length the server rendered).
//!
//! **Fallback:** none needed: every control is a native one, and the switch is still a
//! checkbox where `appearance: none` is not supported.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::default();
//! let name = ui.input("name", "Name").render().into_string();
//! assert!(name.contains(r#"<label for="f-name">Name</label>"#) && name.contains(r#"type="text""#));
//! // Setters as on form fields, and one per input type.
//! let email = ui.input("email", "Email").email().required().value("ada@x.org")
//!     .help("We never share it.").error("Already taken.");
//! let email = email.render().into_string();
//! assert!(email.contains(r#"aria-describedby="f-email-help f-email-error""#));
//! assert!(email.contains(r#"aria-invalid="true""#) && email.contains(r#"type="email""#));
//! let on = ui.switch("digest", "Weekly digest").checked(true).render().into_string();
//! assert!(on.contains(r#"role="switch""#) && on.contains("checked"));
//! let plan = ui.radio_group("plan", "Plan").option("free", "Free").option("pro", "Pro").value("pro");
//! let plan = plan.render().into_string();
//! assert!(plan.contains("<legend>Plan</legend>") && plan.contains(r#"value="pro" checked"#));
//! ```

use maud::{Markup, Render, html};

use crate::Ui;

/// Input type of a field.
#[derive(Clone, Debug)]
pub(crate) enum FieldKind<'a> {
    Text,
    Search,
    Email,
    Password,
    Number { min: Option<i64>, max: Option<i64> },
    Pattern { pattern: &'a str, hint: &'a str },
    Textarea { rows: u8 },
    File { accept: &'a str, multiple: bool },
    Date { min: &'a str, max: &'a str },
    Time { min: &'a str, max: &'a str },
    Select(Vec<&'a str>),
    Checkbox,
    Switch,
    Hidden,
}

/// One field with its current value and server-side error: what [`Ui::input`] builds and
/// what a [`crate::form::Form`] holds a list of.
#[derive(Clone, Debug)]
pub(crate) struct Field<'a> {
    pub(crate) name: &'a str,
    pub(crate) label: &'a str,
    pub(crate) kind: FieldKind<'a>,
    pub(crate) value: &'a str,
    pub(crate) error: Option<&'a str>,
    pub(crate) required: bool,
    pub(crate) help: Option<&'a str>,
    pub(crate) maxlength: Option<usize>,
    pub(crate) placeholder: Option<&'a str>,
    pub(crate) id: Option<&'a str>,
    pub(crate) extra: Extra<'a>,
}

/// Attributes only some fields need, each set by the `Input` setter of the same name.
#[derive(Clone, Debug, Default)]
pub(crate) struct Extra<'a> {
    hide_label: bool,
    list: Option<&'a str>,
    autocomplete: Option<&'a str>,
    autofocus: bool,
    inputmode: Option<&'a str>,
    step: Option<i64>,
    aria_controls: Option<&'a str>,
    class: Option<&'a str>,
    form: Option<&'a str>,
}

impl<'a> Field<'a> {
    pub(crate) fn new(name: &'a str, label: &'a str, kind: FieldKind<'a>) -> Self {
        Field {
            name,
            label,
            kind,
            value: "",
            error: None,
            required: false,
            help: None,
            maxlength: None,
            placeholder: None,
            id: None,
            extra: Extra::default(),
        }
    }

    /// Whether a person sees it (a review lists only these).
    pub(crate) fn shown(&self) -> bool {
        !matches!(self.kind, FieldKind::Hidden)
    }
}

/// A labelled field, made by [`Ui::input`], [`Ui::checkbox`] or [`Ui::switch`].
///
/// **Setters.** Values and items: `.number(..)`, `.pattern(..)`, `.textarea(..)`, `.file(..)`,
/// `.date(..)`, `.time(..)`, `.help(..)`, `.maxlength(..)`, `.value(..)`, `.error(..)`,
/// `.placeholder(..)`, `.list(..)`, `.autocomplete(..)`, `.inputmode(..)`, `.step(..)`,
/// `.aria_controls(..)`, `.form(..)`, `.class(..)`, `.id(..)`; switches: `.email()`,
/// `.password()`, `.multiple()`, `.required()`, `.search()`, `.hide_label()`, `.autofocus()`;
/// from a condition: `.checked(bool)`.
#[derive(Clone, Debug)]
pub struct Input<'a>(Field<'a>);

impl Ui {
    /// A single-line text field named `name` under `label`; the setters change its type.
    pub fn input<'a>(&self, name: &'a str, label: &'a str) -> Input<'a> {
        Input(Field::new(name, label, FieldKind::Text))
    }

    /// A checkbox posting `true` when ticked and nothing when not (so a `bool` with
    /// `#[serde(default)]` reads it), its label beside it.
    pub fn checkbox<'a>(&self, name: &'a str, label: &'a str) -> Input<'a> {
        Input(Field::new(name, label, FieldKind::Checkbox))
    }

    /// A checkbox drawn as an on/off switch, with `role="switch"`; posts like
    /// [`Ui::checkbox`].
    pub fn switch<'a>(&self, name: &'a str, label: &'a str) -> Input<'a> {
        Input(Field::new(name, label, FieldKind::Switch))
    }
}

impl<'a> Input<'a> {
    /// A number field whose bounds a component may not have (the counter's).
    pub(crate) fn number_within(
        name: &'a str,
        label: &'a str,
        min: Option<i64>,
        max: Option<i64>,
    ) -> Self {
        Input(Field::new(name, label, FieldKind::Number { min, max }))
    }

    /// A text box with no visible label: `label` is its `aria-label`.
    pub(crate) fn text_box(name: &'a str, label: &'a str, value: &'a str) -> Self {
        Input(Field::new(name, label, FieldKind::Text))
            .hide_label()
            .value(value)
    }

    /// A search box with no visible label: `label` is its `aria-label`, `value` the query.
    pub(crate) fn search_box(name: &'a str, label: &'a str, value: &'a str) -> Self {
        Input(Field::new(name, label, FieldKind::Search))
            .hide_label()
            .value(value)
    }

    fn kind(mut self, kind: FieldKind<'a>) -> Self {
        self.0.kind = kind;
        self
    }

    /// `type="email"`: the browser checks the shape.
    pub fn email(self) -> Self {
        self.kind(FieldKind::Email)
    }

    /// `type="password"`: the value is never echoed back into the page.
    pub fn password(self) -> Self {
        self.kind(FieldKind::Password)
    }

    /// A whole number from `min` to `max`, inclusive.
    pub fn number(self, min: i64, max: i64) -> Self {
        self.kind(FieldKind::Number {
            min: Some(min),
            max: Some(max),
        })
    }

    /// Text that must match `pattern`; `hint` explains the rule under the field and as the
    /// input's `title`.
    pub fn pattern(self, pattern: &'a str, hint: &'a str) -> Self {
        self.kind(FieldKind::Pattern { pattern, hint })
    }

    /// Multi-line text, `rows` high; it grows with its content where `field-sizing` works.
    pub fn textarea(self, rows: u8) -> Self {
        self.kind(FieldKind::Textarea { rows })
    }

    /// A file picker; `accept` lists MIME types or extensions, empty for any. The form
    /// around it needs `enctype="multipart/form-data"`.
    pub fn file(self, accept: &'a str) -> Self {
        self.kind(FieldKind::File {
            accept,
            multiple: false,
        })
    }

    /// `type="date"`, bounds as `YYYY-MM-DD`; an empty bound is left out.
    pub fn date(self, min: &'a str, max: &'a str) -> Self {
        self.kind(FieldKind::Date { min, max })
    }

    /// `type="time"`, bounds as `HH:MM`; an empty bound is left out.
    pub fn time(self, min: &'a str, max: &'a str) -> Self {
        self.kind(FieldKind::Time { min, max })
    }

    /// The file picker takes several files.
    pub fn multiple(mut self) -> Self {
        if let FieldKind::File { multiple, .. } = &mut self.0.kind {
            *multiple = true;
        }
        self
    }

    /// The `required` attribute, and a `*` after the label.
    pub fn required(mut self) -> Self {
        self.0.required = true;
        self
    }

    /// Help text under the field.
    pub fn help(mut self, help: &'a str) -> Self {
        self.0.help = Some(help);
        self
    }

    /// `maxlength`, counted in an `<output>` under the field.
    pub fn maxlength(mut self, max: usize) -> Self {
        self.0.maxlength = Some(max);
        self
    }

    /// The current value (ignored for files and passwords).
    pub fn value(mut self, value: &'a str) -> Self {
        self.0.value = value;
        self
    }

    /// Tick the checkbox or switch.
    pub fn checked(mut self, checked: bool) -> Self {
        self.0.value = if checked { "true" } else { "" };
        self
    }

    /// A server message under the field; the control gets `aria-invalid`. An empty message
    /// is no error, so a route can pass `if bad { ".." } else { "" }`.
    pub fn error(mut self, message: &'a str) -> Self {
        self.0.error = (!message.is_empty()).then_some(message);
        self
    }

    /// Placeholder text.
    pub fn placeholder(mut self, placeholder: &'a str) -> Self {
        self.0.placeholder = Some(placeholder);
        self
    }

    /// `type="search"`: a filter box, with the browser's clear button.
    pub fn search(self) -> Self {
        self.kind(FieldKind::Search)
    }

    /// Only the control, the label kept as its `aria-label`: a filter box in a toolbar, a
    /// number beside buttons. Help, counter and error are left out.
    pub fn hide_label(mut self) -> Self {
        self.0.extra.hide_label = true;
        self
    }

    /// `list`: the id of a `<datalist>` of suggestions.
    pub fn list(mut self, id: &'a str) -> Self {
        self.0.extra.list = Some(id);
        self
    }

    /// `autocomplete` (`"off"`, `"email"`, `"new-password"`).
    pub fn autocomplete(mut self, value: &'a str) -> Self {
        self.0.extra.autocomplete = Some(value);
        self
    }

    /// `autofocus`: focused when the page or the popover around it opens.
    pub fn autofocus(mut self) -> Self {
        self.0.extra.autofocus = true;
        self
    }

    /// `inputmode` (`"numeric"`), the on-screen keyboard to show.
    pub fn inputmode(mut self, mode: &'a str) -> Self {
        self.0.extra.inputmode = Some(mode);
        self
    }

    /// `step` of a number field.
    pub fn step(mut self, step: i64) -> Self {
        self.0.extra.step = Some(step);
        self
    }

    /// `aria-controls`: the id of the region this field updates (a results list).
    pub fn aria_controls(mut self, id: &'a str) -> Self {
        self.0.extra.aria_controls = Some(id);
        self
    }

    /// `form`: the id of the form this control posts with, when it cannot sit inside it (a
    /// table row being edited).
    pub fn form(mut self, id: &'a str) -> Self {
        self.0.extra.form = Some(id);
        self
    }

    /// A class on the control, for a component's part name.
    pub fn class(mut self, class: &'a str) -> Self {
        self.0.extra.class = Some(class);
        self
    }

    /// The control's id, `f-<name>` by default.
    pub fn id(mut self, id: &'a str) -> Self {
        self.0.id = Some(id);
        self
    }
}

impl Render for Input<'_> {
    fn render(&self) -> Markup {
        self.0.render()
    }
}

impl Render for Field<'_> {
    fn render(&self) -> Markup {
        let f = self;
        let id = f.id.map_or_else(|| format!("f-{}", f.name), str::to_string);
        let help = f.help.or(match f.kind {
            FieldKind::Pattern { hint, .. } => Some(hint),
            _ => None,
        });
        let ids = [
            help.map(|_| format!("{id}-help")),
            f.maxlength.map(|_| format!("{id}-count")),
            f.error.map(|_| format!("{id}-error")),
        ];
        let described: Vec<&str> = ids.iter().flatten().map(String::as_str).collect();
        let described = (!described.is_empty()).then(|| described.join(" "));
        let bound = |s: &str| (!s.is_empty()).then(|| s.to_string());
        let (kind, min, max, pattern, accept, multiple) = match f.kind {
            FieldKind::Text | FieldKind::Textarea { .. } | FieldKind::Select(_) => {
                ("text", None, None, None, None, false)
            }
            FieldKind::Search => ("search", None, None, None, None, false),
            FieldKind::Email => ("email", None, None, None, None, false),
            FieldKind::Password => ("password", None, None, None, None, false),
            FieldKind::Number { min, max } => (
                "number",
                min.map(|m| m.to_string()),
                max.map(|m| m.to_string()),
                None,
                None,
                false,
            ),
            FieldKind::Pattern { pattern, .. } => ("text", None, None, Some(pattern), None, false),
            FieldKind::File { accept, multiple } => (
                "file",
                None,
                None,
                None,
                (!accept.is_empty()).then_some(accept),
                multiple,
            ),
            FieldKind::Date { min, max } => ("date", bound(min), bound(max), None, None, false),
            FieldKind::Time { min, max } => ("time", bound(min), bound(max), None, None, false),
            FieldKind::Checkbox | FieldKind::Switch | FieldKind::Hidden => {
                ("", None, None, None, None, false)
            }
        };
        let invalid = f.error.map(|_| "true");
        if let FieldKind::Hidden = f.kind {
            return html! { input type="hidden" name=(f.name) value=(f.value); };
        }
        if let FieldKind::Checkbox | FieldKind::Switch = f.kind {
            let checked = matches!(f.value, "true" | "on" | "1");
            let switch = matches!(f.kind, FieldKind::Switch);
            return html! {
                div class={ "nojs-field nojs-field-check" @if switch { " nojs-field-switch" } } {
                    label for=(id) {
                        input id=(id) name=(f.name) type="checkbox" value="true" checked[checked] required[f.required]
                            role=[switch.then_some("switch")] class=[switch.then_some("nojs-switch")]
                            aria-invalid=[invalid] aria-describedby=[described.as_deref()];
                        " " (f.label)
                    }
                    @if let Some(h) = help { small id={ (id) "-help" } class="nojs-field-help" { (h) } }
                    @if let Some(e) = f.error { p id={ (id) "-error" } class="nojs-error" role="alert" { (e) } }
                }
            };
        }
        let echo = !matches!(f.kind, FieldKind::File { .. } | FieldKind::Password);
        let x = &f.extra;
        let control = html! {
            input id=(id) class=[x.class] name=(f.name) type=(kind) value=[echo.then_some(f.value)]
                required[f.required] min=[min] max=[max] step=[x.step] pattern=[pattern] title=[pattern.and(help)]
                accept=[accept] multiple[multiple] maxlength=[f.maxlength] placeholder=[f.placeholder]
                form=[x.form] list=[x.list] autocomplete=[x.autocomplete] autofocus[x.autofocus] inputmode=[x.inputmode]
                aria-label=[x.hide_label.then_some(f.label)] aria-controls=[x.aria_controls]
                aria-invalid=[invalid] aria-describedby=[described.as_deref()];
        };
        if x.hide_label && !matches!(f.kind, FieldKind::Textarea { .. } | FieldKind::Select(_)) {
            return control;
        }
        html! {
            div class="nojs-field" {
                label for=(id) { (f.label) @if f.required { " *" } }
                @if let FieldKind::Textarea { rows } = f.kind {
                    textarea id=(id) name=(f.name) rows=(rows) required[f.required] maxlength=[f.maxlength] placeholder=[f.placeholder]
                        aria-invalid=[invalid] aria-describedby=[described.as_deref()] { (f.value) }
                } @else if let FieldKind::Select(options) = &f.kind {
                    select id=(id) name=(f.name) required[f.required] aria-invalid=[invalid] aria-describedby=[described.as_deref()] {
                        @for o in options { option value=(o) selected[*o == f.value] { (o) } }
                    }
                } @else {
                    (control)
                }
                @if let Some(h) = help { small id={ (id) "-help" } class="nojs-field-help" { (h) } }
                @if let Some(max) = f.maxlength {
                    output id={ (id) "-count" } for=(id) class="nojs-field-count" { (f.value.chars().count()) " / " (max) }
                }
                @if let Some(e) = f.error { p id={ (id) "-error" } class="nojs-error" role="alert" { (e) } }
            }
        }
    }
}

/// A set of radio buttons under a legend, made by [`Ui::radio_group`]. Add choices with
/// [`RadioGroup::option`].
///
/// **Setters.** Values and items: `.option(..)`, `.value(..)`, `.help(..)`, `.error(..)`;
/// switches: `.required()`.
#[derive(Clone, Debug)]
pub struct RadioGroup<'a> {
    name: &'a str,
    legend: &'a str,
    options: Vec<(&'a str, &'a str)>,
    value: &'a str,
    required: bool,
    help: Option<&'a str>,
    error: Option<&'a str>,
}

impl Ui {
    /// Radio buttons named `name` in a `<fieldset>` whose `<legend>` is `legend`.
    pub fn radio_group<'a>(&self, name: &'a str, legend: &'a str) -> RadioGroup<'a> {
        RadioGroup {
            name,
            legend,
            options: Vec::new(),
            value: "",
            required: false,
            help: None,
            error: None,
        }
    }
}

impl<'a> RadioGroup<'a> {
    /// One choice posting `value`, labelled `label`.
    pub fn option(mut self, value: &'a str, label: &'a str) -> Self {
        self.options.push((value, label));
        self
    }

    /// The value of the choice that is selected.
    pub fn value(mut self, value: &'a str) -> Self {
        self.value = value;
        self
    }

    /// One choice must be picked before the form submits.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Help text under the choices.
    pub fn help(mut self, help: &'a str) -> Self {
        self.help = Some(help);
        self
    }

    /// A server message under the choices; the group gets `aria-invalid`. An empty message
    /// is no error.
    pub fn error(mut self, message: &'a str) -> Self {
        self.error = (!message.is_empty()).then_some(message);
        self
    }
}

impl Render for RadioGroup<'_> {
    fn render(&self) -> Markup {
        let id = format!("f-{}", self.name);
        let ids = [
            self.help.map(|_| format!("{id}-help")),
            self.error.map(|_| format!("{id}-error")),
        ];
        let described: Vec<&str> = ids.iter().flatten().map(String::as_str).collect();
        let described = (!described.is_empty()).then(|| described.join(" "));
        html! {
            fieldset class="nojs-field nojs-radio-group" aria-describedby=[described.as_deref()]
                aria-invalid=[self.error.map(|_| "true")] {
                legend { (self.legend) @if self.required { " *" } }
                @for (i, (value, label)) in self.options.iter().enumerate() {
                    @let oid = format!("{id}-{i}");
                    label for=(oid) {
                        input id=(oid) type="radio" name=(self.name) value=(value)
                            checked[*value == self.value] required[self.required && i == 0];
                        " " (label)
                    }
                }
                @if let Some(h) = self.help { small id={ (id) "-help" } class="nojs-field-help" { (h) } }
                @if let Some(e) = self.error { p id={ (id) "-error" } class="nojs-error" role="alert" { (e) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
/* Native controls, shadcn sizes: 2.25rem tall, px-3, a 1px --nojs-input border, shadow-xs.
   These bare element rules are the one place inputs are styled: a hand-written field and a
   component's control look the same. */
input, select, textarea { font: inherit; font-size: 0.875rem; line-height: 1.25rem; color: inherit; }
label { font-weight: 500; }
input, select, textarea {
  min-height: 2.25rem; padding: 0.375rem 0.75rem; min-width: 0;
  background: transparent; border: 1px solid var(--nojs-input); border-radius: var(--nojs-radius-sm);
  box-shadow: var(--nojs-shadow-xs); transition: border-color 0.15s, box-shadow 0.15s;
}
textarea { min-height: 4rem; }
input::placeholder, textarea::placeholder { color: var(--nojs-muted); }
/* Native select: no OS chrome, a chevron drawn from two gradients in the muted colour. */
select {
  appearance: none; padding-right: 2rem;
  background-image: linear-gradient(45deg, transparent 50%, var(--nojs-muted) 50%), linear-gradient(135deg, var(--nojs-muted) 50%, transparent 50%);
  background-position: right 1rem center, right 0.75rem center; background-size: 0.25rem 0.25rem; background-repeat: no-repeat;
}
select[multiple], select[size] { padding-right: 0.75rem; background-image: none; }
input:is([type=checkbox], [type=radio]) { width: 1rem; height: 1rem; min-height: 0; padding: 0; margin: 0; accent-color: var(--nojs-primary); vertical-align: -0.15em; }
input[type=range] { min-height: 0; padding: 0; border: 0; box-shadow: none; accent-color: var(--nojs-primary); }
input[type=color] { padding: 0.25rem; }
input[type=file] { padding-block: 0.25rem; }
input::file-selector-button { font: inherit; font-weight: 500; color: var(--nojs-fg); background: transparent; border: 0; padding: 0 0.5rem 0 0; }
:is(input, select, textarea):focus-visible { border-color: var(--nojs-ring); }
:is(input, select, textarea):disabled { opacity: 0.5; cursor: not-allowed; }
.nojs-field { display: grid; gap: 0.5rem; }
.nojs-field label { font-size: 0.875rem; line-height: 1; font-weight: 500; }
.nojs-field-check label, .nojs-radio-group label { display: flex; align-items: center; gap: 0.5rem; }
/* :where keeps this at one class, so a component inside a field (colour, range) sizes itself. */
.nojs-field :where(input:not([type=file], [type=color], [type=range], [type=checkbox], [type=radio]), textarea) { width: 100%; box-sizing: border-box; }
.nojs-field textarea { resize: vertical; field-sizing: content; min-height: 3lh; max-height: 20lh; font: inherit; }
.nojs-field-help { color: var(--nojs-muted); font-size: 0.875rem; }
.nojs-field-count { justify-self: end; color: var(--nojs-muted); font-size: 0.75rem; font-variant-numeric: tabular-nums; }
.nojs-field :is(input, textarea):user-invalid, .nojs-field [aria-invalid=true] { border-color: var(--nojs-danger); }
.nojs-field [aria-invalid=true] ~ label, .nojs-field:has([aria-invalid=true]) > label { color: var(--nojs-danger); }
.nojs-error { color: var(--nojs-danger); margin: 0; font-size: 0.875rem; }
.nojs-radio-group { margin: 0; padding: 0; border: 0; gap: 0.75rem; }
.nojs-radio-group legend { padding: 0; margin-bottom: 0.75rem; font-size: 0.875rem; font-weight: 500; }
.nojs-radio-group[aria-invalid=true] legend { color: var(--nojs-danger); }
/* shadcn Switch: a 2rem by 1.15rem pill in --nojs-input, the primary colour when on, a
   thumb in the page background (on-primary when on) that slides across. */
input.nojs-switch {
  appearance: none; position: relative; flex: none; width: 2rem; height: 1.15rem; margin: 0;
  border: 1px solid transparent; border-radius: 9999px; background: var(--nojs-input);
  box-shadow: var(--nojs-shadow-xs); cursor: pointer; transition: background-color 0.15s;
}
input.nojs-switch::before {
  content: ""; position: absolute; top: 50%; left: 1px; width: 1rem; height: 1rem;
  border-radius: 50%; background: var(--nojs-bg); translate: 0 -50%; transition: translate 0.15s;
}
input.nojs-switch:checked { background: var(--nojs-primary); }
input.nojs-switch:checked::before { translate: calc(2rem - 1rem - 4px) -50%; background: var(--nojs-on-primary); }
@media (prefers-reduced-motion: reduce) { input.nojs-switch, input.nojs-switch::before { transition: none; } }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passwords_and_files_are_never_echoed() {
        let ui = Ui::default();
        let p = ui
            .input("pw", "Password")
            .password()
            .value("hunter2")
            .render()
            .into_string();
        assert!(p.contains(r#"type="password""#) && !p.contains("hunter2"));
        let f = ui
            .input("cv", "CV")
            .file(".pdf")
            .multiple()
            .value("x")
            .render()
            .into_string();
        assert!(f.contains(r#"accept=".pdf" multiple"#) && !f.contains(r#"value="x""#));
    }

    #[test]
    fn a_radio_group_needs_one_choice_and_describes_itself() {
        let ui = Ui::default();
        let g = ui
            .radio_group("size", "Size")
            .option("s", "Small")
            .option("m", "Medium")
            .required()
            .error("Pick one.")
            .render()
            .into_string();
        assert!(
            g.contains(r#"aria-describedby="f-size-error""#)
                && g.contains(r#"aria-invalid="true""#)
        );
        assert_eq!(
            g.matches("required").count(),
            1,
            "one required radio covers the group"
        );
        assert!(
            g.contains(
                r#"<label for="f-size-1"><input id="f-size-1" type="radio" name="size" value="m">"#
            ),
            "{g}"
        );
    }

    #[test]
    fn an_id_overrides_the_default() {
        let m = Ui::default()
            .input("q", "Search")
            .id("search")
            .render()
            .into_string();
        assert!(m.contains(r#"<label for="search">"#) && m.contains(r#"id="search""#));
    }
}
