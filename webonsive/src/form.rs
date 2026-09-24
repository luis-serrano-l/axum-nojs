//! # Form
//!
//! A validated form: the browser blocks bad input before submit, the server checks again and
//! re-renders with messages. Fields come in groups with legends, carry help text, count their
//! characters, take files, dates, times and bounded numbers, and lay out stacked (label above)
//! or inline (label beside). No script needed.
//!
//! **Platform features:**
//! - Constraint validation attributes `required`, `pattern`, `min`, `max`, `maxlength`,
//!   `type=email` (baseline 2015); `type=date` and `type=time` with `min`/`max` (Chrome 20,
//!   Firefox 57, Safari 14.1).
//! - `:user-invalid` / `:user-valid` (baseline 2023): styles only after the user has interacted,
//!   so fields are not red on first paint.
//! - `<fieldset>` + `<legend>` per [`FieldGroup`]; help text and the error are tied to the
//!   field with `aria-describedby`.
//! - `<output>` counts characters for a field with a `maxlength`: the server renders the count
//!   of the value it has, the enhancement script keeps it live while typing.
//! - `<input type=file accept>` (baseline 2015); any file field makes the form
//!   `enctype="multipart/form-data"`.
//! - `field-sizing` (Chrome 123, not yet in Firefox or Safari): a textarea grows with its
//!   content.
//! - Post/Redirect/Get for success; on error the server re-renders the form with values and
//!   messages.
//!
//! **What it does not do without script:** validate against the server as you type, or warn
//! about unsaved changes on leaving the page.
//!
//! **Fallback:** without `field-sizing` a textarea keeps its `rows` and can be resized by
//! hand. Without the script the counter shows the length of the last submitted value and
//! `maxlength` still stops input at the limit. `Caps` is unused.
//!
//! **Enhanced:** the form is a swap root, so with the [`crate::enhance`] script a submit
//! replaces only the form (errors included) and a successful redirect swaps in the result.
//!
//! **Finding:** custom cross-field rules (password confirmation, async uniqueness) only run
//! on the server round trip, and warning about unsaved changes when leaving needs script.
//!
//! ```rust
//! use webonsive::{Caps, form, form_with, Field, FieldKind, form::{FieldGroup, FormLayout, FormOptions}};
//! let fields = [Field::new("email", "Email", FieldKind::Email).required()];
//! let m = form(&Caps::all(), "/form", &[FieldGroup::plain(&fields)]);
//!
//! let about = [
//!     Field::new("bio", "Bio", FieldKind::Textarea { rows: 3 }).maxlength(280).value("Hi").help("Shown on your profile."),
//!     Field::new("avatar", "Avatar", FieldKind::File { accept: "image/png,image/jpeg", multiple: false }),
//!     Field::new("born", "Born", FieldKind::Date { min: "1900-01-01", max: "2026-12-31" }),
//! ];
//! let m = form_with(&Caps::all(), "/profile", &[FieldGroup::new("About you", &about)],
//!              FormOptions::default().submit("Save profile").layout(FormLayout::Inline));
//! let html = m.into_string();
//! assert!(html.contains("enctype=\"multipart/form-data\""));
//! assert!(html.contains("<legend>About you</legend>"));
//! assert!(html.contains(">2 / 280</output>"));
//! assert!(html.contains("accept=\"image/png,image/jpeg\""));
//! ```

use maud::{Markup, html};

use crate::{Caps, enhance};

/// Input type for a [`Field`].
#[derive(Clone, Copy, Debug)]
pub enum FieldKind {
    /// Single-line text.
    Text,
    /// `type="email"`: the browser checks the shape.
    Email,
    /// Integer between `min` and `max`, inclusive.
    Number {
        /// Smallest accepted value.
        min: i64,
        /// Largest accepted value.
        max: i64,
    },
    /// Free text that must match `pattern`; `hint` explains the rule to people.
    Pattern {
        /// HTML `pattern` attribute (a regular expression matched against the whole value).
        pattern: &'static str,
        /// Shown under the field (unless the field has its own help) and as the input's `title`.
        hint: &'static str,
    },
    /// Multi-line text; grows with its content where `field-sizing` is supported.
    Textarea {
        /// Visible rows without `field-sizing`, and the starting height with it.
        rows: u8,
    },
    /// File picker; `accept` lists MIME types or extensions (`image/*,.pdf`), empty for any.
    File {
        /// The `accept` attribute.
        accept: &'static str,
        /// Allow several files.
        multiple: bool,
    },
    /// `type="date"`, bounds as `YYYY-MM-DD`; an empty bound is left out.
    Date {
        /// Earliest date.
        min: &'static str,
        /// Latest date.
        max: &'static str,
    },
    /// `type="time"`, bounds as `HH:MM`; an empty bound is left out.
    Time {
        /// Earliest time.
        min: &'static str,
        /// Latest time.
        max: &'static str,
    },
    /// A checkbox posting `true` when ticked and nothing when not (so a `bool` with
    /// `#[serde(default)]` reads it); ticked when the value is `true`, `on` or `1`.
    Checkbox,
    /// `type="hidden"`: carried with the post, not shown; the label is unused.
    Hidden,
}

