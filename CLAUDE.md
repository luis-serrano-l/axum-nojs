# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`webonsive` is a no-JavaScript-required component library for Rust servers (Axum + Maud).
Every component is a plain `fn name(caps, ...) -> Markup`; interactivity comes from the HTML/CSS
platform (`<dialog>`, `popover`, invoker commands, `<details name>`, `<datalist>`, view
transitions) and ordinary form round trips. **Every page must work with script disabled.** One
optional script, `webonsive/src/enhance.rs`, served at `/wo/enhance.js`, upgrades swap roots
(`id` + `data-wo="swap"`) to fetch-and-replace in place. It is the only `<script>` allowed: a
test enforces that (`demo/src/lib.rs::tests::pages_ship_only_the_enhancement_script`), Blitz
(no script engine) proves every route works without it, and `scripts/browser-check.mjs` drives
headless Firefox to prove the script does its job. Never add inline script, handlers or a
second file; when something needs more, extend `enhance.rs` and keep the no-script path intact.

## Commands

```sh
cargo run -p demo                  # demo server at http://127.0.0.1:3000
cargo dev                          # same, rebuilt and restarted on every save (needs cargo-watch)
cargo test                         # all tests, including the only-one-script test and doctests
cargo test -p demo pages_ship_only_the_enhancement_script   # the single enforcement test
node scripts/browser-check.mjs     # headless Firefox via geckodriver: the script works (needs a built demo)
cargo test -p webonsive --doc      # component doc examples
cargo clippy --all-targets         # must be clean before a roadmap milestone counts as done
cargo test -p webonsive-test       # Blitz layout assertions + screenshots into tests/shots/
cargo bench -p webonsive           # criterion: stylesheet, layout, table with 1 000 rows, paged table, UiState
scripts/bench.sh                   # latency baseline: curl p50/p95 TTFB and Firefox navigation timing on 3001
scripts/verify.sh                  # everything above plus a <script> grep and the browser check; run before committing
```

## Workspace layout

- `wo-caps/` – the detection crate: `Caps`/`Cap` bitset, `@supports` beacons, cookie and
  query parsing, `beacon_cookie`, and an `axum` feature with the extractor and beacon route.
  `webonsive` re-exports it as `webonsive::caps`, so nothing else changes.
- `webonsive/` – the library crate. Depends only on `maud` and `wo-caps`. Feature `http` adds `prg` as an
  `http::Response` and `Streamed` (a chunk stream); feature `axum` adds the `Caps`/`UiState`
  extractors, `IntoResponse` impls and the `/wo/caps` beacon route on top. Everything else is
  plain functions over strings (`Caps::from_cookie_header`, `UiState::from_request`,
  `prg_parts`, `caps::beacon_cookie`). No serde.
- `demo/` – Axum lib + binary, one route per component. Handlers only parse input (query, form,
  cookie) and call `webonsive`; keep each route around 15 lines. The only-one-script test lives here
  and hits every route via `tower::oneshot`, so **add new demo routes to `PATHS`** (the
  screenshot test in `webonsive-test` uses the same list).
- `webonsive-test/` – Blitz-based test harness: `Page::render(router, path, cookie)` then
  `exists / is_visible / bbox / text / display / screenshot`. Blitz gaps go in `FINDINGS.md`
  with an issue link, never as skipped assertions without a comment.

## Component conventions (follow exactly when adding one)

1. One component = one file `webonsive/src/<name>.rs`, registered in `lib.rs` as `pub mod` and
   re-exported with `pub use`.
2. File starts with a `//!` header: what it does, **Platform features** (with browser baseline
   versions), **What it does not do without script**, **Fallback**, and a runnable ```` ```rust ```` usage example (these are doctests).
3. CSS lives beside the component as `pub const CSS: &str` and must be appended to the array in
   `stylesheet()` in `lib.rs`; `layout()` inlines that once per page. Theming only through
   `--wo-*` custom properties defined in `layout.rs`.
4. Root element carries a single `wo-<component>` class; sub-parts use `wo-<component>-<part>`.
   Output should be readable via `curl`.
5. No macros beyond `html!`. Signature order: `caps: &Caps` first, then `id`, required args,
   then options. More than three arguments after `id` means the rest go in a `<Name>Options`
   struct in the same file: `Default` impl, one builder setter per field, re-exported from
   `lib.rs` beside the function. The component then comes in two forms, like `layout` /
   `layout_with`: `<name>(caps, required…)` for the common case with no options (required text
   first; an id the caller does not care about is derived from its label with `crate::slug`),
   and `<name>_with(caps, …, options)` for everything else. The doc header shows the short
   form first and one full `_with` form. Branch on `caps.has(Cap::X)` and emit only one variant, never both.
   A root that should update in place gets `id=(enhance::swap_id(prefix, key))` and
   `data-wo="swap"`; the markup must behave identically without the script.
6. Server-held state (theme, counter, active tab) travels via cookie or `?query=`; mutations use
   `<form method="post">` + redirect (Post/Redirect/Get), never GET side effects.
7. Update the README feature matrix and Findings when a component or its fallback changes.

## Roadmap

`ROADMAP.md` is the work queue: work top to bottom (M1 capability beacons → M2 streaming → M3
state model → M4 Blitz tests → M5 machine-readable spec → M6 release polish). A milestone is
done only when every box is ticked, clippy is clean, tests pass, and README is updated.
