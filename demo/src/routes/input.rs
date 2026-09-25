//! Input: combobox, validated form, wizard, and select, range and colour.

use crate::site::page;
use axum::{
    Form, Router,
    extract::Query,
    routing::{get, post},
};
use loco_ui::prelude::*;
use loco_ui::wizard::Wizard;
use serde::{Deserialize, Serialize};

pub(crate) fn routes() -> Router {
    super::pages(PAGES)
        .route("/combobox/new", post(combobox_new))
        .route("/form", get(form_page).post(form_submit))
        .route("/wizard", get(wizard_page).post(wizard_submit))
        .route("/inputs", get(inputs_page).post(inputs_submit))
}

/// The pages that are their component and a note (`super::pages`).
pub(crate) const PAGES: &[super::Simple] = &[
    (
        "/combobox",
        combobox,
        "Pick several: each result adds a chip, each chip's \u{d7} removes it, and the chips ride along with the next search. Type a language that is not here to get a Create row.",
    ),
    ("/toggle-group", toggle_group, ""),
    ("/otp", otp, ""),
];

/// The other pages' live components, which the index shows too (`site::preview`), empty
/// there.
pub(crate) const PREVIEWS: &[super::Preview] = &[
    ("/form", |ui| signup_form(ui, &[], &[])),
    ("/wizard", |ui| signup(ui, &Signup::default(), &[]).render()),
    ("/inputs", |ui| inputs(ui, &Inputs::default())),
];

fn combobox(ui: &Ui) -> Markup {
    lui! {
            // One swap root around the form and its results: the script searches as you type.
            div id="langs" data-lui="swap" {
                // code: /combobox
                Combobox("q", "/combobox") multiple create="/combobox/new"
                    label="Language" placeholder="Type a language" {
                    group "Systems" (["Rust", "Zig", "Swift"]);
                    group "Scripting" (["Ruby", "Python", "Racket"]);
                    options(["Prolog", "Scala"]);
                }
                // end code
            }
    }
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
    let body = html! {
        p { "Three steps, one form each. The server checks every step; the second can be skipped. Close the tab and come back to " a href="/wizard" { "/wizard" } ": you resume where you left off." }
        (wizard)
    };
    page(ui, "Wizard", body)
}

async fn wizard_page(ui: Ui, Saved(s): Saved<Signup>) -> Page {
    wizard_view(&ui, signup(&ui, &s, &[]))
}

/// Check the posted step: answer 422 with the same step and its messages, or keep the fields
/// (a year, so closing the browser loses nothing) and redirect to the next step.
async fn wizard_submit(
    ui: Ui,
    Saved(mut s): Saved<Signup>,
    posted: Posted,
) -> Result<Redirect, Page> {
    let (step, skip) = (posted.step(), posted.skip());
    for (k, v) in posted
        .pairs()
        .iter()
        .filter(|(k, _)| !skip && k != "step" && k != "skip")
    {
        s.0.retain(|(n, _)| n != k);
        s.0.push((k.clone(), v.clone()));
    }
    let errors = if skip {
        Vec::new()
    } else {
        signup_errors(step, &s)
    };
    let wizard = signup(&ui, &s, &errors).at(step);
    if !errors.is_empty() {
        return Err(wizard_view(&ui, wizard).invalid());
    }
    if wizard.is_last(step) {
        let done = "Account created (well, the cookie was cleared).";
        return Ok(ui.redirect(&wizard.link(0)).flash(done).forget::<Signup>());
    }
    Ok(ui.redirect(&wizard.link(step + 1)).save(&s))
}

