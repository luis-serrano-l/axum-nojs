# webonsive

Interactive HTML components for Rust servers that need no JavaScript. Axum + Maud.

Every component is a plain function returning `Markup`. Interactivity comes from the HTML/CSS
platform and ordinary form round trips. One optional 5 KB script (`/wo/enhance.js`) makes the
same markup update in place: forms and links inside a swap root (`id` + `data-wo="swap"`) are
fetched and only that root is replaced, so a counter clicked five times counts five without a
reload. Every page works identically with the script blocked; that is the only `<script>` tag
allowed, and a test enforces it.

```rust
use maud::html;
use webonsive::{Caps, layout, dialog, Theme};

// `Caps` is what the server knows about the browser; in Axum it is an extractor.
let caps = Caps::all();
let page = layout(&caps, "Hello", Theme::Auto, html! {
    (dialog(&caps, "hi", "Say hi", html! { p { "Hello from a <dialog>." } }, Default::default()))
});
```

## Run the demo

```sh
cargo run -p demo      # http://127.0.0.1:3000: index grouped by what the platform gives, every page links back
cargo dev              # same, restarted on every save (cargo install cargo-watch)
cargo test             # includes: only the enhancement <script> on any route, and Blitz layout tests
scripts/verify.sh      # build + clippy -D warnings + tests + screenshots + <script> grep + Firefox check
```

