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
//! **Theming:** every colour, radius and spacing the components use is a `--nojs-*` custom
//! property. [`Tokens`] holds them for light and dark; `ui.page(..).tokens(&tokens)` emits them once per page
//! as a `<style>` after the stylesheet, so a different palette is a struct, not a CSS file.
//! `docs/theming.md` lists each token and what it affects.
//!
//! ```rust
//! use axum_nojs::{prelude::*, layout::{Palette, Tokens}};
//! let ui = Ui::from(Caps::all());
//! let page = ui.page("Title", html! { p { "body" } });
//! let tokens = Tokens { light: Palette { accent: "#7a3b1e", ..Tokens::default().light }, ..Default::default() };
//! assert!(page.tokens(&tokens).into_string().contains("--nojs-accent: #7a3b1e"));
//! ```

use maud::{DOCTYPE, Markup, PreEscaped, Render, html};

use crate::{Caps, Theme, caps, enhance, stylesheet};

/// One colour scheme's worth of tokens, as CSS colour values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Page background (`--nojs-bg`).
    pub bg: &'static str,
    /// Text on the page background (`--nojs-fg`).
    pub fg: &'static str,
    /// Secondary text: notes, labels, table headers (`--nojs-muted`).
    pub muted: &'static str,
    /// Borders and rules (`--nojs-line`).
    pub line: &'static str,
    /// Raised surfaces: inputs, buttons, dialogs, code (`--nojs-surface`).
    pub surface: &'static str,
    /// Links, primary buttons, the open tab, the focus ring (`--nojs-accent`).
    pub accent: &'static str,
    /// Text on the accent (`--nojs-on-accent`).
    pub on_accent: &'static str,
    /// Errors and "no" (`--nojs-danger`).
    pub danger: &'static str,
    /// Success and "yes" (`--nojs-ok`).
    pub ok: &'static str,
    /// Warnings: worked, but look (`--nojs-warn`).
    pub warn: &'static str,
}

impl Palette {
    /// The custom property declarations for this palette, one per line.
    fn declarations(&self) -> String {
        format!(
            "  --nojs-bg: {}; --nojs-fg: {}; --nojs-muted: {}; --nojs-line: {};\n  --nojs-surface: {}; --nojs-accent: {}; --nojs-on-accent: {};\n  --nojs-danger: {}; --nojs-ok: {}; --nojs-warn: {};\n",
            self.bg, self.fg, self.muted, self.line, self.surface, self.accent, self.on_accent, self.danger, self.ok, self.warn
        )
    }
}

/// Every `--nojs-*` token: a light and a dark palette plus the two shape tokens.
/// `Default` is the crate's own look ("ink and moss": pale sage paper, green-black ink, moss
/// accent; mint on near-black in the dark scheme).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tokens {
    /// Colours for the light scheme and for `data-theme="light"`.
    pub light: Palette,
    /// Colours for `prefers-color-scheme: dark` and for `data-theme="dark"`.
    pub dark: Palette,
    /// Corner radius of buttons, inputs, dialogs, chips (`--nojs-radius`).
    pub radius: &'static str,
    /// The spacing unit every gap and padding is a multiple of (`--nojs-space`).
    pub space: &'static str,
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens {
            light: Palette {
                bg: "#eef1ec", fg: "#14201a", muted: "#566158", line: "#c9d2cb", surface: "#ffffff",
                accent: "#1f6f5f", on_accent: "#ffffff", danger: "#b3261e", ok: "#2f7a3a", warn: "#8a5a00",
            },
            dark: Palette {
                bg: "#0f1512", fg: "#e4ebe6", muted: "#97a59c", line: "#2b3630", surface: "#171f1b",
                accent: "#62c9a8", on_accent: "#08110d", danger: "#ff8a80", ok: "#7bd389", warn: "#e6b450",
            },
            radius: "6px",
            space: "8px",
        }
    }
}