/// One form field with its current value and server-side error.
#[derive(Clone, Debug)]
pub struct Field<'a> {
    /// Form field name, also used for the input id (`f-<name>`).
    pub name: &'a str,
    /// Visible label.
    pub label: &'a str,
    /// Input type and its constraints.
    pub kind: FieldKind,
    /// Current value, re-rendered after a failed submit (ignored for files).
    pub value: &'a str,
    /// Server-side error message for this field.
    pub error: Option<&'a str>,
    /// Adds the `required` attribute and a `*` to the label.
    pub required: bool,
    /// Help text under the field.
    pub help: Option<&'a str>,
    /// `maxlength`, shown as a character counter in an `<output>`.
    pub maxlength: Option<usize>,
}

impl<'a> Field<'a> {
    /// An empty, optional field.
    pub const fn new(name: &'a str, label: &'a str, kind: FieldKind) -> Self {
        Field { name, label, kind, value: "", error: None, required: false, help: None, maxlength: None }
    }
    /// Current value.
    pub const fn value(mut self, value: &'a str) -> Self {
        self.value = value;
        self
    }
    /// Server-side error message.
    pub const fn error(mut self, message: &'a str) -> Self {
        self.error = Some(message);
        self
    }
    /// Required field.
    pub const fn required(mut self) -> Self {
        self.required = true;
        self
    }
    /// Help text under the field.
    pub const fn help(mut self, help: &'a str) -> Self {
        self.help = Some(help);
        self
    }
    /// `maxlength` with a character counter.
    pub const fn maxlength(mut self, max: usize) -> Self {
        self.maxlength = Some(max);
        self
    }
}

/// Fields under one `<fieldset>` and `<legend>`, or with no box around them.
#[derive(Clone, Copy, Debug)]
pub struct FieldGroup<'a> {
    /// The `<legend>`; `None` renders the fields without a fieldset.
    pub legend: Option<&'a str>,
    /// The fields, in order.
    pub fields: &'a [Field<'a>],
}

impl<'a> FieldGroup<'a> {
    /// A fieldset with a legend.
    pub fn new(legend: &'a str, fields: &'a [Field<'a>]) -> Self {
        FieldGroup { legend: Some(legend), fields }
    }
    /// Fields with no fieldset around them.
    pub fn plain(fields: &'a [Field<'a>]) -> Self {
        FieldGroup { legend: None, fields }
    }
}

/// Fields with no fieldset: `(&fields).into()`.
impl<'a> From<&'a [Field<'a>]> for FieldGroup<'a> {
    fn from(fields: &'a [Field<'a>]) -> Self {
        FieldGroup::plain(fields)
    }
}

/// `("Account", &fields)`: a fieldset and its legend.
impl<'a> From<(&'a str, &'a [Field<'a>])> for FieldGroup<'a> {
    fn from((legend, fields): (&'a str, &'a [Field<'a>])) -> Self {
        FieldGroup::new(legend, fields)
    }
}

/// Where labels sit.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FormLayout {
    /// Label above the field.
    #[default]
    Stacked,
    /// Label beside the field on screens wider than 40rem, above it on narrower ones.
    Inline,
}

/// Options for [`form`]; `Default::default()` is a stacked form with a "Submit" button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormOptions<'a> {
    /// Label of the submit button.
    pub submit: &'a str,
    /// Where labels sit.
    pub layout: FormLayout,
    /// Submitted values by field name, as a form post parses them: each field without its
    /// own `value` shows the one named after it.
    pub values: &'a [(String, String)],
    /// Server messages `(field name, message)`: each field without its own `error` shows the
    /// one named after it.
    pub errors: &'a [(&'a str, &'a str)],
    /// What the swap root's id is built from, when two forms on a page post to the same
    /// `action` (one per tab, say); `None` uses the action.
    pub id: Option<&'a str>,
}

