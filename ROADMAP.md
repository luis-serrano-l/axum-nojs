# Roadmap

Rules: work top to bottom. A milestone is done only when every box is ticked, `cargo clippy
--all-targets` is clean, `cargo test` passes, the no-script test still passes, README's feature
matrix and findings are updated, and the work is committed. Unknowns become entries in
`FINDINGS.md`, never reasons to add script.

## M1 · Capability beacons (server-side feature detection, no script)
- [x] `loco_ui::caps`: `Caps` bitset (invokers, anchor, details_content, view_transitions, popover, light_dark, streaming_dsd)
- [x] `caps::beacon_css()` emits `@supports` rules that request `/lui/caps?flag=1` as a background image
- [x] Axum route `/lui/caps` sets/extends a `loco-ui-caps` cookie; `Caps` implements `FromRequestParts`
- [x] Every component takes `&Caps` and emits only the best markup for that browser (dialog: invokers vs `:target`; popover: anchor vs centred; tabs: `::details-content` vs accordion)
- [x] Demo page `/caps` shows what the server thinks the browser supports
- [x] Screenshot verification in Firefox headless; old-Chrome-109 check confirms fallbacks render

## M2 · Out-of-order streaming without script
- [x] `loco_ui::stream`: `Streamed` response type built on `axum::body::Body::from_stream`
- [x] `slot(id, placeholder)` renders `<lui-slot><template shadowrootmode=open><slot name=id>…`
- [x] `fill(id, future)` appends the resolved chunk with `slot=id` later in the stream, any order
- [x] Demo `/stream` with three slow sections (100ms, 800ms, 2s) arriving out of order
- [x] Fallback when DSD unsupported (per `Caps`): render sequentially at the end
- [x] Test: response body is chunked and slots arrive in completion order

## M3 · State model for scriptless apps
- [x] `loco_ui::state`: `UiState` extractor merging query + cookie (open tab, open details, dialog)
- [x] `prg(redirect_to, flash)` helper: Post/Redirect/Get with a one-shot flash cookie
- [x] `flash()` component rendering and clearing the flash
- [x] `details` and `tabs` persist open state via `?open=` links generated from `UiState`
- [x] Demo: settings page with tabs + form + flash that survives a full navigation
- [x] Docs: one page "how state works with no script"

## M4 · Blitz as the test engine
- [x] `loco-ui-test` crate: render a route via `tower::oneshot`, load HTML into `blitz-dom`, resolve layout
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
- [x] Examples in `loco-ui/examples/`
- [x] Publish dry run: `cargo publish --dry-run -p loco-ui`
- [x] Demo visual pass: one palette (`--lui-*` for light and dark, moss accent), one type scale, index grouped by platform feature, toolbar with a back link and the theme switch on every component page

## M7 · Optional enhancement script
- [x] `loco_ui::enhance`: one small script (`/lui/enhance.js`, content-hashed, immutable) that upgrades swap roots (`id` + `data-lui="swap"`) to fetch + replace, queued per root, with focus, flash, title, theme and URL synced
- [x] Counter, form, tabs, accordion, pager, theme toggle are swap roots; combobox searches as you type; range and colour mirror live; `:target` dialog fallback opens as a real modal; `<details>` popover fallback light-dismisses
- [x] Enforcement: exactly one `<script>` per page and it is the enhancement tag; no inline handlers; Blitz suite (no script engine) proves every route works without it
- [x] Headless Firefox check through geckodriver (`scripts/browser-check.mjs`, run by `scripts/verify.sh` when available)
- [x] Docs: README, CLAUDE.md, FINDINGS "with the script" section, spec entry

## M8 · Framework independence
- [x] `Caps::from_cookie_header(&str)` and `Caps::from_query(&str)` as the only entry points; the Axum extractor becomes a thin wrapper behind the `axum` feature
- [x] Every component returns `Markup` that also implements `Render`; a `string` feature (or `.into_string()` docs) shows use without Maud templates
- [x] `state::prg`, `flash` and `stream` compile without Axum: an `http` feature exposes `http::Response` builders, the `axum` feature wraps them
- [x] Example `loco-ui/examples/actix_server.rs` (or `hyper_server.rs`) rendering three components with the beacon route wired by hand
- [x] README: "Use with any server" section; CLAUDE.md workspace notes updated

## M9 · `loco-ui-caps` as its own crate
- [x] Move `caps.rs` (bitset, `@supports` beacons, cookie parsing, beacon route) into `loco-ui-caps/` in the workspace; `loco-ui` depends on it and re-exports `Caps`, `Cap`
- [x] Spec page `docs/caps.md`: how the beacons work, the first-view problem, what each flag tests, cookie format, how to add a flag
- [x] Standalone example: a raw `hyper` handler that reads `Caps` and prints one line per flag
- [x] `cargo publish --dry-run -p loco-ui-caps` passes; README of the sub-crate written for a reader who has never seen loco-ui

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
- [x] `docs/theming.md`: every `--lui-*` token, what it affects, contrast requirements, one worked example with a different palette
- [x] Demo `/theme-demo` (or a query flag on the index) rendering the same page under a second palette; Blitz screenshot pair `index-modern.png` vs `index-alt.png`
- [x] Test: no colour literal outside `layout.rs` (grep for `#[0-9a-f]{3,6}` in component CSS)

## M13 · Publish
- [x] `license`, `repository`, `readme`, `keywords`, `categories` in every publishable `Cargo.toml` (MIT; https://github.com/luis-serrano-l/loco-ui)
- [x] `CHANGELOG.md` with 0.1.0; version bump; `cargo publish --dry-run` for `loco-ui-caps` then `loco-ui`
      (both crates are 0.1.0; `loco-ui-caps` dry-runs clean; `loco-ui` alone cannot until `loco-ui-caps`
      is on crates.io, so `cargo package --workspace --exclude demo --exclude loco-ui-test`
      verifies both together)
- [x] docs.rs metadata (`all-features`), crate-level README rendered on docs.rs checked with `cargo doc --no-deps`
- [ ] The publish itself is an outward action: ask the owner, do not run `cargo publish` without a yes

## M14 · What htmx has that the enhancement script does not
Every item must keep the no-script path intact: the markup is the same, the script only reads
attributes. Blitz proves each route works without it; `scripts/browser-check.mjs` proves the
script does its job.
- [x] Partial swaps with explicit targets: `data-lui-target="#id"` on a form or link swaps that
  root instead of the closest one; `data-lui-swap="inner|outer|append|prepend"` chooses how.
  Without the script the same request is a full navigation to the same page.
- [x] Out-of-band updates: a response may carry extra swap roots marked `data-lui-oob`; the
  script replaces each matching `id` anywhere in the page (flash banner, counter in the header)
  and drops them from the main swap. Without the script the full page already shows them.
- [x] Request lifecycle feedback: `data-lui-busy` class on the root while a request is in flight,
  `aria-busy="true"`, submit buttons disabled, a `--lui-busy` CSS hook; optional
  `data-lui-indicator="#id"` element shown while pending. Failed requests fall back to a normal
  navigation so the user always sees the server's answer.
- [x] History and URL control: `data-lui-push="false"` keeps the URL, `data-lui-replace` uses
  `replaceState`, and Back/Forward restore the swapped roots from a cached copy instead of a
  reload; a `lui:swap` custom event fires after every swap for anything that must react.
- [x] Spec entry for the enhancement script updated, README "How the script works" section,
  Firefox checks for each attribute, FINDINGS on what the platform still cannot do.

## M15 · Components worth using
The components are too basic: each proves a platform feature but stops short of what an app
needs from it. Make each one something a real page would reach for, without giving up what
makes them simple: one function, one options struct, plain HTML you can `curl`, no script
beyond `/lui/enhance.js`, every state a URL or a form. One box per component; each box ends
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
  hash on `/lui/caps` beacon images and `/lui/enhance.js` verified; gzip/br on the demo through
  `tower-http` `CompressionLayer`; release profile with `lto = "fat"`, `codegen-units = 1`,
  `panic = "abort"` for the demo binary.
- [x] Cheap wins, page: the inline stylesheet minified (whitespace and comments stripped at
  build time, a test proves it still parses); beacons `loading="lazy"` and `fetchpriority="low"`
  so they never delay first paint; `<script defer>` stays last; `<link rel="preconnect">` not
  needed (no third party) and recorded as such; the caps cookie small enough to fit one
  `Set-Cookie`.
- [x] Cheap wins, script: `enhance.js` requests carry `Accept: text/html` and the server's
  fragment answer (`Lui-Enhance: 1`) used on every swap route in the demo, not only `/swap`,
  so a swap moves a few hundred bytes instead of the page; `fetch` with `priority: "high"`
  for user actions; prefetch on `mouseenter`/`focus` for same-origin links inside a swap root
  (`data-lui-prefetch`), cached for a few seconds and reused by the click.
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
`use loco_ui::prelude::*`, no `_with` twins, no `XOptions`; handlers lose their cookie
plumbing through `Saved<T>` and `ui.redirect(to).ok(..).save(&value)`.
- [x] Core: `Ui` gains `page(title, body) -> Page` (an `IntoResponse` that writes back changed
  state and clears a shown flash, so no more `(ui, markup)`), `redirect(to) -> Redirect`
  (`.flash/.ok/.warn/.danger/.cookie`, `into_http`), `param`/`params` (decoded query, so
  components read their own input), `From<Caps>`, `Default`. `prg`/`prg_parts` removed.
  `dialog` is no longer remembered in the `lui-ui` cookie.
- [x] `saved.rs` (`axum` feature, serde + serde_urlencoded): `Saved<T>` extractor, cookie
  `lui-<type-name>`, `Redirect::save`/`forget`.
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
- [x] `cargo test -p loco-ui --all-features` green (38 unit, 54 doc), and the crate builds
  with no features and with `--features http`.
- [x] Demo rewrite (`demo/src/lib.rs`): one prelude import, every route in the new form, each
  handler `Page`/`Redirect`, `Saved<Settings>`/`Saved<Count>`/`Saved<Signup>`/`Saved<Inputs>`
  instead of hand-parsed cookies (cookie names change to `lui-*`: check `loco-ui-test`
  and `scripts/browser-check.mjs` for the old `count`, `settings`, `inputs`, `wizard`, `notes`
  cookies), dialog page keeps `.id("confirm")` so `/dialog?dialog=confirm` in `PATHS` still
  works, table + CSV share one `fn files_table(ui)`, palette redirects via `.exact()`,
  wizard uses `.review()` and `Posted`. Goal: the demo visibly shorter; count lines before/after.
- [x] Demo tests (`demo/src/lib.rs` tests), `loco-ui-test` (Blitz) and
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
  `--lui-*` tokens (`.lui-snippet`, `.lui-hl-*` in `layout.rs`); no script. A test checks
  every component page has a snippet and that the box shows exactly that code.
- [x] Visual pass with the frontend-design skill. Plan: keep ink and moss (tokens only), spend
  the boldness on one element, the plate: the live component on a stage (`--lui-surface`)
  with the code box joined under it, one per page. Index rows became a two-column grid
  (name, then a plain-words line of what the component is for, then the chips); the same
  line is the lede under each component title; chips no longer break mid-word. Reviewed
  against the defaults: no card grid, shadows, gradients or eyebrows. Checked in Firefox
  (light and dark, 1280 and 420 wide) and the Blitz PNGs. The colour-literal test no longer
  mistakes `white-space` for a colour.
- [x] Replace the demo's hand-written `highlight()` with `syntect` (asked by the owner): a
  dependency of `demo` only (`default-syntaxes`, `regex-fancy`; no bundled themes, no
  onig C build), never of `loco-ui`. Its parser and Rust grammar decide the scopes; the
  demo maps scope prefixes to the same seven `lui-hl-*` classes coloured by `--lui-*`
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
  `accent` as hover surface, `primary`/`on_primary`, `input`, `ring`), keeping `--lui-*`
  names; `Tokens::default()` is shadcn's neutral (zinc) light and dark, primary near-black /
  near-white. `radius` 0.5rem with derived `--lui-radius-sm/-lg`. ok/warn/danger from Radix
  Colors steps 9/11. Docs and `docs/theming` updated; the "ink and moss" wording removed.
  Done: the old brand token became `--lui-primary`/`--lui-on-primary` everywhere and
  `--lui-accent` is now the hover surface. Status colours use Radix step 11 (red/green/amber),
  the text step, so each clears 4.5:1; danger buttons put `on-primary` on it (5.0 / 8.4).
  `--lui-radius-sm/-lg` are derived as radius ∓ 4px, so a radius needs a unit (`"0px"`).
