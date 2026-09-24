//! # Wizard
//!
//! A form split into steps, no script: each step is one `<form method="post">`, the server
//! checks it and either re-renders it with messages beside the fields or stores it and
//! redirects to the next step. Optional steps can be skipped, a progress bar shows how far
//! along the visitor is, the last step is a review whose every value links back to its step,
//! and a visitor who closes the tab comes back to the step they left.
//!
//! **Platform features:**
//! - Post/Redirect/Get per step: `prg` after each valid POST, so refresh never re-submits.
//!   An invalid POST answers with the same step, the values kept and the messages beside the
//!   fields; mark it with [`Step::error`] and the step list shows it in error.
//! - The current step is a `step.<id>` key in [`crate::UiState`]: it travels in `?step.<id>=n`
//!   and the `wo-ui` cookie, like a tab, so the URL of a step can be shared, Back/Forward in
//!   the browser work, and a bare visit resumes where the cookie says
//!   ([`UiState::remembered`]) with a "Start over" link.
//! - `<ol>` step list with `aria-current="step"` on the current one; done steps are links.
//! - `<fieldset>` + `<legend>` for the step's fields, `<button name="skip" formnovalidate>`
//!   (baseline 2015) to skip an optional step without the browser checking its fields.
//! - `<progress>` (baseline 2015) for the steps done out of the total.
//! - [`summary`] renders the review as a `<dl>` with an "Edit" link per value.
//!
//! **Fallback:** none needed. Everything is a link or a form; `Caps` is unused.
//!
//! **Finding:** the entered values are the app's data, not UI state, so they do not belong
//! in the URL. The demo keeps them in one cookie; a real app would use its session store.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, UiState, wizard, wizard::{Step, WizardOptions, summary}};
//! let state = UiState::parse("/wizard", "step.signup=2", "");
//! let steps = [
//!     Step::new("Account", html! { input name="email"; }),
//!     Step::new("Newsletter", html! { input name="topics"; }).optional(true),
//!     Step::new("Review", summary(&state, "signup", &[("Email", "a@b.c", 0), ("Topics", "", 1)])),
//! ];
//! let m = wizard(&Caps::all(), "signup", "/wizard", &steps, &state, Default::default());
//! let m = wizard(&Caps::all(), "signup", "/wizard", &steps, &state, WizardOptions::default().finish("Create account").progress(true));
//! let html = m.into_string();
//! assert!(html.contains("aria-current=\"step\""));
//! assert!(html.contains("<progress class=\"wo-wizard-progress\" value=\"2\" max=\"2\""));
//! assert!(html.contains("href=\"/wizard?step.signup=0\" aria-label=\"Edit Email\""));
//! ```

use maud::{Markup, html};

use crate::{Caps, UiState, enhance};

/// One step: its title in the step list and its fields (or, on the last step, the review).
pub struct Step {
    /// Shown in the step list and as the fieldset legend.
    pub title: &'static str,
    /// The step's inputs, or the review markup for the last step.
    pub body: Markup,
    /// A "Skip" button beside Next; the handler sees `skip=1` and moves on without checking.
    pub optional: bool,
    /// The server rejected this step: it is marked in the step list and its legend.
    pub error: bool,
}

impl Step {
    /// A required step.
    pub fn new(title: &'static str, body: Markup) -> Self {
        Step { title, body, optional: false, error: false }
    }
    /// Offer a "Skip" button on this step.
    pub fn optional(mut self, optional: bool) -> Self {
        self.optional = optional;
        self
    }
    /// Mark this step as failed server validation.
    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }
}

/// Options for [`wizard`]; `Default::default()` labels the last button "Finish" and shows the
/// progress bar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WizardOptions<'a> {
    /// Label of the last step's submit button.
    pub finish: &'a str,
    /// A `<progress>` bar above the form.
    pub progress: bool,
}

impl Default for WizardOptions<'_> {
    fn default() -> Self {
        WizardOptions { finish: "Finish", progress: true }
    }
}

impl<'a> WizardOptions<'a> {
    /// Label of the last step's submit button.
    pub fn finish(mut self, finish: &'a str) -> Self {
        self.finish = finish;
        self
    }
    /// Show the `<progress>` bar.
    pub fn progress(mut self, progress: bool) -> Self {
        self.progress = progress;
        self
    }
}

