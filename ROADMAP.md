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

## M8 · Framework independence
- [x] `Caps::from_cookie_header(&str)` and `Caps::from_query(&str)` as the only entry points; the Axum extractor becomes a thin wrapper behind the `axum` feature
- [x] Every component returns `Markup` that also implements `Render`; a `string` feature (or `.into_string()` docs) shows use without Maud templates
- [x] `state::prg`, `flash` and `stream` compile without Axum: an `http` feature exposes `http::Response` builders, the `axum` feature wraps them
- [x] Example `webonsive/examples/actix_server.rs` (or `hyper_server.rs`) rendering three components with the beacon route wired by hand
- [x] README: "Use with any server" section; CLAUDE.md workspace notes updated

## M9 · `wo-caps` as its own crate
- [x] Move `caps.rs` (bitset, `@supports` beacons, cookie parsing, beacon route) into `wo-caps/` in the workspace; `webonsive` depends on it and re-exports `Caps`, `Cap`
- [x] Spec page `docs/caps.md`: how the beacons work, the first-view problem, what each flag tests, cookie format, how to add a flag
- [x] Standalone example: a raw `hyper` handler that reads `Caps` and prints one line per flag
- [ ] `cargo publish --dry-run -p wo-caps` passes; README of the sub-crate written for a reader who has never seen webonsive

## M10 · Components admin panels need
- [ ] `table`: server-side sort (`?sort=col&dir=asc` links in `<th>`), column filter (`<form method=get>` with `<search>`), sticky header, `aria-sort`; swap root
- [ ] `paged_table`: the table plus page links (`?page=n`) and a per-page `<select>`; total and range shown; keyboard-reachable
- [ ] `wizard`: multi-step form with PRG state in `UiState`, step list with the current step marked, back link that keeps entered values, review step
- [ ] Demo routes `/table`, `/wizard` added to `PATHS` and `COMPONENTS`; Blitz assertions for sort links and step markers; Firefox check for in-place sort
- [ ] Spec entries, README matrix regenerated, FINDINGS updated

## M11 · Options structs instead of positional arguments
- [ ] Each component with more than three arguments after `id` takes an `Options` struct with `Default` (`DialogOptions { open, close_label, .. }`)
- [ ] Builder-style setters (`.open(true)`) on every options struct; no macros
- [ ] Old signatures removed in the same change, all call sites (demo, examples, doctests, tests) updated
- [ ] Doc headers show the short form `dialog(&caps, "id", "title", body, Default::default())` and one full form

## M12 · Theming guide
- [ ] `layout::Tokens` struct (`accent`, `bg`, `fg`, …, light and dark) with `Default` = ink and moss; `layout_with(&caps, title, theme, &tokens, body)` emits the overrides once per page
- [ ] `docs/theming.md`: every `--wo-*` token, what it affects, contrast requirements, one worked example with a different palette
- [ ] Demo `/theme-demo` (or a query flag on the index) rendering the same page under a second palette; Blitz screenshot pair `index-modern.png` vs `index-alt.png`
- [ ] Test: no colour literal outside `layout.rs` (grep for `#[0-9a-f]{3,6}` in component CSS)

## M13 · Publish
- [ ] `license`, `repository`, `readme`, `keywords`, `categories` in every publishable `Cargo.toml` (needs the owner's answer in BLOCKED.md)
- [ ] `CHANGELOG.md` with 0.1.0; version bump; `cargo publish --dry-run` for `wo-caps` then `webonsive`
- [ ] docs.rs metadata (`all-features`), crate-level README rendered on docs.rs checked with `cargo doc --no-deps`
- [ ] The publish itself is an outward action: ask the owner, do not run `cargo publish` without a yes
