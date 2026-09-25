//! The account pages' bodies, built from `ui.*`. Written by `cargo lui auth`; edit freely.
use loco_ui::prelude::*;

pub fn signin(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    html! {
        (ui.flash())
        h1 { "Sign in" }
        (ui.form("/signin")
            .email("email", "Email").required()
            .password("password", "Password").required()
            .values(values)
            .errors(errors)
            .submit("Sign in"))
        p { a href="/forgot" { "Forgot your password?" } " · " a href="/magic-link" { "Email me a sign-in link" } }
        p { "No account? " a href="/signup" { "Sign up" } }
    }
}

pub fn signup(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    html! {
        h1 { "Sign up" }
        (ui.form("/signup")
            .text("name", "Name").required()
            .email("email", "Email").required()
            .password("password", "Password").required()
            .values(values)
            .errors(errors)
            .submit("Sign up"))
        p { "Have an account? " a href="/signin" { "Sign in" } }
    }
}

pub fn forgot(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    html! {
        h1 { "Forgot password" }
        p { "We will email you a link to choose a new password." }
        (ui.form("/forgot")
            .email("email", "Email").required()
            .values(values)
            .errors(errors)
            .submit("Email me a link"))
        p { a href="/signin" { "Back to sign in" } }
    }
}

/// The new-password form; `action` is the link's own `/reset/<token>`.
pub fn reset(ui: &Ui, action: &str, errors: &[(&str, &str)]) -> Markup {
    html! {
        h1 { "Choose a new password" }
        (ui.form(action)
            .password("password", "New password").required()
            .errors(errors)
            .submit("Change password"))
    }
}

pub fn magic(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    html! {
        h1 { "Email me a link" }
        p { "We will email you a link that signs you in once, no password needed." }
        (ui.form("/magic-link")
            .email("email", "Email").required()
            .values(values)
            .errors(errors)
            .submit("Email me a link"))
        p { a href="/signin" { "Sign in with a password" } }
    }
}

/// A spent or unknown link; `again` is the page that sends a new one.
pub fn expired(ui: &Ui, again: &str) -> Markup {
    html! {
        h1 { "Link expired" }
        p { "That link was used already, is too old, or was mistyped." }
        (ui.link_button("Send a new link", again).primary())
    }
}
