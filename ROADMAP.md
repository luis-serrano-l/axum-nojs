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
- [x] `cargo publish --dry-run -p wo-caps` passes; README of the sub-crate written for a reader who has never seen webonsive

## M10 · Components admin panels need
- [x] `table`: server-side sort (`?sort=col&dir=asc` links in `<th>`), column filter (`<form method=get>` with `<search>`), sticky header, `aria-sort`; swap root
- [x] `paged_table`: the table plus page links (`?page=n`) and a per-page `<select>`; total and range shown; keyboard-reachable
- [x] `wizard`: multi-step form with PRG state in `UiState`, step list with the current step marked, back link that keeps entered values, review step
- [x] Demo routes `/table`, `/wizard` added to `PATHS` and `COMPONENTS`; Blitz assertions for sort links and step markers; Firefox check for in-place sort
- [x] Spec entries, README matrix regenerated, FINDINGS updated

## M11 · Options structs instead of positional arguments
- [x] Each component with more than three arguments after `id` takes an `Options` struct with `Default` (`DialogOptions { open, close_label, .. }`)
- [x] Builder-style setters (`.open(true)`) on every options struct; no macros
- [x] Old signatures removed in the same change, all call sites (demo, examples, doctests, tests) updated
- [x] Doc headers show the short form `dialog(&caps, "id", "title", body, Default::default())` and one full form

## M12 · Theming guide
- [x] `layout::Tokens` struct (`accent`, `bg`, `fg`, …, light and dark) with `Default` = ink and moss; `layout_with(&caps, title, theme, &tokens, body)` emits the overrides once per page
- [x] `docs/theming.md`: every `--wo-*` token, what it affects, contrast requirements, one worked example with a different palette
- [x] Demo `/theme-demo` (or a query flag on the index) rendering the same page under a second palette; Blitz screenshot pair `index-modern.png` vs `index-alt.png`
- [x] Test: no colour literal outside `layout.rs` (grep for `#[0-9a-f]{3,6}` in component CSS)

