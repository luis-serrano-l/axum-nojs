# webonsive

Zero-JavaScript interactive HTML components for Rust servers. Axum + Maud.

Every component is a plain function returning `Markup`. Interactivity comes from the HTML/CSS
platform and ordinary form round trips. No page ships a `<script>` tag, and a test enforces it.

```rust
use maud::html;
use webonsive::{Caps, layout, dialog, Theme};

// `Caps` is what the server knows about the browser; in Axum it is an extractor.
let caps = Caps::all();
let page = layout(&caps, "Hello", Theme::Auto, html! {
    (dialog(&caps, "hi", "Say hi", html! { p { "Hello from a <dialog>." } }))
});
```

## Run the demo

```sh
cargo run -p demo      # http://127.0.0.1:3000
cargo test             # includes: no route may contain "<script"
```

## How to read this crate

- One component = one file in `webonsive/src/`. Each starts with a `//!` header: what it does,
  the platform features it uses (with browser baseline), the fallback, a usage example.
- Signatures are uniform: `fn name(&caps, id, ...required, ...) -> Markup`. No macros beyond `html!`.
- `Caps` is server-side feature detection with no script: `@supports` beacons set one cookie per
  capability, and each component emits only the variant that browser needs (see `/caps`).
- Output HTML is semantic with one `wo-<component>` class per root. `curl` any page and read it.
- CSS lives beside its component as `const CSS`. Theming is via `--wo-*` custom properties only.

## Feature matrix

| Component | Platform feature | Baseline | Fallback | Needs JS? |
|---|---|---|---|---|
| Capability beacons | `@supports` + background-image beacons + cookies | 2015 (`selector()` 2022) | unknown browser gets every fallback | No |
| Dialog | `<dialog>`, `command`/`commandfor` invokers, `closedby`, `<form method=dialog>` | dialog 2022; invokers Chrome 135 / Firefox 144 / Safari 26.2 | `:target` overlay via `href="#id"` link, chosen server-side | No |
| Popover menu | `popover` + `popovertarget`, anchor positioning | popover 2024; anchors Chrome 125 / Firefox 147 / Safari 26 | no anchor: UA-centred popover; no popover: `<details>` dropdown | No |
| Tabs | `<details name>`, `display: contents`, `::details-content` + `order` | details name 2024; ::details-content Chrome 131 / Firefox 143 / Safari 18.4 | accordion markup, chosen server-side | No |
| Accordion | `<details name>`, `::details-content` transition | 2024 | non-exclusive `<details>` | No |
| Combobox | `<input list>` + `<datalist>`, `<search>` | 2020 / 2023 | none needed | Suggestions no; live results **yes** |
| Load-more list | links + `@view-transition { navigation: auto }` + `scroll-margin` | Chrome 126 / Safari 18.2, Firefox flag | plain navigation | Click-to-load no; scroll-to-load **yes** |
| Validated form | `required`/`pattern`/`min`, `:user-invalid`, Post/Redirect/Get | 2015 / 2023 | none needed | No |
| Counter | `<form method=post>`, `<button name value>`, cookie | forever | none needed | No |
| Theme toggle | `prefers-color-scheme`, cookie + `data-theme` | 2020 | OS preference | No |
| UI state | query string + `wo-ui` cookie via `UiState`; tab and accordion titles are links | 1997 | none needed | No |
| Flash + PRG | `303 See Other` + one-shot `wo-flash` cookie | 1997 | none needed | No |
| Streaming | `<template shadowrootmode>` on `<body>`, named `<slot>`s, chunked response | Chrome 111 / Firefox 123 / Safari 16.4 | in-order streaming with in-place splicing | No |

## Findings

**What works better than expected**
- Dialog, popover, tabs, accordion: fully interactive with zero round trips. Light dismiss,
  Escape, focus trapping, top layer, exclusivity: all free from the platform.
- View transitions make server round trips feel like in-place updates. The counter number
  morphs; the list grows without a flash.
- Out-of-order streaming works with no script: the page ships with `<slot>` placeholders inside
  a declarative shadow root on `<body>`, and each slow section is appended whenever it is ready.
  The parser slots it into place. Older browsers get in-order progressive rendering instead.
- `:user-invalid` gives validation UX that used to need a library.

**What needs a fallback today**
- Invoker commands need Chrome 135 / Firefox 144 / Safari 26.2. The server picks the `:target`
  dialog for anything older, so a page never carries both variants.
- CSS cannot feature-detect HTML attributes. `invokers` and `streaming_dsd` are detected through
  CSS features that shipped in the same releases; the proxies and their error bands are listed in
  `FINDINGS.md`.
- The first page view of a new browser is always the fallback variant: the beacons fire during
  that load, the cookie lands, and the second view is tailored. A `curl` client stays on
  fallbacks forever, which is what you want.

**What is impossible without script**
- Filtering results as you type against server data. Datalist covers static suggestions only.
- Infinite scroll. Cumulative pages with one click per page is the ceiling.
- Optimistic UI, offline behaviour, undo without a round trip.
- Drag and drop, resizable panes, canvas or charts drawn from data.
- Keeping `<details>` state across navigations without a round trip. `UiState` makes the round
  trip one link click and remembers it in a cookie; see `docs/state.md`.

**Verdict:** for content sites, admin panels, forms, settings pages and dashboards that refresh
per action, the platform is enough. For editors, real-time collaboration, and anything that
reacts per keystroke, it is not.

## Layout

```
webonsive/src/lib.rs        crate docs, re-exports, stylesheet()
webonsive/src/caps.rs       Caps bitset, @supports beacons, cookie parsing, /wo/caps route
webonsive/src/layout.rs     page shell + base CSS + beacons
webonsive/src/stream.rs     Streamed response: DSD slots out of order, in-order fallback
webonsive/src/state.rs      UiState (query + cookie), prg() redirect with flash
webonsive/src/flash.rs      one-shot status banner
docs/state.md               how state works with no script
webonsive/src/<name>.rs     one component each: dialog, popover, tabs, accordion,
                            combobox, pager, form, counter, theme
demo/src/main.rs            Axum routes, ≤15 lines each, plus the no-script test
FINDINGS.md                 what works, what needs a fallback, what is impossible without JS
```