impl Tokens {
    /// The CSS that sets these tokens: `:root` for light, the dark palette under
    /// `prefers-color-scheme: dark` unless `data-theme="light"`, and again under
    /// `data-theme="dark"`. [`crate::stylesheet`] starts with `Tokens::default().css()`.
    pub fn css(&self) -> String {
        let (light, dark) = (self.light.declarations(), self.dark.declarations());
        format!(
            ":root {{\n  color-scheme: light dark;\n{light}  --nojs-radius: {}; --nojs-space: {};\n}}\n\
             @media (prefers-color-scheme: dark) {{\n  :root:not([data-theme=\"light\"]) {{\n{dark}  }}\n}}\n\
             :root[data-theme=\"dark\"] {{\n  color-scheme: dark;\n{dark}}}\n\
             :root[data-theme=\"light\"] {{ color-scheme: light; }}\n",
            self.radius, self.space
        )
    }
}

/// Wrap `body` in a full page with the default [`Tokens`]. Beacons are added while the
/// browser is still unknown.
pub fn layout(caps: &Caps, title: &str, theme: Theme, body: Markup) -> Markup {
    page(caps, title, theme, None, body)
}

/// [`layout`] under a different set of [`Tokens`]: the overrides are emitted once, in a
/// `<style>` right after the stylesheet, so every component on the page picks them up.
pub fn layout_with(caps: &Caps, title: &str, theme: Theme, tokens: &Tokens, body: Markup) -> Markup {
    page(caps, title, theme, Some(tokens), body)
}

