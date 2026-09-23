//! # Layout
//!
//! The HTML shell every demo page uses: doctype, head, inline stylesheet, `<main>`, the
//! capability beacons that teach the server what this browser supports, and the optional
//! [`crate::enhance`] script tag (the page works the same without it).
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

use crate::{Caps, Theme, caps, enhance, stylesheet};

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
                (enhance::script_tag())
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
            span { "Interactive HTML for Rust servers, no JavaScript" }
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
  --wo-bg: #eef1ec; --wo-fg: #14201a; --wo-muted: #566158; --wo-line: #c9d2cb;
  --wo-surface: #ffffff; --wo-accent: #1f6f5f; --wo-on-accent: #ffffff;
  --wo-danger: #b3261e; --wo-ok: #2f7a3a;
  --wo-radius: 6px;
  --wo-space: 8px;
}
@media (prefers-color-scheme: dark) {
  :root:not([data-theme="light"]) {
    --wo-bg: #0f1512; --wo-fg: #e4ebe6; --wo-muted: #97a59c; --wo-line: #2b3630;
    --wo-surface: #171f1b; --wo-accent: #62c9a8; --wo-on-accent: #08110d;
    --wo-danger: #ff8a80; --wo-ok: #7bd389;
  }
}
:root[data-theme="dark"] {
  color-scheme: dark;
  --wo-bg: #0f1512; --wo-fg: #e4ebe6; --wo-muted: #97a59c; --wo-line: #2b3630;
  --wo-surface: #171f1b; --wo-accent: #62c9a8; --wo-on-accent: #08110d;
  --wo-danger: #ff8a80; --wo-ok: #7bd389;
}
:root[data-theme="light"] { color-scheme: light; }

* { box-sizing: border-box; }
html {
  font-family: "Avenir Next", "Segoe UI Variable Text", "Segoe UI", Ubuntu, Cantarell, "Noto Sans", system-ui, sans-serif;
  line-height: 1.55; -webkit-text-size-adjust: 100%;
}
body { margin: 0; background: var(--wo-bg); color: var(--wo-fg); }
main { max-width: 52rem; margin: 0 auto; padding: calc(var(--wo-space) * 4) calc(var(--wo-space) * 2) calc(var(--wo-space) * 8); }
p, li { max-width: 44rem; }
.wo-header {
  display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 1rem;
  max-width: 52rem; margin: 0 auto; padding: calc(var(--wo-space) * 2);
  color: var(--wo-muted);
}
.wo-header a { color: var(--wo-fg); text-decoration: none; font-size: 1.125rem; letter-spacing: -0.01em; }
.wo-header strong { font-weight: 700; }
h1 { font-size: 2.25rem; line-height: 1.15; letter-spacing: -0.01em; font-weight: 700; margin: 0 0 0.75rem; }
h2 { font-size: 1.375rem; line-height: 1.25; letter-spacing: -0.005em; margin: 2rem 0 0.5rem; }
p { margin: 0 0 1rem; }
a { color: var(--wo-accent); text-underline-offset: 0.15em; text-decoration-thickness: 1px; }
code {
  font-family: ui-monospace, "Cascadia Mono", "JetBrains Mono", Menlo, Consolas, monospace; font-size: 0.875em;
  background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 0.05em 0.35em;
}
button, input, select, textarea { font: inherit; color: inherit; }
button { cursor: pointer; background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 0.5rem 1rem; }
button:hover { border-color: var(--wo-accent); }
button.wo-primary { background: var(--wo-accent); color: var(--wo-on-accent); border-color: transparent; }
input, select, textarea { background: var(--wo-surface); border: 1px solid var(--wo-line); border-radius: var(--wo-radius); padding: 0.5rem 0.75rem; }
:focus-visible { outline: 2px solid var(--wo-accent); outline-offset: 2px; }
table { border-collapse: collapse; width: 100%; }
th, td { text-align: left; padding: 0.5rem; border-bottom: 1px solid var(--wo-line); vertical-align: top; }
th { color: var(--wo-muted); font-weight: 600; }
.wo-note { color: var(--wo-muted); font-size: 0.9rem; }
.wo-yes { color: var(--wo-ok); font-weight: 600; }
.wo-no { color: var(--wo-danger); font-weight: 600; }

/* Demo shell: toolbar with the way back and the theme switch, the lede under a title,
   the "built on" line, and the grouped index. */
.wo-toolbar { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: var(--wo-space); margin: 0 0 calc(var(--wo-space) * 3); min-height: 2.25rem; }
.wo-back { color: var(--wo-muted); text-decoration: none; }
.wo-back::before { content: "\2190"; margin-right: 0.35em; }
.wo-back:hover { color: var(--wo-accent); text-decoration: underline; }
.wo-lede { font-size: 1.125rem; color: var(--wo-muted); margin-bottom: 1.5rem; }
.wo-built { color: var(--wo-muted); font-size: 0.9rem; margin: -0.25rem 0 1.5rem; }
.wo-built code { color: var(--wo-fg); margin-right: 0.25rem; }
.wo-index { max-width: none; }
.wo-index h2 { margin-top: 2.5rem; padding-bottom: 0.35rem; border-bottom: 1px solid var(--wo-line); }
.wo-index ul { list-style: none; margin: 0; padding: 0; }
.wo-index li { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 1rem; padding: 0.6rem 0; max-width: none; }
.wo-index li + li { border-top: 1px solid var(--wo-line); }
.wo-index li a { font-size: 1.25rem; font-weight: 600; text-decoration: none; color: var(--wo-fg); flex: 0 0 12rem; }
.wo-index li a:hover { color: var(--wo-accent); text-decoration: underline; }
.wo-index li span { color: var(--wo-muted); }
.wo-index li code { margin-right: 0.25rem; }
@media (max-width: 40rem) { .wo-index li { flex-direction: column; gap: 0.2rem; } .wo-index li a { flex: none; } }

@media (prefers-reduced-motion: reduce) {
  *, ::before, ::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  ::view-transition-group(*), ::view-transition-old(*), ::view-transition-new(*) { animation: none !important; }
}
"#;
