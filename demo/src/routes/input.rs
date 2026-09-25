//! Input: combobox, validated form, wizard, and select, range and colour.

use crate::site::page;
use axum::{
    Form, Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_nojs::prelude::*;
use axum_nojs::wizard::{Posted, Wizard};
use serde::{Deserialize, Serialize};

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/combobox", get(combobox_page))
        .route("/combobox/new", post(combobox_new))
        .route("/form", get(form_page).post(form_submit))
        .route("/wizard", get(wizard_page).post(wizard_submit))
        .route("/inputs", get(inputs_page).post(inputs_submit))
}

async fn combobox_page(ui: Ui) -> Page {
    page(
        &ui,
        "Combobox",
        nojs! {
            (ui.flash())
            // One swap root around the form and its results: the script searches as you type.
            div id="langs" data-nojs="swap" {
                // code: /combobox
                Combobox("q", "/combobox") multi create="/combobox/new"
                    label="Language" placeholder="Type a language" {
                    group "Systems" (["Rust", "Zig", "Swift"]);
                    group "Scripting" (["Ruby", "Python", "Racket"]);
                    options(["Prolog", "Scala"]);
                }
                // end code
            }
            p class="nojs-note" { "Pick several: each result adds a chip, each chip's \u{d7} removes it, and the chips ride along with the next search. Type a language that is not here to get a Create row." }
        },
    )
}

#[derive(Deserialize)]
struct NewLang {
    name: String,
    #[serde(default)]
    sel: Vec<String>,
}

async fn combobox_new(ui: Ui, Form(f): Form<NewLang>) -> Redirect {
    let name = f.name.trim();
    let to: String = f
        .sel
        .iter()
        .map(String::as_str)
        .chain([name])
        .map(|s| format!("&sel={s}"))
        .collect();
    ui.redirect(&format!("/combobox?q={to}")).flash(&format!(
        "Added {name} (not really: the demo has no database)."
    ))
}

/// What the wizard has collected so far, as the steps posted it.
#[derive(Default, Deserialize, Serialize)]
struct Signup(Vec<(String, String)>);

impl Signup {
    fn get(&self, k: &str) -> &str {
        self.0
            .iter()
            .find(|(n, _)| n == k)
            .map_or("", |(_, v)| v.trim())
    }
}

fn signup<'a>(ui: &'a Ui, s: &'a Signup, errors: &'a [(&'a str, &'a str)]) -> Wizard<'a> {
    // code: /wizard
    ui.wizard("signup", "/wizard")
        .step(
            "Account",
            ui.fields()
                .text("name", "Name")
                .required()
                .email("email", "Email")
                .required(),
        )
        .step(
            "Newsletter",
            ui.fields()
                .select("digest", "Digest", ["daily", "weekly", "never"])
                .text("topics", "Topics")
                .placeholder("rust, html"),
        )
        .optional()
        .review("Review")
        .values(&s.0)
        .errors(errors)
        .finish("Create account")
    // end code
}

/// Server rules for a wizard step: `(field, message)` per problem.
fn signup_errors(step: usize, s: &Signup) -> Vec<(&'static str, &'static str)> {
    let (name, email) = (s.get("name"), s.get("email"));
    match step {
        0 if name.is_empty() => vec![("name", "Enter your name.")],
        0 if !email.contains('@') => vec![("email", "Enter an email address with an @.")],
        0 if email.ends_with("@example.com") => {
            vec![("email", "example.com addresses are not accepted.")]
        }
        _ => Vec::new(),
    }
}

fn wizard_view(ui: &Ui, wizard: Wizard) -> Page {
    page(
        ui,
        "Wizard",
        html! {
            (ui.flash())
            p { "Three steps, one form each. The server checks every step; the second can be skipped. Close the tab and come back to " a href="/wizard" { "/wizard" } ": you resume where you left off." }
            (wizard)
        },
    )
}

async fn wizard_page(ui: Ui, Saved(s): Saved<Signup>) -> Page {
    wizard_view(&ui, signup(&ui, &s, &[]))
}