- [x] Type: system stack only (`ui-sans-serif, system-ui, -apple-system, "Segoe UI", Roboto,
  "Helvetica Neue", Arial, sans-serif`, mono `ui-monospace, SFMono-Regular, Menlo, Consolas`),
  shadcn's scale (text-sm 0.875rem body in controls, 1.25/1.5 line heights, 500/600 weights),
  `-webkit-font-smoothing: antialiased`, tabular numbers in tables and stats.
  Done: the stacks are `--lui-font-sans`/`--lui-font-mono` on `:root` (base rules, not
  `Tokens` fields); every component's sizes snapped to the scale (0.75/0.875/1/1.125/1.25/1.5/
  2.25rem) and 700 weights to 600.
- [x] Base styles in `layout.rs`: shadcn heights and paddings (controls h-9 = 2.25rem, px-3/px-4),
  1px `--lui-input` borders, `shadow-xs` on controls, `shadow-lg` on dialog/popover,
  focus-visible as a 3px `--lui-ring` at 50% opacity, `aria-invalid` red ring, disabled at
  50% opacity, hover as `--lui-accent` surface. Native `<select>`, checkbox, radio, range,
  date and `<details>` restyled to match (`appearance`, `accent-color`).
  Done: `--lui-shadow-xs/-lg` are emitted by `Tokens::css()` beside the radii (rgb literals
  live there, not in component CSS); `--lui-radius-sm` became radius − 2px so controls land
  on shadcn's 6px `rounded-md`. The plain button is shadcn's outline variant. The select
  chevron is two gradients in `--lui-muted` (no data-URI SVG, which would need a literal);
  `select.rs` drops it where `appearance: base-select` draws its own picker icon.
- [x] Every existing component's CSS re-tuned to these values (dialog, drawer as shadcn sheet,
  popover/menu as dropdown-menu, tabs as the muted pill list, accordion, toast as sonner-style
  cards, table, pager as pagination, badge-like chips, skeleton, palette as command).
  Done: `Tokens::css()` also emits `--lui-overlay` (black/50, as shadcn), used by every
  backdrop; all floating layers take `--lui-popover` and `--lui-shadow-lg`. Tabs: each
  summary paints its slice of the muted pill and `.lui-tabs-mark` became the raised chip
  (the Blitz test now checks the chip, inset 3px). Toasts and flashes are neutral cards with
  a level dot. Link-as-button rules repeat the button values until M22 builds them on the
  primitive.
- [x] The demo takes the same look (index, plate, code box); syntect classes recoloured.
  Done: index groups are grids of cards (the title link stretches over the card), "built on"
  items are outline badges, the stage is a preview box over a `--lui-surface` code block, and
  the seven `lui-hl-*` classes use danger/ok/warn/muted/fg (GitHub-like, still tokens only).
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
  same look. The global `button {}` CSS in `layout.rs` moves into `button::CSS` as `.lui-button`.
  Done: modifiers are `lui-button-{primary,danger,ghost,small,icon}`; bare `button` and
  `button.lui-primary`/`.lui-danger` keep the same rules (same specificity as before) so
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
  done early), with the `.lui-field*` CSS. `ui.input` takes the type as a setter
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
  gaps from `--lui-space-*` tokens.
  Done: one file each (`stack.rs`, `cluster.rs`, `grid.rs`, `split.rs`); each takes its
  content as `Markup` (`ui.grid(min, content)`, `ui.split(side, main)`). `Tokens::css()`
  derives `--lui-space-{1,2,3,4,6,8}` (n × 4px by default, Tailwind's steps) and `.gap(n)`
  adds a `lui-gap-n` class from `layout.rs`; default gaps sit in `:where()` so the class
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
  `<summary>` fallbacks carry `class="lui-button"`. Search/filter/number boxes are `Input`
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
  controls got part classes (`lui-color-input`, `lui-range-input`, `lui-counter-input`,
  `lui-table-filter-input`, `lui-table-check`, `lui-palette-input`, `lui-select-filter`,
  `lui-combobox-input`, `lui-paged-table-page`), so no component CSS targets bare `button` or
  `input` any more; the wizard's and the dialog body's field rules became `.lui-field` ones
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
  `.lui-field`), table ("Columns" with a chevron, ghost `⋯` row menus instead of "⋯ ▾"),
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
  `<progress data-lui-progress>` through `XMLHttpRequest` to fill the bar (the served budget
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
  two helpers from the next box, added here: `loco_ui::slug` is public and `Page::css` exists.
- [x] Public helpers users need: `slug`, `enhance::swap_id`, `caps`, and a way to add CSS to
  `ui.page()` (`Ui::with_css` or `Page::css`).
  Done: `loco_ui::slug` and `Page::css` (with the guide, previous box); `enhance::swap_id`
  and `loco_ui::caps` were already public. Added `Ui::link_with(key, value)` and
  `Ui::link_without(key)` (made public, with a doctest): the "same page, one parameter
  changed" links the calendar and table use, which a user component needs just as much.
- [x] A user-land `ui.pricing_card()` in the demo crate, built only from primitives.
  Done: `demo/src/pricing.rs` holds `PricingExt` (the extension trait), `PricingCard` (`.period`,
  `.blurb`, `.feature` per line, `.cta(text, href)`, `.featured()`) and `PRICING_CSS` (classes
  `demo-*`, tokens only). The card is `ui.card` with `ui.badge`, `ui.link_button`, `ui.stack`
  and `Icon::Check`; nothing private is used. `/pricing` shows three tiers in `ui.grid`, a
  monthly/yearly switch of `ui.link_with`/`link_without` links, and adds the CSS with
  `Page::css`. It is the "Your own" group on the index. Blitz test
  `a_component_written_outside_the_library`: three cards, equal heights, the CSS inlined once,
  the featured button primary, yearly prices.

## M25 · Positioning for release
- [x] README opening: "the no-JS UI kit for Rust servers", a layers diagram first
  (primitives → components → widgets → yours), then a comparison with Leptos, Dioxus and
  htmx + hand-written Maud.
  Done: the tagline and one paragraph, `docs/layers.svg` (diagram-design skill, big-text
  profile, a standalone SVG linked from the README: five bands from the HTML/CSS platform up
  to "Yours", which is the one accent band), a paragraph on what the layering buys, and a
  table against Leptos/Dioxus and htmx + Maud (where the UI runs, required JS, behaviour with
  script blocked, components, state) with when to pick which. The screenshot and the code
  sample follow unchanged.
- [x] Demo index grouped by layer.
  Done: the index is four layers, bottom up as in `docs/layers.svg`: Primitives, Components
  (with Overlays, Disclosure, Navigation, Input, Feedback and Server state as sub-headings),
  Widgets (calendar, upload, kanban) and Your own (the pricing card), each with a one-line
  blurb; the lede counts them ("4 primitives, 19 components, 3 widgets and one of your own").
  A demo test fails if an index entry's group sits in no layer.
- [x] M13 (Publish) happens after M26.
  Recorded: M13's publish box stays on hold in BLOCKED.md (the owner's yes, never on their
  behalf), and is not taken before M26 is through; M26's naming box may still change what is
  published.