fn page(caps: &Caps, title: &str, theme: Theme, tokens: Option<&Tokens>, body: Markup) -> Markup {
    // The stylesheet is most of the page: size the buffer once instead of doubling into it.
    let size = stylesheet().len() + body.0.len() + 2048;
    html! {
        (Reserve(size))
        (DOCTYPE)
        html lang="en" data-theme=(theme.as_str()) {
            head {
                (meta(title))
                // Hold a cross-document view transition until `#main` is parsed. Streamed
                // pages must not: their parse ends only when the last slot has filled.
                link rel="expect" href="#main" blocking="render";
                style { (PreEscaped(stylesheet())) }
                @if let Some(t) = tokens { style class="nojs-tokens" { (PreEscaped(t.css())) } }
            }
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
        (Reserve(stylesheet().len() + 256))
        head {
            (meta(title))
            style { (PreEscaped(stylesheet())) }
        }
    }
}

fn meta(title: &str) -> Markup {
    html! {
        meta charset="utf-8";
        meta name="viewport" content="width=device-width, initial-scale=1";
        title { (title) }
    }
}

/// Grows the buffer `html!` is writing into by `.0` bytes and writes nothing: a size hint
/// for a template whose splices dwarf its literals.
pub(crate) struct Reserve(pub(crate) usize);

impl Render for Reserve {
    fn render_to(&self, buffer: &mut String) {
        buffer.reserve(self.0);
    }
}

/// The site header shown on every page.
pub fn header() -> Markup {
    html! {
        header class="nojs-header" {
            a href="/" { strong { "axum-nojs" } }
            span { "Interactive HTML for Rust servers, works without JavaScript" }
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

/* The --nojs-* tokens come first in stylesheet(), from Tokens::default().css(). */

* { box-sizing: border-box; }
html {
  font-family: "Avenir Next", "Segoe UI Variable Text", "Segoe UI", Ubuntu, Cantarell, "Noto Sans", system-ui, sans-serif;
  line-height: 1.55; -webkit-text-size-adjust: 100%;
}
body { margin: 0; background: var(--nojs-bg); color: var(--nojs-fg); }
main { max-width: 52rem; margin: 0 auto; padding: calc(var(--nojs-space) * 4) calc(var(--nojs-space) * 2) calc(var(--nojs-space) * 8); }
p, li { max-width: 44rem; }
.nojs-header {
  display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 1rem;
  max-width: 52rem; margin: 0 auto; padding: calc(var(--nojs-space) * 2);
  color: var(--nojs-muted);
}
.nojs-header a { color: var(--nojs-fg); text-decoration: none; font-size: 1.125rem; letter-spacing: -0.01em; }
.nojs-header strong { font-weight: 700; }
h1 { font-size: 2.25rem; line-height: 1.15; letter-spacing: -0.01em; font-weight: 700; margin: 0 0 0.75rem; }
h2 { font-size: 1.375rem; line-height: 1.25; letter-spacing: -0.005em; margin: 2rem 0 0.5rem; }
p { margin: 0 0 1rem; }
a { color: var(--nojs-accent); text-underline-offset: 0.15em; text-decoration-thickness: 1px; }
code {
  font-family: ui-monospace, "Cascadia Mono", "JetBrains Mono", Menlo, Consolas, monospace; font-size: 0.875em;
  background: var(--nojs-surface); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); padding: 0.05em 0.35em;
}
button, input, select, textarea { font: inherit; color: inherit; }
button { cursor: pointer; background: var(--nojs-surface); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); padding: 0.5rem 1rem; }
button:hover { border-color: var(--nojs-accent); }
button.nojs-primary { background: var(--nojs-accent); color: var(--nojs-on-accent); border-color: transparent; }
button.nojs-danger { background: var(--nojs-danger); color: var(--nojs-on-accent); border-color: transparent; }
input, select, textarea { background: var(--nojs-surface); border: 1px solid var(--nojs-line); border-radius: var(--nojs-radius); padding: 0.5rem 0.75rem; }
:focus-visible { outline: 2px solid var(--nojs-accent); outline-offset: 2px; }
/* A swap root or form with a request in flight (set by the enhancement script only). The
   fade waits so a fast answer never flickers; --nojs-busy: 1 turns it off. */
.nojs-sr { position: absolute; width: 1px; height: 1px; margin: -1px; padding: 0; overflow: hidden; clip-path: inset(50%); text-wrap: nowrap; border: 0; }
[data-nojs-busy] { opacity: var(--nojs-busy, 0.6); transition: opacity 0.15s 0.2s; cursor: progress; }
table { border-collapse: collapse; width: 100%; }
th, td { text-align: left; padding: 0.5rem; border-bottom: 1px solid var(--nojs-line); vertical-align: top; }
th { color: var(--nojs-muted); font-weight: 600; }
.nojs-note { color: var(--nojs-muted); font-size: 0.9rem; }
.nojs-yes { color: var(--nojs-ok); font-weight: 600; }
.nojs-no { color: var(--nojs-danger); font-weight: 600; }

/* Demo shell: toolbar with the way back and the theme switch, the lede under a title,
   the "built on" line, and the grouped index. */
.nojs-toolbar { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: var(--nojs-space); margin: 0 0 calc(var(--nojs-space) * 3); min-height: 2.25rem; }
.nojs-popover-row { display: flex; justify-content: space-between; gap: var(--nojs-space); margin-bottom: calc(var(--nojs-space) * 2); }
.nojs-back { color: var(--nojs-muted); text-decoration: none; }
.nojs-back::before { content: "\2190"; margin-right: 0.35em; }
.nojs-back:hover { color: var(--nojs-accent); text-decoration: underline; }
.nojs-lede { font-size: 1.125rem; color: var(--nojs-muted); margin-bottom: 1.5rem; }
.nojs-built { color: var(--nojs-muted); font-size: 0.9rem; margin: -0.25rem 0 1.5rem; }
.nojs-built code { color: var(--nojs-fg); margin-right: 0.25rem; }
.nojs-index { max-width: none; }
.nojs-index h2 { margin-top: 2.5rem; padding-bottom: 0.35rem; border-bottom: 1px solid var(--nojs-line); }
.nojs-index ul { list-style: none; margin: 0; padding: 0; }
.nojs-index li { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 1rem; padding: 0.6rem 0; max-width: none; }
.nojs-index li + li { border-top: 1px solid var(--nojs-line); }
.nojs-index li a { font-size: 1.25rem; font-weight: 600; text-decoration: none; color: var(--nojs-fg); flex: 0 0 12rem; }
.nojs-index li a:hover { color: var(--nojs-accent); text-decoration: underline; }
.nojs-index li span { color: var(--nojs-muted); }
.nojs-index li code { margin-right: 0.25rem; }
@media (max-width: 40rem) { .nojs-index li { flex-direction: column; gap: 0.2rem; } .nojs-index li a { flex: none; } }

@media (prefers-reduced-motion: reduce) {
  *, ::before, ::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  ::view-transition-group(*), ::view-transition-old(*), ::view-transition-new(*) { animation: none !important; }
}
"#;
