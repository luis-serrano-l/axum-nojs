//! # Form
//!
//! A validated form: the browser blocks bad input before submit, the server checks again and
//! re-renders with messages. No script.
//!
//! **Platform features:**
//! - Constraint validation attributes `required`, `pattern`, `min`, `max`, `type=email`
//!   (baseline 2015).
//! - `:user-invalid` / `:user-valid` (baseline 2023): styles only after the user has interacted,
//!   so fields are not red on first paint.
//! - Post/Redirect/Get for success; on error the server re-renders the form with values and
//!   messages.
//!
//! **Finding:** custom cross-field rules (password confirmation, async uniqueness) only run
//! on the server round trip.
//!
//! ```rust
//! use webonsive::{form, Field, FieldKind};
//! let fields = [Field { name: "email", label: "Email", kind: FieldKind::Email, value: "", error: None, required: true }];
//! let m = form("/form", &fields, "Sign up");
//! ```

use maud::{Markup, html};

/// Input type for a [`Field`].
#[derive(Clone, Copy, Debug)]
pub enum FieldKind {
    Text,
    Email,
    /// Positive integer.
    Number { min: i64, max: i64 },
    /// Free text with a regex `pattern` and a human hint.
    Pattern { pattern: &'static str, hint: &'static str },
}

/// One form field with its current value and server-side error.
#[derive(Clone, Debug)]
pub struct Field<'a> {
    pub name: &'a str,
    pub label: &'a str,
    pub kind: FieldKind,
    pub value: &'a str,
    pub error: Option<&'a str>,
    pub required: bool,
}

/// Render `fields` as a POST form to `action` with a submit button labelled `submit`.
pub fn form(action: &str, fields: &[Field<'_>], submit: &str) -> Markup {
    html! {
        form class="wo-form" method="post" action=(action) {
            @for f in fields {
                @let id = format!("f-{}", f.name);
                @let err_id = format!("f-{}-error", f.name);
                div class="wo-field" {
                    label for=(id) { (f.label) @if f.required { " *" } }
                    @match f.kind {
                        FieldKind::Text => {
                            input id=(id) name=(f.name) type="text" value=(f.value) required[f.required]
                                aria-describedby=[f.error.map(|_| err_id.as_str())];
                        }
                        FieldKind::Email => {
                            input id=(id) name=(f.name) type="email" value=(f.value) required[f.required]
                                aria-describedby=[f.error.map(|_| err_id.as_str())];
                        }
                        FieldKind::Number { min, max } => {
                            input id=(id) name=(f.name) type="number" value=(f.value) required[f.required]
                                min=(min) max=(max) aria-describedby=[f.error.map(|_| err_id.as_str())];
                        }
                        FieldKind::Pattern { pattern, hint } => {
                            input id=(id) name=(f.name) type="text" value=(f.value) required[f.required]
                                pattern=(pattern) title=(hint) aria-describedby=[f.error.map(|_| err_id.as_str())];
                            small class="wo-note" { (hint) }
                        }
                    }
                    @if let Some(e) = f.error {
                        p id=(err_id) class="wo-error" role="alert" { (e) }
                    }
                }
            }
            button type="submit" class="wo-primary" { (submit) }
        }
    }
}

pub const CSS: &str = r#"
.wo-form { display: grid; gap: calc(var(--wo-space) * 2); max-width: 24rem; }
.wo-field { display: grid; gap: 4px; }
.wo-field label { font-weight: 600; }
.wo-field input:user-invalid { border-color: var(--wo-danger); }
.wo-field input:user-valid { border-color: color-mix(in srgb, var(--wo-accent) 60%, transparent); }
.wo-error { color: var(--wo-danger); margin: 0; font-size: 0.9rem; }
"#;
