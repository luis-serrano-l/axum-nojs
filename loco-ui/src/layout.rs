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
//! **Theming:** every colour, radius and spacing the components use is a `--lui-*` custom
//! property. [`Tokens`] holds them for light and dark; `ui.page(..).tokens(&tokens)` emits them once per page
//! as a `<style>` after the stylesheet, so a different palette is a struct, not a CSS file.
//! `docs/theming.md` lists each token and what it affects.
//!
//! ```rust
//! use loco_ui::{prelude::*, layout::{Palette, Tokens}};
//! let ui = Ui::from(Caps::all());
//! let page = ui.page("Title", html! { p { "body" } });
//! let tokens = Tokens { light: Palette { primary: "#7a3b1e", ..Tokens::default().light }, ..Default::default() };
//! assert!(page.tokens(&tokens).into_string().contains("--lui-primary: #7a3b1e"));
//! ```

use maud::{DOCTYPE, Markup, PreEscaped, Render, html};

use crate::{Caps, Theme, caps, enhance, stylesheet};

/// One colour scheme's worth of tokens, as CSS colour values. The roles follow shadcn/ui:
/// `primary` is the brand colour, `accent` is the quiet surface under a hovered item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Palette {
    /// Page background (`--lui-bg`).
    pub bg: &'static str,
    /// Text on the page background (`--lui-fg`).
    pub fg: &'static str,
    /// Secondary text: notes, labels, table headers (`--lui-muted`).
    pub muted: &'static str,
    /// Borders and rules (`--lui-line`).
    pub line: &'static str,
    /// Raised surfaces: `<code>`, the demo stage, open panels (`--lui-surface`).
    pub surface: &'static str,
    /// Cards and stat tiles (`--lui-card`).
    pub card: &'static str,
    /// Dialogs, drawers, popovers, menus and toasts (`--lui-popover`).
    pub popover: &'static str,
    /// Secondary buttons, the tab list, chips and badges (`--lui-secondary`).
    pub secondary: &'static str,
    /// Hover and highlighted surface: menu items, ghost buttons, rows (`--lui-accent`).
    pub accent: &'static str,
    /// Text on the accent surface (`--lui-on-accent`).
    pub on_accent: &'static str,
    /// Links, primary buttons, the current page and step (`--lui-primary`).
    pub primary: &'static str,
    /// Text on the primary colour (`--lui-on-primary`).
    pub on_primary: &'static str,
    /// Borders of inputs, selects and textareas (`--lui-input`).
    pub input: &'static str,
    /// The focus ring, drawn at 50% opacity (`--lui-ring`).
    pub ring: &'static str,
    /// Errors and "no" (`--lui-danger`).
    pub danger: &'static str,
    /// Success and "yes" (`--lui-ok`).
    pub ok: &'static str,
    /// Warnings: worked, but look (`--lui-warn`).
    pub warn: &'static str,
}

impl Palette {
    /// The custom property declarations for this palette, one per line.
    fn declarations(&self) -> String {
        format!(
            "  --lui-bg: {}; --lui-fg: {}; --lui-muted: {}; --lui-line: {};\n  --lui-surface: {}; --lui-card: {}; --lui-popover: {}; --lui-secondary: {};\n  --lui-accent: {}; --lui-on-accent: {}; --lui-primary: {}; --lui-on-primary: {};\n  --lui-input: {}; --lui-ring: {};\n  --lui-danger: {}; --lui-ok: {}; --lui-warn: {};\n",
            self.bg,
            self.fg,
            self.muted,
            self.line,
            self.surface,
            self.card,
            self.popover,
            self.secondary,
            self.accent,
            self.on_accent,
            self.primary,
            self.on_primary,
            self.input,
            self.ring,
            self.danger,
            self.ok,
            self.warn
        )
    }
}

