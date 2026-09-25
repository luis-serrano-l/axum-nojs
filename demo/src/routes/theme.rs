//! The theme builder: colour inputs for the main `--lui-*` roles in light and dark and a
//! radius, sent as a GET form so the theme is in the URL; a preview of a few components under
//! those values; and `theme.css`, the overrides as a file to paste after the stylesheet. No
//! script: the form posts back and the page renders the new values.

use crate::site::page;
use axum::{
    Router,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
};
use loco_ui::layout::{Palette, Tokens};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/theme", get(builder).post(crate::site::theme_submit))
        .route("/theme.css", get(download))
}

/// The roles the builder edits, with the name the form shows.
const ROLES: [(&str, &str); 9] = [
    ("bg", "Background"),
    ("fg", "Text"),
    ("muted", "Muted text"),
    ("line", "Lines"),
    ("primary", "Primary"),
    ("on_primary", "On primary"),
    ("accent", "Accent"),
    ("on_accent", "On accent"),
    ("danger", "Danger"),
];

/// The default value of `role` in the built-in palette.
fn default(p: &Palette, role: &str) -> &'static str {
    match role {
        "bg" => p.bg,
        "fg" => p.fg,
        "muted" => p.muted,
        "line" => p.line,
        "primary" => p.primary,
        "on_primary" => p.on_primary,
        "accent" => p.accent,
        "on_accent" => p.on_accent,
        _ => p.danger,
    }
}

/// The chosen theme, read from the query; anything not a `#rrggbb` colour is the default.
struct Chosen {
    light: Vec<(&'static str, String)>,
    dark: Vec<(&'static str, String)>,
    radius: u32,
}

fn chosen(ui: &Ui) -> Chosen {
    let t = Tokens::default();
    let hex = |v: &str| {
        v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
    };
    let read = |scheme: &str, p: &Palette| {
        ROLES
            .iter()
            .map(|(role, _)| {
                let v = ui.param(&format!("{scheme}.{role}")).filter(|v| hex(v));
                (
                    *role,
                    v.map_or_else(|| default(p, role).to_string(), str::to_lowercase),
                )
            })
            .collect()
    };
    Chosen {
        light: read("light", &t.light),
        dark: read("dark", &t.dark),
        radius: ui
            .param("radius")
            .and_then(|r| r.parse().ok())
            .unwrap_or(8)
            .min(24),
    }
}

/// `--lui-bg: #fff; …` for one scheme, with the roles the builder does not edit derived from
/// the ones it does (cards and popovers on the background, inputs on the lines).
fn declarations(roles: &[(&str, String)]) -> String {
    let get = |r: &str| {
        roles
            .iter()
            .find(|(k, _)| *k == r)
            .map_or("", |(_, v)| v.as_str())
    };
    let mut out: Vec<String> = roles
        .iter()
        .map(|(r, v)| format!("--lui-{}: {v};", r.replace('_', "-")))
        .collect();
    for (derived, from) in [
        ("card", "bg"),
        ("popover", "bg"),
        ("secondary", "accent"),
        ("input", "line"),
        ("ring", "muted"),
    ] {
        out.push(format!("--lui-{derived}: {};", get(from)));
    }
    out.join(" ")
}

/// The overrides as a stylesheet, in the cascade order of `layout::Tokens::css`.
fn css(c: &Chosen) -> String {
    let (light, dark) = (declarations(&c.light), declarations(&c.dark));
    format!(
        "/* loco-ui theme from /theme: paste after the stylesheet (Page::css), or put the same\n   values in a layout::Tokens and pass it to Page::tokens. */\n\
         :root {{ {light} --lui-radius: {}px; }}\n\
         @media (prefers-color-scheme: dark) {{ :root:not([data-theme=\"light\"]) {{ {dark} }} }}\n\
         :root[data-theme=\"dark\"] {{ {dark} }}\n",
        c.radius
    )
}

/// A few components under one scheme's values: the preview.
fn preview(ui: &Ui, scheme: &str, roles: &[(&str, String)], radius: u32) -> Markup {
    let style = format!(
        "{} --lui-radius: {radius}px; color-scheme: {scheme};",
        declarations(roles)
    );
    html! {
        div class="lui-theme-preview" style=(style) {
            (ui.card().title("Invite a teammate").description("They get an email with a link.").body(html! {
                (ui.input(&format!("preview-{scheme}-email"), "Email").email().placeholder("ada@example.com"))
                (ui.cluster(html! {
                    (ui.button("Send invite").primary())
                    (ui.button("Cancel"))
                    (ui.badge("Beta").secondary())
                }))
                (ui.alert("Two seats left").danger().description("Upgrade to add more."))
            }))
        }
    }
}

async fn builder(ui: Ui) -> Page {
    let c = chosen(&ui);
    let query: Vec<String> = c
        .light
        .iter()
        .map(|(r, v)| format!("light.{r}={}", v.replace('#', "%23")))
        .chain(
            c.dark
                .iter()
                .map(|(r, v)| format!("dark.{r}={}", v.replace('#', "%23"))),
        )
        .chain([format!("radius={}", c.radius)])
        .collect();
    let download = format!("/theme.css?{}", query.join("&"));
    let body = lui! {
        // code: /theme
        form method="get" action="/theme" class="lui-theme-builder" {
            @for (scheme, roles) in [("light", &c.light), ("dark", &c.dark)] {
                fieldset {
                    legend { (if scheme == "light" { "Light" } else { "Dark" }) }
                    @for ((role, label), (_, value)) in ROLES.iter().zip(roles.iter()) {
                        @let name = format!("{scheme}.{role}");
                        Color(&name, value) label=(label);
                    }
                }
            }
            Range("radius", i64::from(c.radius)) min=0 max=24 label="Radius (px)";
            Cluster(lui! {
                Button("Preview") primary;
                LinkButton("Download theme.css", &download);
                a href="/theme" { "Reset" }
            })
        }
        div class="lui-theme-previews" {
            (preview(&ui, "light", &c.light, c.radius))
            (preview(&ui, "dark", &c.dark, c.radius))
        }
        // end code
        details { summary { "theme.css" } pre tabindex="0" aria-label="theme.css" { code { (css(&c)) } } }
    };
    page(&ui, "Theme builder", body)
}

/// The chosen overrides as a file.
async fn download(ui: Ui) -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (
                header::CONTENT_DISPOSITION,
                "attachment; filename=\"theme.css\"",
            ),
        ],
        css(&chosen(&ui)),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_colours_reach_the_css() {
        let ui = Ui::from_request(
            "/theme",
            "light.primary=%232f5bea&light.bg=red;x&radius=99",
            "",
        );
        let c = chosen(&ui);
        let out = css(&c);
        assert!(out.contains("--lui-primary: #2f5bea;"), "{out}");
        assert!(
            out.contains("--lui-bg: #ffffff;") && !out.contains("red;x"),
            "{out}"
        );
        assert!(out.contains("--lui-radius: 24px;"), "{out}");
    }
}