/// `steps.last()` is the review step. Each POST to `action` carries `step=<n>` (0-based), the
/// step's fields and `skip=1` when an optional step was skipped. The handler checks them and
/// either answers with the same step (values kept, messages beside the fields,
/// [`Step::error`] set) or stores them and redirects to `state.link("step.<id>", n+1)`.
pub fn wizard(_caps: &Caps, id: &str, action: &str, steps: &[Step], state: &UiState, options: WizardOptions) -> Markup {
    let WizardOptions { finish, progress } = options;
    let key = format!("step.{id}");
    let current = state.step(id).min(steps.len().saturating_sub(1));
    let last = current + 1 == steps.len();
    let step = &steps[current];
    html! {
        div id=(enhance::swap_id("wo-wizard", id)) data-wo="swap" class="wo-wizard" {
            @if current > 0 && state.remembered(&key) {
                p class="wo-wizard-resume" role="status" {
                    "Picked up where you left off, at step " (current + 1) ". "
                    a href=(state.link(&key, "0")) { "Start over" }
                }
            }
            ol class="wo-wizard-steps" {
                @for (i, s) in steps.iter().enumerate() {
                    @let class = match (i == current, i < current, s.error) {
                        (true, _, true) => "wo-wizard-current wo-wizard-error",
                        (true, _, false) => "wo-wizard-current",
                        (false, true, true) => "wo-wizard-done wo-wizard-error",
                        (false, true, false) => "wo-wizard-done",
                        (false, false, true) => "wo-wizard-error",
                        (false, false, false) => "",
                    };
                    li class=[(!class.is_empty()).then_some(class)] aria-current=[(i == current).then_some("step")] {
                        @if i < current { a href=(state.link(&key, &i.to_string())) { (s.title) } } @else { span { (s.title) } }
                        @if s.optional { " " small { "(optional)" } }
                        @if s.error { span class="wo-sr" { " (has errors)" } }
                    }
                }
            }
            @if progress {
                progress class="wo-wizard-progress" value=(current) max=(steps.len().saturating_sub(1).max(1)) aria-label="Progress" {
                    (current) " of " (steps.len().saturating_sub(1)) " steps done"
                }
            }
            form method="post" action=(action) class="wo-wizard-form" {
                input type="hidden" name="step" value=(current);
                fieldset aria-invalid=[step.error.then_some("true")] {
                    legend { "Step " (current + 1) " of " (steps.len()) ": " (step.title) @if step.optional { " (optional)" } }
                    (step.body)
                }
                p class="wo-wizard-actions" {
                    @if current > 0 { a class="wo-wizard-back" href=(state.link(&key, &(current - 1).to_string())) { "Back" } }
                    @if step.optional && !last { button type="submit" name="skip" value="1" formnovalidate { "Skip" } }
                    button type="submit" class="wo-primary" { @if last { (finish) } @else { "Next" } }
                }
            }
        }
    }
}