/// Every `--lui-*` token: a light and a dark palette plus the two shape tokens.
/// `Default` is shadcn/ui's neutral (zinc) theme: white and zinc-950, a near-black primary
/// that turns near-white in the dark scheme, with status colours from Radix Colors step 11
/// (the step made for text, so each clears 4.5:1 on the background).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tokens {
    /// Colours for the light scheme and for `data-theme="light"`.
    pub light: Palette,
    /// Colours for `prefers-color-scheme: dark` and for `data-theme="dark"`.
    pub dark: Palette,
    /// Corner radius of dialogs and popovers (`--lui-radius`); controls use
    /// `--lui-radius-sm` (2px less) and cards `--lui-radius-lg` (4px more). The two
    /// shadows, `--lui-shadow-xs` (controls) and `--lui-shadow-lg` (floating layers), are
    /// emitted beside them and are the same in both schemes, as in shadcn/ui.
    pub radius: &'static str,
    /// The spacing unit every gap and padding is a multiple of (`--lui-space`). The scale
    /// `--lui-space-{1,2,3,4,6,8}` is derived from it: step n is n/2 units (4px each by
    /// default, as Tailwind's `gap-n`), and the layout primitives' `.gap(n)` uses it.
    pub space: &'static str,
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens {
            light: Palette {
                bg: "#ffffff",
                fg: "#09090b",
                muted: "#71717a",
                line: "#e4e4e7",
                surface: "#fafafa",
                card: "#ffffff",
                popover: "#ffffff",
                secondary: "#f4f4f5",
                accent: "#f4f4f5",
                on_accent: "#18181b",
                primary: "#18181b",
                on_primary: "#fafafa",
                input: "#e4e4e7",
                ring: "#a1a1aa",
                danger: "#ce2c31",
                ok: "#218358",
                warn: "#ab6400",
            },
            dark: Palette {
                bg: "#09090b",
                fg: "#fafafa",
                muted: "#a1a1aa",
                line: "#27272a",
                surface: "#18181b",
                card: "#18181b",
                popover: "#18181b",
                secondary: "#27272a",
                accent: "#27272a",
                on_accent: "#fafafa",
                primary: "#e4e4e7",
                on_primary: "#18181b",
                input: "#3f3f46",
                ring: "#71717a",
                danger: "#ff9592",
                ok: "#3dd68c",
                warn: "#ffca16",
            },
            radius: "0.5rem",
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
            ":root {{\n  color-scheme: light dark;\n{light}  --lui-radius: {}; --lui-space: {};\n  --lui-space-1: calc(var(--lui-space) * 0.5); --lui-space-2: var(--lui-space); --lui-space-3: calc(var(--lui-space) * 1.5);\n  --lui-space-4: calc(var(--lui-space) * 2); --lui-space-6: calc(var(--lui-space) * 3); --lui-space-8: calc(var(--lui-space) * 4);\n  --lui-radius-sm: max(0px, var(--lui-radius) - 2px); --lui-radius-lg: calc(var(--lui-radius) + 4px);\n  --lui-shadow-xs: 0 1px 2px 0 rgb(0 0 0 / 0.05);\n  --lui-shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1);\n  --lui-overlay: rgb(0 0 0 / 0.5);\n}}\n\
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
    page(caps, title, theme, None, &[], true, body)
}

/// [`layout`] under a different set of [`Tokens`]: the overrides are emitted once, in a
/// `<style>` right after the stylesheet, so every component on the page picks them up.
pub fn layout_with(
    caps: &Caps,
    title: &str,
    theme: Theme,
    tokens: &Tokens,
    body: Markup,
) -> Markup {
    page(caps, title, theme, Some(tokens), &[], true, body)
}

