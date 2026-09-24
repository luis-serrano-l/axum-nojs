# Roadmap

Rules: work top to bottom. A milestone is done only when every box is ticked, `cargo clippy
--all-targets` is clean, `cargo test` passes, the no-script test still passes, README's feature
matrix and findings are updated, and the work is committed. Unknowns become entries in
`FINDINGS.md`, never reasons to add script.

## M1 · Capability beacons (server-side feature detection, no script)
- [x] `axum_nojs::caps`: `Caps` bitset (invokers, anchor, details_content, view_transitions, popover, light_dark, streaming_dsd)
- [x] `caps::beacon_css()` emits `@supports` rules that request `/nojs/caps?flag=1` as a background image
- [x] Axum route `/nojs/caps` sets/extends a `axum-nojs-caps` cookie; `Caps` implements `FromRequestParts`
- [x] Every component takes `&Caps` and emits only the best markup for that browser (dialog: invokers vs `:target`; popover: anchor vs centred; tabs: `::details-content` vs accordion)
- [x] Demo page `/caps` shows what the server thinks the browser supports
- [x] Screenshot verification in Firefox headless; old-Chrome-109 check confirms fallbacks render

## M2 · Out-of-order streaming without script
- [x] `axum_nojs::stream`: `Streamed` response type built on `axum::body::Body::from_stream`
- [x] `slot(id, placeholder)` renders `<nojs-slot><template shadowrootmode=open><slot name=id>…`
- [x] `fill(id, future)` appends the resolved chunk with `slot=id` later in the stream, any order
- [x] Demo `/stream` with three slow sections (100ms, 800ms, 2s) arriving out of order
- [x] Fallback when DSD unsupported (per `Caps`): render sequentially at the end
- [x] Test: response body is chunked and slots arrive in completion order

## M3 · State model for scriptless apps
- [x] `axum_nojs::state`: `UiState` extractor merging query + cookie (open tab, open details, dialog)
- [x] `prg(redirect_to, flash)` helper: Post/Redirect/Get with a one-shot flash cookie
- [x] `flash()` component rendering and clearing the flash
- [x] `details` and `tabs` persist open state via `?open=` links generated from `UiState`
- [x] Demo: settings page with tabs + form + flash that survives a full navigation
- [x] Docs: one page "how state works with no script"

## M4 · Blitz as the test engine
- [x] `axum-nojs-test` crate: render a route via `tower::oneshot`, load HTML into `blitz-dom`, resolve layout
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
- [x] Examples in `axum-nojs/examples/`
- [x] Publish dry run: `cargo publish --dry-run -p axum-nojs`
- [x] Demo visual pass: one palette (`--nojs-*` for light and dark, moss accent), one type scale, index grouped by platform feature, toolbar with a back link and the theme switch on every component page

## M7 · Optional enhancement script
- [x] `axum_nojs::enhance`: one small script (`/nojs/enhance.js`, content-hashed, immutable) that upgrades swap roots (`id` + `data-nojs="swap"`) to fetch + replace, queued per root, with focus, flash, title, theme and URL synced
- [x] Counter, form, tabs, accordion, pager, theme toggle are swap roots; combobox searches as you type; range and colour mirror live; `:target` dialog fallback opens as a real modal; `<details>` popover fallback light-dismisses
- [x] Enforcement: exactly one `<script>` per page and it is the enhancement tag; no inline handlers; Blitz suite (no script engine) proves every route works without it
- [x] Headless Firefox check through geckodriver (`scripts/browser-check.mjs`, run by `scripts/verify.sh` when available)
- [x] Docs: README, CLAUDE.md, FINDINGS "with the script" section, spec entry

## M8 · Framework independence
- [x] `Caps::from_cookie_header(&str)` and `Caps::from_query(&str)` as the only entry points; the Axum extractor becomes a thin wrapper behind the `axum` feature
- [x] Every component returns `Markup` that also implements `Render`; a `string` feature (or `.into_string()` docs) shows use without Maud templates
- [x] `state::prg`, `flash` and `stream` compile without Axum: an `http` feature exposes `http::Response` builders, the `axum` feature wraps them
- [x] Example `axum-nojs/examples/actix_server.rs` (or `hyper_server.rs`) rendering three components with the beacon route wired by hand
- [x] README: "Use with any server" section; CLAUDE.md workspace notes updated

