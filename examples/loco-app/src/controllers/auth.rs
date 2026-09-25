//! Sign up, sign in and sign out as plain forms. The token Loco's `auth::JWT` checks travels in
//! the `auth` cookie (`auth.jwt.location` in `config/*.yaml`), so no page needs script to send it.
use axum_nojs::{
    loco::{FieldErrors, Submitted},
    prelude::*,
};
use loco_rs::prelude::*;

use crate::{models::users, views};

/// The cookie `config/*.yaml` tells `auth::JWT` to read.
pub const COOKIE: &str = "auth";

fn signed_in(ctx: &AppContext, user: &users::Model) -> Result<String> {
    let jwt = ctx.config.get_jwt_config()?;
    let token = user.generate_jwt(&jwt.secret, jwt.expiration)?;
    Ok(format!(
        "{COOKIE}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        jwt.expiration
    ))
}

#[debug_handler]
async fn signin_page(ui: Ui) -> Result<Page> {
    Ok(ui.page("Sign in", views::auth::signin(&ui, &[], &[])))
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
            .await?
            .filter(|u| u.verify_password(&password)),
        _ => None,
    };
    let Some(user) = user else {
        let errors = form.errors();
        let mut pairs = errors.pairs();
        if pairs.is_empty() {
            pairs.push(("password", "Wrong email or password."));
        }
        let body = views::auth::signin(&ui, form.values(), &pairs);
        return Ok(ui.page("Sign in", body).into_response());
    };
    let cookie = signed_in(&ctx, &user)?;
    Ok(ui
        .redirect("/notes")
        .cookie(cookie)
        .ok(&format!("Signed in as {}.", user.name))
        .into_response())
}

#[debug_handler]
async fn signup_page(ui: Ui) -> Result<Page> {
    Ok(ui.page("Sign up", views::auth::signup(&ui, &[], &[])))
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
            match users::Model::create_with_password(&ctx.db, &name, &email, &password).await {
                Ok(user) => {
                    let cookie = signed_in(&ctx, &user)?;
                    return Ok(ui
                        .redirect("/notes")
                        .cookie(cookie)
                        .ok("Welcome.")
                        .into_response());
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
    let body = views::auth::signup(&ui, form.values(), &pairs);
    Ok(ui.page("Sign up", body).into_response())
}

#[debug_handler]
async fn signout(ui: Ui) -> Result<Redirect> {
    let gone = format!("{COOKIE}=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0");
    Ok(ui.redirect("/").cookie(gone).ok("Signed out."))
}

pub fn routes() -> Routes {
    Routes::new()
        .add("/signin", get(signin_page).post(signin))
        .add("/signup", get(signup_page).post(signup))
        .add("/signout", post(signout))
}
