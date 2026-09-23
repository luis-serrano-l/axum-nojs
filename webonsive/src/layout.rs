//! # Layout
//!
//! The HTML shell every demo page uses: doctype, head, inline stylesheet, `<main>`, and the
//! capability beacons that teach the server what this browser supports.
//!
//! **Platform features:** `@view-transition { navigation: auto }` (Chrome 126+, Safari 18.2+,
//! not Firefox) lets elements with a `view-transition-name` morph across full-page navigations.
//! The root itself swaps instantly: the default 0.25 s cross-fade made every click feel slow.
//! `<link rel="expect" blocking="render">` (Chrome 124+) holds the transition until `<main>`
//! is parsed. `prefers-color-scheme` + custom properties give light/dark with no script.
//!
//! **Fallback:** browsers without view transitions navigate normally. Theme colours are plain
//! custom properties switched by a media query and `data-theme`, so no `light-dark()` needed.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{Caps, layout, Theme};
//! let page = layout(&Caps::all(), "Title", Theme::Auto, html! { p { "body" } });
//! ```

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{Caps, Theme, caps, stylesheet};

/// Wrap `body` in a full page. Beacons are added while the browser is still unknown.
pub fn layout(caps: &Caps, title: &str, theme: Theme, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme=(theme.as_str()) {
            (head_blocking(title))
            body {
                (header())
                main id="main" { (body) }
                (caps::beacons(caps))
            }
        }
    }
}

/// `<head>`: charset, viewport, title and the inline stylesheet. `stream` reuses it.
pub fn head(title: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            title { (title) }
            style { (PreEscaped(stylesheet())) }
        }
    }
}

/// `<head>` plus `<link rel="expect" blocking="render">` on `#main`, so a cross-document view
/// transition starts only once the whole page is parsed. `layout` uses it; streamed pages must
/// not, because their parse ends only when the last slot has filled.
fn head_blocking(title: &str) -> Markup {
    html! {
        head {
            meta charset="utf-8";
            meta name="viewport" content="width=device-width, initial-scale=1";
            title { (title) }
            link rel="expect" href="#main" blocking="render";
            style { (PreEscaped(stylesheet())) }
        }
    }
}

/// The site header shown on every page.
pub fn header() -> Markup {
    html! {
        header class="wo-header" {
            a href="/" { strong { "webonsive" } }
            span { " · zero JavaScript" }
        }
    }
}

/// Styles for this component; included in [`crate::stylesheet`].
pub const CSS: &str = r#"
@view-transition { navigation: auto; }
/* The root does not cross-fade: the swap is instant and only named parts morph, so a
   navigation never feels slower than the plain reload it replaces. */
::view-transition-old(root), ::view-transition-new(root) { animation: none; }
::view-transition-group(*) { animation-duration: 120ms; animation-timing-function: ease-out; }
::view-transition-old(*), ::view-transition-new(*) { animation-duration: 120ms; }
/* The transition overlay must not eat clicks: a counter tapped twice quickly would lose the
   second tap while the first one is still morphing. */
::view-transition { pointer-events: none; }

:root {
  color-scheme: light dark;
  --wo-bg: #fafafa; --wo-fg: #1b1b1f; --wo-muted: #5f6168; --wo-line: #d9d9de;
  --wo-surface: #ffffff; --wo-accent: #2f5bea; --wo-on-accent: #ffffff; --wo-danger: #c62828;
  --wo-radius: 8px;
  --wo-space: 8px;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --wo-bg: #141416; --wo-fg: #e8e8ea; --wo-muted: #9a9ca6; --wo-line: #2e2e34;
    --wo-surface: #1c1c20; --wo-accent: #7c9cff; --wo-on-accent: #0f1220; --wo-danger: #ff7b72;
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --wo-bg: #141416; --wo-fg: #e8e8ea; --wo-muted: #9a9ca6; --wo-line: #2e2e34;
  --wo-surface: #1c1c20; --wo-accent: #7c9cff; --wo-on-accent: #0f1220; --wo-danger: #ff7b72;
}
:root[data-theme="light"] { color-scheme: light; }

* { box-sizing: border-box; }
html { font-family: system-ui, -apple-system, "Segoe UI", Roboto, sans-serif; line-height: 1.5; }
body { margin: 0; background: var(--wo-bg); color: var(--wo-fg); }
main { max-width: 52rem; margin: 0 auto; padding: calc(var(--wo-space) * 3) calc(var(--wo-space) * 2); }
.wo-header { padding: calc(var(--wo-space) * 2); border-bottom: 1px solid var(--wo-line); color: var(--wo-muted); }
.wo-header a { color: var(--wo-fg); text-decoration: none; }
h1 { font-size: 1.75rem; margin: 0 0 1rem; }
h2 { font-size: 1.25rem; margin: 2rem 0 0.5rem; }
p { margin: 0 0 1rem; }
a { color: var(--wo-accent); }
code { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; font-size: 0.9em; background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: 4px; padding: 0 0.3em; }
button, input, select, textarea { font: inherit; color: inherit; }
button { cursor: pointer; background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 0.5rem 1rem; }
button:hover { border-color: var(--wo-accent); }
button.wo-primary { background: var(--wo-accent); color: var(--wo-on-accent); border-color: transparent; }
input, select, textarea { background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 0.5rem 0.75rem; }
:focus-visible { outline: 2px solid var(--wo-accent); outline-offset: 2px; }
table { border-collapse: collapse; width: 100%; }
th, td { text-align: left; padding: 0.5rem; border-bottom: 1px solid var(--wo-line); vertical-align: top; }
.wo-note { color: var(--wo-muted); font-size: 0.9rem; }
.wo-yes { color: #2e7d32; font-weight: 600; }
.wo-no { color: var(--wo-danger); font-weight: 600; }

@media (prefers-reduced-motion: reduce) {
  *, ::before, ::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  ::view-transition-group(*), ::view-transition-old(*), ::view-transition-new(*) { animation: none !important; }
}
"#;
