use axum_nojs::prelude::*;

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
