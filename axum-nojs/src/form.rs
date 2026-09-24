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
//! - `<fieldset>` + `<legend>` per [`Form::group`]; help text and the error are tied to the
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
//! `maxlength` still stops input at the limit.
//!
//! **Enhanced:** the form is a swap root, so with the [`crate::enhance`] script a submit
//! replaces only the form (errors included) and a successful redirect swaps in the result.
//!
//! **Finding:** custom cross-field rules (password confirmation, async uniqueness) only run
//! on the server round trip, and warning about unsaved changes when leaving needs script.
//!
//! ```rust
//! use axum_nojs::prelude::*;
//! let ui = Ui::from(Caps::all());
//! let m = ui.form("/signup").email("email", "Email").required();
//! assert!(m.render().into_string().contains(r#"type="email" value="" required"#));
//!
//! // `required`, `help`, `maxlength`, `value` and friends apply to the field added last;
//! // `group` starts a fieldset for the fields after it.
//! let m = ui.form("/profile")
//!     .group("About you")
//!     .textarea("bio", "Bio", 3).maxlength(280).value("Hi").help("Shown on your profile.")
//!     .file("avatar", "Avatar", "image/png,image/jpeg")
//!     .date("born", "Born", "1900-01-01", "2026-12-31")
//!     .select("digest", "Digest", ["daily", "weekly", "never"]).value("weekly")
//!     .submit("Save profile")
//!     .inline();
//! let html = m.render().into_string();
//! assert!(html.contains("enctype=\"multipart/form-data\""));
//! assert!(html.contains("<legend>About you</legend>"));
//! assert!(html.contains(">2 / 280</output>"));
//! assert!(html.contains("accept=\"image/png,image/jpeg\""));
//! assert!(html.contains(r#"<option value="weekly" selected>"#));
//! ```

use maud::{Markup, Render, html};

use crate::input::{Field, FieldKind};
use crate::{Ui, enhance};

/// A POST form of fields, made by [`Ui::form`], or the fields alone, made by [`Ui::fields`].
/// Fields are added in order; `required`, `help`, `maxlength`, `value`, `error`,
/// `placeholder`, `multiple` and `checked` apply to the field added last. A stacked form
/// with a "Submit" button unless told otherwise.
#[derive(Clone, Debug)]
pub struct Form<'a> {
    action: Option<&'a str>,
    groups: Vec<(Option<&'a str>, Vec<Field<'a>>)>,
    submit: &'a str,
    inline: bool,
    values: &'a [(String, String)],
    errors: &'a [(&'a str, &'a str)],
    id: Option<&'a str>,
}

impl Ui {
    /// A form posting to `action`; add fields with [`Form::text`] and friends.
    pub fn form<'a>(&self, action: &'a str) -> Form<'a> {
        Form {
            action: Some(action),
            ..self.fields()
        }
    }

    /// Fields with no `<form>` around them, for a form built elsewhere (a wizard step, a
    /// dialog's confirm form).
    pub fn fields<'a>(&self) -> Form<'a> {
        Form {
            action: None,
            groups: vec![(None, Vec::new())],
            submit: "Submit",
            inline: false,
            values: &[],
            errors: &[],
            id: None,
        }
    }
}

impl<'a> Form<'a> {
    fn add(mut self, name: &'a str, label: &'a str, kind: FieldKind<'a>) -> Self {
        let field = Field::new(name, label, kind);
        self.groups
            .last_mut()
            .expect("a form always has a group")
            .1
            .push(field);
        self
    }