/// The review: a `<dl>` of `(label, value, step)` with an "Edit" link from each value back to
/// the step that asked for it. Empty values read "(skipped)".
pub fn summary(state: &UiState, id: &str, items: &[(&str, &str, usize)]) -> Markup {
    let key = format!("step.{id}");
    html! {
        dl class="wo-wizard-review" {
            @for (label, value, step) in items {
                dt { (label) }
                dd {
                    @if value.is_empty() { span class="wo-note" { "(skipped)" } } @else { (value) }
                    " " a class="wo-wizard-edit" href=(state.link(&key, &step.to_string())) aria-label={ "Edit " (label) } { "Edit" }
                }
            }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
.wo-wizard-steps { display: flex; flex-wrap: wrap; gap: var(--wo-space); list-style: none; counter-reset: wo-step; margin: 0 0 calc(var(--wo-space) * 2); padding: 0; }
.wo-wizard-steps li { counter-increment: wo-step; color: var(--wo-muted); padding: 0.25rem 0.75rem; border: 1px solid var(--wo-line); border-radius: var(--wo-radius); }
.wo-wizard-steps li::before { content: counter(wo-step) ". "; }
.wo-wizard-steps li a { color: var(--wo-fg); text-decoration: none; }
.wo-wizard-steps li a:hover { text-decoration: underline; }
.wo-wizard-steps small { font-size: 0.8em; }
.wo-wizard-current { color: var(--wo-on-accent) !important; background: var(--wo-accent); border-color: transparent !important; }
.wo-wizard-steps .wo-wizard-error { border-color: var(--wo-danger) !important; }
.wo-wizard-steps .wo-wizard-error::before { content: "! " counter(wo-step) ". "; color: var(--wo-danger); font-weight: 700; }
.wo-wizard-steps .wo-wizard-current.wo-wizard-error { background: var(--wo-danger); }
.wo-wizard-steps .wo-wizard-current.wo-wizard-error::before { color: inherit; }
.wo-wizard-progress { display: block; width: 100%; max-width: 32rem; height: 0.5rem; margin: 0 0 calc(var(--wo-space) * 2); accent-color: var(--wo-accent); }
.wo-wizard-resume { padding: var(--wo-space) calc(var(--wo-space) * 2); border-left: 3px solid var(--wo-accent); background: var(--wo-surface); max-width: none; }
.wo-wizard-form fieldset { border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 1rem 1.25rem; }
.wo-wizard-form fieldset[aria-invalid=true] { border-color: var(--wo-danger); }
.wo-wizard-form legend { padding: 0 0.5rem; color: var(--wo-muted); }
.wo-wizard-form label { display: block; margin: 0.5rem 0; }
.wo-wizard-form input:not([type=checkbox]), .wo-wizard-form select { display: block; width: 100%; max-width: 24rem; margin-top: 0.25rem; }
.wo-wizard-form [aria-invalid=true]:is(input, select) { border-color: var(--wo-danger); }
.wo-wizard-actions { display: flex; align-items: center; gap: calc(var(--wo-space) * 2); margin-top: 1rem; }
.wo-wizard-review { margin: 0; }
.wo-wizard-review dt { color: var(--wo-muted); }
.wo-wizard-review dd { margin: 0 0 0.5rem; }
.wo-wizard-edit { margin-left: var(--wo-space); font-size: 0.875rem; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_list_marks_done_current_and_todo() {
        let steps = [Step::new("A", html! {}), Step::new("B", html! {}), Step::new("C", html! {})];
        let state = UiState::parse("/w", "step.x=1", "");
        let m = wizard(&Caps::NONE, "x", "/w", &steps, &state, WizardOptions::default().finish("Done")).into_string();
        assert!(m.contains("class=\"wo-wizard-done\"><a href=\"/w?step.x=0\">A</a>"), "{m}");
        assert!(m.contains("aria-current=\"step\"><span>B</span>"));
        assert!(m.contains("value=\"1\"") && m.contains(">Next<") && !m.contains(">Done<"));
        assert!(!m.contains("wo-wizard-resume"), "the step came from the query");
        let end = UiState::parse("/w", "step.x=9", "");
        let m = wizard(&Caps::NONE, "x", "/w", &steps, &end, WizardOptions::default().finish("Done")).into_string();
        assert!(m.contains(">Done<") && m.contains("href=\"/w?step.x=1\">Back<"));
    }

    #[test]
    fn errors_skip_and_resume() {
        let steps = [Step::new("A", html! {}), Step::new("B", html! {}).optional(true).error(true), Step::new("C", html! {})];
        let state = UiState::parse("/w", "", "step.x=1");
        let m = wizard(&Caps::NONE, "x", "/w", &steps, &state, Default::default()).into_string();
        assert!(m.contains("class=\"wo-wizard-current wo-wizard-error\" aria-current=\"step\""), "{m}");
        assert!(m.contains("<fieldset aria-invalid=\"true\">"));
        assert!(m.contains("name=\"skip\" value=\"1\" formnovalidate"));
        assert!(m.contains("class=\"wo-wizard-resume\"") && m.contains("href=\"/w?step.x=0\">Start over"));
        assert!(m.contains("value=\"1\" max=\"2\""), "progress: one of two steps done");
        let m = wizard(&Caps::NONE, "x", "/w", &steps, &state, WizardOptions::default().progress(false)).into_string();
        assert!(!m.contains("<progress"));
    }
}
