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
- [x] README "Run the demo" updated (index groups, the plate, the code markers, syntect) and
  CLAUDE.md notes the markers; clippy/tests/`scripts/verify.sh` green, local commit.

## M20 · The shadcn look
The owner found the M19 visual pass not good enough and asked for a modern look borrowed from
the best component libraries rather than an original one. Chosen (asked and answered): follow
shadcn/ui (MIT) closely, drop ink and moss for its neutral default, system fonts only (no web
font). References by job: shadcn/ui for tokens, spacing and states; Basecoat UI (shadcn as
plain HTML/CSS) for markup structure; Radix Colors for status scales; Pico CSS / Open Props for
native elements shadcn replaces with React (`<dialog>`, `<details>`, `<select>`, date, range);
Vercel Geist for dense tables and stats. Paid kits (Tailwind UI, Catalyst) are not copied.
Done before M21 so the primitives are born in this look.
- [x] Tokens: `Palette` gains the shadcn roles it lacks (`card`, `popover`, `secondary`,
  `accent` as hover surface, `primary`/`on_primary`, `input`, `ring`), keeping `--nojs-*`
  names; `Tokens::default()` is shadcn's neutral (zinc) light and dark, primary near-black /
  near-white. `radius` 0.5rem with derived `--nojs-radius-sm/-lg`. ok/warn/danger from Radix
  Colors steps 9/11. Docs and `docs/theming` updated; the "ink and moss" wording removed.
  Done: the old brand token became `--nojs-primary`/`--nojs-on-primary` everywhere and
  `--nojs-accent` is now the hover surface. Status colours use Radix step 11 (red/green/amber),
  the text step, so each clears 4.5:1; danger buttons put `on-primary` on it (5.0 / 8.4).
  `--nojs-radius-sm/-lg` are derived as radius ∓ 4px, so a radius needs a unit (`"0px"`).
- [x] Type: system stack only (`ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto,
  "Helvetica Neue", Arial, sans-serif`, mono `ui-monospace, SFMono-Regular, Menlo, Consolas`),
  shadcn's scale (text-sm 0.875rem body in controls, 1.25/1.5 line heights, 500/600 weights),
  `-webkit-font-smoothing: antialiased`, tabular numbers in tables and stats.
  Done: the stacks are `--nojs-font-sans`/`--nojs-font-mono` on `:root` (base rules, not
  `Tokens` fields); every component's sizes snapped to the scale (0.75/0.875/1/1.125/1.25/1.5/
  2.25rem) and 700 weights to 600.
- [x] Base styles in `layout.rs`: shadcn heights and paddings (controls h-9 = 2.25rem, px-3/px-4),
  1px `--nojs-input` borders, `shadow-xs` on controls, `shadow-lg` on dialog/popover,
  focus-visible as a 3px `--nojs-ring` at 50% opacity, `aria-invalid` red ring, disabled at
  50% opacity, hover as `--nojs-accent` surface. Native `<select>`, checkbox, radio, range,
  date and `<details>` restyled to match (`appearance`, `accent-color`).
  Done: `--nojs-shadow-xs/-lg` are emitted by `Tokens::css()` beside the radii (rgb literals
  live there, not in component CSS); `--nojs-radius-sm` became radius − 2px so controls land
  on shadcn's 6px `rounded-md`. The plain button is shadcn's outline variant. The select
  chevron is two gradients in `--nojs-muted` (no data-URI SVG, which would need a literal);
  `select.rs` drops it where `appearance: base-select` draws its own picker icon.
- [x] Every existing component's CSS re-tuned to these values (dialog, drawer as shadcn sheet,
  popover/menu as dropdown-menu, tabs as the muted pill list, accordion, toast as sonner-style
  cards, table, pager as pagination, badge-like chips, skeleton, palette as command).
  Done: `Tokens::css()` also emits `--nojs-overlay` (black/50, as shadcn), used by every
  backdrop; all floating layers take `--nojs-popover` and `--nojs-shadow-lg`. Tabs: each
  summary paints its slice of the muted pill and `.nojs-tabs-mark` became the raised chip
  (the Blitz test now checks the chip, inset 3px). Toasts and flashes are neutral cards with
  a level dot. Link-as-button rules repeat the button values until M22 builds them on the
  primitive.