## M13 · Publish
- [ ] `license`, `repository`, `readme`, `keywords`, `categories` in every publishable `Cargo.toml` (needs the owner's answer in BLOCKED.md)
- [x] `CHANGELOG.md` with 0.1.0; version bump; `cargo publish --dry-run` for `wo-caps` then `webonsive`
      (both crates are 0.1.0; `wo-caps` dry-runs clean; `webonsive` alone cannot until `wo-caps`
      is on crates.io, so `cargo package --workspace --exclude demo --exclude webonsive-test`
      verifies both together)
- [x] docs.rs metadata (`all-features`), crate-level README rendered on docs.rs checked with `cargo doc --no-deps`
- [ ] The publish itself is an outward action: ask the owner, do not run `cargo publish` without a yes

## M14 · What htmx has that the enhancement script does not
Every item must keep the no-script path intact: the markup is the same, the script only reads
attributes. Blitz proves each route works without it; `scripts/browser-check.mjs` proves the
script does its job.
- [x] Partial swaps with explicit targets: `data-wo-target="#id"` on a form or link swaps that
  root instead of the closest one; `data-wo-swap="inner|outer|append|prepend"` chooses how.
  Without the script the same request is a full navigation to the same page.
- [x] Out-of-band updates: a response may carry extra swap roots marked `data-wo-oob`; the
  script replaces each matching `id` anywhere in the page (flash banner, counter in the header)
  and drops them from the main swap. Without the script the full page already shows them.
- [x] Request lifecycle feedback: `data-wo-busy` class on the root while a request is in flight,
  `aria-busy="true"`, submit buttons disabled, a `--wo-busy` CSS hook; optional
  `data-wo-indicator="#id"` element shown while pending. Failed requests fall back to a normal
  navigation so the user always sees the server's answer.
- [x] History and URL control: `data-wo-push="false"` keeps the URL, `data-wo-replace` uses
  `replaceState`, and Back/Forward restore the swapped roots from a cached copy instead of a
  reload; a `wo:swap` custom event fires after every swap for anything that must react.
- [x] Spec entry for the enhancement script updated, README "How the script works" section,
  Firefox checks for each attribute, FINDINGS on what the platform still cannot do.

## M15 · Components worth using
The components are too basic: each proves a platform feature but stops short of what an app
needs from it. Make each one something a real page would reach for, without giving up what
makes them simple: one function, one options struct, plain HTML you can `curl`, no script
beyond `/wo/enhance.js`, every state a URL or a form. One box per component; each box ends
with a demo route that shows the new behaviour, a Blitz assertion and (where the script is
involved) a Firefox check.
- [x] Dialog: sizes (`sm|md|lg`), a header with title and close, a footer slot for real actions
  (confirm form posting to a URL, cancel), `danger` variant, focus lands on the first field,
  `Escape` and backdrop close honour `closedby`; an optional `returns_to` so the server can
  redirect back to the page that opened it.
- [x] Popover menu: items with icons and keyboard shortcuts shown, separators and section
  headings, disabled and destructive items, a submenu that is another popover, items that are
  `<form method="post">` buttons for actions (not just links), placement options
  (`bottom-start|bottom-end|right`) via `anchor-name`, arrow keys move between items.
- [x] Tabs: lazy panels (a tab that is a link to `?tab.x=n` fetches only when opened), a badge
  count per tab, vertical orientation option, overflow to a `<select>` on narrow screens, the
  active tab underlined with a morphing `view-transition-name`.
- [x] Accordion: a "expand all / collapse all" pair of links, an item can carry a summary line
  and an icon, nested accordions, `open.<group>` accepts a list so several items can be open.
- [x] Combobox: multi-select with removable chips, grouped options (`<optgroup>` in the
  datalist), a "create new" row when nothing matches, keyboard navigation of server results,
  the current selection kept across a re-filter, async results marked with `aria-live`.
- [x] Table: row selection with checkboxes and a bulk-action form, column visibility toggles
  (`?cols=`), a per-row action menu (the popover), expandable detail rows (`<details>` in a
  cell), numeric alignment and column widths, empty and loading states, CSV link for the
  current filter.
- [x] Paged table: jump-to-page form, first/last links, ellipsis for long ranges, page size
  remembered per table in `UiState`, total row count formatted with separators.
- [x] Wizard: per-step server validation with messages beside the field and the step marked
  in error in the step list, optional steps that can be skipped, a progress bar, a summary
  that links each field back to its step, resume from the cookie after closing the tab.
- [x] Form: field groups with legends, help text and character counters (`<output>`),
  file inputs with accepted types, date/time/number inputs with min/max, textarea autosize via
  `field-sizing: content`, inline and stacked layouts, a "dirty" warning link-back is not
  possible without script and is recorded in FINDINGS.
- [x] Counter, range, color, select: stepper with min/max/step and a typed value, range with
  two thumbs (min/max pair as two inputs), colour with a preset swatch row and alpha, select
  with option groups, icons in options and a search box when it has more than ~15 options.
- [x] Flash and status: variants (`info|ok|warn|danger`), dismiss is a link that clears the
  cookie, multiple flashes stack, a `role="alert"` variant for errors, auto-hide via CSS
  animation with reduced-motion respected.
- [x] New: toast list, breadcrumbs, skeleton placeholders for streamed slots, empty states, a
  stat card, a sidebar/drawer navigation (`<dialog>` non-modal or popover), a command palette
  (search + datalist + popover) as the flagship "no script needed" demo.
- [x] Every options struct grows only setters that map to real HTML/CSS; the doc header of each
  component gains a "What it does not do without script" paragraph; README matrix, spec and
  FINDINGS updated as each box lands.

## M16 · Cut latency
Every page is one server round trip, so latency is the whole experience. Measure first, then
take the cheap wins, then the ones that cost structure. Each box records before/after numbers
in FINDINGS.md (`hyperfine` against the demo, Firefox navigation timing from
`scripts/browser-check.mjs`) so a change that does not move a number is reverted.
- [x] Measure: a `scripts/bench.sh` that starts the release demo and reports p50/p95 time to
  first byte and full response for `/`, `/table`, `/stream` cold and warm; Firefox
  `performance.getEntriesByType("navigation")` for the same routes; numbers in FINDINGS.md.
- [x] Cheap wins, server: `stylesheet()` built once (`OnceLock`) instead of per page; the
  `Tokens::css()` string cached; `Content-Length` on every response; `Cache-Control` with a
  hash on `/wo/caps` beacon images and `/wo/enhance.js` verified; gzip/br on the demo through
  `tower-http` `CompressionLayer`; release profile with `lto = "fat"`, `codegen-units = 1`,
  `panic = "abort"` for the demo binary.
- [x] Cheap wins, page: the inline stylesheet minified (whitespace and comments stripped at
  build time, a test proves it still parses); beacons `loading="lazy"` and `fetchpriority="low"`
  so they never delay first paint; `<script defer>` stays last; `<link rel="preconnect">` not
  needed (no third party) and recorded as such; the caps cookie small enough to fit one
  `Set-Cookie`.
- [x] Cheap wins, script: `enhance.js` requests carry `Accept: text/html` and the server's
  fragment answer (`Wo-Enhance: 1`) used on every swap route in the demo, not only `/swap`,
  so a swap moves a few hundred bytes instead of the page; `fetch` with `priority: "high"`
  for user actions; prefetch on `mouseenter`/`focus` for same-origin links inside a swap root
  (`data-wo-prefetch`), cached for a few seconds and reused by the click.
- [x] Speculation rules: a `<script type="speculationrules">` is a `<script>` tag and so out
  of bounds by CLAUDE.md; instead `<link rel="prefetch">` for the index's component links and
  `<link rel="prerender">`-free; record in FINDINGS what the platform cannot prefetch without
  the rules script.
- [x] Streaming everywhere it pays: `layout` sends `<head>` and the shell before the body is
  built (an `http`-feature `Streamed` page for every demo route whose body waits on anything),
  `Transfer-Encoding: chunked` with an early flush after `</head>` so the stylesheet parses
  while the server works; measured on `/stream` and `/table`.
- [x] Structural: `Caps` from a bitset cookie is already O(1); `UiState` parse checked for
  allocations; Maud templates render into a pre-sized `String` (`html!` with capacity hints
  where a component knows its size); `paged_table` builds rows without intermediate `String`s;
  a `cargo bench` (criterion) for `layout`, `table` with 1 000 rows and `stylesheet()`.
- [x] HTTP/2 and HTTP/3 in the hyper example so many beacon images share one connection; a
  note in `docs/caps.md` on why the beacons cost nothing after the first visit (cookie) and
  how to serve them from the same origin as the page.
- [x] Docs: `docs/latency.md` with the numbers, what moved them, what did not, and the order a
  user should apply them to their own server; README gets one line pointing at it.

## M17 · Pleasant to use
Calling a component should read like describing the page. Today a call site can carry eight
positional arguments, a chain of setters and a `jar.get(...).map(...).as_deref()` just to
reach a value; the goal is call sites a newcomer reads once and understands, and that stay
short enough to scan. Every box lands with the demo and the doc headers rewritten to the new
form, and the old form kept only where removing it would break a published signature.
- [x] Audit: list every public signature and every demo call site with its argument count,
  the setters it needs, and what a reader must know to follow it; record the worst ten in
  `docs/ergonomics.md` with a proposed rewrite for each.
- [ ] One obvious way in: each component has a short constructor for the common case
  (`dialog(&caps, "confirm", "Delete account", body)`) and options only for the rest;
  required text first, ids derived from it where the caller does not care.
- [ ] Readable data: items, columns, fields and options built with `From` impls from plain
  tuples and `&str` where that reads better (`["Name", "Size"].into()`), without losing the
  builder form for the rare setting.
- [ ] Less plumbing in handlers: extractors that hand a route the flash, the theme and the
  `UiState` together, so a route reads as "parse input, render components" in a few lines.
- [ ] Fewer calls in the demo, more in the component: where a demo route stitches several
  helper calls around a component (building items, reading state, wrapping markup, formatting
  values), move that work into the component as an option so the route makes one call; the
  refactor should show how much a single component call can do, with the demo shrinking as
  proof and each moved piece covered by a test in the component's file.
- [ ] Names read like HTML: option and setter names match the attribute or element they set
  (`.required()`, `.placeholder()`, `.open()`), one word where one word says it; a pass over
  every doc example so each reads top to bottom without jumping to another file.
- [ ] Docs: `docs/ergonomics.md` shows before/after for each changed call site; README's
  first example is the most pleasant one the library can offer.
