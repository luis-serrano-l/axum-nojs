//! The theme builder: one brand colour and one gray, each grown into a 12-step scale
//! (`layout::Scale::derive`) for light and dark, and a radius, sent as a GET form so the theme
//! is in the URL; a preview of a few components, their shadows and the primary gradient under
//! those values in both schemes; and `theme.css`, the overrides as a file to paste after the
//! stylesheet. No script: the form posts back and the page renders the new values.

use crate::site::page;
use axum::{
    Router,
    http::header,
    response::{IntoResponse, Response},
    routing::get,
};
use loco_ui::layout::{DEPTH_DARK, DEPTH_LIGHT, DerivedScale, Scale, Tokens};
use loco_ui::prelude::*;

pub(crate) fn routes() -> Router {
    Router::new()
        .route("/theme", get(builder).post(crate::site::theme_submit))
        .route("/theme.css", get(download))
}

/// The page's live component, which the index shows too (`site::preview`).
pub(crate) const PREVIEWS: &[super::Preview] = &[("/theme", |ui| pickers(ui, &chosen(ui)))];

/// Radix Colors' step 9 of a few brand scales, offered as swatches.
const BRANDS: [&str; 7] = [
    "#3e63dd", "#0090ff", "#12a594", "#46a758", "#f76b15", "#e93d82", "#6e56cf",
];
/// Radix Colors' step 9 of its grays: slate, gray, mauve, sage, olive, sand.
const GRAYS: [&str; 6] = [
    "#8b8d98", "#8d8d8d", "#8e8c99", "#868e8b", "#898e87", "#8d8d86",
];

/// The chosen theme, read from the query: a clicked swatch (`brand-preset`) wins over the
/// picker, and anything not a `#rrggbb` colour is the default.
struct Chosen {
    brand: String,
    gray: String,
    radius: u32,
    brand_scale: DerivedScale,
    gray_scale: DerivedScale,
}

fn chosen(ui: &Ui) -> Chosen {
    let hex = |v: &str| {
        v.len() == 7 && v.starts_with('#') && v[1..].chars().all(|c| c.is_ascii_hexdigit())
    };
    let pick = |name: &str, default: &str| {
        ui.param(&format!("{name}-preset"))
            .or_else(|| ui.param(name))
            .filter(|v| hex(v))
            .map_or_else(|| default.to_string(), str::to_lowercase)
    };
    let (brand, gray) = (pick("brand", BRANDS[0]), pick("gray", GRAYS[0]));
    Chosen {
        brand_scale: Scale::derive(&brand, &Scale::INDIGO).expect("a #rrggbb colour"),
        gray_scale: Scale::derive(&gray, &Scale::SLATE).expect("a #rrggbb colour"),
        brand,
        gray,
        radius: ui
            .param("radius")
            .and_then(|r| r.parse().ok())
            .unwrap_or(8)
            .min(24),
    }
}

/// The scales of one scheme, plus what follows from the brand in both: white or near-black
/// text on its solid step, and a gradient that keeps near-black text readable.
fn declarations(c: &Chosen, dark: bool) -> String {
    let on = c.brand_scale.on_solid();
    let mut out = c.gray_scale.css("gray", dark) + &c.brand_scale.css("brand", dark);
    out.push_str(&format!("  --lui-on-primary: {on};\n"));
    if on != "#ffffff" {
        out.push_str("  --lui-gradient-primary: linear-gradient(in oklch to bottom, var(--lui-brand-9), var(--lui-brand-10));\n");
    }
    out
}

/// The overrides as a stylesheet, in the cascade order of `layout::Tokens::css`.
fn css(c: &Chosen) -> String {
    let (light, dark) = (declarations(c, false), declarations(c, true));
    format!(
        "/* loco-ui theme from /theme: paste after the stylesheet (Page::css), or put the same\n   scales in a layout::Tokens and pass it to Page::tokens. */\n\
         :root {{\n{light}  --lui-radius: {}px;\n}}\n\
         @media (prefers-color-scheme: dark) {{\n  :root:not([data-theme=\"light\"]) {{\n{dark}  }}\n}}\n\
         :root[data-theme=\"dark\"] {{\n{dark}}}\n",
        c.radius
    )
}

/// A few components under one scheme's values: the preview. The roles and depth tokens are
/// declared again on it, since `--lui-bg: var(--lui-gray-1)` resolves where it is declared
/// (on `:root`); the chosen scales come last so their `--lui-on-primary` wins.
fn preview(ui: &Ui, c: &Chosen, dark: bool) -> Markup {
    let (t, scheme) = (Tokens::default(), if dark { "dark" } else { "light" });
    let roles = if dark { t.dark } else { t.light };
    let depth = if dark { DEPTH_DARK } else { DEPTH_LIGHT };
    let style = format!(
        "{}{}{} --lui-radius: {}px; color-scheme: {scheme};",
        roles.declarations(),
        depth,
        declarations(c, dark),
        c.radius
    )
    .replace('\n', " ");
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
            div class="lui-theme-depth" aria-label="Shadows xs to lg, and the primary gradient" role="img" {
                @for size in ["xs", "sm", "md", "lg"] { span style={ "box-shadow: var(--lui-shadow-" (size) "), var(--lui-highlight)" } { (size) } }
                span class="lui-theme-gradient" {}
            }
        }
    }
}

/// The form that picks the theme, and the previews in both schemes.
fn pickers(ui: &Ui, c: &Chosen) -> Markup {
    let download = format!(
        "/theme.css?brand={}&gray={}&radius={}",
        c.brand.replace('#', "%23"),
        c.gray.replace('#', "%23"),
        c.radius
    );
    lui! {
        // code: /theme
        form method="get" action="/theme" class="lui-theme-builder" {
            fieldset {
                legend { "Scales" }
                div {
                    Color("brand", "Brand") value=(&c.brand) presets=(&BRANDS);
                    Color("gray", "Gray") value=(&c.gray) presets=(&GRAYS);
                }
            }
            Range("radius", "Radius (px)") value=(i64::from(c.radius)) min=0 max=24;
            Cluster(lui! {
                Button("Preview") primary;
                LinkButton("Download theme.css", &download);
                a href="/theme" { "Reset" }
            })
        }
        div class="lui-theme-previews" {
            (preview(ui, c, false))
            (preview(ui, c, true))
        }
        // end code
    }
}

async fn builder(ui: Ui) -> Page {
    let c = chosen(&ui);
    let rust = c.brand_scale.rust("BRAND") + &c.gray_scale.rust("GRAY");
    let body = lui! {
        (pickers(&ui, &c))
        details { summary { "theme.css" } pre tabindex="0" aria-label="theme.css" { code { (css(&c)) } } }
        details { summary { "The scales in Rust" } pre tabindex="0" aria-label="The scales in Rust" { code { (rust) } } }
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
        let ui = Ui::from_request("/theme", "brand=%2312a594&gray=red;x&radius=99", "");
        let out = css(&chosen(&ui));
        assert!(out.contains("--lui-brand-9: #12a594;"), "{out}");
        assert!(
            out.contains("--lui-gray-9: #8b8d98;") && !out.contains("red;x"),
            "{out}"
        );
        assert!(out.contains("--lui-radius: 24px;"), "{out}");
    }

    #[test]
    fn a_swatch_wins_and_a_light_brand_gets_dark_text() {
        let ui = Ui::from_request("/theme", "brand=%2312a594&brand-preset=%23ffe629", "");
        let out = css(&chosen(&ui));
        assert!(out.contains("--lui-brand-9: #ffe629;"), "{out}");
        assert!(out.contains("--lui-on-primary: #111113;"), "{out}");
    }
}
