# Roadmap

Rules: work top to bottom. A milestone is done only when every box is ticked, `cargo clippy
--all-targets` is clean, `cargo test` passes, the no-script test still passes, README's feature
matrix and findings are updated, and the work is committed. Unknowns become entries in
`FINDINGS.md`, never reasons to add script.

## M1 · Capability beacons (server-side feature detection, no script)
- [x] `webonsive::caps`: `Caps` bitset (invokers, anchor, details_content, view_transitions, popover, light_dark, streaming_dsd)
- [x] `caps::beacon_css()` emits `@supports` rules that request `/wo/caps?flag=1` as a background image
- [x] Axum route `/wo/caps` sets/extends a `wo-caps` cookie; `Caps` implements `FromRequestParts`
- [x] Every component takes `&Caps` and emits only the best markup for that browser (dialog: invokers vs `:target`; popover: anchor vs centred; tabs: `::details-content` vs accordion)
- [x] Demo page `/caps` shows what the server thinks the browser supports
- [x] Screenshot verification in Firefox headless; old-Chrome-109 check confirms fallbacks render

## M2 · Out-of-order streaming without script
- [x] `webonsive::stream`: `Streamed` response type built on `axum::body::Body::from_stream`
- [x] `slot(id, placeholder)` renders `<wo-slot><template shadowrootmode=open><slot name=id>…`
- [x] `fill(id, future)` appends the resolved chunk with `slot=id` later in the stream, any order
- [x] Demo `/stream` with three slow sections (100ms, 800ms, 2s) arriving out of order
- [x] Fallback when DSD unsupported (per `Caps`): render sequentially at the end
- [x] Test: response body is chunked and slots arrive in completion order

## M3 · State model for scriptless apps
- [x] `webonsive::state`: `UiState` extractor merging query + cookie (open tab, open details, dialog)
- [x] `prg(redirect_to, flash)` helper: Post/Redirect/Get with a one-shot flash cookie
- [x] `flash()` component rendering and clearing the flash
- [x] `details` and `tabs` persist open state via `?open=` links generated from `UiState`
- [x] Demo: settings page with tabs + form + flash that survives a full navigation
- [x] Docs: one page "how state works with no script"

## M4 · Blitz as the test engine
- [x] `webonsive-test` crate: render a route via `tower::oneshot`, load HTML into `blitz-dom`, resolve layout
- [x] Assertions: element exists, is visible, bounding box, computed style
- [x] Screenshot every demo route through Blitz's painter; store PNGs under `tests/shots`
- [x] CI-style script `just verify` (or `scripts/verify.sh`): build, clippy, tests, screenshots, no-script grep
- [x] File Blitz issues for anything it cannot render; link them from FINDINGS.md

## M5 · Machine-readable component spec
- [x] `spec/` JSON: per component, features used, baseline per browser, fallback, needs_js verdict
- [x] `cargo run -p demo -- spec` prints the JSON; README feature matrix is generated from it
- [x] `FINDINGS.md` consolidated: what works, what needs fallback, what is impossible without JS
- [x] Doc comment headers in every component checked against the spec by a test

## M6 · Polish for release
- [x] `<select>` with `<selectedcontent>` component, `<input type=range>` and colour with server round trip
- [x] Crate docs on docs.rs style: every pub item documented, `#![warn(missing_docs)]`
- [x] Examples in `webonsive/examples/`
- [x] Publish dry run: `cargo publish --dry-run -p webonsive`
- [x] Demo visual pass: one palette (`--wo-*` for light and dark, moss accent), one type scale, index grouped by platform feature, toolbar with a back link and the theme switch on every component page

## M7 · Optional enhancement script
- [x] `webonsive::enhance`: one small script (`/wo/enhance.js`, content-hashed, immutable) that upgrades swap roots (`id` + `data-wo="swap"`) to fetch + replace, queued per root, with focus, flash, title, theme and URL synced
- [x] Counter, form, tabs, accordion, pager, theme toggle are swap roots; combobox searches as you type; range and colour mirror live; `:target` dialog fallback opens as a real modal; `<details>` popover fallback light-dismisses
- [x] Enforcement: exactly one `<script>` per page and it is the enhancement tag; no inline handlers; Blitz suite (no script engine) proves every route works without it
- [x] Headless Firefox check through geckodriver (`scripts/browser-check.mjs`, run by `scripts/verify.sh` when available)
- [x] Docs: README, CLAUDE.md, FINDINGS "with the script" section, spec entry
