//! # Layout
//!
//! The HTML shell every demo page uses: doctype, head, inline stylesheet, `<main>`.
//!
//! **Platform features:** `@view-transition { navigation: auto }` (Chrome 126+, Safari 18.2+)
//! makes full-page navigations cross-fade, so server round trips feel in-place.
//! `color-scheme` + custom properties give light/dark with no script.
//!
//! **Fallback:** browsers without view transitions navigate normally.
//!
//! ```rust
//! use maud::html;
//! use webonsive::{layout, Theme};
//! let page = layout("Title", Theme::Auto, html! { p { "body" } });
//! ```

use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::{Theme, stylesheet};

/// Wrap `body` in a full page.
pub fn layout(title: &str, theme: Theme, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" data-theme=(theme.as_str()) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                style { (PreEscaped(stylesheet())) }
            }
            body {
                header class="wo-header" {
                    a href="/" { strong { "webonsive" } }
                    span { " · zero JavaScript" }
                }
                main { (body) }
            }
        }
    }
}

pub const CSS: &str = r#"
@view-transition { navigation: auto; }

:root {
  color-scheme: light dark;
  --wo-bg: light-dark(#fafafa, #141416);
  --wo-fg: light-dark(#1b1b1f, #e8e8ea);
  --wo-muted: light-dark(#5f6168, #9a9ca6);
  --wo-line: light-dark(#d9d9de, #2e2e34);
  --wo-surface: light-dark(#ffffff, #1c1c20);
  --wo-accent: light-dark(#2f5bea, #7c9cff);
  --wo-on-accent: light-dark(#ffffff, #0f1220);
  --wo-danger: light-dark(#c62828, #ff7b72);
  --wo-radius: 8px;
  --wo-space: 8px;
}
:root[data-theme="light"] { color-scheme: light; }
:root[data-theme="dark"]  { color-scheme: dark; }

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

@media (prefers-reduced-motion: reduce) {
  *, ::before, ::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  ::view-transition-group(*), ::view-transition-old(*), ::view-transition-new(*) { animation: none !important; }
}
"#;