/// The sign-up form, with the values and messages of a post the server refused.
fn signup_form(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Markup {
    let inline = ui.param("layout") == Some("inline");
    // code: /form
    let form = lui! {
        Form("/form") submit="Sign up" values=(values) errors=(errors) inline[inline] {
            group "Account";
            text "name" "Name" required;
            email "email" "Email" required;
            number "age" "Age" 13 120 required;
            pattern "handle" "Handle" "[a-z0-9_]{3,16}" "3–16 lowercase letters, digits or _" required;
            select "plan" "Plan" ([("free", "Free"), ("team", "Team"), ("enterprise", "Enterprise")]);
            group "Profile";
            textarea "bio" "Bio" 3 maxlength=160 help="Grows as you type where the browser supports it.";
            file "avatar" "Avatar" "image/png,image/jpeg" help="PNG or JPEG.";
            date "start" "Start date" "2026-01-01" "2027-12-31";
            time "call" "Best time to call" "09:00" "17:00" help="Office hours, 09:00 to 17:00.";
            datetime "demo" "Demo" "2026-01-01T09:00" "2027-12-31T17:00" help="A date and a time in one field.";
        }
    };
    // end code
    form
}

fn form_view(ui: &Ui, values: &[(String, String)], errors: &[(&str, &str)]) -> Page {
    let inline = ui.param("layout") == Some("inline");
    let body = html! {
        p { "Labels " @if inline { "beside the fields. " a href="/form" { "Put them above" } } @else { "above the fields. " a href="/form?layout=inline" { "Put them beside" } } "." }
        p class="lui-note" { "The handle " code { "admin" } " and " code { "@example.com" } " addresses pass the browser's checks and fail the server's: the form comes back with an error summary on top that takes the focus and links to each field. " a href="/form?errors=1" { "See it" } "." }
        (signup_form(ui, values, errors))
    };
    page(ui, "Validated form", body)
}

/// `?errors=1` shows the answer to a post the server refused, without posting.
async fn form_page(ui: Ui) -> Page {
    if ui.param("errors").is_none() {
        return form_view(&ui, &[], &[]);
    }
    let values = [
        ("name", "Ada"),
        ("email", "ada@example.com"),
        ("handle", "admin"),
    ];
    let values: Vec<(String, String)> = values.map(|(n, v)| (n.into(), v.into())).into();
    let errors = [
        ("email", "example.com addresses are not accepted."),
        ("handle", "That handle is reserved."),
    ];
    form_view(&ui, &values, &errors)
}

/// A multipart post (the avatar is a file): server rules, then PRG with a flash or the form
/// again with 422.
async fn form_submit(ui: Ui, posted: Posted) -> Result<Redirect, Page> {
    let mut errors = Vec::new();
    if posted.get("handle") == "admin" {
        errors.push(("handle", "That handle is reserved."));
    }
    if posted.get("email").ends_with("@example.com") {
        errors.push(("email", "example.com addresses are not accepted."));
    }
    if !errors.is_empty() {
        return Err(form_view(&ui, posted.pairs(), &errors).invalid());
    }
    let files: String = posted
        .files()
        .iter()
        .map(|f| format!(" with {} ({} bytes)", f.file_name, f.bytes.len()))
        .collect();
    let handle = posted.get("handle");
    Ok(ui
        .redirect("/form")
        .flash(&format!("Signed up as {handle}{files}.")))
}

/// The inputs page's values: from the query while filtering (unsaved), else saved; a missing
/// one is its default.
#[derive(Deserialize, Serialize)]
#[serde(default)]
struct Inputs {
    size: String,
    volume: i64,
    accent: String,
    #[serde(rename = "accent-alpha")]
    alpha: u8,
    #[serde(rename = "accent-preset")]
    preset: Option<String>,
    price_min: i64,
    price_max: i64,
    country: String,
}

impl Default for Inputs {
    fn default() -> Self {
        Inputs {
            size: "m".into(),
            volume: 40,
            accent: ACCENTS[0].into(),
            alpha: 100,
            preset: None,
            price_min: 20,
            price_max: 80,
            country: "es".into(),
        }
    }
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

/// Select, range and colour in one form, saved in `lui-inputs`. The country filter is a GET
/// through the same form, so while filtering the values come from the query.
async fn inputs_page(ui: Ui, Query(q): Query<Inputs>, Saved(saved): Saved<Inputs>) -> Page {
    let v = if ui.param("country-q").is_some() {
        q
    } else {
        saved
    };
    let body = lui! {
        (inputs(&ui, &v))
        p class="lui-note" { "Without the enhancement script the outputs and the swatch show the last saved values and update on submit, and the country filter needs its button." }
    };
    page(&ui, "Select, range, colour", body)
}

/// The form of select, range and colour, showing `v`.
fn inputs(ui: &Ui, v: &Inputs) -> Markup {
    lui! {
        // code: /inputs
        Form("/inputs") submit="Save" {
            Select("size", "Size") value=(&v.size) options=(SIZES);
            Select("country", "Country") value=(&v.country) groups=(COUNTRIES) search="/inputs";
            Range("volume", "Volume") value=(v.volume) step=5;
            RangePair("price", "Price") values=(v.price_min, v.price_max) step=5;
            Color("accent", "Accent") value=(&v.accent) presets=(&ACCENTS) alpha=(v.alpha);
        }
        // end code
    }
}

/// Only known sizes, countries and `#rrggbb` colours are kept; numbers are clamped.
async fn inputs_submit(ui: Ui, Form(f): Form<Inputs>) -> Redirect {
    let d = Inputs::default();
    let hex = |c: &String| c.len() == 7 && c.starts_with('#');
    let size = SIZES.iter().any(|(v, ..)| *v == f.size);
    let country = COUNTRIES
        .iter()
        .flat_map(|(_, cs)| cs)
        .any(|(v, ..)| *v == f.country);
    let accent = f.preset.filter(hex).or(Some(f.accent).filter(hex));
    let (lo, hi) = loco_ui::range::order(f.price_min.clamp(0, 100), f.price_max.clamp(0, 100));
    let clean = Inputs {
        size: if size { f.size } else { d.size },
        country: if country { f.country } else { d.country },
        accent: accent.unwrap_or(d.accent),
        alpha: f.alpha.min(100),
        volume: f.volume.clamp(0, 100),
        price_min: lo,
        price_max: hi,
        preset: None,
    };
    ui.redirect("/inputs").flash("Inputs saved.").save(&clean)
}

/// One pick (alignment) and several (style), sent with the form they sit in, which says what
/// it was sent with.
fn toggle_group(ui: &Ui) -> Markup {
    let picked = |name| ui.params(name).collect::<Vec<_>>().join(", ");
    lui! {
        Form("/toggle-group") get submit="Apply" {
            // code: /toggle-group
            ToggleGroup("align", "Alignment") { option "left" "Left"; option "center" "Center"; option "right" "Right"; }
            ToggleGroup("style", "Text style") multiple {
                option "bold" "Bold" icon=(Icon::Bold); option "italic" "Italic" icon=(Icon::Italic);
            }
            // end code
            @if ui.param("align").is_some() { p class="lui-note" { "Alignment: " (picked("align")) ". Style: " (picked("style")) "." } }
        }
    }
}

/// A one-time code: one field the phone offers to fill from the message.
fn otp(ui: &Ui) -> Markup {
    lui! {
        Form("/otp") get submit="Verify" {
            // code: /otp
            InputOtp("code", "Code from the text message");
            // end code
            @if let Some(code) = ui.param("code") { p class="lui-note" { "Sent " code { (code) } "." } }
        }
    }
}