## M9 · `axum-nojs-caps` as its own crate
- [x] Move `caps.rs` (bitset, `@supports` beacons, cookie parsing, beacon route) into `axum-nojs-caps/` in the workspace; `axum-nojs` depends on it and re-exports `Caps`, `Cap`
- [x] Spec page `docs/caps.md`: how the beacons work, the first-view problem, what each flag tests, cookie format, how to add a flag
- [x] Standalone example: a raw `hyper` handler that reads `Caps` and prints one line per flag
- [x] `cargo publish --dry-run -p axum-nojs-caps` passes; README of the sub-crate written for a reader who has never seen axum-nojs

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
- [x] `docs/theming.md`: every `--nojs-*` token, what it affects, contrast requirements, one worked example with a different palette
- [x] Demo `/theme-demo` (or a query flag on the index) rendering the same page under a second palette; Blitz screenshot pair `index-modern.png` vs `index-alt.png`
- [x] Test: no colour literal outside `layout.rs` (grep for `#[0-9a-f]{3,6}` in component CSS)

## M13 · Publish
- [x] `license`, `repository`, `readme`, `keywords`, `categories` in every publishable `Cargo.toml` (MIT; https://github.com/luis-serrano-l/axum-nojs)
- [x] `CHANGELOG.md` with 0.1.0; version bump; `cargo publish --dry-run` for `axum-nojs-caps` then `axum-nojs`
      (both crates are 0.1.0; `axum-nojs-caps` dry-runs clean; `axum-nojs` alone cannot until `axum-nojs-caps`
      is on crates.io, so `cargo package --workspace --exclude demo --exclude axum-nojs-test`
      verifies both together)
- [x] docs.rs metadata (`all-features`), crate-level README rendered on docs.rs checked with `cargo doc --no-deps`
- [ ] The publish itself is an outward action: ask the owner, do not run `cargo publish` without a yes

## M14 · What htmx has that the enhancement script does not
Every item must keep the no-script path intact: the markup is the same, the script only reads
attributes. Blitz proves each route works without it; `scripts/browser-check.mjs` proves the
script does its job.
- [x] Partial swaps with explicit targets: `data-nojs-target="#id"` on a form or link swaps that
  root instead of the closest one; `data-nojs-swap="inner|outer|append|prepend"` chooses how.
  Without the script the same request is a full navigation to the same page.
- [x] Out-of-band updates: a response may carry extra swap roots marked `data-nojs-oob`; the
  script replaces each matching `id` anywhere in the page (flash banner, counter in the header)
  and drops them from the main swap. Without the script the full page already shows them.
- [x] Request lifecycle feedback: `data-nojs-busy` class on the root while a request is in flight,
  `aria-busy="true"`, submit buttons disabled, a `--nojs-busy` CSS hook; optional
  `data-nojs-indicator="#id"` element shown while pending. Failed requests fall back to a normal
  navigation so the user always sees the server's answer.
- [x] History and URL control: `data-nojs-push="false"` keeps the URL, `data-nojs-replace` uses
  `replaceState`, and Back/Forward restore the swapped roots from a cached copy instead of a
  reload; a `nojs:swap` custom event fires after every swap for anything that must react.
- [x] Spec entry for the enhancement script updated, README "How the script works" section,
  Firefox checks for each attribute, FINDINGS on what the platform still cannot do.

## M15 · Components worth using
The components are too basic: each proves a platform feature but stops short of what an app
needs from it. Make each one something a real page would reach for, without giving up what
makes them simple: one function, one options struct, plain HTML you can `curl`, no script
beyond `/nojs/enhance.js`, every state a URL or a form. One box per component; each box ends
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
  hash on `/nojs/caps` beacon images and `/nojs/enhance.js` verified; gzip/br on the demo through
  `tower-http` `CompressionLayer`; release profile with `lto = "fat"`, `codegen-units = 1`,
  `panic = "abort"` for the demo binary.
- [x] Cheap wins, page: the inline stylesheet minified (whitespace and comments stripped at
  build time, a test proves it still parses); beacons `loading="lazy"` and `fetchpriority="low"`
  so they never delay first paint; `<script defer>` stays last; `<link rel="preconnect">` not
  needed (no third party) and recorded as such; the caps cookie small enough to fit one
  `Set-Cookie`.
- [x] Cheap wins, script: `enhance.js` requests carry `Accept: text/html` and the server's
  fragment answer (`Nojs-Enhance: 1`) used on every swap route in the demo, not only `/swap`,
  so a swap moves a few hundred bytes instead of the page; `fetch` with `priority: "high"`
  for user actions; prefetch on `mouseenter`/`focus` for same-origin links inside a swap root
  (`data-nojs-prefetch`), cached for a few seconds and reused by the click.
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
- [x] One obvious way in: each component has a short constructor for the common case
  (`dialog(&caps, "confirm", "Delete account", body)`) and options only for the rest;
  required text first, ids derived from it where the caller does not care.
- [x] Readable data: items, columns, fields and options built with `From` impls from plain
  tuples and `&str` where that reads better (`["Name", "Size"].into()`), without losing the
  builder form for the rare setting.
- [x] Less plumbing in handlers: extractors that hand a route the flash, the theme and the
  `UiState` together, so a route reads as "parse input, render components" in a few lines.
- [x] Fewer calls in the demo, more in the component: where a demo route stitches several
  helper calls around a component (building items, reading state, wrapping markup, formatting
  values), move that work into the component as an option so the route makes one call; the
  refactor should show how much a single component call can do, with the demo shrinking as
  proof and each moved piece covered by a test in the component's file.
- [x] Names read like HTML: option and setter names match the attribute or element they set
  (`.required()`, `.placeholder()`, `.open()`), one word where one word says it; a pass over
  every doc example so each reads top to bottom without jumping to another file.
- [x] Docs: `docs/ergonomics.md` shows before/after for each changed call site; README's
  first example is the most pleasant one the library can offer.

## M18 · Every component starts from `ui`
The owner looked at the demo after M17 and found the call sites still heavy (a 70-name import
line, `x_with(&ui, …, XOptions::default()…)`, `.state(&ui.state)` repeating `ui`, cookies parsed
by hand). Chosen shape (asked and answered): methods on `Ui` returning builders that render in
`html!` (`ui.dialog("Delete account").title(..).danger().confirm(..).body(html!{..})`), one
`use axum_nojs::prelude::*`, no `_with` twins, no `XOptions`; handlers lose their cookie
plumbing through `Saved<T>` and `ui.redirect(to).ok(..).save(&value)`.
- [x] Core: `Ui` gains `page(title, body) -> Page` (an `IntoResponse` that writes back changed
  state and clears a shown flash, so no more `(ui, markup)`), `redirect(to) -> Redirect`
  (`.flash/.ok/.warn/.danger/.cookie`, `into_http`), `param`/`params` (decoded query, so
  components read their own input), `From<Caps>`, `Default`. `prg`/`prg_parts` removed.
  `dialog` is no longer remembered in the `nojs-ui` cookie.
- [x] `saved.rs` (`axum` feature, serde + serde_urlencoded): `Saved<T>` extractor, cookie
  `nojs-<type-name>`, `Redirect::save`/`forget`.
- [x] Every component converted to a builder with `impl Ui { fn x(..) }` in its own file:
  flash, toasts, breadcrumbs (`.link().here()`), theme_toggle, stat, skeleton, empty_state,
  counter (`.apply(op, typed)` for the handler), range/range_pair, color, select
  (`.options/.group/.groups`, `.search(action)` reads `<name>-q`), combobox (reads `?q`/`?sel`,
  results default to matching suggestions), pager (reads `?page`, `.rows(|i| ..)`), dialog
  (id = slug of trigger, `.small()/.large()`, `.cancel()`), drawer (`.nav().body()`), menu
  (`ui.menu`, item modifiers apply to the last item, `.submenu(text, items)`), tabs
  (`.tab/.lazy/.badge`), accordion (`.item/.icon/.summary`), form (`ui.form(action)`/`ui.fields()`,
  `.text/.email/.number/.pattern/.textarea/.file/.date/.time/.select/.checkbox/.hidden`,
  last-field modifiers, `.group(legend)`), wizard (`.step(title, fields|markup)`,
  `.optional()`, `.review(title)` generated from the fields, `.errors()`, `Posted::from_pairs`,
  `.link(n)`, `.current()`), table (`ui.table(id, href).column().sortable().numeric().width()`,
  reads sort/q/page/cols, `.sort()/.filter()/.visible()/.page()/.per_page()` for the route,
  `.rows()`, `.paged(total)`), palette (`.group/.command/.commands/.keywords`, `.exact()`),
  stream (`ui.stream`, `ui.slot`). lib.rs: `prelude`, slim re-exports, tests rewritten.
  Examples (render_page, axum_server, hyper_server) and the bench rewritten.
- [x] `cargo test -p axum-nojs --all-features` green (38 unit, 54 doc), and the crate builds
  with no features and with `--features http`.
- [x] Demo rewrite (`demo/src/lib.rs`): one prelude import, every route in the new form, each
  handler `Page`/`Redirect`, `Saved<Settings>`/`Saved<Count>`/`Saved<Signup>`/`Saved<Inputs>`
  instead of hand-parsed cookies (cookie names change to `nojs-*`: check `axum-nojs-test`
  and `scripts/browser-check.mjs` for the old `count`, `settings`, `inputs`, `wizard`, `notes`
  cookies), dialog page keeps `.id("confirm")` so `/dialog?dialog=confirm` in `PATHS` still
  works, table + CSV share one `fn files_table(ui)`, palette redirects via `.exact()`,
  wizard uses `.review()` and `Posted`. Goal: the demo visibly shorter; count lines before/after.
- [x] Demo tests (`demo/src/lib.rs` tests), `axum-nojs-test` (Blitz) and
  `scripts/browser-check.mjs` pass; look at `tests/shots/` (form select field and wizard
  review are new markup).
- [x] Docs: README first example and "How to read this crate" (signatures section is stale:
  describes `name`/`name_with`/`XOptions`), the "Use with any server" table (`prg` rows),
  `docs/ergonomics.md` before/after for the M18 shape, `docs/state.md`, CLAUDE.md component
  convention #5 (now: builder struct + `impl Ui` method in the component file, modifiers on
  the last item, `impl Render`), FINDINGS if Blitz changes. Grep for `_with(`, `Options`,
  `prg`, `popover_menu`, `command_palette`, `Streamed::page` across docs.
