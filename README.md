# webonsive

Interactive HTML components for Rust servers that work with JavaScript turned off. Axum + Maud.

Every component is a plain function returning `Markup`. Interactivity comes from the HTML/CSS
platform and ordinary form round trips. One optional 10 KB script (`/wo/enhance.js`) makes the
same markup update in place; see "How the script works" below. Every page works identically
with the script blocked; that is the only `<script>` tag allowed, and a test enforces it.

```rust
use axum::{Router, routing::get};
use maud::{Markup, html};
use webonsive::{Ui, dialog};

// `Ui` is what the server knows about this browser, its theme and the page's UI state.
async fn hello(ui: Ui) -> Markup {
    ui.layout("Hello", html! {
        (dialog(&ui, "hi", "Say hi", html! { p { "Hello from a <dialog>." } }))
    })
}

let app = Router::new()
    .route("/", get(hello))
    .merge(webonsive::caps::router())     // the beacons that tell the server what the browser supports
    .merge(webonsive::enhance::router()); // the optional script
```

With `webonsive = { features = ["axum"] }`. Without Axum, `dialog(&Caps::all(), …)` returns
the same `Markup`; `webonsive/examples/hyper_server.rs` shows a raw hyper server.

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
- Signatures are uniform: `name(&caps, ...required) -> Markup` for the common case and
  `name_with(&caps, ...required, options)` for the rest, where options is a plain `XOptions`
  struct with `Default` and one setter per field named after the attribute it sets
  (`DialogOptions::default().danger().state(&ui.state)`). `&ui` works wherever `&caps` does.
  No macros beyond `html!`.
- `Caps` is server-side feature detection with no script: `@supports` beacons set one cookie per
  capability, and each component emits only the variant that browser needs (see `/caps`).
  `?caps=popover,anchor` on any URL forces a set. The protocol is three plain functions
  (`Caps::from_cookie_header`, `Caps::from_query`, `caps::beacon_cookie`); Axum only wraps them.
- Output HTML is semantic with one `wo-<component>` class per root. `curl` any page and read it.
- CSS lives beside its component as `const CSS`. Theming is via `--wo-*` custom properties only
  (`bg`, `surface`, `fg`, `muted`, `line`, `accent`, `on-accent`, `danger`, `ok`, `warn`, `radius`, `space`).
  `layout::Tokens` holds them for light and dark, `layout_with` applies another set once per
  page, and `docs/theming.md` says what each one affects and which pairs must keep contrast.
  `/?palette=linen` in the demo is the same index under a second palette.
  A test fails if any component CSS names a colour instead of a token.

## How the script works

`/wo/enhance.js` is one file, plain ES2020, served with a content hash so it caches forever
and compatible with `script-src 'self'`. It never changes what the server sends: it reads a
few `data-wo-*` attributes and does in place what the browser would have done as a full
navigation. Without it every attribute is inert and every control is a normal form or link.

| Attribute | On | What the script does | Without the script |
|---|---|---|---|
| `id` + `data-wo="swap"` | a root element | Forms and links inside it are fetched; the element of the same `id` in the answer replaces the root. Flash, `<title>`, `data-theme` and the URL follow. Requests on one root are queued. | Normal navigation to the same URL. |
| `data-wo-target="#id"` | a form or link, or an ancestor | Swaps that root instead of the closest one, so a control can sit anywhere. | Same navigation. |
| `data-wo-swap="outer\|inner\|append\|prepend"` | with `data-wo-target` | How the answer lands: replace the root, replace its children, add at the end or the start. | Same navigation; the full page already shows the result. |
| `data-wo-oob="outer\|inner\|…"` | an element in the answer | Replaces the element of the same `id` anywhere in the page and is dropped from the main swap. | The full page shows it in place. |
| `Wo-Enhance: 1` | the request header | Sent on every enhanced request, so a handler may answer with only the fragment it needs to (`/swap` does). | Not sent; the handler returns the page. |
| `data-wo-busy` + `aria-busy="true"` | set by the script on the root and the form | Present while a request is in flight; submit buttons are disabled meanwhile; `[data-wo-busy]` fades to `--wo-busy` (0.6). | Never set. |
| `data-wo-indicator="#id"` | a form or link | The named element (authored with `hidden`) is shown while the request runs. | Stays hidden. |
| `data-wo-push="false"` | a form or link | The URL does not change. Links push a history entry by default, forms replace it. | Normal navigation. |
| `data-wo-replace` | a form or link | `replaceState` instead of `pushState`. | Normal navigation. |
| Back and Forward | | Each swap stores a copy of every root in the history entry; Back and Forward restore from it with no request. Entries without a copy are re-fetched. | Normal history. |
| `wo:swap` | a bubbling `CustomEvent` on the swapped root | `detail` is `{ id, url, mode }`, for anything that must react; no listener ships with the crate. | Never fires. |

A request that fails (network down, the answer has no element of that `id`) becomes the
navigation the browser would have made, so the server's answer is always seen. The script also
mirrors `<input type=range>` and `type=color` values while they move, opens the `:target`
dialog fallback as a real modal, closes the `<details>` popover fallback on outside click, and
searches a combobox as you type. `scripts/browser-check.mjs` proves each of these in headless
Firefox; the Blitz suite proves every route with no script engine at all.

