# loco-ui on Loco

[Loco](https://loco.rs) is a Rails-style framework on Axum, so a Loco controller is an Axum
handler and takes `ui: Ui` like any other. The `loco` feature adds the one piece of wiring
Loco needs, a form helper and a scaffold. The API reference is the `loco_ui::loco` module
docs, whose examples are compiled and run by `cargo test -p loco-ui --features loco --doc
loco`. [`examples/loco-app`](../examples/loco-app) is all of it in one app: sign-in and a
notes model, every page working with script off, tested through Loco's own router and Blitz.

## Install

One command sets a Loco app up: it adds the two dependencies below, the initializer line,
the scaffold templates and a `src/views/layout.rs` (a page with a header, registered in
`src/views/mod.rs`). Running it again changes nothing, and a file you edited is kept
(`--force` overwrites the templates and the layout).

```sh
cargo install --git https://github.com/luis-serrano-l/loco-ui loco-ui --bin cargo-lui
cargo lui install            # in the app's directory, or `cargo lui install path/to/app`
```

`loco-ui/tests/install.rs` runs it on a fresh `loco new` app and `cargo check`s the result.
By hand, the same steps are the rest of this page:

```toml
# Cargo.toml of the Loco app (the crate is not on crates.io yet: use git or a path)
loco-ui = { git = "https://github.com/luis-serrano-l/loco-ui", features = ["loco"] }
maud = "0.27" # `html!` expands to `maud::` paths, so the app names it too
```

## The initializer line

```rust
// src/app.rs, in `impl Hooks for App`
async fn initializers(_ctx: &AppContext) -> Result<Vec<Box<dyn Initializer>>> {
    Ok(vec![Box::new(loco_ui::loco::Initializer)])
}
```

It mounts the enhancement script (`/lui/enhance.js`) and the capability beacon
(`/lui/caps`) beside the app's routes, plus the layer that drops the stylesheet from
enhanced responses. Without it pages still work: the script 404s and every visitor gets the
baseline variant.

## A controller

`Page`, `Redirect` and Loco's `Error` all implement `IntoResponse`, so a handler returns
`Result<Page>` and uses `?` on model calls, or `Result<Response>` when branches answer
differently. Mutations are forms that post and redirect (Post/Redirect/Get), with a flash
message for the next page:

```rust
use loco_ui::prelude::*;
use loco_rs::prelude::*;

async fn show(ui: Ui, State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Page> {
    let note = notes::Entity::find_by_id(id).one(&ctx.db).await?.ok_or(Error::NotFound)?;
    Ok(ui.page(&note.title, views::notes::show(&ui, &note)))
}

async fn remove(ui: Ui, State(ctx): State<AppContext>, Path(id): Path<i64>) -> Result<Redirect> {
    notes::Entity::delete_by_id(id).exec(&ctx.db).await?;
    Ok(ui.redirect("/notes").ok("Deleted."))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("/notes")
        .add("/{id}", get(show))
        .add("/{id}/delete", post(remove))
}
```

The flash and the UI state are unsigned `lui-*` cookies, so there is no key to configure and
no clash with Loco's JWT cookie. Paging with SeaORM (`paginate`, `num_items`, `fetch_page`
into `.paged(total)`) is a doctest in the module docs.

## A form with validation

On bad input the handler shows the form again with what was typed and a message on each
field. `Valid<T>` is the extractor for that: where Loco's `FormValidate` answers a bad form
with an error response, `Valid` hands the handler `Ok(T)` or `Err(Invalid)` (the messages
by field and what was posted), so create and update are one `match`. The scaffold uses it.

```rust
use loco_ui::{loco::Valid, prelude::*};
use loco_rs::prelude::*;
use serde::Deserialize;

#[derive(Deserialize, Validate)]
struct NewNote {
    #[validate(length(max = 80, message = "At most 80 characters."))]
    title: String,
    due: Option<Date>,
    #[serde(default, deserialize_with = "loco_ui::loco::checkbox")]
    done: bool,
}

async fn create(
    ui: Ui,
    State(ctx): State<AppContext>,
    Valid(form): Valid<NewNote>,
) -> Result<Response> {
    let note = match form {
        Ok(note) => note,
        Err(bad) => {
            let body = html! {
                (ui.form("/notes").text("title", "Title").required()
                    .date("due", "Due", "1900-01-01", "2100-12-31").checkbox("done", "Done")
                    .values(&bad.values).errors(&bad.errors.pairs()).submit("Save"))
            };
            return Ok(ui.page("New note", body).into_response());
        }
    };
    let note = notes::ActiveModel {
        title: Set(note.title), due: Set(note.due), done: Set(note.done), ..Default::default()
    }
    .insert(&ctx.db)
    .await?;
    Ok(ui.redirect(&format!("/notes/{}", note.id)).ok("Created.").into_response())
}
```

How it reads the form: values are trimmed and empty ones left out, so an empty field is
`None` for an `Option` and "This field is required." otherwise; a value that does not parse
gets "Check this field."; then the `#[validate(..)]` rules run. Every field in error gets its
message at once, not only the first. A checkbox posts `on`, which serde does not read as a
`bool`, hence `loco_ui::loco::checkbox`.

Two lower-level pieces, for handlers that do not fit a struct:

- `FieldErrors::from(&errors)` turns `validator`'s `ValidationErrors`, or the
  `ModelValidationErrors` inside `Error::Validation` / `ModelError::Validation` (a model's
  own rules, checked on save), into `(field, message)` pairs for `.errors(..)`.
- `Submitted`, from `Form<Vec<(String, String)>>`, parses field by field: `required::<T>`,
  `optional::<T>` and `checkbox`, recording the same messages as it goes. The account pages
  from `cargo lui auth` use it, since sign-in checks the pair against the database.