- [x] `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`,
  `scripts/verify.sh`, then one local commit "M18: every component starts from ui". Do not push.

## M19 · A demo that teaches
Asked by the owner during M18: each component page shows how it is written, and the demo
gets a second visual pass.
- [x] Code snippet on every component page, in a box under the live component: the lines
  between `// code: <href>` and `// end code` markers in `demo/src/lib.rs`, cut from the file
  itself (`include_str!`) so the page and the code cannot drift (markers instead of the planned
  const, which would have been a second copy). Highlighted on the server by `highlight()`
  (keywords, strings, numbers, types, comments, macros, methods), coloured only with
  `--nojs-*` tokens (`.nojs-snippet`, `.nojs-hl-*` in `layout.rs`); no script. A test checks
  every component page has a snippet and that the box shows exactly that code.
- [x] Visual pass with the frontend-design skill. Plan: keep ink and moss (tokens only), spend
  the boldness on one element, the plate: the live component on a stage (`--nojs-surface`)
  with the code box joined under it, one per page. Index rows became a two-column grid
  (name, then a plain-words line of what the component is for, then the chips); the same
  line is the lede under each component title; chips no longer break mid-word. Reviewed
  against the defaults: no card grid, shadows, gradients or eyebrows. Checked in Firefox
  (light and dark, 1280 and 420 wide) and the Blitz PNGs. The colour-literal test no longer
  mistakes `white-space` for a colour.
- [x] Replace the demo's hand-written `highlight()` with `syntect` (asked by the owner): a
  dependency of `demo` only (`default-syntaxes`, `regex-fancy`; no bundled themes, no
  onig C build), never of `axum-nojs`. Its parser and Rust grammar decide the scopes; the
  demo maps scope prefixes to the same seven `nojs-hl-*` classes coloured by `--nojs-*`
  tokens, rather than `ClassedHTMLGenerator`, whose class-per-scope spans were about ten times
  the markup. Every snippet is highlighted once (`LazyLock`, warmed on a thread when the router
  is built; about 1 s in a debug build). Maud's `@if` shows `if` as a keyword and the `@` plain.
- [ ] README "Run the demo" updated, clippy/tests/`scripts/verify.sh` green, local commit.
