# Roadmap

Rules: work top to bottom. A milestone is done only when every box is ticked, `cargo clippy
--all-targets` is clean, `cargo test` passes, the no-script test still passes, README's feature
matrix and findings are updated, and the work is committed. Unknowns become entries in
`FINDINGS.md`, never reasons to add script.

## M1 · Capability beacons (server-side feature detection, no script)
- [ ] `webonsive::caps`: `Caps` bitset (invokers, anchor, details_content, view_transitions, popover, light_dark, streaming_dsd)
- [ ] `caps::beacon_css()` emits `@supports` rules that request `/wo/caps?flag=1` as a background image
- [ ] Axum route `/wo/caps` sets/extends a `wo-caps` cookie; `Caps` implements `FromRequestParts`
- [ ] Every component takes `&Caps` and emits only the best markup for that browser (dialog: invokers vs `:target`; popover: anchor vs centred; tabs: `::details-content` vs accordion)
- [ ] Demo page `/caps` shows what the server thinks the browser supports
- [ ] Screenshot verification in Firefox headless; old-Chrome-109 check confirms fallbacks render

## M2 · Out-of-order streaming without script
- [ ] `webonsive::stream`: `Streamed` response type built on `axum::body::Body::from_stream`
- [ ] `slot(id, placeholder)` renders `<wo-slot><template shadowrootmode=open><slot name=id>…`
- [ ] `fill(id, future)` appends the resolved chunk with `slot=id` later in the stream, any order
- [ ] Demo `/stream` with three slow sections (100ms, 800ms, 2s) arriving out of order
- [ ] Fallback when DSD unsupported (per `Caps`): render sequentially at the end
- [ ] Test: response body is chunked and slots arrive in completion order

## M3 · State model for scriptless apps
- [ ] `webonsive::state`: `UiState` extractor merging query + cookie (open tab, open details, dialog)
- [ ] `prg(redirect_to, flash)` helper: Post/Redirect/Get with a one-shot flash cookie
- [ ] `flash()` component rendering and clearing the flash
- [ ] `details` and `tabs` persist open state via `?open=` links generated from `UiState`
- [ ] Demo: settings page with tabs + form + flash that survives a full navigation
- [ ] Docs: one page "how state works with no script"

## M4 · Blitz as the test engine
- [ ] `webonsive-test` crate: render a route via `tower::oneshot`, load HTML into `blitz-dom`, resolve layout
- [ ] Assertions: element exists, is visible, bounding box, computed style
- [ ] Screenshot every demo route through Blitz's painter; store PNGs under `tests/shots`
- [ ] CI-style script `just verify` (or `scripts/verify.sh`): build, clippy, tests, screenshots, no-script grep
- [ ] File Blitz issues for anything it cannot render; link them from FINDINGS.md

## M5 · Machine-readable component spec
- [ ] `spec/` JSON: per component, features used, baseline per browser, fallback, needs_js verdict
- [ ] `cargo run -p demo -- spec` prints the JSON; README feature matrix is generated from it
- [ ] `FINDINGS.md` consolidated: what works, what needs fallback, what is impossible without JS
- [ ] Doc comment headers in every component checked against the spec by a test

## M6 · Polish for release
- [ ] `<select>` with `<selectedcontent>` component, `<input type=range>` and colour with server round trip
- [ ] Crate docs on docs.rs style: every pub item documented, `#![warn(missing_docs)]`
- [ ] Examples in `webonsive/examples/`
- [ ] Publish dry run: `cargo publish --dry-run -p webonsive`
