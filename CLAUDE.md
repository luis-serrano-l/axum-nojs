# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`axum-nojs` is a no-JavaScript-required component library for Rust servers (Axum + Maud).
Every component is a plain `fn name(caps, ...) -> Markup`; interactivity comes from the HTML/CSS
platform (`<dialog>`, `popover`, invoker commands, `<details name>`, `<datalist>`, view
transitions) and ordinary form round trips. **Every page must work with script disabled.** One
optional script, `axum-nojs/src/enhance.rs`, served at `/nojs/enhance.js`, upgrades swap roots
(`id` + `data-nojs="swap"`) to fetch-and-replace in place. It is the only `<script>` allowed: a
test enforces that (`demo/src/lib.rs::tests::pages_ship_only_the_enhancement_script`), Blitz
(no script engine) proves every route works without it, and `scripts/browser-check.mjs` drives
headless Firefox to prove the script does its job. Never add inline script, handlers or a
second file; when something needs more, extend `enhance.rs` and keep the no-script path intact.

## Commands

```sh
cargo run -p demo                  # demo server at http://127.0.0.1:3000
cargo dev                          # same, rebuilt and restarted when crate source or a manifest changes (needs cargo-watch)
cargo test                         # all tests, including the only-one-script test and doctests
cargo test -p demo pages_ship_only_the_enhancement_script   # the single enforcement test
node scripts/browser-check.mjs     # headless Firefox via geckodriver: the script works (needs a built demo)
cargo test -p axum-nojs --doc      # component doc examples
cargo clippy --all-targets         # must be clean before a roadmap milestone counts as done
cargo test -p axum-nojs-test       # Blitz layout assertions + screenshots into tests/shots/
cargo bench -p axum-nojs           # criterion: stylesheet, layout, table with 1 000 rows, paged table, UiState
scripts/bench.sh                   # latency baseline: curl p50/p95 TTFB and Firefox navigation timing on 3001
scripts/look.sh                    # Firefox shots of every page (light/dark, 1280/420) beside the shadcn docs, into target/look/
scripts/verify.sh                  # everything above plus a <script> grep and the browser check; run before committing
```

## Workspace layout

- `axum-nojs-caps/` – the detection crate: `Caps`/`Cap` bitset, `@supports` beacons, cookie and
  query parsing, `beacon_cookie`, and an `axum` feature with the extractor and beacon route.
  `axum-nojs` re-exports it as `axum_nojs::caps`, so nothing else changes.
- `axum-nojs/` – the library crate. Depends only on `maud` and `axum-nojs-caps`. Feature `http` adds
  `Redirect::into_http` and `Streamed` (a chunk stream); feature `axum` adds the `Ui` extractor,
  `IntoResponse` for `Page`/`Redirect`/`Streamed`, the `/nojs/caps` beacon route, and
  `Saved<T>` (the only use of serde). Everything else is plain functions over strings
  (`Ui::from_request`, `Page::into_string`, `Redirect::set_cookies`, `caps::beacon_cookie`).
- `demo/` – Axum lib + binary, one route per component, one `use axum_nojs::prelude::*`.
  Each component page shows the code between its `// code: <href>` and `// end code`
  markers (a test fails if a component page has none), so keep the markers around the
  component call when editing a route.
  Handlers take `ui: Ui` (plus `Saved<T>` / `Form<T>` when they need them) and return `Page`
  or `Redirect`; they only parse input and call `axum-nojs`; keep each route around 15 lines. The only-one-script test lives here
  and hits every route via `tower::oneshot`, so **add new demo routes to `PATHS`** (the
  screenshot test in `axum-nojs-test` uses the same list).
- `axum-nojs-test/` – Blitz-based test harness: `Page::render(router, path, cookie)` then
  `exists / is_visible / bbox / text / display / screenshot`. Blitz gaps go in `FINDINGS.md`
  with an issue link, never as skipped assertions without a comment.

## Component conventions (follow exactly when adding one)

1. One component = one file `axum-nojs/src/<name>.rs`, registered in `lib.rs` as `pub mod`.
   Only types a caller names go in the `prelude` (most never do: builders are reached from `ui`).
2. File starts with a `//!` header: what it does, **Platform features** (with browser baseline
   versions), **What it does not do without script**, **Fallback**, and a runnable ```` ```rust ```` usage example (these are doctests).
3. CSS lives beside the component as `pub const CSS: &str` and must be appended to the array in
   `stylesheet()` in `lib.rs`; `ui.page()` inlines that once per page. Theming only through
   `--nojs-*` custom properties defined in `layout.rs`.
4. Root element carries a single `nojs-<component>` class; sub-parts use `nojs-<component>-<part>`.
   Output should be readable via `curl`.
5. No macros beyond `html!`. A component is a builder struct holding `&Ui` (or what it needs
   from it) plus an `impl Ui { pub fn <name>(&self, required…) -> <Name> }` in the same file,
   and `impl Render for <Name>`, so a route writes `(ui.<name>(..).setter()..)` inside `html!`.
   Required arguments stay in the call (text first); everything else is a chained setter. An
   id the caller does not care about is derived from the label with `crate::slug`, with an
   `.id()` override. Components read their own input from `ui` (`ui.param`, `ui.params`,
   `ui.state`) instead of taking it as an argument. For lists (fields, menu items, tabs,
   columns, commands), an adder per item (`.text(..)`, `.link(..)`, `.tab(..)`) and
   modifiers that apply to the item added last (`.required()`, `.icon()`, `.badge()`). No
   `_with` twins and no `XOptions` structs. The doc header shows the common call first, then
   one with the setters. Setter names follow the HTML attribute or element they set
   (`.maxlength()`, `.placeholder()`, `.closedby()`). A setter with no argument switches
   something on (`.required()`, `.danger()`, `.multi()`); one that takes a `bool` is one a
   route sets from a condition (`.open(..)`, `.loading(..)`). Branch on `caps.has(Cap::X)` and
   emit only one variant, never both.
   A root that should update in place gets `id=(enhance::swap_id(prefix, key))` and
   `data-nojs="swap"`; the markup must behave identically without the script.
6. Server-held state (theme, counter, active tab) travels via cookie or `?query=`; mutations use
   `<form method="post">` + redirect (Post/Redirect/Get), never GET side effects.
7. Update the README feature matrix and Findings when a component or its fallback changes.
8. A component builds its parts from the primitives: `ui.button` / `Button::new(caps, ..)`
   (or `class="nojs-button"` on a `<summary>`), `ui.input` / `Input` with `.hide_label()` for a
   bare control, `ui.badge`, `ui.card`, `Icon`, and the layouts (`ui.stack`, `ui.cluster`,
   `ui.grid`, `ui.split`). Never a raw `<button>` or visible `<input>` with its own CSS: a
   component's CSS styles its parts by class (`.nojs-<component>-<part>`), and a test
   (`only_the_primitives_select_bare_buttons_and_inputs`) fails if it selects a bare `button`
   or `input`. Hidden inputs, `<select>`, range sliders and menu items are the exceptions.

## Roadmap

`ROADMAP.md` is the work queue: work top to bottom (M1 capability beacons → M2 streaming → M3
state model → M4 Blitz tests → M5 machine-readable spec → M6 release polish). A milestone is
done only when every box is ticked, clippy is clean, tests pass, and README is updated.
