//! Accounts as plain forms: sign up, sign in, sign out, forgot and reset password, email
//! verification and magic link, on the starter's `users` model and `AuthMailer`. The token
//! Loco's `auth::JWT` checks travels in the `HttpOnly` `auth` cookie (`auth.jwt.location`
//! in `config/*.yaml`), so no page needs script to send it. Every change is a form post and
//! a redirect with a flash (Post/Redirect/Get). Written by `cargo lui auth`; edit freely.
use loco_rs::prelude::*;
use loco_ui::{
    loco::{FieldErrors, Submitted},
    prelude::*,
};

use crate::{
    mailers::auth::AuthMailer,
    models::users::{self, RegisterParams},
    views,
};

/// The cookie `config/*.yaml` tells `auth::JWT` to read.
pub const COOKIE: &str = "auth";
/// Where signing in lands.
pub const HOME: &str = "/";
/// What the forgot-password and magic-link forms say, whether or not the email has an account.
const SENT: &str = "If an account uses that email, a link is on its way. It works once.";

/// The `Set-Cookie` value that signs `user` in.
fn session(ctx: &AppContext, user: &users::Model) -> Result<String> {
    let jwt = ctx.config.get_jwt_config()?;
    let token = user.generate_jwt(&jwt.secret, jwt.expiration)?;
    Ok(format!(
        "{COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        jwt.expiration
    ))
}

/// Sign `user` in and go home with a greeting.
fn welcome(ui: &Ui, ctx: &AppContext, user: &users::Model, message: &str) -> Result<Response> {
    let cookie = session(ctx, user)?;
    Ok(ui.redirect(HOME).cookie(cookie).ok(message).into_response())
}

#[debug_handler]
async fn signin_page(ui: Ui) -> Result<Page> {
    Ok(ui.page("Sign in", views::account::signin(&ui, &[], &[])))
}

#[debug_handler]
async fn signin(
    ui: Ui,
    State(ctx): State<AppContext>,
    Form(posted): Form<Vec<(String, String)>>,
) -> Result<Response> {
    let mut form = Submitted::new(posted);
    let email = form.required::<String>("email");
    let password = form.required::<String>("password");
    let user = match (email, password) {
        (Some(email), Some(password)) => users::Model::find_by_email(&ctx.db, &email)
            .await
            .ok()
            .filter(|u| u.verify_password(&password)),
        _ => None,
    };
    if let Some(user) = user {
        return welcome(&ui, &ctx, &user, &format!("Signed in as {}.", user.name));
    }
    let errors = form.errors();
    let mut pairs = errors.pairs();
    if pairs.is_empty() {
        pairs.push(("password", "Wrong email or password."));
    }
    let body = views::account::signin(&ui, form.values(), &pairs);
    Ok(ui.page("Sign in", body).into_response())
}

#[debug_handler]
async fn signup_page(ui: Ui) -> Result<Page> {
    Ok(ui.page("Sign up", views::account::signup(&ui, &[], &[])))
}

#[debug_handler]
async fn signup(
    ui: Ui,
    State(ctx): State<AppContext>,
    Form(posted): Form<Vec<(String, String)>>,
) -> Result<Response> {
    let mut form = Submitted::new(posted);
    let name = form.required::<String>("name");
    let email = form.required::<String>("email");
    let password = form.required::<String>("password");
    let mut taken = false;
    let errors = match (name, email, password) {
        (Some(name), Some(email), Some(password)) => {
            let params = RegisterParams {
                name,
                email,
                password,
            };
            match users::Model::create_with_password(&ctx.db, &params).await {
                Ok(user) => {
                    let user = user
                        .into_active_model()
                        .set_email_verification_sent(&ctx.db)
                        .await?;
                    AuthMailer::send_welcome(&ctx, &user).await?;
                    let message =
                        format!("Welcome. A link to verify {} is on its way.", user.email);
                    return welcome(&ui, &ctx, &user, &message);
                }
                Err(ModelError::EntityAlreadyExists) => {
                    taken = true;
                    FieldErrors::default()
                }
                Err(ModelError::Validation(e)) => FieldErrors::from(&e),
                Err(e) => return Err(e.into()),
            }
        }
        _ => form.errors(),
    };
    let mut pairs = errors.pairs();
    if taken {
        pairs.push(("email", "That email is taken."));
    }
    let body = views::account::signup(&ui, form.values(), &pairs);
    Ok(ui.page("Sign up", body).into_response())
}

#[debug_handler]
async fn signout(ui: Ui) -> Result<Redirect> {
    let gone = format!("{COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");
    Ok(ui.redirect("/").cookie(gone).ok("Signed out."))
}