- [x] The demo takes the same look (index, plate, code box); syntect classes recoloured.
  Done: index groups are grids of cards (the title link stretches over the card), "built on"
  items are outline badges, the stage is a preview box over a `--nojs-surface` code block, and
  the seven `nojs-hl-*` classes use danger/ok/warn/muted/fg (GitHub-like, still tokens only).
- [x] Side-by-side check: for each component, a Firefox screenshot of the demo (light and dark,
  1280 and 420 wide) next to the shadcn docs page for the same component; mismatches fixed
  or noted. `tests/shots/` refreshed; the colour-literal test still passes.
  Done with `scripts/look.sh` (17 pages × light/dark × 1280/420, plus the shadcn docs page,
  into `target/look/`). Fixed: the dialog footer now stacks full width under 40rem (confirm
  on top, as shadcn's `flex-col-reverse`) and the dialog wraps under its trigger instead of
  squeezing beside it; table number cells no longer wrap and `code` in cells is plain text.
  Noted, kept: shadcn centres its preview in a tall box, the demo stage is left-aligned
  because it carries explanatory text; toasts only appear after a POST, so Firefox cannot
  shoot them (the Blitz toast shot covers them); the wizard has no shadcn counterpart.
- [x] README (screenshot, theming section) and FINDINGS updated; clippy, tests,
  `scripts/verify.sh` green; local commit.
  Done: `docs/screenshot.png` (the table demo split light/dark, from Firefox) heads the README;
  the theming bullet names the derived radius/shadow/overlay tokens, the font properties and
  `scripts/look.sh`; FINDINGS has an M20 section.

## M21 · Primitives
The owner found the library "in the middle of nowhere": 30 components, but no button (buttons
are only global element CSS in `layout.rs`) and no calendar, and components barely reuse each
other. A "React for Rust" was discussed and rejected: client-side reactivity contradicts the
no-script rule, and Leptos/Dioxus/Yew own that space. Chosen direction (asked and answered): a
layered design system: primitives → existing components rebuilt on them → flagship widgets
→ a documented way to write your own. The first thing a user looks for is a button.
- [x] `button.rs`: `ui.button(text)` with `.primary()/.danger()/.ghost()/.small()/.icon()`,
  `.submit()/.reset()`, `.command(cmd, target)` (invoker), `.popovertarget()`, `.form(id)`,
  `.name().value()`, `.disabled()`, `.loading(bool)`; `ui.link_button(text, href)` with the
  same look. The global `button {}` CSS in `layout.rs` moves into `button::CSS` as `.nojs-button`.
  Done: modifiers are `nojs-button-{primary,danger,ghost,small,icon}`; bare `button` and
  `button.nojs-primary`/`.nojs-danger` keep the same rules (same specificity as before) so
  hand-written and not-yet-migrated buttons look unchanged until M22. The button submits by
  default and becomes `type="button"` with a command or popover target; without invoker
  commands a popover command falls back to `popovertarget`/`popovertargetaction`. Also
  `.label()` (aria-label, for icon buttons) and `.class()` (a component's part name, for M22).
  A loading button is `disabled` + `aria-busy` with a CSS spinner; a disabled link loses its
  `href`.
- [x] `input.rs`: `ui.input(name, label)`, one labelled field (label, hint, error,
  `aria-describedby`), same setters as form fields; `ui.checkbox`, `ui.radio_group`,
  `ui.switch` (checkbox with `role=switch`).
  Done: the field renderer, `Field` and `FieldKind` moved out of `form.rs` into `input.rs`,
  and `ui.form` now renders its fields through it (one renderer, which is M22's second box
  done early), with the `.nojs-field*` CSS. `ui.input` takes the type as a setter
  (`.email()`, `.password()` (never echoed back), `.number()`, `.pattern()`, `.textarea()`,
  `.file()`, `.date()`, `.time()`) plus `.id()`; `ui.checkbox`/`ui.switch` share the builder
  (`.checked(bool)`); `ui.radio_group(name, legend).option(value, label)` is a fieldset with
  one `required` radio. The switch is `appearance: none` with a `::before` thumb.
- [x] `badge.rs`, `card.rs` (`.header/.body/.footer`), `icon.rs` (small inline-SVG set, no
  font), `avatar.rs`.
  Done: `ui.badge(text)` (primary fill; `.secondary/.danger/.outline/.ok/.warn`, `.href`);
  `ui.card()` with `.title/.description/.header(markup)` (the header markup is the top-right
  action), `.body`, `.footer`, `.id`; `Icon` (28 Lucide shapes, ISC, in the prelude, renders
  decoratively on its own) and `ui.icon(Icon::X).label(..)` for a named one; `ui.avatar(name)`
  with `.src/.small/.large`: initials under the `<img alt="">`, so a failed load shows them
  with no `onerror` (the img overhangs the clipped circle by 2px to hide Firefox's broken-image
  frame). Found on the way: a crate using `html!` from the prelude still needs its own `maud`
  dependency, since the macro expands to `maud::` paths; M24's docs must say so.
- [x] Layout primitives: `ui.stack()`, `ui.cluster()`, `ui.grid(min)`, `ui.split()`, CSS-only,
  gaps from `--nojs-space-*` tokens.
  Done: one file each (`stack.rs`, `cluster.rs`, `grid.rs`, `split.rs`); each takes its
  content as `Markup` (`ui.grid(min, content)`, `ui.split(side, main)`). `Tokens::css()`
  derives `--nojs-space-{1,2,3,4,6,8}` (n × 4px by default, Tailwind's steps) and `.gap(n)`
  adds a `nojs-gap-n` class from `layout.rs`; default gaps sit in `:where()` so the class
  always wins. Extras: `cluster.between()/.end()`, `split.side_width()/.side_end()`. The grid
  minimum and split width travel as a custom property in a `style` attribute (a note for
  M26's CSP box: that needs `style-src-attr`, which the inline `<style>` already implies).
- [x] Demo pages, `PATHS` entries, doctests and README matrix for each; Blitz test for button
  variants and the focus ring.
  Done: four pages in a new "Primitives" group (`/button`, `/field`, `/card`, `/layout`),
  in `PATHS` and `COMPONENTS`; ten `SPECS` entries (README matrix regenerated). Blitz tests
  `buttons_badges_and_icons` and `fields_cards_and_layouts`. The focus ring is checked in
  Firefox (`browser-check.mjs` tabs onto the button: `:focus-visible`, 3px outline), because
  Blitz never matches `:focus-visible` (#839). Found and handled: the test harness now enables
  Blitz's `svg` feature (icons were unpainted); Taffy lays out one column when a grid track
  minimum uses `min()`, so `ui.grid` only uses it under 30rem; the switch is a plain box in
  Blitz (#258). `.error("")` means no error. All in FINDINGS.

## M22 · Components rebuilt on primitives
One change to the button restyles every dialog, pager and table.
- [x] Every component that renders a `button`, `input`, label+field or chip uses the primitive
  builders internally: dialog, drawer, popover/menu, counter, pager, table, paged_table,
  form, wizard, select, combobox, color, range, theme, tabs, palette, empty_state.
  Public API unchanged.
  Done: every visible button, and every link or `<summary>` drawn as one, is `Button` (menu
  triggers, dialog/drawer open-cancel-close with `Icon::X`, counter ±/Reset/Set, "Load more",
  pagination links as ghost/outline link buttons, filter/Go/Show, bulk actions, wizard
  Back/Skip/Next, theme toggle group, colour swatches, empty-state actions, palette trigger);
  `<summary>` fallbacks carry `class="nojs-button"`. Search/filter/number boxes are `Input`
  (`.hide_label()` keeps the label as `aria-label`). `Button` now holds `Caps` rather than
  `&Ui` (components holding only caps build one with `Button::new`), and gained `.content()`,
  `.id/.role/.title/.style/.aria_haspopup/.pressed/.accesskey/.aria_keyshortcuts/.rel/.current`
  and `.formmethod/.formaction/.formnovalidate`; `Input` gained `.search/.hide_label/.list/
  .autocomplete/.autofocus/.inputmode/.step/.aria_controls/.class`. A table row's menu trigger
  is a small ghost icon button (`Icon::Ellipsis`, "Row actions"). Left as they are, on purpose:
  menu items (shadcn's DropdownMenuItem is not a Button either), tab summaries, `<select>`,
  range sliders (range.rs is itself the slider primitive), row checkboxes and hidden inputs.
  The demo's own raw buttons moved to `ui.button` too.
- [x] `form.rs` fields delegate to `input.rs` (one field renderer, not two).
  Done in M21's input box: `Field`, `FieldKind` and the renderer live in `input.rs`; `form.rs`
  holds a list of `Field`s and renders each with `(f)`.
- [x] Delete the per-component button/input CSS the primitives now carry; measure
  `stylesheet()` bytes and the bench before and after.
  Done: the native-control rules (inputs, selects, checkboxes, file, focus and disabled
  states) moved from `layout.rs` into `input.rs` and `button.rs`; the copies of the button look
  in dialog, drawer, pager, paged table, table, palette and popover are gone. Component-owned
  controls got part classes (`nojs-color-input`, `nojs-range-input`, `nojs-counter-input`,
  `nojs-table-filter-input`, `nojs-table-check`, `nojs-palette-input`, `nojs-select-filter`,
  `nojs-combobox-input`, `nojs-paged-table-page`), so no component CSS targets bare `button` or
  `input` any more; the wizard's and the dialog body's field rules became `.nojs-field` ones
  (the demo's dialog field is `ui.input` now).
  Measured against pre-M21 (`167dfd1`), release build, same machine:
  | | pre-M21 | before this box | after |
  |---|---|---|---|
  | `stylesheet()` bytes (gzip) | 49 305 (8 702) | 51 841 (9 455) | 51 605 (9 404) |
  | `/table` HTML, 25 rows | 81 365 | 96 038 | 88 263 |
  | bench `layout` | 12.9 µs | | 12.9 µs |
  | bench `table 1000 rows` | 162 µs | | 164 µs (noise ±10%) |
  | bench `paged_table 25 of 1000` | 8.2 µs | 16.6 µs | 12.2 µs |
  The stylesheet is 2.3 KB (0.7 KB gzip) larger than before M21 for ten new primitives; the
  duplicates removed here were small because M20 had already reduced them to overrides.
  Two regressions found and cut: paged_table built a link string for every page (41) instead
  of the ~11 shown, and each row's menu carried an inline SVG ellipsis (≈330 B a row), now the
  `⋯` glyph. The remaining paged-table cost is the richer markup (ghost/outline page buttons
  with icons and attributes).
- [x] A test fails if a component's CSS styles bare `button`/`input` selectors outside
  `button.rs`/`input.rs`.
  Done: `lib.rs::tests::only_the_primitives_select_bare_buttons_and_inputs` reads every
  selector prelude of the minified component CSS (skipping at-rules), anywhere in it,
  `:is()`/`:where()` arguments included, and names the offending selector. `select`,
  `textarea` and `summary` are not covered: the box asks for buttons and inputs, and
  components still style their own `<select>` (select.rs) and `<summary>` (tabs, accordion).
- [x] CLAUDE.md component convention 8: a component builds its parts from primitives
  (`ui.button`, `ui.input`, `ui.card`…), never raw `button`/`input` with its own CSS.
  Update `docs/` and FINDINGS.
  Done: convention 8 names the primitives, the part-class rule, the test that enforces it and
  the exceptions; `docs/theming.md` says buttons and controls are styled in one place each;
  FINDINGS has an M22 section.
- [x] `tests/shots/` compared before and after; only intended visual diffs.
  Done: Blitz shots from `999e74b` (end of M21) against now, pixel-diffed (all 55 exist on
  both sides, same size). Every page differs only in the theme toggle (small ghost buttons in
  the group); beyond that: counter (± icons, ghost "Reset"), dialog (the body field is
  `ui.input`, 7px tighter), toast (buttons in a cluster), wizard (field spacing from
  `.nojs-field`), table ("Columns" with a chevron, ghost `⋯` row menus instead of "⋯ ▾"),
  popover (chevron icons on the triggers), palette (search icon in the trigger), dashboard (the
  empty-state link is an outline button). All intended; nothing else moved.

## M23 · Flagship widgets
The showcase for "wait, this needs no JS?".
- [x] `calendar.rs`: server-rendered month grid, previous/next month as links (`?month=`),
  day cells as links or radio inputs, `.min/.max/.disabled(fn)`, `.events(..)`, week start;
  swap root for in-place month changes.
  Done: `ui.calendar(name)`; the picked day is `?<name>=YYYY-MM-DD`, the month
  `?month.<name>=YYYY-MM` (named like the other state keys; plain `?month=` would clash with
  two calendars on a page). `calendar::Date` is a small civil date (parse, weekday,
  add_days/add_months, today in UTC) so no date crate is needed. Setters: `.today()` (override
  the server clock), `.min/.max`, `.disabled(fn(Date) -> bool)`, `.event(date, text)` (the
  adder form of `.events`), `.sunday_first()` (Monday by default), `.radio()` and
  `.required()`. `Ui::link_with(key, value)` builds the "this URL with one parameter changed"
  links. Demo `/calendar` (weekends off, two events); in `PATHS`, `COMPONENTS` and `SPECS`.
- [x] `date_picker.rs`: calendar inside a popover (no popover → inline grid) writing a form
  field; native `input type=date` when the caller asks for `.native()`.
  Done: `ui.date_picker(name, label)` with `.value/.min/.max/.disabled/.required/.native`.
  The popover holds the calendar in radio mode (checked radio = the posted value), anchored
  under the button where anchor positioning exists. After a month link the calendar comes
  back laid out in the page, since a popover cannot arrive open. The button shows the saved
  date ("24 September 2026"); without script it does not follow a new pick until the form is
  sent. `Calendar::value()` added so a form's saved date shows. Demo: a GET form on
  `/calendar`; the browser check opens the popover and picks a day.
- [x] `table` upgrades: row selection with bulk actions (checkboxes + one form), inline edit
  row via PRG, sticky header.
  Done: selection with bulk actions (M10's `form=` checkboxes and one bulk form) and the sticky
  header were already there; inline edit is new. `Table::edit(action)` plus `.editable()` per
  column and `Row::values(..)` (the raw text of each cell): every keyed row gets an "Edit"
  link to `?edit.<id>=<key>` (all other parameters kept, page included), and that row draws
  its editable columns as text boxes tied by `form=` to one POST form after the table (a form
  cannot wrap a `<tr>`), with Save and Cancel. The route saves and redirects to the posted
  `returns_to`. Demo: `/table`'s Kind column, saved per visitor in a cookie (`/table/edit`
  only redirects back to `/table…`). Blitz test `table_row_edits_in_place`; the browser check
  edits, saves and reads the new value. `Ui::link_without` and `Input::form` added.
- [x] `upload.rs`: file input with a preview list after the round trip; progress only through
  the enhance script, no-script path intact.
  Done: `ui.upload(action, name)` with `.accept/.multiple/.hint`, `.file(name, bytes)` plus
  `.preview(src)` and `.href(url)` for the file added last, and `.remove(action)` (a Remove
  button per file posting `<name>=<file>`). A dashed drop zone around the file input, then the
  Upload button, then the list. `enhance.js` sends a multipart form holding
  `<progress data-nojs-progress>` through `XMLHttpRequest` to fill the bar (the served budget
  goes from 10 to 11 KB, now 10,564 bytes). `Icon::File` added (29 icons). Demo `/upload`:
  per-visitor, capped, in memory; raster images inline, anything else as an attachment. The
  browser check uploads a real file in place.
- [x] `kanban.rs`: moving a card is a form post per column.
  Done: `ui.kanban(action)` with `.column(key, title)`, `.limit(n)` (shows `n / limit`, red past
  it; the server decides whether to refuse), `.card(key, title)` and `.note(text)` for the card
  added last. Each card has ghost arrow buttons to the neighbouring columns posting
  `card=<key>&to=<column>`; the route moves it (last in the new column) and redirects. A swap
  root, and `view-transition-name` per card where supported, so the card slides across with the
  script. The board scrolls sideways with snap on narrow screens. Demo `/kanban` (per visitor,
  in a cookie); the browser check moves a card and back.
- [x] Demo pages, Blitz screenshots, browser-check steps for the calendar swap.
  Done: `/calendar` (calendar plus a date-picker form), `/upload`, `/kanban` and the table's
  edit state are in `PATHS` (Blitz shots for each) and `COMPONENTS`. The Blitz test
  `calendar_date_picker_upload_and_kanban` checks the week rows, picked and blocked days, the
  month link, the closed popover, the in-page calendar after a month link, the multipart form
  and the kanban columns. The browser check changes month and picks a day in place, opens the
  date picker, uploads a file, moves a kanban card and edits a table row. Blitz draws no file
  picker (#258, in FINDINGS). README's summary lists the new widgets.

## M24 · Write your own component
The React idea worth keeping: a component model users extend, not a closed catalogue.
- [x] `docs/components.md`: a component from primitives in about 30 lines (builder holding
  `&Ui`, an extension trait for `impl Ui` in user crates, `impl Render`, CSS const, swap id).
  Done: a newsletter box (`ui.subscribe(action)`: a card with `ui.input` and `ui.button`,
  a thank-you state read from `?subscribed`, a swap root, a CSS const on tokens), its routes
  and assertions on the output; the whole page is included under `#[cfg(doctest)]` so its
  code runs with `cargo test`. The rules follow: extension trait, input from `ui`, primitives
  only, tokens only, an own class prefix, `Page::css`, one variant per request, swap roots,
  `Saved<T>` needs named fields, Blitz for tests, and `maud` as a direct dependency. Needed
  two helpers from the next box, added here: `axum_nojs::slug` is public and `Page::css` exists.
- [x] Public helpers users need: `slug`, `enhance::swap_id`, `caps`, and a way to add CSS to
  `ui.page()` (`Ui::with_css` or `Page::css`).
  Done: `axum_nojs::slug` and `Page::css` (with the guide, previous box); `enhance::swap_id`
  and `axum_nojs::caps` were already public. Added `Ui::link_with(key, value)` and
  `Ui::link_without(key)` (made public, with a doctest): the "same page, one parameter
  changed" links the calendar and table use, which a user component needs just as much.
- [ ] A user-land `ui.pricing_card()` in the demo crate, built only from primitives.

## M25 · Positioning for release
- [ ] README opening: "the no-JS UI kit for Rust servers", a layers diagram first
  (primitives → components → widgets → yours), then a comparison with Leptos, Dioxus and
  htmx + hand-written Maud.
- [ ] Demo index grouped by layer.
- [ ] M13 (Publish) happens after M26.

## M26 · Compete on the guarantee, not the catalogue
`maud-ui` (crates.io, MIT, 0.20.3 on 2026-09-23) already ships the same stack and look: Maud,
shadcn styling, 84 components, 31 blocks, 15 JS widget shells. It needs htmx plus an 89 KB
(24 KB gzip) script and 313 KB (44 KB gzip) of CSS, its no-script claim is untested (its docs
say the no-JS browser check "was not run"), and its API is `Props { .., ..Default::default() }`.
Do not chase its breadth or wrap JS widgets. Win on what it cannot promise: zero required
script, proven in CI, with server-side flows included.
- [ ] Name: `maud-ui` rules out a generic `*-ui`; the differentiator belongs in the name.
  Proposed `nojs-ui` (+ `nojs-ui-caps`, `nojs-ui-test`, Axum stays the `axum` feature; free
  on crates.io as of 2026-09-24), or keep `axum-nojs`. Ask the owner before renaming; the
  `nojs-*` classes, `--nojs-*` tokens and `/nojs/` routes stay either way.
- [ ] Measure and publish: bytes shipped per demo page (HTML, CSS, script = 0 required),
  `stylesheet()` size raw and gzip, next to maud-ui's numbers; a bench or test keeps them
  from regressing.
- [ ] Strict CSP: the demo sends `Content-Security-Policy: script-src 'none'` (and `'self'`
  only when the enhancement script is on); a test asserts every route renders under it and
  README documents the header.
- [ ] README badge line: "0 KB JavaScript required · verified by a script-less renderer (Blitz)
  in CI", linking the only-one-script test and the Blitz suite.
- [ ] Comparison page in docs (`docs/comparison.md`): maud-ui, htmx + hand-written Maud,
  Leptos/Dioxus on required JS, CSP, no-script proof, API shape (builder vs `Props`), server
  state; side-by-side of `ui.button("Ship it").primary()` vs `button::render(button::Props {..})`.
- [ ] API shape stays builders (asked and answered after the owner weighed maud-ui's
  `Props { .., ..Default::default() }`; M11 tried options structs and M18 removed them). Why:
  components read their input from `ui` (params, state, caps, swap ids), which a detached
  struct cannot; list adders with last-item modifiers (`.tab(..).badge(3)`) beat nested
  `vec![Item { .., ..Default::default() }]`; short calls stay short inside `html!`; setters
  carry meaning (no-arg switches on, `bool` from a condition, `.id()` derived by `slug`).
  Take the two things Props does better:
  - [ ] Builders are plain data: every builder derives `Clone` and `Debug` so a route can
    build one in a loop, keep it in a variable, or pass it around; a test or clippy-style
    check fails on a builder missing either.
  - [ ] Every option discoverable in one place: each component's rustdoc groups its setters
    (required call, switches, conditions, list adders and their modifiers) on the builder
    type, and `docs/comparison.md` shows the builder next to the equivalent Props call.
- [ ] Complete flows, not just widgets: a demo "app" section with sign-in with server
  validation errors, create/edit/delete via PRG with flash, a filterable paged table, and a
  multi-step wizard, all with script off; each a Blitz test.
- [ ] Close the shadcn must-have gaps M21 does not cover: `tooltip` (popover `hint` /
  `title` fallback), `alert`, `progress`/`meter`, `separator`, `textarea` field. Then stop:
  no JS widget shells (editors, grids, maps).
- [ ] Audience pages in docs: public-sector / GOV.UK-style services, strict-CSP environments,
  low bandwidth and old devices, Tor Browser "Safest", internal tools, each with the
  guarantee it relies on.
- [ ] Launch material: live demo host, a post "shadcn look, zero JavaScript, verified",
  crates.io keywords `no-js`, `progressive-enhancement`, `maud`, `ssr`, `components`.
  Posting to r/rust / This Week in Rust and hosting are outward actions: ask the owner first.

## M27 · Loco fit (only if the owner picks Loco)
The owner thinks [Loco](https://loco.rs) (Rails-style, built on Axum) is the best home for
this library. Loco controllers are Axum handlers, so `Ui`, `Page`, `Redirect` and `Saved<T>`
should already work there; this milestone makes it a first-class fit instead of an accident.
Loco's default views are Tera templates and its scaffolds generate them, so the gap is
wiring, generators and docs, not components. Check every Loco API named below against the
loco-rs source (fetch it into `~/.cargo/registry` with a scratch crate) before relying on it.
- [ ] Decision: commit to Loco as the primary target? Owner only; if no, skip this milestone.
  If yes, it also bears on M26's naming box (a `loco-nojs`/`nojs-ui` split, or one crate with
  a `loco` feature).
- [ ] `loco` feature (or `axum-nojs-loco` crate): an `Initializer` whose `after_routes` mounts
  `/nojs/enhance.js` and the `/nojs/caps` beacon route, so an app adds one line to
  `app.rs::initializers`.
- [ ] Handlers return Loco's `Result<Response>`: `Page`, `Redirect` and `Streamed` convert with
  `?`/`.into_response()`, no wrapper; a doctest shows a Loco controller using `ui: Ui`.
- [ ] Validation errors: Loco models validate with the `validator` crate; a helper maps
  `ValidationErrors` into `Form::errors(..)`/`Input::error(..)` so server errors land on the
  right field.
- [ ] Data: SeaORM's paginator feeds `paged_table`/`pager` (`?page=` and page size) without
  loading every row; an example query in the docs.
- [ ] Flash and PRG: `Redirect` + `ui.flash()` work with Loco's cookie setup (the private
  cookie key from `config/*.yaml` if we use signed cookies); no conflict with Loco's
  session or auth middleware.
- [ ] Views: document Maud views beside Loco's Tera default (a `views/` module of functions
  returning `Markup`), and decide whether a Tera function bridge (`{{ nojs_button(..) }}`) is
  worth it; default answer: no, Maud only, stated in the docs.
- [ ] Generator: a scaffold override (`cargo loco generate override` templates, or our own
  template set) that emits Maud views built from `ui.*` for list/show/new/edit, PRG included.
- [ ] `examples/loco-app`: a minimal Loco app (one model, CRUD, sign-in) with script off;
  Blitz renders its routes and the only-one-script test covers them.
- [ ] `docs/loco.md` and a README section: install, the initializer line, a controller, a
  form with validation, the generator.
