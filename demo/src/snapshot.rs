//! The static snapshot for GitHub Pages: every GET page in [`PATHS`] rendered to a file, with
//! the enhancement script taken out, links rewritten to relative `.html` files and a banner
//! saying what needs the real server. `cargo run -p demo -- snapshot <dir>` (or
//! `scripts/snapshot.sh`) writes it; `<dialog>`, `popover`, `<details>` and tooltips keep
//! working there because they never needed the server.
//!
//! Each page is rendered twice, as a browser with every capability and as one with none. The
//! modern render is `<name>.html`; where the baseline render differs it is written beside it
//! as `<name>.baseline.html`, and the banner links the two.

use crate::{PATHS, router};
use axum::{body::Body, http::Request};
use axum_nojs::caps::Cap;
use axum_nojs::prelude::*;
use tower::ServiceExt;

/// The text every exported page carries (the test looks for it).
pub const BANNER: &str = "Static snapshot";

/// One exported file: its name under the output directory and its HTML.
pub struct File {
    pub name: String,
    pub html: String,
}

/// Every page of the snapshot, modern variants first in [`PATHS`] order.
pub async fn pages() -> Vec<File> {
    let modern = Cap::ALL
        .map(|c| format!("nojs-cap-{}=1", c.name()))
        .join("; ");
    let names = names();
    let mut files = Vec::new();
    for (path, name) in PATHS.iter().zip(&names) {
        let rich = render(path, &modern).await;
        let plain = render(path, "").await;
        let baseline = (plain != rich).then(|| format!("{name}.baseline"));
        let other = baseline.as_deref();
        files.push(File {
            name: format!("{name}.html"),
            html: finish(&rich, &names, other, false),
        });
        if let Some(b) = other {
            files.push(File {
                name: format!("{b}.html"),
                html: finish(&plain, &names, Some(name), true),
            });
        }
    }
    files
}

/// The file name (without `.html`) for each entry of [`PATHS`]: `/` is `index`, `/app/signin`
/// is `app-signin`, and a route that appears again with another query gets `-2`, `-3`.
pub fn names() -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    for path in PATHS {
        let route = route(path);
        let base = match route.trim_matches('/') {
            "" => "index".to_string(),
            r => r.replace('/', "-"),
        };
        let seen = PATHS
            .iter()
            .take_while(|p| **p != path)
            .filter(|p| self::route(p) == route)
            .count();
        names.push(if seen == 0 {
            base
        } else {
            format!("{base}-{}", seen + 1)
        });
    }
    names
}

fn route(path: &str) -> &str {
    path.split(['?', '#']).next().unwrap_or(path)
}

async fn render(path: &str, cookie: &str) -> String {
    let req = Request::get(path)
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap();
    let res = router().oneshot(req).await.unwrap();
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    // The capability beacons ask `/nojs/caps` for a cookie; a static host has neither.
    let beacons = r#"<div class="nojs-caps" aria-hidden="true">"#;
    match html.find(beacons) {
        Some(i) => {
            let end = i + html[i..].find("</div>").map_or(0, |j| j + "</div>".len());
            format!("{}{}", &html[..i], &html[end..])
        }
        None => html,
    }
}

/// Drop the script, rewrite `href`/`action`, and put the banner at the top of `<main>`.
fn finish(html: &str, names: &[String], other: Option<&str>, baseline: bool) -> String {
    let html = html.replace(&axum_nojs::enhance::script_tag().into_string(), "");
    let html = rewrite(&html, "href=\"", names);
    let html = rewrite(&html, "action=\"", names);
    let at = html
        .find("<main")
        .or_else(|| html.find("<body"))
        .and_then(|i| html[i..].find('>').map(|j| i + j + 1))
        .unwrap_or(0);
    format!("{}{}{}", &html[..at], banner(other, baseline).into_string(), &html[at..])
}

/// Point every root-relative URL that names an exported page at its file: the exact path and
/// query if [`PATHS`] has it, else the first export of that route. Anything else (POST
/// targets, `/nojs/*`, `/table.csv`) stays as it is and fails on a static host, as the banner
/// says.
fn rewrite(html: &str, attr: &str, names: &[String]) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(i) = rest.find(attr) {
        let start = i + attr.len();
        out.push_str(&rest[..start]);
        rest = &rest[start..];
        let end = rest.find('"').unwrap_or(rest.len());
        let url = &rest[..end];
        out.push_str(&target(url, names).unwrap_or_else(|| url.to_string()));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn target(url: &str, names: &[String]) -> Option<String> {
    if !url.starts_with('/') || url.starts_with("//") {
        return None;
    }
    let url = url.replace("&amp;", "&");
    let (no_frag, frag) = match url.split_once('#') {
        Some((u, f)) => (u, format!("#{f}")),
        None => (url.as_str(), String::new()),
    };
    let i = PATHS
        .iter()
        .position(|p| *p == no_frag)
        .or_else(|| PATHS.iter().position(|p| route(p) == route(no_frag)))?;
    Some(format!("{}.html{frag}", names[i]))
}

fn banner(other: Option<&str>, baseline: bool) -> Markup {
    let ui = Ui::default();
    let body = html! {
        p { "Forms, cookies and paging need the real server: clone the repository and run "
            code { "cargo run -p demo" } ". Dialogs, popovers, disclosures and tooltips work here." }
        @if let Some(o) = other {
            p { a href={ (o) ".html" } {
                @if baseline { "See this page as a current browser gets it." }
                @else { "See this page as a browser without the newer CSS features gets it." }
            } }
        }
    };
    html! { div style="margin-block-end: var(--nojs-space-4)" { (ui.alert(BANNER).warn().body(body)) } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn every_exported_page_has_the_banner_and_no_script() {
        let pages = pages().await;
        assert!(pages.len() >= PATHS.len());
        let names: Vec<&str> = pages.iter().map(|p| p.name.as_str()).collect();
        for page in &pages {
            assert!(page.html.contains(BANNER), "{}: no banner", page.name);
            assert_eq!(page.html.matches("<script").count(), 0, "{}", page.name);
            assert!(!page.html.contains(r#"class="nojs-caps""#), "{}: beacons", page.name);
            // Every link to a page is to a file that exists.
            for (i, _) in page.html.match_indices("href=\"") {
                let url = &page.html[i + 6..];
                let url = &url[..url.find('"').unwrap()];
                let file = url.split('#').next().unwrap();
                if file.ends_with(".html") {
                    assert!(names.contains(&file), "{}: {url} is not exported", page.name);
                }
            }
        }
    }
}