## M26 · Compete on the guarantee, not the catalogue
`maud-ui` (crates.io, MIT, 0.20.3 on 2026-09-23) already ships the same stack and look: Maud,
shadcn styling, 84 components, 31 blocks, 15 JS widget shells. It needs htmx plus an 89 KB
(24 KB gzip) script and 313 KB (44 KB gzip) of CSS, its no-script claim is untested (its docs
say the no-JS browser check "was not run"), and its API is `Props { .., ..Default::default() }`.
Do not chase its breadth or wrap JS widgets. Win on what it cannot promise: zero required
script, proven in CI, with server-side flows included.
- [x] Name: `maud-ui` rules out a generic `*-ui`; the differentiator belongs in the name.
  Proposed `nojs-ui` (+ `nojs-ui-caps`, `nojs-ui-test`, Axum stays the `axum` feature; free
  on crates.io as of 2026-09-24), or keep `axum-nojs`.
  Answered 2026-09-24: keep `axum-nojs`. Reversed 2026-09-25: renamed to `loco-ui` (prefix
  `lui-`, macro `lui!`), because "nojs" promised no JavaScript while an optional script ships;
  history kept by moving the directories in a commit of their own.
- [x] Measure and publish: bytes shipped per demo page (HTML, CSS, script = 0 required),
  `stylesheet()` size raw and gzip, next to maud-ui's numbers; a bench or test keeps them
  from regressing.
  Done: README "What a page weighs": stylesheet 57.6 KB (10.3 KB gzip), demo pages 62–80 KB
  (11.8–13.5 KB gzip) with the stylesheet inlined, 0 required script, the optional one 10.6 KB
  (3.6 KB gzip), beside maud-ui's 313 KB CSS / 89 KB script. Tests: `stylesheet()` under
  64 KB (loco-ui), every `PATHS` page under 96 KB in both caps variants (demo); the
  shadow-DOM stream carries the stylesheet twice (117 KB), gets 128 KB, and is in FINDINGS.
- [x] Strict CSP: the demo sends `Content-Security-Policy: script-src 'none'` (and `'self'`
  only when the enhancement script is on); a test asserts every route renders under it and
  README documents the header.
  Done: `enhance::CSP` (`script-src 'self'`) and `enhance::CSP_NO_SCRIPT` (`'none'`), sent by
  the `enhance::csp` middleware on HTML answers without a policy of their own; which one is
  decided by `Page::without_script()` (a response extension, nothing on the wire). The demo
  layers it and serves any page script-less with `?script=off`. Test
  `every_route_is_served_under_a_strict_csp` checks the header on every `PATHS` route and the
  `'none'` variant; the Firefox check passes with the policy in force. Styles need
  `'unsafe-inline'` (inlined stylesheet, a few `style` attributes); README documents it.
- [x] README badge line: "0 KB JavaScript required · verified by a script-less renderer (Blitz)
  in CI", linking the only-one-script test and the Blitz suite.
  Done: a text line under the title (no image badge: no third-party badge service), linking
  `loco-ui-test/tests/demo.rs` (Blitz) and `demo/src/lib.rs` (the only-one-script test), plus
  "strict CSP". True as written: `.github/workflows/rust.yml` runs `cargo test --workspace`,
  which runs both.
