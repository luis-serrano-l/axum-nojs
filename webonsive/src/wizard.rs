//! # Wizard
//!
//! A form split into steps, no script: each step is one `<form method="post">`, the server
//! stores what was entered and redirects to the next step, and the last step is a review of
//! everything before the real submit. Back is a link, so nothing entered is lost.
//!
//! **Platform features:**
//! - Post/Redirect/Get per step: `prg` after each POST, so refresh never re-submits.
//! - The current step is a `step.<id>` key in [`crate::UiState`]: it travels in `?step.<id>=n`
//!   and the `wo-ui` cookie, like a tab, so the URL of a step can be shared and Back/Forward
//!   in the browser work.
//! - `<ol>` step list with `aria-current="step"` on the current one; done steps are links.
//! - `<fieldset>` + `<legend>` for the step's fields, `<button name="step">` for Next.
//!
//! **Fallback:** none needed. Everything is a link or a form; `Caps` is unused.
//!
//! **Finding:** the entered values are the app's data, not UI state, so they do not belong
//! in the URL. The demo keeps them in one cookie; a real app would use its session store.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, UiState, wizard, wizard::{Step, WizardOptions}};
//! let steps = [Step { title: "Account", body: html! { input name="email"; } },
//!              Step { title: "Review", body: html! { p { "email: a@b.c" } } }];
//! let state = UiState::parse("/wizard", "step.signup=1", "");
//! let m = wizard(&Caps::all(), "signup", "/wizard", &steps, &state, Default::default());
//! let m = wizard(&Caps::all(), "signup", "/wizard", &steps, &state, WizardOptions::default().finish("Create account"));
//! assert!(m.into_string().contains("aria-current=\"step\""));
//! ```

use maud::{Markup, html};

use crate::{Caps, UiState, enhance};

/// One step: its title in the step list and its fields (or, on the last step, the review).
pub struct Step {
    /// Shown in the step list and as the fieldset legend.
    pub title: &'static str,
    /// The step's inputs, or the review markup for the last step.
    pub body: Markup,
}

/// Options for [`wizard`]; `Default::default()` labels the last button "Finish".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WizardOptions<'a> {
    /// Label of the last step's submit button.
    pub finish: &'a str,
}

impl Default for WizardOptions<'_> {
    fn default() -> Self {
        WizardOptions { finish: "Finish" }
    }
}

impl<'a> WizardOptions<'a> {
    /// Label of the last step's submit button.
    pub fn finish(mut self, finish: &'a str) -> Self {
        self.finish = finish;
        self
    }
}

/// `steps.last()` is the review step. Each POST to `action` carries `step=<n>` (0-based) and
/// the step's fields; the handler stores them and redirects to `state.link("step.<id>", n+1)`.
pub fn wizard(_caps: &Caps, id: &str, action: &str, steps: &[Step], state: &UiState, options: WizardOptions) -> Markup {
    let WizardOptions { finish } = options;
    let key = format!("step.{id}");
    let current = state.step(id).min(steps.len().saturating_sub(1));
    let last = current + 1 == steps.len();
    html! {
        div id=(enhance::swap_id("wo-wizard", id)) data-wo="swap" class="wo-wizard" {
            ol class="wo-wizard-steps" {
                @for (i, step) in steps.iter().enumerate() {
                    @if i == current {
                        li class="wo-wizard-current" aria-current="step" { span { (step.title) } }
                    } @else if i < current {
                        li class="wo-wizard-done" { a href=(state.link(&key, &i.to_string())) { (step.title) } }
                    } @else {
                        li { span { (step.title) } }
                    }
                }
            }
            form method="post" action=(action) class="wo-wizard-form" {
                input type="hidden" name="step" value=(current);
                fieldset {
                    legend { "Step " (current + 1) " of " (steps.len()) ": " (steps[current].title) }
                    (steps[current].body)
                }
                p class="wo-wizard-actions" {
                    @if current > 0 { a class="wo-wizard-back" href=(state.link(&key, &(current - 1).to_string())) { "Back" } }
                    button type="submit" class="wo-primary" { @if last { (finish) } @else { "Next" } }
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
.wo-wizard-current { color: var(--wo-on-accent) !important; background: var(--wo-accent); border-color: transparent !important; }
.wo-wizard-form fieldset { border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 1rem 1.25rem; }
.wo-wizard-form legend { padding: 0 0.5rem; color: var(--wo-muted); }
.wo-wizard-form label { display: block; margin: 0.5rem 0; }
.wo-wizard-form input:not([type=checkbox]), .wo-wizard-form select { display: block; width: 100%; max-width: 24rem; margin-top: 0.25rem; }
.wo-wizard-actions { display: flex; align-items: center; gap: calc(var(--wo-space) * 2); margin-top: 1rem; }
.wo-wizard-review { margin: 0; }
.wo-wizard-review dt { color: var(--wo-muted); }
.wo-wizard-review dd { margin: 0 0 0.5rem; }
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_list_marks_done_current_and_todo() {
        let steps = [
            Step { title: "A", body: html! {} },
            Step { title: "B", body: html! {} },
            Step { title: "C", body: html! {} },
        ];
        let state = UiState::parse("/w", "step.x=1", "");
        let m = wizard(&Caps::NONE, "x", "/w", &steps, &state, WizardOptions::default().finish("Done")).into_string();
        assert!(m.contains("class=\"wo-wizard-done\"><a href=\"/w?step.x=0\">A</a>"), "{m}");
        assert!(m.contains("aria-current=\"step\"><span>B</span>"));
        assert!(m.contains("value=\"1\"") && m.contains(">Next<") && !m.contains(">Done<"));
        let end = UiState::parse("/w", "step.x=9", "");
        let m = wizard(&Caps::NONE, "x", "/w", &steps, &end, WizardOptions::default().finish("Done")).into_string();
        assert!(m.contains(">Done<") && m.contains("href=\"/w?step.x=1\">Back<"));
    }
}