## Feature matrix

Generated from `webonsive::spec::SPECS` by `cargo run -p demo -- spec write` (a test fails if it
drifts). Versions are the first release of each engine with the feature, from MDN
browser-compat-data; `no` means unshipped, so that browser gets the fallback.

<!-- matrix:start -->
| Component | Platform features | Chrome / Firefox / Safari | Fallback | Needs JS? |
|---|---|---|---|---|
| Enhancement script | `fetch`, `history.pushState`, `document.startViewTransition`, `CustomEvent` | 42 / 39 / 10.1; 5 / 4 / 5; 111 / 144 / 18; 15 / 11 / 6 | none needed: without the script every form and link is a normal navigation and every data-wo-* attribute is inert | No |
| Layout | `@view-transition`, `prefers-color-scheme`, `custom properties` | 126 / no / 18.2; 76 / 67 / 12.1; 49 / 31 / 9.1 | plain navigations (root never cross-fades); colours still switch by media query and data-theme | No |
| Capability beacons | `@supports`, `selector()`, `background images`, `cookies` | 28 / 22 / 9; 83 / 69 / 14.1; 1 / 1 / 1; 1 / 1 / 1 | unknown browser gets every fallback; the first view always does | No |
| Dialog | `<dialog>`, `command="show-modal"`, `<form method="dialog">`, `closedby` | 37 / 98 / 15.4; 135 / 144 / 26.2; 37 / 98 / 15.4; 134 / 141 / 26 | link to #id opens it through a :target rule, chosen server-side; the confirm footer is a plain form either way | No |
| Popover menu | `popover`, `anchor-name` | 114 / 125 / 17; 125 / 147 / 26 | no anchor: UA-centred popover; no popover: <details> dropdown (submenus nested); actions are plain post forms either way | No |
| Tabs | `<details name`, `display: contents`, `::details-content`, `view-transition-name` | 120 / 130 / 17.2; 65 / 37 / 11.1; 131 / 143 / 18.4; 111 / 144 / 18 | accordion markup, chosen server-side; the narrow-screen select has a Go button | No |
| Accordion | `<details name`, `::details-content`, `interpolate-size` | 120 / 130 / 17.2; 131 / 143 / 18.4; 129 / no / no | plain <details>: no exclusivity, no animation; expand/collapse and every toggle are links either way | No |
| Combobox | `<datalist>`, `<optgroup>`, `<search>`, `aria-live` | 20 / 4 / 12.1; 20 / 4 / 12.1; 118 / 118 / 17; 1 / 1 / 1 | none needed: chips, results and the create row are links and forms | Partly: static suggestions and per-submit results; live filtering and arrow keys into the results need script |
| Load-more list | `view-transition-name`, `scroll-margin` | 111 / 144 / 18; 69 / 90 / 14.1 | plain navigation to ?page=n#more | Partly: click-to-load; scroll-to-load needs script |
| Table | `?sort=<col>&dir=asc|desc`, `<search>`, `aria-sort`, `form attribute`, `<details>`, `<colgroup>`, `tabular-nums`, `position: sticky`, `view-transition-name`, `aria-busy` | 1 / 1 / 1; 118 / 118 / 17; 1 / 1 / 1; 10 / 4 / 5.1; 12 / 49 / 6; 1 / 1 / 1; 52 / 34 / 9.1; 56 / 32 / 13; 111 / 144 / 18; 1 / 1 / 1 | none needed: sorting, filtering, column choice and the bulk form are plain navigations and posts; no select-all without script | No |
| Paged table | `?page=n`, `<select>`, `<input type="number">`, `<output>` | 1 / 1 / 1; 1 / 1 / 1; 6 / 29 / 5.1; 10 / 4 / 7 | none needed: every control is a link or a form | No |
| Wizard | `<form method="post">`, `aria-current="step"`, `<fieldset>`, `formnovalidate`, `<progress>` | 1 / 1 / 1; 1 / 1 / 1; 1 / 1 / 1; 4 / 4 / 5; 8 / 16 / 6 | none needed: one form per step, PRG between them | No |
| Validated form | `required`, `pattern`, `:user-invalid`, `<fieldset>`, `<output>`, `type=date`, `accept`, `field-sizing` | 4 / 4 / 5; 4 / 4 / 5; 119 / 88 / 16.5; 1 / 1 / 1; 10 / 4 / 7; 20 / 57 / 14.1; 1 / 1 / 1; 123 / no / no | server re-renders with messages; no early styling; textareas keep their rows; the counter shows the submitted length | No |
| Counter | `<form method="post">`, `<button name value>`, `<input type="number">`, `cookie` | 1 / 1 / 1; 1 / 1 / 1; 6 / 29 / 5.1; 1 / 1 / 1 | none needed | No |
| Theme toggle | `prefers-color-scheme`, `color-scheme`, `cookie` | 76 / 67 / 12.1; 81 / 96 / 13; 1 / 1 / 1 | OS preference | No |
| Flash | `cookie`, `role="status"`, `role="alert"`, `@keyframes`, `prefers-reduced-motion` | 1 / 1 / 1; 1 / 1 / 1; 1 / 1 / 1; 43 / 16 / 9; 74 / 63 / 10.1 | without CSS animations the message stays | No |
| UI state | `links`, `cookies`, `303 See Other` | 1 / 1 / 1; 1 / 1 / 1; 1 / 1 / 1 | without cookies, state still travels in links on one page | No |
| Select | `<selectedcontent>`, `appearance: base-select`, `<optgroup label>`, `formmethod` | 135 / no / 27; 135 / no / 27; 1 / 1 / 1; 9 / 4 / 5.1 | plain <select>, chosen server-side | No |
| Range | `<input type="range">`, `<datalist>`, `pointer-events` | 4 / 23 / 3.1; 20 / 110 / 12.1; 1 / 1 / 1 | ticks not drawn | Partly: value shown after submit; live mirroring needs script |
| Color | `<input type="color">`, `color-mix()` | 20 / 29 / 12.1; 111 / 113 / 16.2 | text field accepting #rrggbb | No |
| Streaming | `<template shadowrootmode="open">`, `<slot name`, `Chunked transfer` | 111 / 123 / 16.4; 53 / 63 / 10; 1 / 1 / 1 | in-order streaming with in-place splicing | No |
| Toast | `position: fixed`, `role="status"`, `role="alert"`, `@keyframes`, `prefers-reduced-motion` | 1 / 1 / 1; 1 / 1 / 1; 1 / 1 / 1; 43 / 16 / 9; 74 / 63 / 10.1 | without CSS animations toasts stay until the next page | No |
| Breadcrumbs | `aria-current="page"`, `::before`, `<details>` | 1 / 1 / 1; 1 / 1 / 1; 12 / 49 / 6 | none needed | No |
| Skeleton | `aria-busy`, `role="status"`, `@keyframes`, `prefers-reduced-motion` | 1 / 1 / 1; 1 / 1 / 1; 43 / 16 / 9; 74 / 63 / 10.1 | without CSS animations the bars are still | No |
| Empty state | `<form method="post">` | 1 / 1 / 1 | none needed | No |
| Stat | `repeat(auto-fit` | 57 / 52 / 10.1 | none needed | No |
| Drawer | `<dialog>`, `command="show-modal"`, `closedby`, `@starting-style`, `@media` | 37 / 98 / 15.4; 135 / 144 / 26.2; 134 / 141 / 26; 117 / 129 / 17.5; 1 / 1 / 1 | link to #id and a :target rule; open from the server | No |
| Command palette | `popover`, `<datalist>`, `<search>`, `accesskey` | 114 / 125 / 17; 20 / 4 / 12.1; 118 / 118 / 17; 1 / 1 / 1 | a <details> disclosure with the same form | Partly: arrow keys through live results and a global Ctrl+K need script |
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
- **Impossible without script:** filtering as you type against server data, mirroring a
  slider's value while it moves, and a modal opened on load (the optional script adds these
  three and moving the arrow keys into combobox results), infinite scroll, persisting client-side `<details>` toggles, optimistic UI, offline,
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
webonsive/src/ui.rs         Ui: caps, theme and UiState in one extractor; ui.flash(), ui.layout()
webonsive/src/flash.rs      one-shot status banners: levels, stacked, dismiss, auto-hide
webonsive/src/select.rs     <select> with <selectedcontent> where supported
webonsive/src/range.rs      <input type=range> with ticks and a server-rendered <output>
webonsive/src/color.rs      <input type=color> with a swatch of the saved value
webonsive/src/spec.rs       SPECS: features, per-browser baselines, fallback, needs_js
webonsive/examples/         render_page (no server), axum_server (--features axum), hyper_server (--features http)
spec/components.json        generated from SPECS (cargo run -p demo -- spec write)
docs/state.md               how state works with no script
docs/caps.md                how the beacons work, cookie format, the first view, adding a flag
docs/theming.md             every --wo-* token, contrast pairs, a second palette as a Tokens value
docs/ergonomics.md          audit of every call site and how M17 makes them shorter
docs/latency.md             what made pages faster, what did not, and the order to apply it to your server
webonsive/src/<name>.rs     one component each: dialog, popover, tabs, accordion, table, paged_table, wizard,
                            combobox, pager, form, counter, theme, toast, breadcrumbs, skeleton,
                            empty_state, stat, drawer, palette (command palette)
demo/src/lib.rs             Axum routes, ≤15 lines each, plus the no-script test
webonsive-test/src/lib.rs   Page: render a route through Blitz, assert layout, screenshot
webonsive-test/tests/       every route rendered and captured; layout assertions
webonsive-test/examples/probe.rs   render any HTML file through Blitz, print boxes
tests/shots/                PNG per route and capability level, from Blitz
scripts/verify.sh           the full verification pass
FINDINGS.md                 what works, what needs a fallback, what is impossible without JS
CHANGELOG.md                what each version added; both crates share the version
```

## License

MIT, see [LICENSE](LICENSE).