/// The whole document: `tokens` overrides and then `css` (a user component's styles, see
/// `Page::css`) follow the stylesheet in the head, each once.
pub(crate) fn page(
    caps: &Caps,
    title: &str,
    theme: Theme,
    tokens: Option<&Tokens>,
    css: &[&str],
    script: bool,
    body: Markup,
) -> Markup {
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
                @if let Some(t) = tokens { style class="lui-tokens" { (PreEscaped(t.css())) } }
                @if !css.is_empty() { style class="lui-user" { @for c in css { (PreEscaped(crate::minify_css(c))) } } }
            }
            body {
                (header())
                main id="main" { (body) }
                (caps::beacons(caps))
                @if script { (enhance::script_tag()) }
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
        header class="lui-header" {
            a href="/" { strong { "loco-ui" } }
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

/* The --lui-* tokens come first in stylesheet(), from Tokens::default().css(). */

* { box-sizing: border-box; }
/* Type: the system stack only, no web font. Body text is 1rem/1.5; controls, tables and
   menus use shadcn's text-sm (0.875rem/1.25rem); weights are 500 for labels and buttons,
   600 for headings. Numbers in tables and stats are tabular. */
:root {
  --lui-font-sans: ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  --lui-font-mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
}
html {
  font-family: var(--lui-font-sans); line-height: 1.5;
  -webkit-text-size-adjust: 100%; -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale;
}
body { margin: 0; background: var(--lui-bg); color: var(--lui-fg); }
main { max-width: 52rem; margin: 0 auto; padding: calc(var(--lui-space) * 4) calc(var(--lui-space) * 2) calc(var(--lui-space) * 8); }
p, li { max-width: 44rem; }
.lui-header {
  display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 1rem;
  max-width: 52rem; margin: 0 auto; padding: calc(var(--lui-space) * 2);
  color: var(--lui-muted);
}
.lui-header a { color: var(--lui-fg); text-decoration: none; font-size: 1rem; letter-spacing: -0.01em; }
.lui-header span { font-size: 0.875rem; }
.lui-header strong { font-weight: 600; }
h1 { font-size: 2.25rem; line-height: 2.5rem; letter-spacing: -0.025em; font-weight: 600; margin: 0 0 0.75rem; }
h2 { font-size: 1.5rem; line-height: 2rem; letter-spacing: -0.0125em; font-weight: 600; margin: 2rem 0 0.5rem; }
h3 { font-size: 1.125rem; line-height: 1.75rem; font-weight: 600; }
p { margin: 0 0 1rem; }
a { color: var(--lui-primary); text-underline-offset: 0.15em; text-decoration-thickness: 1px; }
code {
  font-family: var(--lui-font-mono); font-size: 0.875em;
  background: var(--lui-surface); border: 1px solid var(--lui-line); border-radius: var(--lui-radius-sm); padding: 0.05em 0.35em;
}
/* Controls are styled in button.rs and input.rs. Focus everywhere is a 3px ring at 50%;
   aria-invalid turns borders and the ring to --lui-danger. */
details > summary { cursor: pointer; font-weight: 500; }
:focus-visible { outline: 3px solid color-mix(in srgb, var(--lui-ring) 50%, transparent); outline-offset: 0; }
[aria-invalid=true] { border-color: var(--lui-danger); }
[aria-invalid=true]:focus-visible { outline-color: color-mix(in srgb, var(--lui-danger) 20%, transparent); }
/* A swap root or form with a request in flight (set by the enhancement script only). The
   fade waits so a fast answer never flickers; --lui-busy: 1 turns it off. */
/* Gap steps for the layout primitives (stack, cluster, grid, split); their default gaps sit
   in :where() so one of these always wins. */
.lui-gap-0 { gap: 0; }
.lui-gap-1 { gap: var(--lui-space-1); }
.lui-gap-2 { gap: var(--lui-space-2); }
.lui-gap-3 { gap: var(--lui-space-3); }
.lui-gap-4 { gap: var(--lui-space-4); }
.lui-gap-6 { gap: var(--lui-space-6); }
.lui-gap-8 { gap: var(--lui-space-8); }
.lui-sr { position: absolute; width: 1px; height: 1px; margin: -1px; padding: 0; overflow: hidden; clip-path: inset(50%); text-wrap: nowrap; border: 0; }
[data-lui-busy] { opacity: var(--lui-busy, 0.6); transition: opacity 0.15s 0.2s; cursor: progress; }
table { border-collapse: collapse; width: 100%; font-size: 0.875rem; line-height: 1.25rem; font-variant-numeric: tabular-nums; }
/* shadcn Table: h-10 heads in the muted colour, p-2 cells, a rule under each row, muted/50 hover. */
th, td { text-align: left; padding: 0.5rem; border-bottom: 1px solid var(--lui-line); vertical-align: middle; }
th { height: 2.5rem; color: var(--lui-muted); font-weight: 500; white-space: nowrap; }
tbody tr { transition: background-color 0.15s; }
tbody tr:hover { background: color-mix(in srgb, var(--lui-accent) 50%, transparent); }
.lui-note { color: var(--lui-muted); font-size: 0.875rem; }
/* The dashed boxes on the demo's layout page. */
.lui-layout-tile { padding: calc(var(--lui-space) * 1.5); border: 1px dashed var(--lui-input); border-radius: var(--lui-radius-sm); background: var(--lui-surface); font-size: 0.875rem; }
.lui-yes { color: var(--lui-ok); font-weight: 600; }
.lui-no { color: var(--lui-danger); font-weight: 600; }

/* Demo shell, after the shadcn docs: toolbar with the way back and the theme switch, the
   lede under a title, "built on" as outline badges, the plate (a preview box over a muted
   code block), and the index as a grid of cards per group. */
.lui-toolbar { display: flex; flex-wrap: wrap; justify-content: space-between; align-items: center; gap: var(--lui-space); margin: 0 0 calc(var(--lui-space) * 3); min-height: 2.25rem; }
.lui-popover-row { display: flex; justify-content: space-between; gap: var(--lui-space); margin-bottom: calc(var(--lui-space) * 2); }
.lui-back { color: var(--lui-muted); text-decoration: none; font-size: 0.875rem; font-weight: 500; }
.lui-back::before { content: "\2190"; margin-right: 0.35em; }
.lui-back:hover { color: var(--lui-fg); }
.lui-lede { font-size: 1.125rem; line-height: 1.75rem; color: var(--lui-muted); margin-bottom: 1rem; }
.lui-built { color: var(--lui-muted); font-size: 0.875rem; margin: 0 0 1.5rem; }
.lui-built code, .lui-index li code {
  display: inline-block; margin: 0 0.25rem 0.25rem 0; padding: 0.125rem 0.5rem; white-space: nowrap;
  font-size: 0.75rem; line-height: 1rem; font-weight: 500; color: var(--lui-fg);
  background: transparent; border: 1px solid var(--lui-line); border-radius: var(--lui-radius-sm);
}
/* A component page's plate: the live component on a stage, the code that drew it joined
   underneath. One per page; no transform, overflow or contain on the stage, so dialogs,
   drawers and toasts still escape it. */
.lui-plate { margin: 0 0 2rem; }
.lui-stage {
  padding: calc(var(--lui-space) * 5) calc(var(--lui-space) * 4); background: var(--lui-bg);
  border: 1px solid var(--lui-line); border-bottom: 0; border-radius: var(--lui-radius-lg) var(--lui-radius-lg) 0 0;
}
.lui-stage > :last-child { margin-bottom: 0; }
.lui-stage h2:first-child { margin-top: 0; }
@media (max-width: 40rem) { .lui-stage { padding: calc(var(--lui-space) * 3) calc(var(--lui-space) * 2); } }
.lui-snippet {
  margin: 0; max-width: none; overflow: hidden;
  border: 1px solid var(--lui-line); border-radius: 0 0 var(--lui-radius-lg) var(--lui-radius-lg);
  background: var(--lui-surface);
}
.lui-snippet figcaption {
  display: flex; flex-wrap: wrap; justify-content: space-between; gap: 0.25rem 1rem;
  padding: 0.5rem calc(var(--lui-space) * 2); border-bottom: 1px solid var(--lui-line);
  color: var(--lui-muted); font-size: 0.75rem;
}
.lui-snippet figcaption span:first-child { color: var(--lui-fg); font-weight: 500; font-family: var(--lui-font-mono); }
.lui-snippet pre { margin: 0; padding: calc(var(--lui-space) * 2); overflow-x: auto; scrollbar-color: var(--lui-line) transparent; line-height: 1.7; tab-size: 4; }
.lui-snippet pre code { background: none; border: 0; padding: 0; font-size: 0.8125rem; color: var(--lui-fg); }
/* Highlighted Rust in the status colours, as a GitHub-like theme: keywords in danger, strings
   in ok, numbers and types in warn, comments muted, macros bold. */
.lui-hl-k { color: var(--lui-danger); }
.lui-hl-s { color: var(--lui-ok); }
.lui-hl-n, .lui-hl-t { color: var(--lui-warn); }
.lui-hl-c { color: var(--lui-muted); font-style: italic; }
.lui-hl-m { color: var(--lui-fg); font-weight: 600; }
.lui-hl-f { color: var(--lui-fg); }
/* The demo's props tables under the snippet: one <details> per builder. */
.lui-props { max-width: none; margin: 0 0 2rem; }
.lui-props details { border: 1px solid var(--lui-line); border-radius: var(--lui-radius); margin: 0 0 0.5rem; }
.lui-props summary { display: flex; flex-wrap: wrap; align-items: baseline; gap: 0.25rem 0.5rem; padding: 0.5rem 0.75rem; cursor: pointer; }
.lui-props summary span { margin-left: auto; color: var(--lui-muted); font-size: 0.8125rem; }
.lui-props-scroll { overflow-x: auto; border-top: 1px solid var(--lui-line); }
.lui-props table { width: 100%; border-collapse: collapse; font-size: 0.8125rem; }
.lui-props th, .lui-props td { text-align: left; vertical-align: top; padding: 0.375rem 0.75rem; border-bottom: 1px solid var(--lui-line); }
.lui-props th { color: var(--lui-muted); font-weight: 500; white-space: nowrap; }
.lui-props tbody tr:last-child td { border-bottom: 0; }
.lui-props td:nth-child(-n+2), .lui-props td:nth-child(4), .lui-props td:nth-child(5) { white-space: nowrap; }
.lui-props td:nth-child(3) { min-width: 10rem; }
.lui-props td:last-child { min-width: 16rem; }
.lui-index { max-width: none; }
.lui-index h2 { margin: 3rem 0 0.25rem; font-size: 1.5rem; line-height: 2rem; }
.lui-index-layer { margin: 0 0 1rem; color: var(--lui-muted); font-size: 0.875rem; }
.lui-index h3 { margin: 1.5rem 0 0.75rem; font-size: 0.875rem; font-weight: 500; color: var(--lui-muted); text-transform: uppercase; letter-spacing: 0.05em; }
.lui-index ul { list-style: none; margin: 0; padding: 0; display: grid; grid-template-columns: repeat(auto-fill, minmax(15rem, 1fr)); gap: 1rem; }
/* Each component is a card; its title link stretches over the whole card. */
.lui-index li {
  position: relative; display: grid; align-content: start; gap: 0.5rem; padding: 1.25rem; max-width: none;
  background: var(--lui-card); border: 1px solid var(--lui-line); border-radius: var(--lui-radius-lg);
  box-shadow: var(--lui-shadow-xs); transition: background-color 0.15s;
}
.lui-index li:hover { background: color-mix(in srgb, var(--lui-accent) 50%, var(--lui-card)); }
.lui-index li a { font-size: 1rem; font-weight: 600; line-height: 1.5rem; text-decoration: none; color: var(--lui-fg); }
.lui-index li a::after { content: ""; position: absolute; inset: 0; border-radius: inherit; }
.lui-index li a:focus-visible { outline: none; }
.lui-index li:has(a:focus-visible) { outline: 3px solid color-mix(in srgb, var(--lui-ring) 50%, transparent); }
.lui-index li p { margin: 0 0 0.5rem; font-size: 0.875rem; color: var(--lui-muted); }
.lui-index li span { display: block; }

@media (prefers-reduced-motion: reduce) {
  *, ::before, ::after { animation-duration: 0.01ms !important; transition-duration: 0.01ms !important; }
  ::view-transition-group(*), ::view-transition-old(*), ::view-transition-new(*) { animation: none !important; }
}
"#;
