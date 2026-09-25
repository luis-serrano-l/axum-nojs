//! Complete flows: the demo app under `/app`, sign in then notes.

use crate::site::page;
use axum::{
    Form, Router,
    extract::Query,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use loco_ui::Row;
use loco_ui::prelude::*;
use serde::{Deserialize, Serialize};

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/app/signin", get(signin_page).post(signin_submit))
        .route("/app/signout", post(signout))
        .route("/app/notes", get(notes_page).post(note_add))
        .route("/app/notes/edit", post(note_edit))
        .route("/app/notes/delete", post(note_delete))
}

/// Who is signed in to the demo app (`/app`), in the saved cookie.
#[derive(Default, Deserialize, Serialize)]
struct Session {
    email: String,
}

/// The signed-in visitor's notes, `(id, text)`, newest last, at most 20.
#[derive(Default, Deserialize, Serialize)]
struct AppNotes(Vec<(String, String)>);

/// Each page's live component, which the index shows too (`site::preview`): the sign-in card,
/// and the notes of someone who has none yet.
pub(crate) const PREVIEWS: &[super::Preview] = &[
    ("/app/signin", |ui| signin_card(ui, &[], &[])),
    ("/app/notes", |ui| notes(ui, &AppNotes::default())),
];

fn signin_view(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Page {
    page(
        ui,
        "Sign in",
        html! {
            (ui.flash())
            (signin_card(ui, values, errors))
        },
    )
}

/// The sign-in form in a card, with the values and messages of a refused post.
fn signin_card(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    // code: /app/signin
    let form = lui! {
        Form("/app/signin") submit="Sign in" values=(values) errors=(errors) {
            email "email" "Email" required placeholder="you@example.com";
            password "password" "Password" required
                help="At least 8 characters. Try a short one to see the server's answer.";
        }
    };
    // end code
    html! { (ui.card().title("Sign in").description("Any email and a password of 8 characters or more.").body(html! { (form) })) }
}

async fn signin_page(ui: Ui) -> Page {
    signin_view(&ui, &[], &[])
}

/// The server checks what the browser already did, and more; mistakes re-render the form with
/// the email kept and the password not.
async fn signin_submit(ui: Ui, Form(f): Form<Vec<(String, String)>>) -> Response {
    let get = |k: &str| {
        f.iter()
            .find(|(n, _)| n == k)
            .map_or("", |(_, v)| v.as_str())
    };
    let (email, password) = (get("email").trim().to_string(), get("password"));
    let mut errors = Vec::new();
    if !email.contains('@') || email.len() > 80 {
        errors.push(("email", "Enter an email address, like you@example.com."));
    }
    if password.chars().count() < 8 {
        errors.push(("password", "The password needs at least 8 characters."));
    }
    if errors.is_empty() {
        let msg = format!("Signed in as {email}.");
        return ui
            .redirect("/app/notes")
            .ok(&msg)
            .save(&Session { email })
            .into_response();
    }
    signin_view(&ui, &[("email".to_string(), email)], &errors).into_response()
}

async fn signout(ui: Ui) -> Redirect {
    ui.redirect("/app/signin")
        .flash("Signed out.")
        .forget::<Session>()
}

async fn notes_page(ui: Ui, Saved(session): Saved<Session>, Saved(saved): Saved<AppNotes>) -> Page {
    if session.email.is_empty() {
        return page(
            &ui,
            "Notes",
            html! {
                (ui.flash())
                (ui.empty_state("Sign in to see your notes")
                    .body(html! { "Your notes are kept in a cookie for this browser." })
                    .link("Sign in", "/app/signin"))
            },
        );
    }
    page(
        &ui,
        "Notes",
        html! {
            (ui.flash())
            (ui.stack().gap(6).body(html! {
                (ui.cluster().between().body(html! {
                    span class="lui-note" { "Signed in as " strong { (session.email) } }
                    form method="post" action="/app/signout" { (ui.button("Sign out").ghost().small()) }
                }))
                (notes(&ui, &saved))
            }))
        },
    )
}

/// The form that adds a note over the table of notes: filter, sort, rename, delete, pages.
fn notes(ui: &Ui, notes: &AppNotes) -> Markup {
    // code: /app/notes
    let t = ui
        .table("notes", "/app/notes")
        .column("text", "Note")
        .sortable()
        .editable()
        .column("id", "No.")
        .numeric()
        .width("5rem");
    let q = t.filter().to_lowercase();
    let mut rows: Vec<&(String, String)> = notes
        .0
        .iter()
        .filter(|(_, n)| n.to_lowercase().contains(&q))
        .collect();
    if let Some((_, desc)) = t.sort() {
        rows.sort_by_key(|n| n.1.to_lowercase());
        if desc {
            rows.reverse();
        }
    }
    let total = rows.len();
    let deletes: Vec<String> = rows
        .iter()
        .map(|(id, _)| format!("/app/notes/delete?id={id}"))
        .collect();
    let t = t
        .rows(rows.iter().zip(&deletes).map(|((id, text), del)| {
            Row::new([html! { (text) }, html! { (id) }])
                .key(id)
                .values([text.as_str(), ""])
                .menu([MenuItem::action("Delete", del).danger()])
        }))
        .paged(total)
        .edit("/app/notes/edit")
        .empty("No notes yet: add one above.");
    let add = lui! {
        Form("/app/notes") submit="Add note" {
            text "text" "New note" required maxlength=60 placeholder="Buy milk";
        }
    };
    // end code
    html! { (add) (t) }
}

#[derive(Deserialize)]
struct NewNote {
    text: String,
}

/// Add, then back to the list with a flash (Post/Redirect/Get). Twenty notes at most.
async fn note_add(ui: Ui, Saved(mut notes): Saved<AppNotes>, Form(n): Form<NewNote>) -> Redirect {
    let text: String = n.text.trim().chars().take(60).collect();
    if text.is_empty() {
        return ui.redirect("/app/notes").warn("A note needs some text.");
    }
    let next = notes
        .0
        .iter()
        .filter_map(|(id, _)| id.parse::<u32>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    notes.0.push((next.to_string(), text));
    let over = notes.0.len().saturating_sub(20);
    notes.0.drain(..over);
    ui.redirect("/app/notes").ok("Note added.").save(&notes)
}

#[derive(Deserialize)]
struct EditedNote {
    key: String,
    text: String,
    returns_to: String,
}

async fn note_edit(
    ui: Ui,
    Saved(mut notes): Saved<AppNotes>,
    Form(e): Form<EditedNote>,
) -> Redirect {
    let text: String = e.text.trim().chars().take(60).collect();
    if let Some(n) = notes
        .0
        .iter_mut()
        .find(|(id, _)| *id == e.key)
        .filter(|_| !text.is_empty())
    {
        n.1 = text;
    }
    let back = if e.returns_to.starts_with("/app/notes") {
        &e.returns_to
    } else {
        "/app/notes"
    };
    ui.redirect(back).ok("Note saved.").save(&notes)
}

#[derive(Deserialize)]
struct NoteId {
    id: String,
}

async fn note_delete(
    ui: Ui,
    Saved(mut notes): Saved<AppNotes>,
    Query(q): Query<NoteId>,
) -> Redirect {
    notes.0.retain(|(id, _)| *id != q.id);
    ui.redirect("/app/notes").warn("Note deleted.").save(&notes)
}