`webonsive-test` renders every route through [Blitz](https://github.com/DioxusLabs/blitz)
(Stylo + Taffy + vello_cpu, no script engine) and writes a PNG per route and capability level
to `tests/shots/`, which is also the proof that every route works with no script. What Blitz
cannot render is listed with issue links in `FINDINGS.md`. `scripts/browser-check.mjs` drives
headless Firefox through geckodriver to check the enhancement script (in-place counter, tabs,
search as you type, in-place table sort, wizard steps, live range output, theme).

## Use with any server

The crate depends on Maud alone. Axum is an optional feature, and everything it does is a
thin wrapper over plain functions on strings, so any server can do the same in a few lines:

| You need | Without a framework | With `--features http` | With `--features axum` |
|---|---|---|---|
| What the browser supports | `Caps::from_query(query)` then `Caps::from_cookie_header(cookies)` | same | `caps: Caps` extractor |
| The beacon route `GET /wo/caps?flag=x` | `caps::beacon_cookie(query)` → 204 + `Set-Cookie`, or 404 | same | `caps::router()` |
| Tab, accordion, dialog state | `UiState::from_request(path, query, cookies)`, `state.set_cookies()` | same | `state: UiState` extractor, return `(state, page)` |
| Post/Redirect/Get with a flash | `state::prg_parts(to, flash)` → 303, `Location`, `Set-Cookie` | `prg(to, flash)` → `http::Response<B>` | `prg(to, flash)` → `Response` |
| Out-of-order streaming | | `Streamed::into_stream()` → chunks | `impl IntoResponse for Streamed` |
| The optional script | serve `enhance::JS` at `enhance::SCRIPT_PATH` | same | `enhance::router()` |

`webonsive/examples/hyper_server.rs` is the whole of it on raw hyper: three components, the
beacon route, a POST answered with PRG, the script served by hand. `.into_string()` on any
component gives the HTML to another template engine.

## How to read this crate

- One component = one file in `webonsive/src/`. Each starts with a `//!` header: what it does,
  the platform features it uses (with browser baseline), the fallback, a usage example.
- Signatures are uniform: `fn name(&caps, id, ...required, options) -> Markup`. Anything past the
  required arguments is a plain `XOptions` struct with `Default` and builder setters
  (`DialogOptions::default().open(true)`), so the short call is `Default::default()`. No macros beyond `html!`.
- `Caps` is server-side feature detection with no script: `@supports` beacons set one cookie per
  capability, and each component emits only the variant that browser needs (see `/caps`).
  `?caps=popover,anchor` on any URL forces a set. The protocol is three plain functions
  (`Caps::from_cookie_header`, `Caps::from_query`, `caps::beacon_cookie`); Axum only wraps them.
- Output HTML is semantic with one `wo-<component>` class per root. `curl` any page and read it.
- CSS lives beside its component as `const CSS`. Theming is via `--wo-*` custom properties only
  (`bg`, `surface`, `fg`, `muted`, `line`, `accent`, `on-accent`, `danger`, `ok`, `radius`, `space`).
  `layout::Tokens` holds them for light and dark, `layout_with` applies another set once per
  page, and `docs/theming.md` says what each one affects and which pairs must keep contrast.
  `/?palette=linen` in the demo is the same index under a second palette.

## Feature matrix

Generated from `webonsive::spec::SPECS` by `cargo run -p demo -- spec write` (a test fails if it
drifts). Versions are the first release of each engine with the feature, from MDN
browser-compat-data; `no` means unshipped, so that browser gets the fallback.

<!-- matrix:start -->
| Component | Platform features | Chrome / Firefox / Safari | Fallback | Needs JS? |
|---|---|---|---|---|
| Enhancement script | `fetch`, `history.pushState`, `document.startViewTransition` | 42 / 39 / 10.1; 5 / 4 / 5; 111 / 144 / 18 | none needed: without the script every form and link is a normal navigation | No |
| Layout | `@view-transition`, `prefers-color-scheme`, `custom properties` | 126 / no / 18.2; 76 / 67 / 12.1; 49 / 31 / 9.1 | plain navigations (root never cross-fades); colours still switch by media query and data-theme | No |
| Capability beacons | `@supports`, `selector()`, `background images`, `cookies` | 28 / 22 / 9; 83 / 69 / 14.1; 1 / 1 / 1; 1 / 1 / 1 | unknown browser gets every fallback; the first view always does | No |
| Dialog | `<dialog>`, `command="show-modal"`, `<form method="dialog">` | 37 / 98 / 15.4; 135 / 144 / 26.2; 37 / 98 / 15.4 | link to #id opens it through a :target rule, chosen server-side | No |
| Popover menu | `popover`, `anchor-name` | 114 / 125 / 17; 125 / 147 / 26 | no anchor: UA-centred popover; no popover: <details> dropdown | No |
| Tabs | `<details name`, `display: contents`, `::details-content` | 120 / 130 / 17.2; 65 / 37 / 11.1; 131 / 143 / 18.4 | accordion markup, chosen server-side | No |
| Accordion | `<details name`, `::details-content`, `interpolate-size` | 120 / 130 / 17.2; 131 / 143 / 18.4; 129 / no / no | plain <details>: no exclusivity, no animation | No |
| Combobox | `<datalist>`, `<search>` | 20 / 4 / 12.1; 118 / 118 / 17 | none needed | Partly: static suggestions and per-submit results; live filtering needs script |
| Load-more list | `view-transition-name`, `scroll-margin` | 111 / 144 / 18; 69 / 90 / 14.1 | plain navigation to ?page=n#more | Partly: click-to-load; scroll-to-load needs script |
| Table | `?sort=<col>&dir=asc|desc`, `<search>`, `aria-sort`, `position: sticky`, `view-transition-name` | 1 / 1 / 1; 118 / 118 / 17; 1 / 1 / 1; 56 / 32 / 13; 111 / 144 / 18 | none needed: sorting and filtering are plain navigations | No |
| Paged table | `?page=n`, `<select name="per">`, `<output>` | 1 / 1 / 1; 1 / 1 / 1; 10 / 4 / 7 | none needed: every control is a link or a form | No |
| Wizard | `<form method="post">`, `aria-current="step"`, `<fieldset>` | 1 / 1 / 1; 1 / 1 / 1; 1 / 1 / 1 | none needed: one form per step, PRG between them | No |
| Validated form | `required`, `pattern`, `:user-invalid` | 4 / 4 / 5; 4 / 4 / 5; 119 / 88 / 16.5 | server re-renders with messages; no early styling | No |
| Counter | `<form method="post">`, `<button name value>`, `cookie` | 1 / 1 / 1; 1 / 1 / 1; 1 / 1 / 1 | none needed | No |
| Theme toggle | `prefers-color-scheme`, `color-scheme`, `cookie` | 76 / 67 / 12.1; 81 / 96 / 13; 1 / 1 / 1 | OS preference | No |
| Flash | `cookie`, `role="status"` | 1 / 1 / 1; 1 / 1 / 1 | none needed | No |
| UI state | `links`, `cookies`, `303 See Other` | 1 / 1 / 1; 1 / 1 / 1; 1 / 1 / 1 | without cookies, state still travels in links on one page | No |
| Select | `<selectedcontent>`, `appearance: base-select` | 135 / no / 27; 135 / no / 27 | plain <select>, chosen server-side | No |
| Range | `<input type="range">`, `<datalist>` | 4 / 23 / 3.1; 20 / 110 / 12.1 | ticks not drawn | Partly: value shown after submit; live mirroring needs script |
| Color | `<input type="color">` | 20 / 29 / 12.1 | text field accepting #rrggbb | No |
| Streaming | `<template shadowrootmode="open">`, `<slot name`, `Chunked transfer` | 111 / 123 / 16.4; 53 / 63 / 10; 1 / 1 / 1 | in-order streaming with in-place splicing | No |
<!-- matrix:end -->

## Findings

The short version. `FINDINGS.md` has the reasons, the proxies, the error bands and the Blitz
issues.

- **Works with no script:** dialog, popover, tabs, accordion, server-side feature detection,
  out-of-order streaming, URL + cookie state with Post/Redirect/Get, cross-navigation view
  transitions, constraint validation with `:user-invalid`, sortable/filterable/paged tables,
  multi-step wizards, headless layout tests through Blitz.
- **Needs a fallback today:** invoker commands, anchor positioning, `popover`,
  `::details-content`, `<details name>`, cross-document view transitions in Firefox,
  declarative shadow DOM, and the first page view of every browser (beacons not fired yet).
  All fallbacks are chosen server-side from `Caps`; a page never carries both variants.
- **Impossible without script:** filtering as you type against server data, infinite scroll,
  mirroring a slider's value while it moves,
  a modal opened on load, persisting client-side `<details>` toggles, optimistic UI, offline,
  undo, drag and drop, inline cell editing, canvas, and feature-detecting HTML attributes from CSS.

**Verdict:** for content sites, admin panels, forms, settings pages and dashboards that refresh
per action, the platform is enough. For editors, real-time collaboration, and anything that
reacts per keystroke, it is not.

## Layout

```
webonsive/src/lib.rs        crate docs, re-exports, stylesheet()
wo-caps/src/lib.rs          Caps bitset, @supports beacons, cookie parsing, /wo/caps route (own crate)
wo-caps/examples/hyper.rs   the beacons on raw hyper, one line per flag
webonsive/src/layout.rs     page shell + base CSS + beacons
webonsive/src/stream.rs     Streamed response: DSD slots out of order, in-order fallback (http feature)
webonsive/src/state.rs      UiState (query + cookie), prg_parts()/prg() redirect with flash
webonsive/src/flash.rs      one-shot status banner
webonsive/src/select.rs     <select> with <selectedcontent> where supported
webonsive/src/range.rs      <input type=range> with ticks and a server-rendered <output>
webonsive/src/color.rs      <input type=color> with a swatch of the saved value
webonsive/src/spec.rs       SPECS: features, per-browser baselines, fallback, needs_js
webonsive/examples/         render_page (no server), axum_server (--features axum), hyper_server (--features http)
spec/components.json        generated from SPECS (cargo run -p demo -- spec write)
docs/state.md               how state works with no script
docs/caps.md                how the beacons work, cookie format, the first view, adding a flag
docs/theming.md             every --wo-* token, contrast pairs, a second palette as a Tokens value
webonsive/src/<name>.rs     one component each: dialog, popover, tabs, accordion, table, paged_table, wizard,
                            combobox, pager, form, counter, theme
demo/src/lib.rs             Axum routes, ≤15 lines each, plus the no-script test
webonsive-test/src/lib.rs   Page: render a route through Blitz, assert layout, screenshot
webonsive-test/tests/       every route rendered and captured; layout assertions
webonsive-test/examples/probe.rs   render any HTML file through Blitz, print boxes
tests/shots/                PNG per route and capability level, from Blitz
scripts/verify.sh           the full verification pass
FINDINGS.md                 what works, what needs a fallback, what is impossible without JS
```