    fn last(mut self, change: impl FnOnce(&mut Field<'a>)) -> Self {
        if let Some(f) = self.groups.last_mut().and_then(|g| g.1.last_mut()) {
            change(f);
        }
        self
    }

    /// A `<fieldset>` with this `<legend>` around the fields added after it.
    pub fn group(mut self, legend: &'a str) -> Self {
        self.groups.push((Some(legend), Vec::new()));
        self
    }

    /// Single-line text.
    pub fn text(self, name: &'a str, label: &'a str) -> Self {
        self.add(name, label, FieldKind::Text)
    }

    /// `type="email"`: the browser checks the shape.
    pub fn email(self, name: &'a str, label: &'a str) -> Self {
        self.add(name, label, FieldKind::Email)
    }

    /// A whole number from `min` to `max`, inclusive.
    pub fn number(self, name: &'a str, label: &'a str, min: i64, max: i64) -> Self {
        self.add(name, label, FieldKind::Number { min, max })
    }

    /// Text that must match `pattern` (the HTML `pattern` attribute); `hint` explains the
    /// rule under the field and as the input's `title`.
    pub fn pattern(self, name: &'a str, label: &'a str, pattern: &'a str, hint: &'a str) -> Self {
        self.add(name, label, FieldKind::Pattern { pattern, hint })
    }

    /// Multi-line text, `rows` high; it grows with its content where `field-sizing` works.
    pub fn textarea(self, name: &'a str, label: &'a str, rows: u8) -> Self {
        self.add(name, label, FieldKind::Textarea { rows })
    }

    /// A file picker; `accept` lists MIME types or extensions (`image/*,.pdf`), empty for
    /// any. The form becomes `multipart/form-data`.
    pub fn file(self, name: &'a str, label: &'a str, accept: &'a str) -> Self {
        self.add(
            name,
            label,
            FieldKind::File {
                accept,
                multiple: false,
            },
        )
    }

    /// `type="date"`, bounds as `YYYY-MM-DD`; an empty bound is left out.
    pub fn date(self, name: &'a str, label: &'a str, min: &'a str, max: &'a str) -> Self {
        self.add(name, label, FieldKind::Date { min, max })
    }

    /// `type="time"`, bounds as `HH:MM`; an empty bound is left out.
    pub fn time(self, name: &'a str, label: &'a str, min: &'a str, max: &'a str) -> Self {
        self.add(name, label, FieldKind::Time { min, max })
    }

    /// A `<select>` of `options`, each its own value and text.
    pub fn select(
        self,
        name: &'a str,
        label: &'a str,
        options: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        self.add(
            name,
            label,
            FieldKind::Select(options.into_iter().collect()),
        )
    }

    /// A checkbox posting `true` when ticked and nothing when not (so a `bool` with
    /// `#[serde(default)]` reads it).
    pub fn checkbox(self, name: &'a str, label: &'a str) -> Self {
        self.add(name, label, FieldKind::Checkbox)
    }

    /// `type="hidden"`: posted with the form, not shown.
    pub fn hidden(self, name: &'a str, value: &'a str) -> Self {
        self.add(name, "", FieldKind::Hidden).value(value)
    }

    /// The `required` attribute, and a `*` after the label.
    pub fn required(self) -> Self {
        self.last(|f| f.required = true)
    }

    /// Help text under the field.
    pub fn help(self, help: &'a str) -> Self {
        self.last(|f| f.help = Some(help))
    }

    /// `maxlength`, counted in an `<output>` under the field.
    pub fn maxlength(self, max: usize) -> Self {
        self.last(|f| f.maxlength = Some(max))
    }

    /// The field's current value (ignored for files). A checkbox is ticked by `true`, `on`
    /// or `1`.
    pub fn value(self, value: &'a str) -> Self {
        self.last(|f| f.value = value)
    }

    /// Tick the checkbox.
    pub fn checked(self, checked: bool) -> Self {
        self.last(|f| f.value = if checked { "true" } else { "" })
    }

    /// A server message beside the field.
    pub fn error(self, message: &'a str) -> Self {
        self.last(|f| f.error = Some(message))
    }

    /// Placeholder text.
    pub fn placeholder(self, placeholder: &'a str) -> Self {
        self.last(|f| f.placeholder = Some(placeholder))
    }

    /// The file picker takes several files.
    pub fn multiple(self) -> Self {
        self.last(|f| {
            if let FieldKind::File { multiple, .. } = &mut f.kind {
                *multiple = true;
            }
        })
    }

    /// Label of the submit button.
    pub fn submit(mut self, label: &'a str) -> Self {
        self.submit = label;
        self
    }

    /// Labels beside the fields on screens wider than 40rem, above them on narrower ones.
    pub fn inline(mut self) -> Self {
        self.inline = true;
        self
    }

    /// Submitted values by field name, as a form post parses them: each field without its
    /// own `value` shows the one named after it.
    pub fn values(mut self, values: &'a [(String, String)]) -> Self {
        self.values = values;
        self
    }

    /// Server messages `(field name, message)`: each field without its own `error` shows the
    /// one named after it.
    pub fn errors(mut self, errors: &'a [(&'a str, &'a str)]) -> Self {
        self.errors = errors;
        self
    }

    /// What the swap root's id is built from, when two forms on a page post to the same
    /// action (one per tab, say); the action by default.
    pub fn id(mut self, id: &'a str) -> Self {
        self.id = Some(id);
        self
    }

    /// Every field, with the values and errors filled in by name.
    pub(crate) fn filled(&self) -> impl Iterator<Item = (Option<&'a str>, Vec<Field<'a>>)> + '_ {
        self.groups
            .iter()
            .filter(|(legend, fs)| legend.is_some() || !fs.is_empty())
            .map(|(legend, fs)| {
                let fs = fs.iter().map(|f| {
                    let value = if f.value.is_empty() {
                        self.values
                            .iter()
                            .find(|(n, _)| n == f.name)
                            .map_or("", |(_, v)| v.as_str())
                    } else {
                        f.value
                    };
                    let error = f.error.or_else(|| {
                        self.errors
                            .iter()
                            .find(|(n, _)| *n == f.name)
                            .map(|(_, m)| *m)
                    });
                    Field {
                        value,
                        error,
                        ..f.clone()
                    }
                });
                (*legend, fs.collect())
            })
    }