/// Check the posted step: answer 422 with the same step and its messages, or keep the fields
/// (a year, so closing the browser loses nothing) and redirect to the next step.
async fn wizard_submit(
    ui: Ui,
    Saved(mut s): Saved<Signup>,
    Form(pairs): Form<Vec<(String, String)>>,
) -> Response {
    let posted = Posted::from_pairs(&pairs);
    for (k, v) in pairs
        .into_iter()
        .filter(|(k, _)| !posted.skip && k != "step" && k != "skip")
    {
        s.0.retain(|(n, _)| *n != k);
        s.0.push((k, v));
    }
    let errors = if posted.skip {
        Vec::new()
    } else {
        signup_errors(posted.step, &s)
    };
    let wizard = signup(&ui, &s, &errors).at(posted.step);
    if !errors.is_empty() {
        return (StatusCode::UNPROCESSABLE_ENTITY, wizard_view(&ui, wizard)).into_response();
    }
    if wizard.is_last(posted.step) {
        return ui
            .redirect(&wizard.link(0))
            .flash("Account created (well, the cookie was cleared).")
            .forget::<Signup>()
            .into_response();
    }
    ui.redirect(&wizard.link(posted.step + 1))
        .save(&s)
        .into_response()
}

fn form_view(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Page {
    let inline = ui.param("layout") == Some("inline");
    // code: /form
    let form = nojs! {
        Form("/form") submit="Sign up" values=(values) errors=(errors) inline[inline] {
            group "Account";
            text "name" "Name" required;
            email "email" "Email" required;
            number "age" "Age" 13 120 required;
            pattern "handle" "Handle" "[a-z0-9_]{3,16}" "3–16 lowercase letters, digits or _" required;
            select "plan" "Plan" (["Free", "Team", "Enterprise"]);
            group "Profile";
            textarea "bio" "Bio" 3 maxlength=160 help="Grows as you type where the browser supports it.";
            file "avatar" "Avatar" "image/png,image/jpeg" help="PNG or JPEG.";
            date "start" "Start date" "2026-01-01" "2027-12-31";
            time "call" "Best time to call" "09:00" "17:00" help="Office hours, 09:00 to 17:00.";
        }
    };
    // end code
    page(
        ui,
        "Validated form",
        html! {
            (ui.flash())
            p { "Labels " @if inline { "beside the fields. " a href="/form" { "Put them above" } } @else { "above the fields. " a href="/form?layout=inline" { "Put them beside" } } "." }
            @if !errors.is_empty() { p class="nojs-error" { "Server-side checks failed. Browser validation passed, these rules only live on the server." } }
            (form)
        },
    )
}

async fn form_page(ui: Ui) -> Page {
    form_view(&ui, &[], &[])
}

/// A multipart post (the avatar is a file): server rules, then PRG with a flash or the form again.
async fn form_submit(ui: Ui, mut parts: axum::extract::Multipart) -> Response {
    let (mut values, mut files) = (Vec::new(), Vec::new());
    while let Ok(Some(part)) = parts.next_field().await {
        let (name, file) = (
            part.name().unwrap_or("").to_string(),
            part.file_name().map(str::to_string),
        );
        match file {
            Some(f) => {
                let n = part.bytes().await.map_or(0, |b| b.len());
                if !f.is_empty() {
                    files.push(format!(" with {f} ({n} bytes)"));
                }
            }
            None => values.push((name, part.text().await.unwrap_or_default())),
        }
    }
    let get = |k: &str| {
        values
            .iter()
            .find(|(n, _)| n == k)
            .map_or("", |(_, v)| v.as_str())
    };
    let mut errors = Vec::new();
    if get("handle") == "admin" {
        errors.push(("handle", "That handle is reserved."));
    }
    if get("email").ends_with("@example.com") {
        errors.push(("email", "example.com addresses are not accepted."));
    }
    if errors.is_empty() {
        return ui
            .redirect("/form")
            .flash(&format!(
                "Signed up as {}{}.",
                get("handle"),
                files.concat()
            ))
            .into_response();
    }
    (
        StatusCode::UNPROCESSABLE_ENTITY,
        form_view(&ui, &values, &errors),
    )
        .into_response()
}

/// The inputs page's values: from the query while filtering (unsaved), else saved.
#[derive(Deserialize, Serialize, Default)]
struct Inputs {
    size: Option<String>,
    volume: Option<i64>,
    accent: Option<String>,
    #[serde(rename = "accent-alpha")]
    alpha: Option<u8>,
    #[serde(rename = "accent-preset")]
    preset: Option<String>,
    price_min: Option<i64>,
    price_max: Option<i64>,
    country: Option<String>,
}

const SIZES: [(&str, &str, &str); 3] = [
    ("s", "Small", "🐭"),
    ("m", "Medium", "🐕"),
    ("l", "Large", "🐘"),
];
const ACCENTS: [&str; 5] = ["#1f6f5f", "#2f5bea", "#b3261e", "#8a5a00", "#6b3fa0"];
/// `(value, name, flag)`.
type Country = (&'static str, &'static str, &'static str);
const COUNTRIES: [(&str, [Country; 7]); 3] = [
    (
        "Europe",
        [
            ("es", "Spain", "🇪🇸"),
            ("fr", "France", "🇫🇷"),
            ("de", "Germany", "🇩🇪"),
            ("it", "Italy", "🇮🇹"),
            ("pt", "Portugal", "🇵🇹"),
            ("nl", "Netherlands", "🇳🇱"),
            ("se", "Sweden", "🇸🇪"),
        ],
    ),
    (
        "Americas",
        [
            ("us", "United States", "🇺🇸"),
            ("ca", "Canada", "🇨🇦"),
            ("mx", "Mexico", "🇲🇽"),
            ("br", "Brazil", "🇧🇷"),
            ("ar", "Argentina", "🇦🇷"),
            ("cl", "Chile", "🇨🇱"),
            ("co", "Colombia", "🇨🇴"),
        ],
    ),
    (
        "Asia",
        [
            ("jp", "Japan", "🇯🇵"),
            ("kr", "South Korea", "🇰🇷"),
            ("in", "India", "🇮🇳"),
            ("id", "Indonesia", "🇮🇩"),
            ("vn", "Vietnam", "🇻🇳"),
            ("th", "Thailand", "🇹🇭"),
            ("ph", "Philippines", "🇵🇭"),
        ],
    ),
];

/// Select, range and colour in one form, saved in `nojs-inputs`. The country filter is a GET
/// through the same form, so while filtering the values come from the query.
async fn inputs_page(ui: Ui, Query(q): Query<Inputs>, Saved(saved): Saved<Inputs>) -> Page {
    let v = if ui.param("country-q").is_some() {
        q
    } else {
        saved
    };
    page(
        &ui,
        "Select, range, colour",
        nojs! {
            (ui.flash())
            form id="inputs" data-nojs="swap" class="nojs-form" method="post" action="/inputs" {
                // code: /inputs
                Select("size", v.size.as_deref().unwrap_or("m")) options=(SIZES) label="Size";
                Select("country", v.country.as_deref().unwrap_or("es")) groups=(COUNTRIES) search="/inputs" label="Country";
                Range("volume", v.volume.unwrap_or(40)) step=5 label="Volume";
                RangePair("price", (v.price_min.unwrap_or(20), v.price_max.unwrap_or(80))) step=5 label="Price";
                Color("accent", v.accent.as_deref().unwrap_or("#1f6f5f")) presets=(&ACCENTS) alpha=(v.alpha.unwrap_or(100)) label="Accent";
                // end code
                (ui.button("Save").primary())
            }
            p class="nojs-note" { "Without the enhancement script the outputs and the swatch show the last saved values and update on submit, and the country filter needs its button." }
        },
    )
}

/// Only known sizes, countries and `#rrggbb` colours are kept; numbers are clamped.
async fn inputs_submit(ui: Ui, Form(f): Form<Inputs>) -> Redirect {
    let known = |v: &Option<String>, ok: &dyn Fn(&str) -> bool| v.clone().filter(|v| ok(v));
    let hex = |c: &str| c.len() == 7 && c.starts_with('#');
    let (lo, hi) = axum_nojs::range::order(
        f.price_min.unwrap_or(20).clamp(0, 100),
        f.price_max.unwrap_or(80).clamp(0, 100),
    );
    let clean = Inputs {
        size: known(&f.size, &|s| SIZES.iter().any(|(v, ..)| *v == s)),
        country: known(&f.country, &|c| {
            COUNTRIES
                .iter()
                .flat_map(|(_, cs)| cs)
                .any(|(v, ..)| *v == c)
        }),
        accent: known(&f.preset, &hex).or(known(&f.accent, &hex)),
        alpha: Some(f.alpha.unwrap_or(100).min(100)),
        volume: Some(f.volume.unwrap_or(40).clamp(0, 100)),
        price_min: Some(lo),
        price_max: Some(hi),
        preset: None,
    };
    ui.redirect("/inputs").flash("Inputs saved.").save(&clean)
}