## Sign-in without script

`cargo lui auth` (after `cargo lui install`) writes every account page as a no-script form on
the starter's `users` model and `AuthMailer`: sign in, sign up (which mails a verification
link), sign out, forgot and reset password, email verification and magic link. It writes
`src/controllers/account.rs` and `src/views/account.rs`, registers them, points the
starter's mail links at the pages (`/verify/<token>`, `/reset/<token>`,
`/magic-link/<token>`) and does the cookie setting below in each `config/*.yaml`. The
forgot and magic-link forms give the same answer whether or not an account exists, and each
mailed link works once. `examples/loco-app` runs exactly these files, through Loco's router
and Blitz. Mail needs a `mailer:` section (`smtp` on 1025 with Mailpit in development,
`stub: true` in tests).

Loco's `auth::JWT` extractor reads the token from a header by default, which a plain form
cannot send. Point it at a cookie and have the sign-in form set that cookie on its redirect:

```yaml
# config/development.yaml
auth:
  jwt:
    location:
      from: Cookie
      name: auth
    secret: <your secret>
    expiration: 604800
```

```rust
let token = user.generate_jwt(&jwt.secret, jwt.expiration)?;
let cookie = format!("auth={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}", jwt.expiration);
Ok(ui.redirect("/").cookie(cookie).ok("Signed in."))
```

Signing out posts to a route that sets the same cookie with `Max-Age=0`.
`examples/loco-app/src/controllers/account.rs` has every account page in full. A
route with `auth::JWT` answers Loco's JSON 401 to a signed-out visitor; link to the sign-in
page from anywhere a visitor may arrive signed out.

## The generator

`loco-ui/loco-templates/` overrides Loco's scaffold so that `cargo loco generate scaffold`
writes an HTML controller and Maud views instead of a JSON API and DTOs:

```sh
cargo lui install   # or: cp -r path/to/loco-ui/loco-ui/loco-templates .loco-templates
cargo loco generate scaffold note title:string! body:text done:bool! due:date
cargo fmt
```

That writes the migration and model as usual, then `src/controllers/notes.rs` (list with
paging, show, new, create, edit, update, delete; every change a form post and a redirect with
a flash; the form re-rendered with values and messages on bad input) and `src/views/notes.rs`
(`list`, `show`, `form`, `values`), and registers the routes in `app.rs`. The handlers take
`auth::JWT` unless you pass `--no-auth`. Needs: a `src/views/mod.rs`, a
`tests/models/mod.rs` (the model generator adds a test there), and `sea-orm-cli` 2 for the
entities step. Field kinds map to inputs: `text` → textarea, `bool` → checkbox, `date` → date
picker, enums → select, date-times → a text field in `2026-01-31T09:00:00` form, the rest →
a text field parsed with `FromStr`. Array columns are not supported.

## Loco settings that affect pages

- `secure_headers` with the default `github` preset sends `script-src https:`: over plain
  `http://` in development the enhancement script is blocked and pages behave as without it.
- The `owasp` preset sends `Clear-Site-Data: "cache","cookies","storage"` on every response,
  which wipes the flash, the theme and the sign-in cookie on each page. Use `github`, or
  override that header.
- loco-ui's strict `csp` layer is not added for you; add
  `axum::middleware::from_fn(loco_ui::enhance::csp)` in `after_routes` to use it.

## Views: Maud, not Tera

Loco's generators write Tera templates under `assets/views/` and render them with
`format::render().view(&v, "notes/list.html", data!({..}))`. With loco-ui the views are Rust:
a `src/views/` module of plain functions that take `&Ui` and the data and return `Markup`. A
controller calls one and returns the page.

```rust
// src/views/notes.rs
use loco_ui::{prelude::*, table::Row};
use crate::models::_entities::notes;

pub fn list(ui: &Ui, rows: &[notes::Model], total: usize) -> Markup {
    let table = ui.table("notes", "/notes").column("title", "Title").sortable();
    html! {
        (ui.flash())
        (table.rows(rows.iter().map(|n| Row::new([html! { a href={ "/notes/" (n.id) } { (n.title) } }]))).paged(total))
        (ui.link_button("New note", "/notes/new"))
    }
}

pub fn form(ui: &Ui, action: &str, title: &str, errors: &[(&str, &str)]) -> Markup {
    html! {
        (ui.form(action).text("title", "Title").required().value(title).errors(errors).submit("Save"))
    }
}
```

```rust
// src/controllers/notes.rs
async fn list(ui: Ui, State(ctx): State<AppContext>) -> Result<Page> {
    let (rows, total) = /* the paging query in the `loco` module docs */;
    Ok(ui.page("Notes", views::notes::list(&ui, &rows, total)))
}
```

Tera and Maud views live side by side: each controller picks how it renders, so an existing
app can move one controller at a time. The two need nothing from each other.

**No Tera bridge.** A Tera function such as `{{ lui_button(label="Save") }}` was considered
and left out:

- A component starts from `ui`: the browser's capabilities, the query and the `lui-ui` state
  (which tab is open, the page size, the flash). A Tera function gets only JSON arguments, so
  every template would have to pass that state through by hand, and a forgotten argument
  renders the wrong variant silently.
- Builders are typed chains (`.required()`, `.maxlength(80)`, `.column(..).sortable()`); as
  keyword arguments they lose the compiler's checks, and a typo is a runtime error on the page.
- Maud escapes by default and checks the markup at compile time; Tera output would have to be
  marked `| safe`.

A Tera page can still link to or embed a Maud-rendered fragment from its own route; there is
nothing to share at template level.