impl Default for FormOptions<'_> {
    fn default() -> Self {
        FormOptions { submit: "Submit", layout: FormLayout::Stacked, values: &[], errors: &[], id: None }
    }
}

impl<'a> FormOptions<'a> {
    /// Label of the submit button.
    pub fn submit(mut self, submit: &'a str) -> Self {
        self.submit = submit;
        self
    }
    /// Stacked or inline labels.
    pub fn layout(mut self, layout: FormLayout) -> Self {
        self.layout = layout;
        self
    }
    /// Fill each field's value from the submitted pairs, by name.
    pub fn values(mut self, values: &'a [(String, String)]) -> Self {
        self.values = values;
        self
    }
    /// Attach each server message to the field it names.
    pub fn errors(mut self, errors: &'a [(&'a str, &'a str)]) -> Self {
        self.errors = errors;
        self
    }
    /// Tell this form apart from another posting to the same action.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }
}

/// A stacked form with the default submit button.
/// [`form_with`] takes the options.
pub fn form(caps: &Caps, action: &str, groups: &[FieldGroup<'_>]) -> Markup {
    form_with(caps, action, groups, Default::default())
}

/// Render `groups` as a POST form to `action`.
pub fn form_with(_caps: &Caps, action: &str, groups: &[FieldGroup<'_>], options: FormOptions) -> Markup {
    let FormOptions { submit, layout, id, .. } = options;
    let multipart = groups.iter().flat_map(|g| g.fields).any(|f| matches!(f.kind, FieldKind::File { .. }));
    let class = match layout { FormLayout::Stacked => "wo-form", FormLayout::Inline => "wo-form wo-form-inline" };
    html! {
        form id=(enhance::swap_id("wo-form", id.unwrap_or(action))) data-wo="swap" class=(class) method="post" action=(action)
            enctype=[multipart.then_some("multipart/form-data")] {
            (fields(groups, options))
            div class="wo-form-actions" { button type="submit" class="wo-primary" { (submit) } }
        }
    }
}

/// The groups and their fields without the `<form>` around them, for a form that is built
/// elsewhere (a wizard step, a dialog's confirm form). `values` and `errors` fill by name as
/// in [`form_with`]; `submit` and `layout` belong to the form and are not used.
pub fn fields(groups: &[FieldGroup<'_>], options: FormOptions) -> Markup {
    let filled = |f: &Field| {
        let value = if f.value.is_empty() {
            options.values.iter().find(|(n, _)| n == f.name).map_or("", |(_, v)| v.as_str())
        } else {
            f.value
        };
        let error = f.error.or_else(|| options.errors.iter().find(|(n, _)| *n == f.name).map(|(_, m)| *m));
        field(&Field { value, error, ..*f })
    };
    html! {
        @for g in groups {
            @if let Some(legend) = g.legend {
                fieldset class="wo-form-group" { legend { (legend) } @for f in g.fields { (filled(f)) } }
            } @else {
                @for f in g.fields { (filled(f)) }
            }
        }
    }
}

fn field(f: &Field) -> Markup {
    let id = format!("f-{}", f.name);
    let help = f.help.or(match f.kind { FieldKind::Pattern { hint, .. } => Some(hint), _ => None });
    let ids = [
        help.map(|_| format!("{id}-help")),
        f.maxlength.map(|_| format!("{id}-count")),
        f.error.map(|_| format!("{id}-error")),
    ];
    let described: Vec<&str> = ids.iter().flatten().map(String::as_str).collect();
    let described = (!described.is_empty()).then(|| described.join(" "));
    let bound = |s: &'static str| (!s.is_empty()).then_some(s.to_string());
    let (kind, min, max, pattern, accept, multiple) = match f.kind {
        FieldKind::Text | FieldKind::Textarea { .. } => ("text", None, None, None, None, false),
        FieldKind::Email => ("email", None, None, None, None, false),
        FieldKind::Number { min, max } => ("number", Some(min.to_string()), Some(max.to_string()), None, None, false),
        FieldKind::Pattern { pattern, .. } => ("text", None, None, Some(pattern), None, false),
        FieldKind::File { accept, multiple } => ("file", None, None, None, (!accept.is_empty()).then_some(accept), multiple),
        FieldKind::Date { min, max } => ("date", bound(min), bound(max), None, None, false),
        FieldKind::Time { min, max } => ("time", bound(min), bound(max), None, None, false),
        FieldKind::Checkbox | FieldKind::Hidden => ("", None, None, None, None, false),
    };
    let invalid = f.error.map(|_| "true");
    if let FieldKind::Hidden = f.kind {
        return html! { input type="hidden" name=(f.name) value=(f.value); };
    }
    if let FieldKind::Checkbox = f.kind {
        let checked = matches!(f.value, "true" | "on" | "1");
        return html! {
            div class="wo-field wo-field-check" {
                label for=(id) {
                    input id=(id) name=(f.name) type="checkbox" value="true" checked[checked] required[f.required]
                        aria-invalid=[invalid] aria-describedby=[described.as_deref()];
                    " " (f.label)
                }
                @if let Some(h) = help { small id={ (id) "-help" } class="wo-field-help" { (h) } }
                @if let Some(e) = f.error { p id={ (id) "-error" } class="wo-error" role="alert" { (e) } }
            }
        };
    }
    html! {
        div class="wo-field" {
            label for=(id) { (f.label) @if f.required { " *" } }
            @if let FieldKind::Textarea { rows } = f.kind {
                textarea id=(id) name=(f.name) rows=(rows) required[f.required] maxlength=[f.maxlength]
                    aria-invalid=[invalid] aria-describedby=[described.as_deref()] { (f.value) }
            } @else {
                input id=(id) name=(f.name) type=(kind) value=[(kind != "file").then_some(f.value)]
                    required[f.required] min=[min] max=[max] pattern=[pattern] title=[pattern.and(help)]
                    accept=[accept] multiple[multiple] maxlength=[f.maxlength]
                    aria-invalid=[invalid] aria-describedby=[described.as_deref()];
            }
            @if let Some(h) = help { small id={ (id) "-help" } class="wo-field-help" { (h) } }
            @if let Some(max) = f.maxlength {
                output id={ (id) "-count" } for=(id) class="wo-field-count" { (f.value.chars().count()) " / " (max) }
            }
            @if let Some(e) = f.error { p id={ (id) "-error" } class="wo-error" role="alert" { (e) } }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-form { display: grid; gap: calc(var(--wo-space) * 2); max-width: 28rem; }
.wo-form-inline { max-width: 40rem; }
.wo-form-group { display: grid; gap: calc(var(--wo-space) * 2); margin: 0; padding: calc(var(--wo-space) * 2); border: 1px solid var(--wo-line); border-radius: var(--wo-radius); }
.wo-form-group legend { padding: 0 0.5rem; font-weight: 600; }
.wo-field { display: grid; gap: 4px; }
.wo-field label { font-weight: 600; }
.wo-field-check label { font-weight: 400; }
/* :where keeps this at one class, so a component inside a field (colour, range) sizes itself. */
.wo-field :where(input:not([type=file], [type=color], [type=range], [type=checkbox], [type=radio]), textarea) { width: 100%; box-sizing: border-box; }
.wo-field textarea { resize: vertical; field-sizing: content; min-height: 3lh; max-height: 20lh; font: inherit; }
.wo-field-help { color: var(--wo-muted); font-size: 0.875rem; }
.wo-field-count { justify-self: end; color: var(--wo-muted); font-size: 0.8rem; font-variant-numeric: tabular-nums; }
.wo-field :is(input, textarea):user-invalid, .wo-field [aria-invalid=true] { border-color: var(--wo-danger); }
.wo-field :is(input, textarea):user-valid { border-color: color-mix(in srgb, var(--wo-accent) 60%, transparent); }
.wo-error { color: var(--wo-danger); margin: 0; font-size: 0.9rem; }
@media (min-width: 40rem) {
  .wo-form-inline .wo-field { grid-template-columns: 10rem 1fr; column-gap: calc(var(--wo-space) * 2); }
  .wo-form-inline .wo-field > :not(label) { grid-column: 2; }
  /* The label sits on the input's row, centred on it; help, counter and error stack below. */
  .wo-form-inline .wo-field > label { grid-column: 1; grid-row: 1; align-self: center; }
  .wo-form-inline .wo-field-check > label { grid-column: 2; }
  .wo-form-inline .wo-form-actions { padding-left: calc(10rem + var(--wo-space) * 2); }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkbox_and_hidden_fields() {
        let fs = [Field::new("tab", "", FieldKind::Hidden).value("1"), Field::new("notify", "Email me", FieldKind::Checkbox)];
        let values = [("notify".to_string(), "true".to_string())];
        let m = fields(&[FieldGroup::plain(&fs)], FormOptions::default().values(&values)).into_string();
        assert!(m.starts_with(r#"<input type="hidden" name="tab" value="1">"#), "{m}");
        assert!(m.contains(r#"type="checkbox" value="true" checked"#) && m.contains(" Email me</label>"), "{m}");
        let off = fields(&[FieldGroup::plain(&fs)], FormOptions::default()).into_string();
        assert!(!off.contains("checked"));
    }

    #[test]
    fn values_and_errors_fill_fields_by_name() {
        let fs = [
            Field::new("name", "Name", FieldKind::Text),
            Field::new("email", "Email", FieldKind::Email).value("own@x.org"),
            Field::new("bio", "Bio", FieldKind::Textarea { rows: 2 }).error("Own message."),
        ];
        let values = [("name".to_string(), "Ada".to_string()), ("email".to_string(), "posted@x.org".to_string()), ("bio".to_string(), "Hi".to_string())];
        let errors = [("name", "Too short."), ("bio", "Posted message.")];
        let m = form_with(&Caps::NONE, "/p", &[FieldGroup::plain(&fs)], FormOptions::default().values(&values).errors(&errors)).into_string();
        assert!(m.contains(r#"name="name" type="text" value="Ada""#), "{m}");
        assert!(m.contains(r#"value="own@x.org""#) && !m.contains("posted@x.org"), "a field's own value wins");
        assert!(m.contains(">Too short.</p>") && m.contains(r#"aria-invalid="true""#));
        assert!(m.contains("Own message.") && !m.contains("Posted message."), "a field's own error wins");
        assert!(m.contains(">Hi</textarea>"));
        let bare = fields(&[FieldGroup::plain(&fs)], FormOptions::default().values(&values)).into_string();
        assert!(!bare.contains("<form") && bare.contains(r#"value="Ada""#));
    }

    #[test]
    fn help_counter_and_error_describe_the_field() {
        let fields = [Field::new("bio", "Bio", FieldKind::Textarea { rows: 2 }).value("héllo").maxlength(10).help("Short.").error("Too dull.")];
        let m = form(&Caps::NONE, "/p", &[FieldGroup::plain(&fields)]).into_string();
        assert!(m.contains("aria-describedby=\"f-bio-help f-bio-count f-bio-error\""), "{m}");
        assert!(m.contains("<output id=\"f-bio-count\" for=\"f-bio\" class=\"wo-field-count\">5 / 10</output>"), "counts chars, not bytes");
        assert!(m.contains("aria-invalid=\"true\"") && m.contains(">héllo</textarea>"));
        assert!(!m.contains("<fieldset") && !m.contains("enctype"));
    }

    #[test]
    fn kinds_map_to_attributes() {
        let fields = [
            Field::new("d", "D", FieldKind::Date { min: "2026-01-01", max: "" }),
            Field::new("t", "T", FieldKind::Time { min: "09:00", max: "17:00" }),
            Field::new("f", "F", FieldKind::File { accept: "", multiple: true }).value("ignored"),
            Field::new("h", "H", FieldKind::Pattern { pattern: "[a-z]+", hint: "Lowercase." }),
        ];
        let m = form_with(&Caps::NONE, "/p", &[FieldGroup::new("G", &fields)], FormOptions::default().layout(FormLayout::Inline)).into_string();
        assert!(m.contains("type=\"date\" value=\"\" min=\"2026-01-01\">"), "an empty bound is left out: {m}");
        assert!(m.contains("type=\"time\" value=\"\" min=\"09:00\" max=\"17:00\""));
        assert!(m.contains("type=\"file\" multiple") && !m.contains("ignored") && !m.contains("accept="));
        assert!(m.contains("pattern=\"[a-z]+\" title=\"Lowercase.\"") && m.contains("id=\"f-h-help\""));
        assert!(m.contains("class=\"wo-form wo-form-inline\"") && m.contains("<legend>G</legend>"));
    }
}