/// The link in the welcome email.
#[debug_handler]
async fn verify(
    ui: Ui,
    State(ctx): State<AppContext>,
    Path(token): Path<String>,
) -> Result<Redirect> {
    let Ok(user) = users::Model::find_by_verification_token(&ctx.db, &token).await else {
        return Ok(ui
            .redirect("/signin")
            .danger("That verification link is not valid."));
    };
    if user.email_verified_at.is_none() {
        user.into_active_model().verified(&ctx.db).await?;
    }
    Ok(ui.redirect(HOME).ok("Email verified."))
}

#[debug_handler]
async fn forgot_page(ui: Ui) -> Result<Page> {
    Ok(ui.page("Forgot password", views::account::forgot(&ui, &[], &[])))
}

/// Mails a reset link. The answer is the same whether or not the email has an account, so the
/// form does not reveal who signed up.
#[debug_handler]
async fn forgot(
    ui: Ui,
    State(ctx): State<AppContext>,
    Form(posted): Form<Vec<(String, String)>>,
) -> Result<Response> {
    let mut form = Submitted::new(posted);
    let Some(email) = form.required::<String>("email") else {
        let errors = form.errors();
        let body = views::account::forgot(&ui, form.values(), &errors.pairs());
        return Ok(ui.page("Forgot password", body).into_response());
    };
    if let Ok(user) = users::Model::find_by_email(&ctx.db, &email).await {
        let user = user
            .into_active_model()
            .set_forgot_password_sent(&ctx.db)
            .await?;
        AuthMailer::forgot_password(&ctx, &user).await?;
    }
    Ok(ui.redirect("/signin").ok(SENT).into_response())
}

/// The link in the forgot-password email: a form for the new password, or a way to ask again.
#[debug_handler]
async fn reset_page(
    ui: Ui,
    State(ctx): State<AppContext>,
    Path(token): Path<String>,
) -> Result<Response> {
    if users::Model::find_by_reset_token(&ctx.db, &token)
        .await
        .is_err()
    {
        return Ok(expired(&ui, "/forgot").into_response());
    }
    let body = views::account::reset(&ui, &format!("/reset/{token}"), &[]);
    Ok(ui.page("Choose a new password", body).into_response())
}

#[debug_handler]
async fn reset(
    ui: Ui,
    State(ctx): State<AppContext>,
    Path(token): Path<String>,
    Form(posted): Form<Vec<(String, String)>>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_reset_token(&ctx.db, &token).await else {
        return Ok(expired(&ui, "/forgot").into_response());
    };
    let mut form = Submitted::new(posted);
    let Some(password) = form.required::<String>("password") else {
        let errors = form.errors();
        let body = views::account::reset(&ui, &format!("/reset/{token}"), &errors.pairs());
        return Ok(ui.page("Choose a new password", body).into_response());
    };
    user.into_active_model()
        .reset_password(&ctx.db, &password)
        .await?;
    Ok(ui
        .redirect("/signin")
        .ok("Password changed. Sign in with the new one.")
        .into_response())
}

#[debug_handler]
async fn magic_page(ui: Ui) -> Result<Page> {
    Ok(ui.page("Email me a link", views::account::magic(&ui, &[], &[])))
}

/// Mails a one-time sign-in link; the same answer whether or not the email has an account.
#[debug_handler]
async fn magic(
    ui: Ui,
    State(ctx): State<AppContext>,
    Form(posted): Form<Vec<(String, String)>>,
) -> Result<Response> {
    let mut form = Submitted::new(posted);
    let Some(email) = form.required::<String>("email") else {
        let errors = form.errors();
        let body = views::account::magic(&ui, form.values(), &errors.pairs());
        return Ok(ui.page("Email me a link", body).into_response());
    };
    if let Ok(user) = users::Model::find_by_email(&ctx.db, &email).await {
        let user = user.into_active_model().create_magic_link(&ctx.db).await?;
        AuthMailer::send_magic_link(&ctx, &user).await?;
    }
    Ok(ui.redirect("/signin").ok(SENT).into_response())
}

/// The link in the magic-link email: signs in once, then the link is spent.
#[debug_handler]
async fn magic_signin(
    ui: Ui,
    State(ctx): State<AppContext>,
    Path(token): Path<String>,
) -> Result<Response> {
    let Ok(user) = users::Model::find_by_magic_token(&ctx.db, &token).await else {
        return Ok(expired(&ui, "/magic-link").into_response());
    };
    let user = user.into_active_model().clear_magic_link(&ctx.db).await?;
    welcome(&ui, &ctx, &user, &format!("Signed in as {}.", user.name))
}

/// A spent or unknown link, with the form that sends a new one.
fn expired(ui: &Ui, again: &str) -> Page {
    ui.page("Link expired", views::account::expired(ui, again))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/signin", get(signin_page).post(signin))
        .add("/signup", get(signup_page).post(signup))
        .add("/signout", post(signout))
        .add("/verify/{token}", get(verify))
        .add("/forgot", get(forgot_page).post(forgot))
        .add("/reset/{token}", get(reset_page).post(reset))
        .add("/magic-link", get(magic_page).post(magic))
        .add("/magic-link/{token}", get(magic_signin))
}