    /// Whether any field has a server message.
    pub(crate) fn has_errors(&self) -> bool {
        self.filled()
            .any(|(_, fs)| fs.iter().any(|f| f.error.is_some()))
    }

    fn fields(&self) -> Markup {
        html! {
            @for (legend, fs) in self.filled() {
                @if let Some(legend) = legend {
                    fieldset class="nojs-form-group" { legend { (legend) } @for f in &fs { (f) } }
                } @else {
                    @for f in &fs { (f) }
                }
            }
        }
    }
}

impl Render for Form<'_> {
    fn render(&self) -> Markup {
        let Some(action) = self.action else {
            return self.fields();
        };
        let multipart = self
            .groups
            .iter()
            .flat_map(|g| &g.1)
            .any(|f| matches!(f.kind, FieldKind::File { .. }));
        let class = if self.inline {
            "nojs-form nojs-form-inline"
        } else {
            "nojs-form"
        };
        html! {
            form id=(enhance::swap_id("nojs-form", self.id.unwrap_or(action))) data-nojs="swap" class=(class) method="post" action=(action)
                enctype=[multipart.then_some("multipart/form-data")] {
                (self.fields())
                div class="nojs-form-actions" { button type="submit" class="nojs-primary" { (self.submit) } }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.nojs-form { display: grid; gap: calc(var(--nojs-space) * 3); max-width: 28rem; }
.nojs-form-inline { max-width: 40rem; }
.nojs-form-group { display: grid; gap: calc(var(--nojs-space) * 2); margin: 0; padding: calc(var(--nojs-space) * 3); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius-lg); }
.nojs-form-group legend { padding: 0 0.5rem; font-weight: 600; }
@media (min-width: 40rem) {
  .nojs-form-inline .nojs-field { grid-template-columns: 10rem 1fr; column-gap: calc(var(--nojs-space) * 2); }
  .nojs-form-inline .nojs-field > :not(label) { grid-column: 2; }
  /* The label sits on the input's row, centred on it; help, counter and error stack below. */
  .nojs-form-inline .nojs-field > label { grid-column: 1; grid-row: 1; align-self: center; }
  .nojs-form-inline .nojs-field-check > label { grid-column: 2; }
  .nojs-form-inline .nojs-form-actions { padding-left: calc(10rem + var(--nojs-space) * 2); }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    fn html(f: Form) -> String {
        f.render().into_string()
    }

    #[test]
    fn checkbox_and_hidden_fields() {
        let ui = Ui::default();
        let fs = || {
            ui.fields()
                .hidden("tab", "1")
                .checkbox("notify", "Email me")
        };
        let m = html(fs().checked(true));
        assert!(
            m.starts_with(r#"<input type="hidden" name="tab" value="1">"#),
            "{m}"
        );
        assert!(
            m.contains(r#"type="checkbox" value="true" checked"#)
                && m.contains(" Email me</label>"),
            "{m}"
        );
        assert!(!html(fs()).contains("checked"));
    }

    #[test]
    fn values_and_errors_fill_fields_by_name() {
        let ui = Ui::default();
        let values = [
            ("name".to_string(), "Ada".to_string()),
            ("email".to_string(), "posted@x.org".to_string()),
            ("bio".to_string(), "Hi".to_string()),
        ];
        let errors = [("name", "Too short."), ("bio", "Posted message.")];
        let fs = |f: Form<'static>| {
            f.text("name", "Name")
                .email("email", "Email")
                .value("own@x.org")
                .textarea("bio", "Bio", 2)
                .error("Own message.")
        };
        let m = html(fs(ui.form("/p")).values(&values).errors(&errors));
        assert!(m.contains(r#"name="name" type="text" value="Ada""#), "{m}");
        assert!(
            m.contains(r#"value="own@x.org""#) && !m.contains("posted@x.org"),
            "a field's own value wins"
        );
        assert!(m.contains(">Too short.</p>") && m.contains(r#"aria-invalid="true""#));
        assert!(
            m.contains("Own message.") && !m.contains("Posted message."),
            "a field's own error wins"
        );
        assert!(m.contains(">Hi</textarea>"));
        let bare = html(fs(ui.fields()).values(&values));
        assert!(!bare.contains("<form") && bare.contains(r#"value="Ada""#));
    }

    #[test]
    fn help_counter_and_error_describe_the_field() {
        let f = Ui::default()
            .form("/p")
            .textarea("bio", "Bio", 2)
            .value("héllo")
            .maxlength(10)
            .help("Short.")
            .error("Too dull.");
        let m = html(f);
        assert!(
            m.contains("aria-describedby=\"f-bio-help f-bio-count f-bio-error\""),
            "{m}"
        );
        assert!(m.contains("<output id=\"f-bio-count\" for=\"f-bio\" class=\"nojs-field-count\">5 / 10</output>"), "counts chars, not bytes");
        assert!(m.contains("aria-invalid=\"true\"") && m.contains(">héllo</textarea>"));
        assert!(!m.contains("<fieldset") && !m.contains("enctype"));
    }

    #[test]
    fn kinds_map_to_attributes() {
        let f = Ui::default()
            .form("/p")
            .group("G")
            .date("d", "D", "2026-01-01", "")
            .time("t", "T", "09:00", "17:00")
            .file("f", "F", "")
            .multiple()
            .value("ignored")
            .pattern("h", "H", "[a-z]+", "Lowercase.")
            .text("p", "P")
            .placeholder("Type")
            .inline();
        let m = html(f);
        assert!(
            m.contains("type=\"date\" value=\"\" min=\"2026-01-01\">"),
            "an empty bound is left out: {m}"
        );
        assert!(m.contains("type=\"time\" value=\"\" min=\"09:00\" max=\"17:00\""));
        assert!(
            m.contains("type=\"file\" multiple")
                && !m.contains("ignored")
                && !m.contains("accept=")
        );
        assert!(
            m.contains("pattern=\"[a-z]+\" title=\"Lowercase.\"") && m.contains("id=\"f-h-help\"")
        );
        assert!(m.contains("placeholder=\"Type\""));
        assert!(
            m.contains("class=\"nojs-form nojs-form-inline\"") && m.contains("<legend>G</legend>")
        );
    }
}