- [x] Comparison page in docs (`docs/comparison.md`): maud-ui, htmx + hand-written Maud,
  Leptos/Dioxus on required JS, CSP, no-script proof, API shape (builder vs `Props`), server
  state; side-by-side of `ui.button("Ship it").primary()` vs `button::render(button::Props {..})`.
  Done: a table on where the UI runs, required JS, behaviour with script blocked, proof,
  strict CSP, CSS shipped, components, server state and API shape; the button side by side
  (the Props field names marked illustrative, maud-ui's source not read here) with why the
  builder fits; when to pick which. Linked from the README.
- [x] API shape stays builders (asked and answered after the owner weighed maud-ui's
  `Props { .., ..Default::default() }`; M11 tried options structs and M18 removed them). Why:
  components read their input from `ui` (params, state, caps, swap ids), which a detached
  struct cannot; list adders with last-item modifiers (`.tab(..).badge(3)`) beat nested
  `vec![Item { .., ..Default::default() }]`; short calls stay short inside `html!`; setters
  carry meaning (no-arg switches on, `bool` from a condition, `.id()` derived by `slug`).
  Take the two things Props does better:
  - [x] Builders are plain data: every builder derives `Clone` and `Debug` so a route can
    build one in a loop, keep it in a variable, or pass it around; a test or clippy-style
    check fails on a builder missing either.
    Done: `Pager` and `Tabs` held boxed closures; they are `Rc` now, so both are `Clone`,
    with a hand-written `Debug` that prints a closure as `<fn>`. `Streamed` owns its fill
    futures and is `Debug` only (documented). Test `every_builder_is_clone_and_debug` reads
    every `pub struct` in `src/`.
  - [x] Every option discoverable in one place: each component's rustdoc groups its setters
    (required call, switches, conditions, list adders and their modifiers) on the builder
    type, and `docs/comparison.md` shows the builder next to the equivalent Props call.
    Done: 38 builders carry a "**Setters.**" paragraph grouping values and items, no-argument
    switches and `bool` conditions (the required call is the `ui.x(..)` method it names);
    test `every_setter_is_listed_on_its_builder` fails when a setter is missing from it (it
    caught 14 on the first run). `docs/comparison.md` has the side by side.
- [x] Complete flows, not just widgets: a demo "app" section with sign-in with server
  validation errors, create/edit/delete via PRG with flash, a filterable paged table, and a
  multi-step wizard, all with script off; each a Blitz test.
  Done: a "Complete flows" group on the index. `/app/signin` (server checks; mistakes
  re-render the form with messages beside the fields, the email kept and the password never
  echoed; success sets a session cookie and redirects) and `/app/notes` (add, rename in place,
  delete from the row menu, each Post/Redirect/Get with a flash; the list is a filterable,
  sortable, paged table). `Form::password` added. The wizard was already at `/wizard` with its
  Blitz test. Blitz test `a_whole_app_flow_with_no_script` walks it with a cookie jar: bad
  sign-in, good sign-in, add, edit in place, delete, rendering each state.
- [x] Close the shadcn must-have gaps M21 does not cover: `tooltip` (popover `hint` /
  `title` fallback), `alert`, `progress`/`meter`, `separator`, `textarea` field. Then stop:
  no JS widget shells (editors, grids, maps).
  Done: `tooltip.rs` (CSS only: shown on `:hover`/`:focus-within`, named by
  `aria-describedby`, off on touch screens; chosen over `popover="hint"`, which is
  Chromium-only and needs script to open on hover), `alert.rs` (neutral, danger with
  `role="alert"`, warn, ok; tone icons), `progress.rs` (styled `<progress>`, indeterminate
  when `max` is 0), `meter.rs` (`<meter>` whose low/high/optimum pick ok, warn or danger),
  `separator.rs` (`<hr>`, labelled, vertical). The textarea field already existed
  (`ui.input(..).textarea(rows)`, `Form::textarea`). Demo `/feedback`; SPECS and README matrix;
  the browser check focuses a tooltip's trigger and sees it shown. Stopped there.
- [x] Audience pages in docs: public-sector / GOV.UK-style services, strict-CSP environments,
  low bandwidth and old devices, Tor Browser "Safest", internal tools, each with the
  guarantee it relies on.
  Done: `docs/audiences.md`, one section each: the need, the guarantee it relies on, and the
  proof (which test, script or number), plus who it is not for. Linked from the README.
- [ ] Launch material: live demo host, a post "shadcn look, zero JavaScript, verified",
  crates.io keywords `no-js`, `progressive-enhancement`, `maud`, `ssr`, `components`.
  Posting to r/rust / This Week in Rust and hosting are outward actions: ask the owner first.
  Partly done: keywords set (`no-js`, `maud`, `ssr`, `components`, `axum`; the 23-character
  `progressive-enhancement` exceeds crates.io's 20-character limit, so it is in the
  description), and the post drafted in `docs/launch-post.md`.
  Hosting answered 2026-09-24: a static snapshot on GitHub Pages (no server). Posting stays
  with the owner (BLOCKED.md).
  - [x] Static snapshot: export every GET page in `PATHS` to HTML (both caps variants where
    they differ, the enhancement script off), rewrite links to relative `.html`, and add a
    banner on every page: forms, cookies and paging need the real server (`cargo run -p demo`).
    `<dialog>`, `popover`, `<details>` and tooltips keep working. A script (`scripts/snapshot.sh`)
    writes it to `target/site/`, and a test checks that every exported page has the banner and no `<script>`.
    Done: `demo/src/snapshot.rs` (`cargo run -p demo -- snapshot <dir>`, wrapped by
    `scripts/snapshot.sh`). Each path renders with every capability as `<name>.html` and with
    none as `<name>.baseline.html` where the two differ (15 of 33), linked from the banner; the
    capability beacons are stripped. Links and GET form actions to an exported path point at its
    file (the exact query if `PATHS` has it, else the route's first export); POST targets and
    `/table.csv` stay absolute and fail, as the banner says. `.nojekyll` is written too. The test
    also checks every `.html` link resolves.
  - [x] A Pages workflow (`.github/workflows/pages.yml`) that builds the snapshot and deploys it.
    Enabling Pages in the repository settings and pushing are the owner's actions.
    Done: on push to `main` (and by hand), `scripts/snapshot.sh` then
    `actions/upload-pages-artifact` and `actions/deploy-pages`. Settings → Pages → Source
    "GitHub Actions" and the push are the owner's (BLOCKED.md). `scripts/verify.sh` now also
    runs `cargo fmt --check`, as CI does.

## M27 · Loco fit (only if the owner picks Loco)
The owner thinks [Loco](https://loco.rs) (Rails-style, built on Axum) is the best home for
this library. Loco controllers are Axum handlers, so `Ui`, `Page`, `Redirect` and `Saved<T>`
should already work there; this milestone makes it a first-class fit instead of an accident.
Loco's default views are Tera templates and its scaffolds generate them, so the gap is
wiring, generators and docs, not components. Check every Loco API named below against the
loco-rs source (fetch it into `~/.cargo/registry` with a scratch crate) before relying on it.
- [x] Decision: commit to Loco as the primary target? Owner only; if no, skip this milestone.
  Answered 2026-09-24: yes, as a `loco` feature on `loco-ui` (not a separate crate), and
  the crate keeps its name (M26).
- [x] `loco` feature: an `Initializer` whose `after_routes` mounts
  `/lui/enhance.js` and the `/lui/caps` beacon route, so an app adds one line to
  `app.rs::initializers`.
  Done: `loco-ui/src/loco.rs`, feature `loco` (loco-rs 1.2, `default-features = false`,
  plus `async-trait`, which Loco's trait uses). `Box::new(loco_ui::loco::Initializer)`;
  `after_routes` merges `caps::router()` and `enhance::router()` and adds the `slim` layer
  (not `csp`: the app owns its policy). `loco::mount(router)` does the same by hand; a test
  checks both routes answer. `verify.sh` and CI run clippy and the tests with `--features loco`.
- [x] Handlers return Loco's `Result<Response>`: `Page`, `Redirect` and `Streamed` convert with
  `?`/`.into_response()`, no wrapper; a doctest shows a Loco controller using `ui: Ui`.
  Done: nothing to add, since Loco's `Error` implements `IntoResponse`: a handler returns
  `Result<Page>`/`Result<Redirect>` and uses `?`, or `Result<Response>` with `.into_response()`.
  The `loco` module doc has a controller (`Routes::new().prefix(..).add(..)`) as a doctest, and
  a test checks `Result<Page>` answers 200 with the page and 404 for `Error::NotFound`.
- [x] Validation errors: Loco models validate with the `validator` crate; a helper maps
  `ValidationErrors` into `Form::errors(..)`/`Input::error(..)` so server errors land on the
  right field.
  Done: `loco_ui::loco::FieldErrors`, `From` both `validator::ValidationErrors` and Loco's
  `ModelValidationErrors`, plus `from_error(&loco_rs::Error)`; `.pairs()` for `Form::errors`,
  `.get(field)` for `Input::error`. First message per field; a rule without `message` gets a
  sentence from its code. Doctest: a `create` controller re-rendering the form. Gotchas noted
  there: Loco's prelude makes `.validate()` ambiguous (call `Validate::validate`), and the
  pairs borrow, so build the form inside `html!`. Before this, the `loco` doctests were not run
  (`cargo test <filter>` skipped them); `verify.sh` and CI now run `--doc` explicitly.
- [x] Data: SeaORM's paginator feeds `paged_table`/`pager` (`?page=` and page size) without
  loading every row; an example query in the docs.
  Done: no new API; the table already exposes `page()`, `per_page()`, `sort()` and `filter()`.
  The `loco` module doc has `notes_table(ui, db)`: filter, sort, `paginate(db, per_page)`,
  `num_items()` and `fetch_page(page - 1)`, then `.paged(total)`. It runs against SeaORM's
  `MockDatabase` (dev-dependency `sea-orm` 2.0, Loco 1.2's version) and asserts the range
  (`11–20 of 42`) and that the SQL carries `LIMIT`/`OFFSET`. The "Load more" pager's query
  (`limit(pager.shown())`) is stated beside it.
- [x] Flash and PRG: `Redirect` + `ui.flash()` work with Loco's cookie setup (the private
  cookie key from `config/*.yaml` if we use signed cookies); no conflict with Loco's
  session or auth middleware.
  Done: our cookies are unsigned (`lui-*`), so no key is needed; Loco 1.2 core has no session
  middleware, and its JWT cookie is app-named. Test `flash_survives_locos_default_middleware`
  builds the router as Loco's boot does (routes, `default_middleware_stack`, `with_state`, our
  `after_routes`) with `tests_cfg::app::get_app_context` (dev-dependency `loco-rs` with
  `testing`), then POST → 303 + flash cookie → GET shows the flash and clears it; the script is
  served too. Documented: `secure_headers` `github` blocks the script on plain http; `owasp`'s
  `Clear-Site-Data: "cookies"` wipes every cookie per response.
- [x] Views: document Maud views beside Loco's Tera default (a `views/` module of functions
  returning `Markup`), and decide whether a Tera function bridge (`{{ lui_button(..) }}`) is
  worth it; default answer: no, Maud only, stated in the docs.
  Done: `docs/loco.md` (started here; the last box finishes it) has "Views: Maud, not Tera":
  a `src/views/notes.rs` of `fn(&Ui, data) -> Markup`, the controller calling it, Tera and
  Maud side by side per controller. Decision: no Tera bridge (a Tera function gets JSON, not
  `ui`'s caps and state; typed builder chains would become unchecked keyword arguments; output
  would need `| safe`). The `loco` module doc has the same views module as a doctest.
- [x] Generator: a scaffold override (`cargo loco generate override` templates, or our own
  template set) that emits Maud views built from `ui.*` for list/show/new/edit, PRG included.
  Done: `loco-ui/loco-templates/scaffold/api/{controller,dto}.t`, copied into an app's
  `.loco-templates/` (Loco reads overrides there by the built-in file names). `controller.t`
  writes an HTML controller (list with `.paged`, show, new, create, edit, update, delete; PRG
  and flash; the form re-rendered with values and `FieldErrors` on bad input); `dto.t` writes
  `src/views/<plural>.rs` instead of a DTO. Posts are parsed field by field with
  `loco::Submitted`. Proven by generating `examples/loco-app`'s notes with the real
  `cargo loco generate scaffold`, which found four template bugs (FINDINGS, M27).
- [x] `examples/loco-app`: a minimal Loco app (one model, CRUD, sign-in) with script off;
  Blitz renders its routes and the only-one-script test covers them.
  Done: trimmed from `loco new` (sqlite, no assets, blocking workers): users + sign up/in/out
  as forms, the JWT in an `HttpOnly` `auth` cookie (`auth.jwt.location`), the loco-ui
  initializer line, and `notes` from `cargo loco generate scaffold note title:string!
  body:text done:bool! due:date`. A workspace member, so `cargo test` and verify.sh run it.
  `tests/pages.rs` boots Loco's router: signed out → 401; sign-up/sign-in errors on the form;
  create with a missing title re-renders with values; then each GET page has exactly one
  script, the enhancement one, and renders in Blitz to `tests/shots/loco-*.png`; update and
  delete redirect. Blitz leaves the textarea and date value blank (FINDINGS; the HTML is
  asserted).
- [x] `docs/loco.md` and a README section: install, the initializer line, a controller, a
  form with validation, the generator.
  Done: `docs/loco.md` now covers install (git dependency plus `maud`), the initializer line,
  a controller (`Result<Page>`/`Result<Redirect>`, PRG, flash), a form with validation
  (`FieldErrors` and `Submitted`), sign-in without script (JWT from a cookie), the generator
  (what it writes and needs, field kinds) and the Loco settings that affect pages
  (`secure_headers` presets, `csp`). README has a "Use with Loco" section linking both the
  doc and `examples/loco-app`.

## M28 · Props written like attributes, and props you can list
The owner read maud-ui's getting-started page (2026-09-24) and liked two things: the props
(named values at the call site) and how fast its swaps feel. They want a component call to
read like the Maud element around it, `name=value` the way `a href="/tabs?tab.demo=2"` does,
instead of `.name(value)`, keeping the dot form where it reads better (a route building a
component in a loop, a value from a condition). And they want the props introspectable.
M26 kept builders over `Props { .., ..Default::default() }` for good reasons (components read
`ui`, list adders with last-item modifiers, short calls stay short); nothing here undoes that:
the attribute form must compile down to the same builder, so both forms stay one code path.
- [x] Decision (owner): which attribute form. Options shown with the `/dialog` call and a
  tabs strip built from data: A, a `lui!` macro (Maud plus components); B, public fields plus
  struct update (`Tabs { vertical: true, ..ui.tabs("demo") }`); C, builders only.
  Answered 2026-09-24: A. B was dropped because it reads longer than the dot form, freezes
  every internal field as public API (`#[non_exhaustive]` forbids struct update outside the
  crate) and brings back nested item structs. CLAUDE.md's "no macros beyond `html!`" becomes
  "no macros beyond `html!` and `lui!`".
- [x] One source of truth per component: a `PROPS: &[Prop]` const on each builder
  (`Prop { name, kind: Text | Switch | Condition | Number | Item | Modifier, default, attr,
  doc }`), where `attr` is the HTML attribute or element it maps to. The existing
  "**Setters.**" paragraph is generated from it (or checked against it by
  `every_setter_is_listed_on_its_builder`), so docs, macro and introspection cannot drift.
  Done: `props.rs` (`Prop`, `PropKind`: value, number, switch, condition, item, modifier)
  and a `PROPS` const on all 43 builders that have a **Setters.** paragraph (263 setters),
  generated once from the source and then kept by hand. `args` is the Rust argument list;
  `attr` is filled where the setter writes that attribute; `default` is `off` for switches
  and conditions and the constructor's literal where there is one. The paragraph is not
  generated (rustdoc cannot read a const); it is checked: `every_setter_is_listed_on_its_builder`
  (paragraph ↔ source) and `every_setter_is_in_props` (PROPS ↔ source: names, arguments,
  and a switch has none, a condition one `bool`). Both share one source reader. Bulk adders
  (`.options(iter)`, `.rows(iter)`) are values; items add exactly one thing.
- [x] `loco-ui-macros` (proc macro, re-exported as `loco_ui::lui` and in the prelude)
  with `lui!`, a superset of `html!` that expands to the builder chain, so the dot form and
  the attribute form are one code path. Target shape:
  ```rust
  lui! {
      Dialog("Delete account") id="confirm" title="Delete account?" small danger
          confirm=("Delete account", "/dialog/delete") cancel="Keep it" {
          p { "This cannot be undone." }
          Input("reason", "Tell us why (optional)") placeholder="Moving on";
      }
      Tabs("projects") vertical[narrow] {
          @for p in &projects {
              tab (p.name) badge=(p.open_issues) { p { (p.summary) } }
          }
          lazy "Archive" { (archive(&ui)) }
      }
  }
  ```
  Rules, each with a test:
  - A capitalized name is a component: `Tabs(..)` is `ui.tabs(..)` (snake case of the name),
    `(..)` holds the required arguments, text first as today. A user's own component (M24)
    works the same once it has an `impl Ui` method (an extension trait outside the crate).
  - `x="v"` or `x=(expr)` is `.x(v)`; `x=(a, b)` is `.x(a, b)`; a bare `x` is `.x()`;
    `x[cond]` calls `.x()` only when `cond` holds and `x=[option]` calls `.x(v)` only for
    `Some(v)` (Maud's toggle and optional syntax, checked in maud_macros 0.27).
  - Lowercase names inside a component's block are its item adders (`tab`, `item`, `link`,
    `column`, `text`..): `tab "Use" badge=3 { .. }` is `.tab("Use", html!{..}).badge(3)`, so an
    item's attributes are the last-item modifiers. An adder that takes a closure (`lazy`) gets
    `|| html!{..}` when the caller writes `lazy "Why" || { .. }`. A proc macro cannot read
    `PROPS` (a const in another crate), so the closure is marked in the markup, not looked up.
  - `@for`, `@if`, `@match` and `@let` among items expand to a fold over the builder, so items
    built from data stay inline. Plain Maud elsewhere passes through untouched, so `lui!` can
    replace `html!` in any route; a component's block that is not items becomes `.body(..)`.
  - `ui` is taken from scope by that name (documented); `lui!(ctx => ..)` names another.
  - A misspelled prop is rustc's own method-not-found error ("did you mean `vertical`?"), so
    every expanded call must keep the span of the attribute it came from; a `trybuild` test
    pins the error text for a typo, a missing required argument and a wrong value type.
  - Record in FINDINGS whatever rust-analyzer does and does not offer inside `lui!`.
  Done: `loco-ui-macros` (proc-macro2 + quote, no syn), `pub use` as `loco_ui::lui` and in
  the prelude; rules and syntax in its crate doc (with a doctest). Two rules were settled while
  building it: an item is a lowercase name followed by an argument (a literal or `(..)`; an
  item with none is `name()`), since in Maud an element name is never followed by one, which
  keeps `select`, `link`, `option`, `summary`, `time`, `header` usable both as items and as
  elements; and a component with no required arguments leaves out the parentheses
  (`Card title="Plan" { .. }`). `loco-ui/tests/lui.rs` has one test per rule, each
  comparing against the dot form: component and `;`, body block, every attribute form,
  items (arguments, modifiers, `||` closures, `()`, nested items continuing the chain in a
  kanban), `@for`/`@if`/`@else if`/`@else`/`@match`/`@let` among items, plain Maud passing
  through (a markup `@match` on `Some(..)`, brace attribute values), and `ctx =>`. `trybuild`
  pins four errors: typo, missing argument, wrong type, markup mixed into items (our own
  message). rust-analyzer: FINDINGS, M28 (hover, go to definition and setter completion work).
- [x] Both forms tested: a test renders each component once in the dot form and once in
  `lui!` and asserts identical HTML; doctests show both in every component header
  (common call first, per convention 5).
  Done: all 41 component headers (every spec module with a `ui.<name>(..)`; `enhance`,
  `layout`, `caps`, `state` and `paged_table` have none) end their main doctest with a
  `// The same in \`lui!\`:` twin and `assert_eq!` on the HTML, so the doctests are the
  per-component test; `every_component_header_shows_the_lui_form` fails when one is
  missing. Every component could be written in `lui!` (the tooltip's trigger is a nested
  `lui!`, the stream twin is `Slot(..)`). Switches sit on the component as attributes; any
  setter may also stand in an items block (`search "/shop";`) when a chain's order matters,
  now stated in the macro doc.
- [x] Introspection: `loco_ui::props()` lists every component with its `PROPS`; the M5 spec
  JSON (`spec/components.json`, `cargo run -p demo -- spec`) gains a `props` array per
  component; each demo component page shows a props table (name, kind, default, HTML
  attribute, doc) under its code snippet, generated from the same list; `Debug` on a builder
  prints only the props set away from their default.
  Done: `loco_ui::props()` returns every builder as a `props::Component` (module, builder,
  constructors as written, `PROPS`), with `.lui()` for its `lui!` name; 44 builders, a
  test (`props_lists_every_builder_and_constructor`) fails on a builder or `ui.<name>(..)`
  missing from it. `spec/components.json` gains `builders` per component (builder, lui name,
  calls, and `props` with name, kind, args, default, attr, doc). Each demo page ends with a
  "Props" section: one `<details>` per builder its snippet calls (found from `ui.<name>(`,
  `Type::new(` or the `lui!` name), the first open, a table with the doc's backticks as
  `<code>`; test `component_pages_show_their_props`. Widening the setter scan to
  `pub const fn` found two setters no list had (`Row::key`, `SelectOption::icon`).
  Not done, by choice: a `Debug` that prints only changed props. The derived `Debug` prints
  every field; a filtered one needs a hand-written impl per builder (44) that would drift
  from `PROPS`, the opposite of this milestone's one source of truth.
- [x] Swap speed: measure the enhancement script's click-to-paint on a table sort, a tab and
  a pager (Firefox via `scripts/browser-check.mjs` timings) beside an htmx swap of the same
  fragment; if ours is slower, close the gap in `enhance.rs` (answer with only the swap
  root's fragment when asked, prefetch on `pointerdown`, skip the view transition on fast
  answers). Numbers go in README "What a page weighs"; the no-script path stays as it is.
  Done: `scripts/bench-swap.mjs` (geckodriver, the release demo, a proxy serving the same
  pages with htmx 2.0.11 in place of the script and the same answers). The script was behind
  on table sort (51 vs 41 ms p50) and pager (47 vs 22); the view transition around every swap
  cost about a frame. Now an answer within 150 ms swaps directly, slower ones morph, and a
  root with `data-lui-morph` (the kanban) always morphs: table 35 vs 41, tab 13 vs 14, pager
  32 vs 22 (p90 35 vs 35; traced to frame alignment, not script work, FINDINGS M28). Fragment
  answers (`slim`) and prefetch already existed; nothing to add there. Numbers in README
  "How fast an update lands".
- [x] Demo: routes move to `lui!` where it reads better (every snippet between the `// code:`
  markers, so each component page teaches the attribute form); the dot form stays where a route
  keeps a builder in a variable. Count route lines before and after.
  Done: 41 `lui!` blocks across the route files; every component page whose snippet can be
  written that way now teaches it, including the kanban and upload (items from data through
  `@for`, `@if let` and `x=[option]` instead of a mutable builder in a loop) and the
  "write your own" page (`PricingCard(..)` from its own `impl Ui` method). Kept in the dot
  form, where a route keeps or reads a builder: `/wizard`, `/table`, the notes table,
  `/counter`, `/palette` (`.exact()`), `/stream`, the toast and settings redirects. Every
  page's HTML is byte-identical outside the snippet box (48 exported pages diffed) except
  `/settings`, which gains a Form props table now that `Form(..)` is inside its snippet.
  Route lines 1791 → 1798: nested blocks take a line more, loops a few less.
- [x] Docs: CLAUDE.md convention 5 and the macro rule (`lui!`, `PROPS`), README first example,
  `docs/ergonomics.md` before/after, `docs/comparison.md` (maud-ui's `Props` vs ours, now
  with names at the call site and a listable prop table, which maud-ui's docs say it lacks).
  Done: CLAUDE.md convention 5 (the macro rule is now `html!` and `lui!`; `PROPS`,
  `props::COMPONENTS`, the header's `lui!` twin and the tests that enforce them; demo
  snippets in `lui!` unless a builder is kept), the workspace layout lists
  `loco-ui-macros`; README's first example in `lui!` (tested as `the_readme_example`) with
  a paragraph on the attribute rules and `props()`; `docs/ergonomics.md` "M28" with the
  kanban and dialog before/after and the counts; `docs/comparison.md` gains the `lui!`
  button, names at the call site, and the listable props.


## M29 · Beat maud-ui on Loco
Evaluation of 2026-09-25 against maud-ui 0.20.3 (84 components, 31 blocks, a `/theme`
customiser; htmx plus an 89 KB script; its own README lists 21 components that need JS, among
them Dialog, Popover, Menu, Select, Sheet and Data Table; no Loco integration, no CLI; 6
stars) and the kits worth copying: shadcn/ui and templUI (a CLI and registry), Phoenix 1.8
(`phx.gen.auth`, form-bound `<.input>`), Rails ViewComponent + Lookbook + Primer (previews,
component status, accessibility linting), GOV.UK Frontend (error summary, translatable
strings), Basecoat and Oat (CSS-first, a list of what needs JS). The opening: according to
Loco's generators reference, Loco 1.0 removed the `--html`/`--htmx` scaffolds in favour of a
JSON API plus a React SPA, so server-rendered CRUD on Loco has no first-party answer. loco-ui
already beats maud-ui on the guarantee (0 KB required, proven by Blitz, strict CSP) and on Loco
wiring; it falls behind on install, ready-made pages, i18n and checked accessibility. Check
every Loco API named below against loco-rs 1.2 before relying on it, as M27 did.

### Loco adoption
- [ ] Publish: the M13 box. Everything below matters less while install is a git dependency.
  Owner only.
- [x] One-command install: a subcommand (`cargo run -p loco-ui --features loco -- install`,
  or a small `cargo-lui` binary) that writes `.loco-templates/`, adds the initializer line
  to `app.rs`, a `views/layout.rs` and the `views/mod.rs` entries; idempotent, and a test
  runs it on a fresh `loco new` copy and then `cargo check`s the result.
  Done: `cargo-lui`, a standard-library-only binary in `loco-ui` (`src/bin/cargo-lui.rs`;
  `cargo install --git .. loco-ui --bin cargo-lui`, then `cargo lui install [APP_DIR]`, or
  `cargo run -p loco-ui -- install APP_DIR` from a checkout). It writes the two scaffold
  templates (embedded with `include_str!`), `src/views/layout.rs` (`page(ui, title, body)`,
  a header then the view) plus `pub mod layout;`, puts the initializer first in
  `fn initializers`' `vec![..]`, and adds `loco-ui` (git, or `--dep-path`) and `maud` under
  `[dependencies]`. A second run prints `unchanged` for all six; an edited file is `kept`
  unless `--force`. `tests/install.rs` runs it on `tests/fresh-loco-app` (`loco new` 1.2.0,
  sqlite, blocking, no assets; trimmed to `Cargo.toml`, `Cargo.lock`, `src/`, `migration/`,
  excluded from the package), and an `--ignored` test `cargo check`s the result (about a
  minute cold), run by verify.sh and CI.
- [x] Auth generator (Phoenix's `phx.gen.auth` as the model): templates for sign-in, sign-up,
  forgot and reset password, email verification and magic link, as no-script forms with
  the JWT in an `HttpOnly` cookie (the `examples/loco-app` pattern); `examples/loco-app` is
  regenerated from them and its Blitz tests cover every page.
  Done: `cargo lui auth` (after `install`) writes `loco-templates/auth/{controller,views}.rs`
  as `src/controllers/account.rs` and `src/views/account.rs` on the starter's `users` model
  and `AuthMailer`, registers them, rewrites the starter's six mail links (JSON API and SPA
  paths) to `/verify/<token>`, `/reset/<token>`, `/magic-link/<token>`, and adds the cookie
  `location` under `auth.jwt` in each `config/*.yaml`; idempotent. Plain Rust files, not
  Tera: nothing in them varies per app. `examples/loco-app` now has the starter's full users
  model and mailers and runs these files byte for byte (a test fails on drift); its
  `tests/pages.rs` walks sign-up, verify, sign-in, forgot/reset (old password refused after),
  magic link (spent after one use), and renders sign-in, sign-up, forgot, reset, magic link
  and the expired-link page in Blitz. The fresh-app test runs `auth` too and `cargo check`s
  it. FINDINGS M29: the mail links, and `include_dir!` paths inside a workspace.
- [x] Validation that re-renders: an extractor (`loco::Valid<T>`) that gives the handler
  `Ok(T)` or `Err((FieldErrors, values))` instead of Loco's `FormValidate` error response,
  so a create/update handler is one `match`: render the form again or redirect. The scaffold
  uses it.
  Done: `loco::Valid<T>(pub Result<T, Invalid>)`, `Invalid { errors: FieldErrors, values }`
  (a named struct rather than a tuple, so the handler reads `bad.values`/`bad.errors`), for
  any `T: Deserialize + Validate`. Values are trimmed and empty ones dropped (`None` for an
  `Option`, "This field is required." otherwise); `serde_path_to_error` (new optional dep of
  `loco`) names the field a parse error is about ("Check this field."); a bad field is stood
  in for so every field is reported at once, then the `#[validate]` rules run.
  `loco::checkbox` reads `on`/`true` for `#[serde(deserialize_with)]`. `Valid::check(pairs)`
  is the same without a request. The scaffold's `Params` derives `Deserialize, Validate` and
  create/update match on `Valid<Params>`; `examples/loco-app`'s notes were regenerated with
  the real `cargo loco generate scaffold` (views unchanged). Loco-only, so no demo route.
- [x] Error summary (GOV.UK): `ui.error_summary(&errors)`, a `role="alert"` list at the top
  of the form linking to each field in error by id, focused on load via `autofocus` on its
  heading link; `Form` shows it when it has errors. Demo on `/app/signin` and `/form`.
  Done: `error_summary.rs` (`ErrorSummary`, setter `.title(..)`, default "There is a
  problem"; PROPS, spec, props entry). Links go to `#f-<name>`; inside a `Form` they use the
  field's own id and read "Label: message", and messages naming no field are listed unlinked
  (the heading then takes `tabindex="-1" autofocus` itself). `Form` renders it first when any
  message is set. The enhancement script now focuses the `autofocus` element of swapped-in
  markup (else the old focus, as before), so the enhanced submit behaves like the load.
  Demo: `/form?errors=1` (new PATHS entry, Blitz shot) shows the server's answer on GET and
  replaces the ad-hoc error line; `/app/signin` shows it on a refused post (demo test). The
  browser check asserts the focus both after an in-place swap and on a full load.
- [x] A pager fed straight from Loco: `.paged_from(&PagerMeta)` (or `From<&PageResponse<T>>`)
  instead of the hand-written `num_items` + `fetch_page`; a doctest against `MockDatabase`.
  Done: `Table::paged_from(&PagerMeta)` (feature `loco`; in PROPS), which is
  `.paged(meta.total_items)`. The handler asks Loco for the table's page with
  `query::fetch_page(db, select, &PaginationQuery { page: table.page(), page_size:
  table.per_page() })`; the `loco` module doctest does that with a sort and a filter against
  `MockDatabase` (count, then `LIMIT`/`OFFSET`). `PagerMeta` lives behind Loco's `with-db`, so
  the `loco` feature now turns `with-db` on (any app with a database has it). The scaffold's
  list uses `fetch_page` and `views::<plural>::list(ui, rows, &PagerMeta)`; the example's
  notes were regenerated with the real generator. Loco-only, so no demo route.
- [x] Scaffold field kinds: `references` fields become a select (or combobox past N rows) of
  the parent model, `bool`, `date`, `datetime`, `decimal` and enum columns get their own
  controls; check the templates against Loco 1.x's adaptive scaffold (`--no-auth`, the
  `frontend/` detection) and note the result in FINDINGS.
  Done: a reference (`<x>_id: i64`; Loco's context does not mark references, FINDINGS) is a
  select of `<x>`'s plural table, rows labelled by the new `loco::label` (name, title, label
  or email via `Serialize`), loaded by a generated `refs()` into `views::<plural>::Refs`;
  `date_time`/`tstz` are `datetime-local` and `time` is `type=time`, read by the new
  `loco::local` (browsers post no seconds; a `tstz` without offset is UTC); decimals and floats
  a number `pattern`; ints `type=number`; `bool`, `date` and enums as before. Library:
  `Form::datetime`/`Input::datetime` and `Form::select` taking `&str` or `(value, label)`
  (`input::Choice`), both in the `/form` demo. Default: a select, not a combobox past N rows
  (a combobox needs a search route per parent; the generated comment says when to swap).
  `examples/loco-app` scaffolds `task` with one field of each kind; its test posts bad and good
  values and checks the edit form round-trips them. `--no-auth` and the `frontend/` detection
  (React pages still emitted beside ours) are in FINDINGS.

### Library quality
- [x] i18n: `lang="en"` is hard-coded in `layout.rs` and built-in strings ("Next", "Close",
  "Load more", "Search", "Loading", "Page", "Cancel") are literals across components. Add
  `ui.lang()` (from `Accept-Language` or a cookie, set on `<html lang>`) and one string table
  (`Strings`, English default, overridable per app); a test fails on an English literal a
  component renders outside the table. Bridge to Loco's `fluent-templates` under `loco`.
  Done: `i18n.rs`: `Text` (113 keys, `Text::key()` = `lui-<kebab>`), `Strings` (English
  `const`, `Strings::new("es").with(Text::Next, "Siguiente")` in a `static`, `fill` with `{}`
  and `{0}` placeholders so a language can reorder), `i18n::languages(&[..])` to register,
  `Redirect::lang(tag)` for the `lui-lang` cookie. `Ui` carries `strings`; the cookie wins,
  then `Accept-Language` by `q` and primary subtag (the Axum extractor reads the header;
  `Ui::accept_language` for other servers); `ui.lang()`, `ui.text(..)`, `ui.fill(..)`;
  `<html lang>` in pages and streamed pages. 25 components moved every word they write to
  the table (month and weekday names and date order included). Test
  `components_write_no_english_outside_the_string_table` reads component code for text-like
  literals. Fluent: `Strings::from_lookup(lang, |key| LOCALES.try_lookup(..))`, no new
  dependency; documented in the `loco` module (doctest) and `docs/loco.md`. Demo: a full
  Spanish table (`demo/src/spanish.rs`, a test that nothing is left in English) and an
  English/Español switch beside the theme toggle; demo test for cookie, header and switch.
  Not covered: the form messages of `Valid`/`Submitted` (English, set your own in
  `#[validate(message)]`) and the site header line in `layout::header`.
- [x] Automated accessibility: run axe-core inside `scripts/browser-check.mjs` over every
  `PATHS` route, both caps variants, light and dark (axe is injected by the test driver only,
  never served, so the one-script rule holds); CI fails on any violation of serious or
  critical impact. Each component header gains an **Accessibility** line (roles, keyboard,
  what was checked), and README's badge line adds "axe-clean".
  Done: axe-core 4.12.1 as a test-only npm package (`scripts/package.json`, lock committed,
  `node_modules` ignored), injected by the driver into 34 routes × modern/old caps ×
  light/dark = 136 page views; serious or critical fails. CI gains a "Browser check and
  accessibility" step (the runner image has Firefox and geckodriver); verify.sh installs axe
  when missing. Fixed: keyboard access to scrollable code and props tables (demo), contrast
  of ok/warn badges, highlighter colours, palette `kbd` and unselected tabs (tones mixed with
  `--lui-fg`), `aria-pressed` on a calendar link (now hidden text), the combobox's
  listbox-of-links (now a plain list). Two patterns let through with reasons in FINDINGS:
  links filling a `<summary>` (the no-script tabs/accordion design) and the customisable
  select's button that only a forced cookie shows Firefox. 44 component headers have an
  **Accessibility** line (test `every_component_header_has_an_accessibility_line`); README
  says axe-clean.
- [x] Blocks, no script: app shell with sidebar, auth pages, settings page, record show/edit
  page, dashboard of stats, and error pages (404, 500) in Maud usable as Loco's fallback. One
  file each under `loco-ui/src/blocks/`, one demo route each, one Blitz test each.
  Done: `loco-ui/src/blocks/{app_shell,auth_page,settings_page,record_page,dashboard_page,
  error_page}.rs`, each a builder from `ui` (`ui.app_shell(name)`, `ui.error_page(status)`, …)
  with PROPS, a `props::COMPONENTS` entry, an Accessibility line and a doctest ending in its
  `lui!` twin (the source-reading tests now include `src/blocks/`); their words are in the
  i18n table (Delete, Sign out, the 404/500 texts; Spanish too). Composed from the drawer,
  avatar, button, form, stat and badge; 3.8 KB of CSS. `blocks::not_found` is a
  `Router::fallback` handler (and `ErrorPage` is `IntoResponse` with its status): the demo
  router and `examples/loco-app` (`App::before_routes`, Loco's dev fallback switched off; test)
  use it. Demo: a "Blocks" layer with `/blocks/{shell,auth,settings,record,dashboard,error}`
  (PATHS, SOURCES, props tables); Blitz test `blocks_lay_out` checks each block's parts and
  the 404 page (`Page::render_expecting` for non-200); axe covers the new routes. The DSD
  stream page budget went from 128 to 136 KB (FINDINGS).
- [x] Server-rendered SVG charts (bar, line, sparkline): `ui.chart(..)` writes `<svg>` with
  `<title>`/`<desc>` and a visually hidden data table as the accessible fallback; theme
  colours from `--lui-*` tokens. No JS, which maud-ui and most kits need here.
  Done: `chart.rs`: `ui.chart(title)` with `.point(label, value)` items, `.bar()` (default),
  `.line()`, `.sparkline()`, `.description(..)` (`<desc>`), `.unit(..)`, `.id(..)`; PROPS,
  spec, `lui!` twin. A `<figure>` with `role="img"` SVG named by `<title>`/`<desc>`, a `<title>`
  per bar and point (native tooltip), a y axis in 1/2/5 steps (negative values hang below
  zero), and the data in a visually hidden table (a sparkline, being phrasing content, is a
  `<span>` with the values as hidden text). Fills and strokes are `--lui-*` tokens in CSS,
  with presentation attributes for renderers without the page's CSS (Blitz draws inline SVG
  through usvg: FINDINGS). Demo `/chart` (bars, a line, a sparkline in a sentence), checked
  in Firefox light and dark (`look.sh` now shoots it and the blocks) and by axe. The
  stylesheet budget went from 64 to 68 KB (blocks and chart).
- [x] Missing shadcn components: sidebar, navigation menu, description list, toggle group,
  context menu (a popover on a secondary button), input OTP (one field with
  `autocomplete="one-time-code"` and `inputmode="numeric"`). Each by the component
  conventions, with PROPS, a `lui!` twin and a demo route.
  Done: `sidebar.rs` (groups, icon and badge modifiers, current path marked), `nav_menu.rs`
  (links and `.panel(..)`s reusing the popover menu), `description_list.rs` (`.stacked()`),
  `toggle_group.rs` (radios or `.multi()` checkboxes drawn as segments, pressed from the
  query), `context_menu.rs` (a thing plus the popover menu on a "More actions" button),
  `input_otp.rs` (one labelled field, `autocomplete="one-time-code"`, `inputmode="numeric"`,
  `pattern`, cells drawn by the background). Each has PROPS, a `props::COMPONENTS` entry, a
  spec entry, an Accessibility line and a doctest ending in its `lui!` twin; two new icons
  (Bold, Italic); two new texts (More actions, "{} digits", Spanish too). Demo routes
  `/sidebar`, `/nav-menu`, `/description-list`, `/toggle-group`, `/otp`, `/context-menu`
  (PATHS, index, `look.sh`); axe clean on all 47 routes. Budgets raised with the stylesheet
  (69.6 KB now): 72 KB stylesheet, 104 KB page, 152 KB shadow-DOM stream; README "What a page
  weighs" re-measured.

### Developer experience
- [x] Playground per component (Lookbook, phoenix_storybook): each demo page's props table
  becomes a GET form, generated from `PROPS`, that re-renders the component with the chosen
  props and shows the matching `lui!` snippet. Works with script off; the enhancement script
  swaps it in place.
  Done: `demo/src/playground.rs`. Rust cannot call a setter by name, so each playable builder
  has an entry: its `lui!` call, the props it offers (switch, condition, text, number) and a
  function mapping them to setters; 17 builders (Button, Badge, Alert, Card, Avatar, Progress,
  Meter, Separator, Skeleton, EmptyState, Stat, Chart, DescriptionList, InputOtp, Input,
  Range, ErrorPage). Their props table gains a "Try" column (checkboxes and text boxes built
  from the primitives) inside a GET form (`pg.<Builder>.<prop>`), then the component as chosen
  and its `lui!` line, all in one swap root; the tried builder's `<details>` opens. Builders
  fed lists, rows or markup (tabs, table, kanban, …) keep the read-only table: the default,
  since a form cannot build their data. Tests: every entry's props exist in its `PROPS`, a
  tried page renders the choice and its line; the browser check ticks a prop and sees the
  swap in place; axe clean. Blitz counts scoped to the demo stage now that previews repeat
  components.
- [x] Theme builder without script: `/theme` with colour inputs and a radius, posted to the
  server, a live preview of a few components, and a `theme.css` download of the `--lui-*`
  overrides (maud-ui's `/theme`, without its script). `docs/theming.md` links it.
  Done: `demo/src/routes/theme.rs`: `GET /theme` (the theme toggle keeps `POST /theme`), a
  form of colour inputs (`ui.color`) for nine roles (bg, fg, muted, line, primary,
  on-primary, accent, on-accent, danger) in light and dark plus a radius (`ui.range`); the
  roles it does not edit are derived (card and popover from bg, input from line, ring from
  muted, secondary from accent). Only `#rrggbb` values are accepted (a test). The preview is a
  card with an input, buttons, a badge and an alert under each scheme's values as inline
  custom properties; `/theme.css` downloads the overrides in `Tokens::css`'s cascade order
  (test). Default: a GET form rather than a post, so a theme is a shareable link and no state
  is kept. Index entry, PATHS, `docs/theming.md` links it; axe clean.
- [x] Component status (Primer): `stable | beta` in `props::COMPONENTS`, shown on the demo
  page, the index and the spec JSON; README says what each status promises.
  Done: `props::Status { Stable, Beta }` and a `status` on every `props::Component`; beta are
  the 14 builders new in M29 (the six blocks, Chart, ErrorSummary, Sidebar, NavMenu,
  DescriptionList, ToggleGroup, ContextMenu, InputOtp), the rest stable. The spec JSON's
  builders carry `"status"`; each builder's props summary shows its status, a page with a
  beta builder has a "beta" badge by its title and in the index (demo test). README
  "Component status" says what each promises (stable: breaking changes only in a breaking
  release with a note; beta: may change in any release until one release passes unchanged).
- [ ] Copy-paste mode (shadcn, templUI), last: `cargo lui add <component>` vendors a
  component's file into the app, from the spec JSON as the registry. Components read `ui`, so
  first decide whether a vendored file keeps `use loco_ui::…` for `Ui` or copies it; ask
  the owner before starting.
  Blocked on the owner: the question and a suggested answer (keep `use loco_ui::…`) are in
  BLOCKED.md; not started.

### Positioning
- [x] README and `docs/loco.md` lead with "server-rendered scaffolds for Loco 1.x, zero
  JavaScript"; `docs/comparison.md` gains the maud-ui numbers above and a Loco column.
  Done: README and `docs/loco.md` open with "Server-rendered scaffolds for Loco 1.x, zero
  JavaScript required" and the three commands (`cargo lui install`, `cargo loco generate
  scaffold`, `cargo lui auth`). "Required" keeps the owner's "HTML first, script optional"
  (the optional script is still there). `docs/comparison.md`'s table gains a "Loco 1.x on its
  own" column (React SPA scaffolds, no HTML ones since 1.0), rows "On Loco" and "Languages",
  maud-ui's 21 components that need JS and "no integration", and current numbers (69.6 KB CSS,
  58 builders, axe-clean); "When to pick which" gains loco-ui on Loco.
- [ ] Ask to be linked from Loco's docs or discussions once the crate is published. Outward
  action: owner only (BLOCKED.md).
  Blocked on the owner: in BLOCKED.md with a suggested message; waits for the publish.

## M30 · The Linear / Magic UI look
The owner found the shadcn look (M20) simple and useful, but wanted something more modern and
attractive. The direction was picked through questions on 2026-09-25:
- Linear / Magic UI as the reference.
- Layered shadows rather than hairlines.
- Radix-style 12-step colour scales with gradient accents.
- System fonts kept (no web font).
- Showpiece motion, CSS only.
- Every showpiece effect is an opt-in setter, so the defaults stay restrained.

Geist (flat hairlines, calm motion) was considered and dropped because it clashes with the
shadows and motion chosen. The no-script rules are unchanged:
- Every effect is CSS.
- Every effect honours `prefers-reduced-motion`.
- Every page still works in Blitz and old Chrome 109.
- Paid kits are still not copied. Magic UI (MIT) and Radix Colors (MIT) may be read for
  technique.

- [ ] Tokens: replace the shadcn zinc palette with 12-step gray and accent scales in
  `layout.rs`, one set for light and one for dark. Each step has one job:
  - 1–2 backgrounds
  - 3–5 component surfaces (normal, hover, pressed)
  - 6–8 borders and the focus ring
  - 9–10 solid fills
  - 11–12 text

  The existing `Palette` roles (`card`, `popover`, `accent`, `primary`, `input`, `ring`, …)
  become aliases onto the steps, so components and `--lui-*` names do not change. Colours are
  written in oklch, and every text/background pair must still clear 4.5:1 (the existing
  contrast test).
- [ ] Depth and gradient tokens:
  - `--lui-shadow-{xs,sm,md,lg}` become stacked shadows (a tight contact shadow plus a soft
    ambient one).
  - New `--lui-highlight` is an inset top highlight for raised surfaces.
  - New `--lui-gradient-primary` and `--lui-gradient-ring` are oklch gradients built from the
    accent scale.
  - Dark mode gets its own shadows and highlight (lighter edge, deeper ambient), not the light
    values reused.
- [ ] Theme builder (`/theme`) and `docs/theming.md`: pick one accent and one gray, and derive
  all 12 steps from them, rather than editing nine separate colours. The preview shows the
  shadows and the gradient in both schemes, and `theme.css` still downloads as a link.
- [ ] Surfaces:
  - Buttons, inputs, cards, stat tiles, dialogs, popovers, menus, sheets and toasts use the
    new shadows and highlight.
  - Primary buttons and the focus ring use the gradient.
  - Ghost and secondary buttons stay flat.
  - `scripts/look.sh` shots are compared against linear.app and magicui.design, not the
    shadcn docs.
- [ ] Motion, on by default but restrained:
  - Dialog, popover, sheet, drawer, menu and toast animate in and out with `@starting-style`
    and `transition-behavior: allow-discrete`.
  - Springs use `linear()` easing, from a `--lui-ease-spring` token.
  - Tabs and page navigation use view transitions, gated on `Cap::ViewTransitions`.
  - Under `prefers-reduced-motion: reduce`, every duration is 0.
  - A Blitz test checks that the final layout is unchanged.
- [ ] Showpiece setters, all opt-in. Each one gets an entry in `PROPS`, a playground control, a
  demo snippet and a `lui!` doctest:
  - `.shimmer()`: a light sweep across buttons and badges.
  - `.beam()`: a border beam on cards, drawn with a conic gradient and `@property` angle.
  - `.glow()`: a spotlight on cards, CSS only. The light sits at a fixed spot and brightens
    and grows on hover. It does not follow the pointer: the owner chose this over adding
    pointer tracking to `enhance.rs` and over a CSS grid of hover cells, which puts 64 spans
    on every card. Both alternatives were asked and answered on 2026-09-25.
  - `.gradient_border()`: a gradient border on cards and inputs.
  - `ui.marquee(..)`: a new component, a CSS-only looping row with pause on hover and focus.
  - `.reveal()`: fade and rise as the element scrolls into view, using scroll-driven
    `animation-timeline: view()`; browsers without it show the element at rest.

  Each effect has a `@supports` fallback that shows the element at rest, and Blitz shots prove
  it.
- [ ] Budget: measure the stylesheet size before and after, keep the growth under 15 KB
  gzipped, and put the figure in README. Also run `cargo bench -p loco-ui` for the stylesheet.
- [ ] README feature matrix, findings and `docs/comparison.md` say "Linear / Magic UI look"
  instead of "shadcn look". FINDINGS gets the Blitz gaps for `@starting-style`, `@property`
  and `animation-timeline`, each with an issue link.

## M31 · The index shows the components
The owner asked on 2026-09-25 for the main page to show the components themselves, not cards
with their names, and for the grouping and navigation to move into a sidebar.

- [ ] Index as a gallery: `/` renders each component live, with the same call its own page
  shows in a small stage, under its name and linked to that page. Components that need a
  full page (dialogs, drawers, toasts, sheets) show their trigger. The page stays one
  request and readable with `curl`; a Blitz test checks that every group has a live
  component on the index.
- [ ] Sidebar: the groups (layers and the index groups in `site.rs`) become a sidebar on the
  index and on every component page, listing each component under its group with the
  current one marked (`aria-current="page"`). It is a plain `<nav>` with links; on narrow
  screens it collapses into a `<details>` above the content. No script, and a Blitz shot at
  1280 and 420 wide.
