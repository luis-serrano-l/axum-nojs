# Findings

What the HTML/CSS platform can and cannot do for a server-rendered app with no script, as
learned building and testing this crate. Three lists first; the milestone notes with the
reasons follow. The per-component feature matrix (with per-browser versions) is generated
from `axum_nojs::spec` into `spec/components.json` and README.

## 1. What works with no script

- **Dialog, popover, tabs, accordion:** open, close, exclusivity, light dismiss, Escape, focus
  trapping and the top layer all come from the platform. Zero round trips.
- **Server-side feature detection:** `@supports` beacons plus one cookie per capability let
  the server emit only the markup a browser needs. Verified Firefox 155 (all flags) and
  Chrome 109 (none).
- **Out-of-order streaming:** a declarative shadow root on `<body>` with named slots; chunks
  appended in completion order land in place. Verified with `curl -N` timings and Firefox.
- **State without a framework:** the URL holds what you look at, a cookie remembers it, POST
  changes data, `303 See Other` plus a one-shot cookie carries the flash. Reload-safe.
- **View transitions across navigations:** counters morph and lists grow without a flash in
  Chrome and Safari; Firefox navigates normally. The root does not cross-fade (see "Feel"
  below): the default 0.25 s fade made every click feel slow.
- **Validation UX:** `required`/`pattern` block bad input; `:user-invalid` styles only after
  interaction. The server re-checks and re-renders with messages.
- **Testing without a browser:** Blitz renders every route headless with real layout, so
  "it has a box and sits below the strip" is a unit test, and every route has a PNG.
- **Admin tables and multi-step forms:** sort, filter and page are links and GET forms, every
  state is a URL; a wizard is one POST per step with the entered values on the server and
  the current step in `UiState`. Verified in Firefox with and without the script.

## 2. What needs a fallback today

| Feature | Missing in | Fallback (chosen server-side from `Caps`) |
|---|---|---|
| Invoker commands (`command`/`commandfor`) | Chrome < 135, Firefox < 144, Safari < 26.2 | link to `#id` + `:target` overlay |
| Anchor positioning | Chrome < 125, Firefox < 147, Safari < 26 | UA-centred popover |
| `popover` attribute | Chrome < 114, Firefox < 125, Safari < 17 | `<details>` dropdown, no light dismiss |
| `::details-content` | Chrome < 131, Firefox < 143, Safari < 18.4 | tabs render as an accordion |
| `<details name>` | Chrome < 120, Firefox < 130, Safari < 17.2 | several sections can be open |
| `@view-transition` (cross-document) | Firefox (unshipped) | plain navigation |
| Declarative shadow DOM | Chrome < 111, Firefox < 123, Safari < 16.4 | in-order streaming, spliced in place |
| `light-dark()` | Chrome < 123, Firefox < 120, Safari < 17.5 | not used: media query + `data-theme` instead |
| First page view of any browser | everyone | the beacons have not fired yet; `Caps::ASSUMED` (popover only) |

## 3. What is impossible without script

Each of these is now covered by the optional enhancement script (`axum_nojs::enhance`,
section 4). The list stays true for a browser with script disabled.

- Filtering results as you type against server data. `<datalist>` covers static suggestions;
  results update per submit. Choosing, removing (chips) and creating are links and forms, so
  they need nothing; only the arrow keys into the result list are script.
- Mirroring a slider or colour picker while it moves. The `<output>` and the swatch show the
  value the server last saved; they update on submit.
- Infinite scroll. Cumulative pages with one click per page is the ceiling.
- A modal opened on page load. `<dialog open>` is visible but not modal; only `showModal()`
  gives a backdrop and focus trap. `#id` + `:target` fakes the overlay.
- Persisting a native `<details>` toggle. The title link now fills the summary so every
  click is a navigation; an instant client-only toggle would be undone by the next render.
- Clicking faster than a document reloads. Every counter press is POST, 303, GET and a fresh
  document; a click that lands while the old one is unloading is dropped. The floor is the
  browser's reload time, tens of milliseconds on localhost, and nothing in HTML batches it.
  The transition overlay at least no longer swallows clicks (`::view-transition {
  pointer-events: none }`).
- Optimistic UI, offline behaviour, undo without a round trip.
- Drag and drop, resizable panes, canvas or charts drawn from data.
- Feature-detecting HTML attributes. CSS can only test CSS; `invokers` and `streaming_dsd`
  use CSS features that shipped in the same releases as proxies (see M1 below).

**Verdict:** content sites, admin panels, forms, settings pages and dashboards that refresh per
action need no script. Editors, real-time collaboration and per-keystroke reactions do.

## 4. What the optional script adds

One 10 KB file, no framework, no build step, `script-src 'self'` compatible. It never changes
what the server sends; it changes what the browser does with it.

- **Swap roots.** A root with `id` and `data-nojs="swap"` has its forms and links fetched in the
  background; the response document is parsed and only the root (plus flash, title, theme and
  URL) is replaced, inside `document.startViewTransition` when available. Requests on one root
  are queued, so five fast counter clicks count five.
- **Targets and modes.** `data-nojs-target="#id"` lets a control anywhere name its root;
  `data-nojs-swap` picks outer, inner, append or prepend; `data-nojs-oob` elements in the answer
  update their twin anywhere on the page. The `Nojs-Enhance: 1` header lets a handler answer
  with a fragment. Without the script the same request is the full page, which shows all of it.
- **Lifecycle.** `data-nojs-busy` and `aria-busy` on the root and form while a request runs,
  submit buttons disabled, a `data-nojs-indicator` element shown, `--nojs-busy` for the fade. A
  failed request becomes the navigation the browser would have made.
- **History.** Links push an entry, forms replace it, `data-nojs-push="false"` keeps the URL and
  `data-nojs-replace` rewrites the entry. Every swap stores a copy of the roots in the entry, so
  Back and Forward restore in place with no request. `nojs:swap` fires after every swap.
- **Search as you type** in a combobox inside a swap root, debounced, focus and caret kept.
- **Live mirroring** of range output and colour swatch while the control moves.
- **Real modal** for the `:target` dialog fallback; light dismiss for the `<details>` popover
  fallback.

Proven by `scripts/browser-check.mjs`: headless Firefox 155 through geckodriver, asserting
`performance.getEntriesByType("navigation").length` stays at 1 across every action.

What stays impossible even with it, because the script owns no state and never runs before
the server answers: offline behaviour and retry queues; optimistic updates (the counter shows
the new value when the server says so, not on click); client-side validation beyond what
`required`, `pattern` and `:user-invalid` give; polling or server push (`data-nojs-oob` only
rides on a request the user made; a `<meta http-equiv="refresh">` or a streamed page is the
no-script answer); drag and drop, keyboard shortcuts and anything driven by pointer position;
a "dirty form" warning before leaving; and restoring scroll position or form input inside a
root that Back and Forward replaced from the cached copy (the copy is the markup as swapped,
not what the user typed since).

## Notes by milestone

### M1 · Capability beacons

**One cookie per flag, not one list cookie.** The beacons load in parallel. Seven responses
each setting `nojs-caps=<old list + me>` would overwrite one another and keep one flag. Separate
cookies `nojs-cap-<name>=1` cannot race. The roadmap's "sets/extends a `axum-nojs-caps` cookie" is
implemented as this prefix family.

**Proxies for HTML attributes** (versions from MDN browser-compat-data, September 2026):

| Flag | Real feature | Proxy `@supports` test | Error band |
|---|---|---|---|
| `invokers` | `button[command]`: Chrome 135, Firefox 144, Safari 26.2 | `selector(::picker(select))` (Chrome 135, Safari 27) or `(-moz-appearance: none) and (view-transition-class: x)` (Firefox 144) | Safari 26.2 reported as lacking invokers, gets the `:target` dialog (still works) |
| `streaming_dsd` | `<template shadowrootmode>`: Chrome 111, Firefox 123, Safari 16.4 | `selector(:popover-open)`: Chrome 114, Firefox 125, Safari 17 | Chrome 111-113, Firefox 123-124, Safari 16.4-16.x reported as lacking DSD, get in-order streaming |

Both err on the conservative side: they never claim a feature the browser lacks.

**`view_transitions` means same-document names.** `@view-transition` for cross-document
navigations is Chrome 126 / Safari 18.2 and not in Firefox; `@supports at-rule()` does not
exist in Firefox either. The flag tests `view-transition-class`, and the layout always emits
the at-rule, which browsers without it ignore.

**`light_dark` is informational.** Before M1 the theme variables used `light-dark()` and
Chrome 109 rendered unstyled buttons. A media query plus `data-theme` costs a few lines more
CSS and works everywhere.

**The first view used to be all fallbacks.** A `<meta http-equiv=refresh>` could force a reload
but would double the first-load cost for `curl` and crawlers too, so it is not done. Instead an
unprobed browser gets `Caps::ASSUMED`: `popover` only, at baseline in every engine since early
2024. Everything younger (invokers, `::details-content`, anchors) still waits for the cookie,
so the first and second page view differ only where the fallback is harmless.

**Cookie lifetime is 30 days.** No negative cache: an unsupported feature simply has no cookie,
and `probed` says the beacons ran. Once `probed` is set the beacon elements are not emitted.

### M2 · Out-of-order streaming

**The host must be an ancestor of the late chunks.** Slots only match direct light-DOM
children of the host. A per-section `<nojs-slot>` host cannot receive a chunk appended at the end
of the document, so the roadmap's per-slot host became one host: `<body>`. `slot()` emits a
plain `<slot name>` inside it.

**Shadow trees do not see document stylesheets.** The page inside the shadow root needs its
own `<style>`; the slotted chunks are light DOM and need the document one. The stylesheet is
inlined twice in DSD mode (about 12 KB extra). A `<link>` to a cached `/nojs.css` in both places
would cost one fetch instead; not done because the layout's contract is one inline stylesheet.

**Fallback is still streaming.** Without DSD `slot()` leaves `<!--nojs-slot:id-->` and the
response is spliced in document order: the bytes before the first marker leave immediately,
each section as soon as it and its predecessors resolve. A slow first section delays the rest.

### M3 · State model

**The `nojs-ui` cookie is written on GET.** Preference state has no other trigger without script:
the tab link is a GET. It is idempotent and never touches application data, which stays
POST-only. `UiState` only emits `Set-Cookie` when the query actually changed something.

**Title links versus summary toggles.** A `<summary>` toggles on click without a round trip; a
link inside it navigates. The title is the persisted link, the padding around it the instant
toggle. The open accordion title links to `open.<group>=` (close), so it always toggles too.

**"Collapse all" that did nothing.** `UiState::link(key, "")` used to drop the key from the
URL; with the key gone the cookie's memory (`open.faq=0,2`) won and every section stayed open.
Closing the open section in an exclusive accordion had the same hole. The empty value now stays
in the link as `open.faq=`: an explicit nothing beats the cookie and is remembered as such.

**Saved values are percent-encoded twice.** `Saved<T>` form-encodes the value
(`name=Ada&notify=true`) and then percent-encodes that for the cookie
(`name%3DAda%26notify%3Dtrue`), so a `#` in a colour reads `%2523` in the raw header. Tests
that look at raw `Set-Cookie` headers must expect that. `serde_urlencoded` writes a newtype
(`struct Notes(Vec<(String, String)>)`) but refuses to read one back, so `Saved` reads through
a small deserializer that unwraps newtypes (`saved.rs`).

### M4 · Blitz as the test engine

`axum-nojs-test` renders any demo route through `tower::oneshot`, parses it with `blitz-html`,
resolves Stylo styles and Taffy layout, and paints it with `vello_cpu`. Every route is
screenshotted twice (modern and no-capability cookies) into `tests/shots/`. Blitz has no script
engine, so passing there is proof the page needs none.

**What Blitz 0.3.0-beta.2 cannot render, with issues:**

| Symptom here | Blitz issue |
|---|---|
| `/stream` in DSD mode renders nothing: `<template shadowrootmode>` is inert | [#923](https://github.com/DioxusLabs/blitz/issues/923) (filed), related [#889](https://github.com/DioxusLabs/blitz/issues/889), [#892](https://github.com/DioxusLabs/blitz/pull/892) |
| Theme toggle buttons stay lowercase: `text-transform: capitalize` ignored | [#924](https://github.com/DioxusLabs/blitz/issues/924) (filed) |
| The form's `type=number` field is an 18 px strip | [#925](https://github.com/DioxusLabs/blitz/issues/925) (filed) |
| Tables with `border-collapse: collapse` get a 2 px black grid on every edge | [#386](https://github.com/DioxusLabs/blitz/issues/386), [#504](https://github.com/DioxusLabs/blitz/issues/504) |
| `<dialog open>` is 114 px wide: absolutely positioned box sized by its DOM parent | [#764](https://github.com/DioxusLabs/blitz/issues/764) |
| Header reads "axum-nojs· zero": leading space of a span after an inline is trimmed | [#857](https://github.com/DioxusLabs/blitz/pull/857) (open PR, whitespace collapsing across spans) |
| `/tabs` vertical strip: the panel's left rule spans one row, not the column: Blitz builds boxes only for `::before`/`::after` (`blitz-dom/src/layout/construct.rs`), so `::details-content` never gets one; the panel's own padding is what keeps the shot readable | tracked under [#119](https://github.com/DioxusLabs/blitz/issues/119) (roadmap: pseudo-elements); no dedicated issue |
| `/inputs`: selects render as an empty box (option text and `<selectedcontent>` not drawn), range inputs as a plain box with no thumb, the colour input as a blank box; the two-thumb pair shows only its track. `blitz-paint` `render/form_controls.rs` draws only checkboxes and radios. The Blitz test asserts on attributes and geometry (both thumbs share one track) instead | [#258](https://github.com/DioxusLabs/blitz/issues/258) ("Tracking: Form controls") |
| `/table`: the sticky header cells paint at the top of the viewport, leaving an empty row in the table | `stylo_taffy::convert::position` maps `sticky` to `relative` with a `TODO`; tracked under [#389](https://github.com/DioxusLabs/blitz/issues/389) ("position sticky") |

The DSD gap is pinned by a test (`blitz_has_no_declarative_shadow_dom`) that fails the day
Blitz gains it, so the exception in the screenshot loop gets removed then. Do not expect that
soon: the published beta.2 has no shadow DOM at all (every shadow hook in its Stylo glue
returns `None`), shadow roots and slots arrive in the open PR #892, and the declarative
`<template shadowrootmode>` step is explicitly out of that PR's scope. Once #892 merges the
remaining work is one `TreeSink::attach_declarative_shadow` implementation in `html_sink.rs`
(html5ever 0.39 already routes the template's children through `get_template_contents`),
so the fix will reach a crates.io release only after both land. Until then `/stream` stays
tested through Firefox screenshots, not Blitz. Blitz's `svg`
feature is off: `usvg 0.48` wants `base64 ^0.23`, which the local index did not resolve.

### M6 · Polish

**Customisable `<select>` is a real detection, not a proxy.** `@supports
selector(::picker(select))` is exactly the feature `<selectedcontent>` ships with (Chrome 135,
Safari 27; Firefox behind two flags), so `Cap::BaseSelect` has no error band. Older parsers
drop a `<button>` inside `<select>`, which is why the enhanced markup is only sent to browsers
that asked for it. Neither Firefox 155 nor Chrome 109 on this machine has it, so both got the
plain select; the markup branch is covered by the unit tests.

**Blitz and form controls.** Blitz 0.3.0-beta.2 paints `<select>` and `<input type=color>` as
empty boxes and `<input type=range>` as an inert box ([#456](https://github.com/DioxusLabs/blitz/issues/456);
all three under the form-controls tracking issue [#258](https://github.com/DioxusLabs/blitz/issues/258)).
The `/inputs` screenshot therefore shows the layout, not the controls.

**Publishing.** `cargo publish --dry-run -p axum-nojs` packages and builds cleanly. A real
publish still needs a `license` field, which is a decision for the owner (see `BLOCKED.md`).

### Feel · why navigations felt slow

The server answers in under a millisecond; every delay was client-side CSS.

**The root cross-fade was the cost.** `@view-transition { navigation: auto }` with the default
`::view-transition-old/new(root)` animation freezes the old page, waits for the new one and
fades for 0.25 s on every click, redirect included. Now the root swaps instantly and only named
parts (counter, list) morph for 160 ms. `<link rel="expect" href="#main" blocking="render">`
holds the transition until `<main>` is parsed; streamed pages omit it because their parse ends
with the last slot. Firefox has no cross-document transitions at all (MDN: unshipped), so it
always did a plain reload.

**A named button is a moving block.** The pager's "Load more" link carried its own
`view-transition-name`. On every load the old snapshot of the button morphed from its old spot
to its new one, 8 rows lower, sliding over the freshly added rows (seen frame by frame in
Firefox 155 with the animation slowed to 4 s). Only the list keeps a name now: the new rows
fade in under the old ones and the button simply re-renders where it belongs.

**A named tab title is the same block, sideways.** The first tabs morph put the name on the
open `<summary>`; every switch slid the old title's snapshot, text included, across the strip
and cross-faded it into the new one: a flicker and a bounce to the left. The name now sits on an
empty `.nojs-tabs-mark` (then a 2px underline, since M20 shadcn's raised chip behind the open
title), so the highlight glides and the titles stay still. Rule of
thumb: name the highlight, never the thing that holds text.

**Two behaviours on one summary.** The tab and accordion titles were links inside a padded
summary: clicking the text navigated, clicking the padding toggled `<details>` natively and the
next server render snapped it back. The link now fills the summary. The accordion draws its own
marker with `::before` because a block-level link pushed the native marker onto its own line.

**`height: auto` does not interpolate.** The `::details-content` transition snapped open while
`content-visibility ... allow-discrete` still delayed the close. `interpolate-size:
allow-keywords` (Chrome 129) makes both directions animate; elsewhere it snaps symmetrically.

**Theme toggle went home.** The demo redirected to `/` after every theme change. It now
redirects to the same-origin `Referer` path, `/` when there is none.

### M5 · Machine-readable spec

`axum_nojs::spec::SPECS` is the single source: `spec/components.json` and the README matrix
are generated from it (`cargo run -p demo -- spec write`) and tests fail when they drift. A
test also reads every component's `//!` header and requires each spec feature name to appear
in it verbatim, so the header and the spec cannot disagree about which features a component
uses.

### M10 · Components admin panels need

**Everything the table does is a URL.** Sort (`?sort=size&dir=desc`), filter (`?q=a`), page
(`?page=2`) and page size (`?per=5`) are all query keys, so a view can be bookmarked or sent
to a colleague, and the browser's Back button is the undo. The component never sorts or
filters: it renders rows it is given and emits links that ask for others. That keeps it usable
from `curl` and keeps the data path (a database query) in the handler where it belongs.
The `<search>` form keeps the sort in hidden inputs and the sort links keep the filter in the
query, so no action forgets the others; `paged_table` threads `per` through both.

**A remembered page size that still shares.** With a `UiState` the size is the state key
`per.<table>`, so the `nojs-ui` cookie brings back the size a visitor picked. Every page link
still names it, so a URL someone sends shows the same rows for everyone; only a bare `/table`
differs per visitor.

**"You have unsaved changes" cannot be done without script.** Warning before leaving a
half-filled form needs a `beforeunload` listener and a comparison of the fields with what was
loaded; no attribute or CSS state says "this form differs from its defaults", and the server
never learns of edits that were not submitted. The honest no-script substitute is to make
leaving cheap: the wizard stores each step as it is posted and resumes from the cookie. The
character counter is the same shape of problem: `maxlength` enforces the limit natively, but
the `<output>` only moves while typing when the enhancement script is there.

**A long select filters through the form it lives in.** A search box cannot live inside a
`<select>` picker, and a second `<form>` cannot nest inside the one that saves, so the filter
is a button with `formmethod="get"` and `formaction`: it re-requests the page with every field
of the form in the query, unsaved, and the server renders only the matching options (plus the
selected one). The enhancement script had to learn to honour a submitter's `formmethod` and
`formaction`; before that it would have posted, and saved, on every keystroke.

**Two thumbs, two inputs.** There is no native dual-thumb range. Two range inputs in one grid
cell with `pointer-events: none` on the inputs and `auto` on their thumbs give it without
script; the thumbs can cross, so the server orders the pair (`range::order`). The fill between
them would need the live values, so the track stays neutral.

**`field-sizing: content` is Chrome only (123).** Firefox and Safari keep the textarea at its
`rows` with a vertical resize handle, which is the fallback and needs no branch on `Caps`.

**A wizard step that fails answers, it does not redirect.** A valid step is stored and
redirected (PRG); an invalid one answers `422` with the same step, the typed values and the
messages beside the fields, because a redirect would need the messages parked somewhere for
one request. The enhancement script swaps the `422` body like any other, so the in-place path
and the no-script path show the same thing. Skipping an optional step uses `formnovalidate`,
so the browser does not demand its fields first.

**Resuming is the cookie doing its job.** The step already lived in the `nojs-ui` cookie; a bare
visit to the wizard lands on it. `UiState::remembered` tells the two apart (the key came from
the cookie, not the link), so the page can say "Picked up where you left off" and offer Start
over. The entered values need their own persistent cookie (the demo keeps them a week) or a
session store; a session cookie would be lost with the browser.

**The pager went stale behind an in-place sort.** The table was the swap root and the pager
sat outside it, so after a sort through the enhancement script the page links still carried
the old sort. The paged table is now the swap root and its inner table is not, so a sort,
filter, page or size change replaces both.

**Sticky headers are one line of CSS and one Blitz gap.** `position: sticky; top: 0` on
`thead th` keeps the columns labelled while a long table scrolls. `stylo_taffy` maps
`sticky` to `relative` with a `TODO` and Blitz then paints the cells at the viewport top,
leaving an empty row in the table (screenshot `table-*.png`; tracked under
[#389](https://github.com/DioxusLabs/blitz/issues/389)). Firefox renders it correctly.

**A wizard is three forms, not one.** Each step is its own `<form method="post">` carrying
`step=<n>`; the handler merges the fields into stored data and answers with PRG to the next
step's URL. Refresh never re-submits, the browser's Back returns to the previous step with
its values, and the "Back" link is the same URL. The current step is a `step.<id>` key in
`UiState`, so it travels in the query and the `nojs-ui` cookie like a tab; the entered values
are application data and live in the app's store (a cookie in the demo). Putting them in
the URL would leak names and emails into history and logs.

**Not possible without script, still:** inline cell editing, drag to reorder columns, row
selection that survives paging without a form round trip, and "unsaved changes" warnings
when leaving a wizard step.

**A flash stack is one cookie.** Several messages travel as one `nojs-flash` value, one line
each, prefixed with their level (`ok:Saved.\nwarn:Look.`); plain text stays `info`, so a lone
`ui.redirect(to).flash("Saved.")` sends the text as is. Dismiss is a link back to the page: reading the
flash already queued the cookie's deletion, so the next render is clean. Auto-hide is a CSS
animation on info and ok only (an error that vanishes before it is read is worse than one that
stays), and `prefers-reduced-motion: reduce` turns it off. Blitz renders the first frame of an
animation, so the Blitz shots always show the message; Firefox confirms the computed
`animation-name` in `scripts/browser-check.mjs`. What it cannot do without script is vanish
in place when dismissed: the dismiss link is a navigation.

**A command palette needs no script, only a server that answers "what did they mean".**
The palette is a `popover` holding a GET `<search>` form whose input has a `<datalist>` of
every command, so the browser filters suggestions as the user types. Enter sends `q`: an
exact name answers `303` to its page (`palette::exact`), anything else renders the matches
(`palette::matches`). The opener carries `accesskey="k"`, so Alt+Shift+K opens it with no
key handler. A popover cannot arrive open, so the results of a search are rendered in the
page below the opener; without `Popover` the palette is a `<details>` that arrives open with
the results inside. What still needs script: arrow keys through a live list and a global
Ctrl+K.

**A sidebar and a drawer are one `<dialog>`.** Above 60rem a media query shows the closed
dialog in a grid column and hides the menu button; below, the button opens it as a modal
with `command="show-modal"`, sliding in through `@starting-style`. Blitz renders the sidebar
(the closed `<dialog>` with `display: block` lays out like any block), so the layout is
asserted there; the modal and the slide are checked in Firefox.

**Toasts are flashes in another place.** Same cookie, same `level:` lines, rendered as a
`position: fixed` list. Danger never fades; the others pause on `:hover`/`:focus-within`.
Blitz paints them at the right edge of its viewport as Firefox does.

### M16 · Latency baseline

`scripts/bench.sh` (release demo on 3001, 30 samples, x86_64 with 12 cores, loopback) before any
M16 change. Cold is a new connection per request; warm is one kept-alive connection.

| Route | Cold TTFB p50 / p95 | Warm TTFB p50 / p95 | Full response p50 | Firefox responseStart / DOMContentLoaded / load |
|---|---|---|---|---|
| `/` | 0.73 / 1.73 ms | 0.33 / 0.45 ms | 0.81 ms | 1 / 66 / 66 ms |
| `/table` | 0.93 / 2.64 ms | 0.42 / 0.61 ms | 1.01 ms | 1 / 41 / 54 ms |
| `/stream` | 0.93 / 1.27 ms | 0.68 / 0.87 ms | 2905 ms | 2 / 2010 / 2010 ms |

On loopback the server is under a millisecond; almost all of what a person waits for is the
browser parsing and painting (tens of ms), and on `/stream` the deliberate slot delays. So the
cheap wins that count are the ones that shrink the page (inline stylesheet, compression) and
the swap payload, not the handler. `/stream` finishes at about 2.9 s in curl but DOMContentLoaded
fires at 2.0 s in Firefox: the slots Firefox gets depend on its caps cookie, and the last slot
arrives after the document it cares about has been parsed.

**Server cheap wins.** `stylesheet()` is built once in a `OnceLock` and returned as
`&'static str` (it held the `Tokens::default().css()` string too, so that is cached with it);
the demo router gets `tower-http`'s `CompressionLayer` for gzip and br, restricted to bodies
with a known size so `/stream` keeps its chunks (gzip would hold them until its buffer
fills); the release profile is `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`.
Every non-streamed response already carries `Content-Length` (axum sets it from the body's
exact size; a test checks it). `/nojs/enhance.js` is served `immutable` under a `?v=<hash>`
URL, verified. The `/nojs/caps` beacons stay `no-store` on purpose: each one sets a cookie, and a
cached answer would never set it again after the cookie is cleared; they are only requested
while the cookie lacks the flag, so they cost nothing after the first visit.

| Route | Warm TTFB p50 before → after | Cold TTFB p50 | Bytes plain / gzip / br | Firefox DOMContentLoaded |
|---|---|---|---|---|
| `/` | 0.33 → 0.17 ms | 0.73 → 0.32 ms | 48 906 / 10 479 / 11 135 | 66 → 33 ms |
| `/table` | 0.42 → 0.19 ms | 0.93 → 0.37 ms | 58 140 / 11 103 / 11 714 | 41 → 41 ms |
| `/stream` | 0.68 → 0.59 ms | unchanged | not compressed | unchanged |

Pages are about 80 % inline stylesheet, so compression shrinks them to a fifth; br at
`tower-http`'s default quality comes out slightly larger than gzip here. Firefox on loopback
barely notices the bytes; on a real network the fifth-size page is the win that counts.

**Page cheap wins.** `stylesheet()` now runs through `minify_css` once per process: comments
out, whitespace collapsed outside quoted strings, a space kept only between two words and
before a `:` (so `.a :focus` keeps its meaning). A test checks a sample byte for byte, that
braces and parentheses stay balanced, that no comment survives and that minifying twice
changes nothing; the Blitz layout assertions and the Firefox browser check pass on the
minified sheet, which is the proof that it still parses. `/` goes from 48 906 to 41 847 bytes
(8 806 gzipped, was 10 479) and `/table` from 58 140 to 51 081 (9 419 gzipped, was 11 103).

The rest of the box, checked and left as is:
- The beacons are CSS `background-image`s on 1px `<i>` elements inside `@supports`, not
  `<img>`, so `loading="lazy"` and `fetchpriority` do not apply. Background images never block
  first paint, browsers fetch them at low priority after layout, and the whole block is gone
  once `nojs-cap-probed` is set.
- `<script src="/nojs/enhance.js?v=…" defer>` is the last element of `<body>` (unchanged).
- `<link rel="preconnect">` is not needed: the page has no third-party origin.
- Caps travel as one cookie per flag (`nojs-cap-<name>=1`, nine flags, about 170 bytes on every
  request once all are set), and each beacon answer is exactly one `Set-Cookie`. Folding them
  into a single bitset cookie would need the server to merge flags, and the beacons fire in
  parallel, so two answers would race and one flag would be lost; the per-flag cookies stay.

**Script cheap wins.** Every request from `enhance.js` sends `Nojs-Enhance: 1` and
`Accept: text/html`, and user actions fetch with `priority: "high"`. Rather than a
hand-written fragment per route, one middleware (`enhance::slim`, on the whole demo router)
answers any enhanced request with the page minus its inline stylesheet, which the requesting
document already has; every HTML answer says `Vary: Nojs-Enhance`. `/swap` keeps its own
smaller answer. Links under `data-nojs-prefetch` (the demo's first tab strip) are fetched at
low priority on hover or focus and a click within five seconds reuses the answer; the browser
check proves one request, no reload. Links to the page already shown are not prefetched: after
a swap the link under the pointer is a new element and gets a fresh `mouseover`.

| Swap request | Full page | Enhanced answer | Enhanced + gzip |
|---|---|---|---|
| `/tabs?tab.demo=1` | 39 904 B | 3 226 B | 1 153 B |
| `/table?page=2` | 51 289 B | 14 611 B | 2 392 B |
| `/counter` | 38 900 B | 2 222 B | 926 B |

The script grew past 10 KB with prefetch; it is now served without comment lines and
indentation (`enhance::served()`, 9 398 bytes) and the budget applies to that, while the
source keeps its comments (a second test caps the source at 12 KB).

**Prefetch without speculation rules.** `<script type="speculationrules">` is a `<script>` tag,
so it is out. The index instead emits one `<link rel="prefetch">` per component page (body-ok
`link`, baseline in Firefox and Chrome, not Safari). In Firefox the click on `/dialog` then
comes from the cache: navigation `transferSize` 0 and `responseStart` 0 ms, against 8.9 KB and
24–46 ms without the hints (debug build, loopback). The cost is the 19 pages fetched at idle
priority on each index view, about 170 KB gzipped.

A first try served a stale page: after switching to dark on the index, `/dialog` came from the
prefetch cache still light. Every page depends on cookies (caps, theme, flash), so
`enhance::slim` now sends `Vary: Nojs-Enhance, Cookie` on every HTML answer; with it the cached
copy is dropped when the cookie changes and the page is fetched fresh (checked by hand in
Firefox; the browser check covers the cache hit).

What the platform cannot do without the rules script: **prerender** (`<link rel="prerender">`
is gone from every engine; only speculation rules prerender, in Chromium), eagerness levels
(`moderate` = on hover, `conservative` = on pointer down) and document rules that match links
by selector. The hover-time version lives in `enhance.js` as `data-nojs-prefetch`; `rel=prefetch`
is all-or-nothing at page load.

**Streaming where it pays.** `Streamed::into_stream` now yields the document up to `</head>`
as its own first chunk, so the stylesheet parses while the shell is written and the slow
fills wait; the Axum response adds `X-Accel-Buffering: no` so an nginx in front passes chunks
on instead of buffering them. `/stream` in Firefox: first contentful paint at 50 ms while
DOMContentLoaded waits 2 009 ms for the slowest slot, TTFB 0.9 ms cold (`scripts/bench.sh`
now reports first paint).

`/table` stays a whole response on purpose. Its body waits on nothing (rows are built in
memory in well under a millisecond: 0.44 ms cold TTFB, 37 ms first paint), and streaming it
would drop `Content-Length` and, through the compression predicate, gzip: 51 KB on the wire
instead of 9 KB. The rule for a real server: stream a route only when its body awaits I/O
(a database, another service); give it a `slot` per slow part, or one `slot` around the whole
body if nothing can be shown early, and the head still goes out first.

**Structural.** `cargo bench -p axum-nojs` (criterion, `axum-nojs/benches/render.rs`), before
→ after on this machine:

| bench | before | after | what changed |
|---|---|---|---|
| `stylesheet()` | 1.6 ns | 1.0 ns | nothing: already a `OnceLock` read |
| `layout` (whole page, 42 KB) | 15.6 µs | 11.9 µs | one `Reserve` splice sizes the buffer for the stylesheet and body; the head is no longer a separate `Markup` copied in |
| `table`, 1 000 rows | 45.2 µs | 41.7 µs | none kept (see below) |
| `paged_table`, page 20 of 1 000 | 4.9 µs | 4.2 µs | page numbers and page links render through `Display` straight into the buffer; carried pairs borrow instead of cloning |
| `UiState::from_request` | 884 ns | 596 ns | keys are checked before decoding, and `decode` borrows when there is no `%` or `+` |

`Caps` from the cookie is a fixed loop over the flags into a bitset: nothing to do. Maud's
`html!` already starts from `String::with_capacity(<template literal size>)`, so a size hint
only pays where splices dwarf the literals, which is the page shell. A hint on the 1 000-row
table made it 12% *slower*: the estimate overshot, touching pages it never filled, while a
growing `String` above a page reallocates in place (`mremap`) without copying. Not kept.

### M20 · The shadcn look

Every component now takes shadcn/ui's neutral theme through tokens alone: the palette grew to
shadcn's roles (`card`, `popover`, `secondary`, `accent` as the hover surface, `primary` as the
brand, `input`, `ring`), and `Tokens::css()` derives the small/large radii, two shadows and the
backdrop overlay, so component CSS still names no colour. Three things learned on the way:

- **The colour-literal test reads comments too.** It greps all component CSS for `#hex`,
  `rgb(`, and colour words, and a comment saying a border "turns red" or a backdrop is
  "black/50" fails it. Kept as is: rewording a comment is cheaper than parsing CSS.
- **Tabs' moving mark became shadcn's raised chip.** It is the same element carrying the view
  transition name, now inset 3px inside the open summary instead of a 2px underline; Blitz
  lays it out correctly and the test checks the inset.
- **What `scripts/look.sh` cannot see.** Toasts only exist after a POST, so a Firefox
  `--screenshot` of a URL never shows one; the Blitz toast shot is the only picture. The
  demo stage is left-aligned where shadcn centres its preview, on purpose, because it carries
  prose. No new Blitz gaps. The sticky header is still the known one
  ([#389](https://github.com/DioxusLabs/blitz/issues/389)); the 2 px black grid
  ([#386](https://github.com/DioxusLabs/blitz/issues/386)) no longer shows on `/table`, because
  the framed table now uses `border-collapse: separate` with row rules only.

### M21 · Primitives

Button, input, badge, card, icon, avatar and four layout helpers. Everything renders in
Blitz with no script. The Blitz tests and the Firefox check found three things:

- **The keyboard focus ring can only be checked in Firefox.** Blitz hard-codes
  `:focus-visible` to false (`blitz-dom` `stylo.rs`,
  [#839](https://github.com/DioxusLabs/blitz/issues/839)). `scripts/browser-check.mjs` now
  tabs onto the primary button and asserts `:focus-visible` and a 3px outline. The Blitz test
  covers the variants' sizes and fills.
- **The switch is a plain box in Blitz.** It is a checkbox with `appearance: none`, and its
  thumb is an `::before` pseudo-element. Blitz paints neither the custom look nor the pseudo
  on the `<input>`: it is still a working checkbox, only its drawing differs (form controls
  are tracked in [#258](https://github.com/DioxusLabs/blitz/issues/258)). Firefox draws the
  track and thumb.
- **Taffy picks one column when a grid track minimum uses `min()`.**
  `repeat(auto-fill, minmax(min(16rem, 100%), 1fr))` gives one column at any width, while
  `minmax(16rem, 1fr)` and `minmax(var(--m), 1fr)` are fine. No upstream issue exists yet;
  filing one is the owner's call. `ui.grid` avoids it: it uses the plain minimum, and only
  under `@media (max-width: 30rem)` caps it with `min(<min>, 100%)`, so a wide minimum cannot
  overflow a phone. Blitz tests run at 1000px, so they never reach that rule.
  Inline SVG needs Blitz's `svg` feature. The test harness had turned it off, so icons were
  laid out but not painted; `axum-nojs-test` now enables it (a first-party inline-SVG plan
  is [#701](https://github.com/DioxusLabs/blitz/issues/701)).

A crate that uses `html!` from the prelude still needs its own `maud` dependency, because
the macro expands to `maud::` paths (M24's guide must say so). `.error("")` on a field is
now "no error", so a route can pass `if bad { "…" } else { "" }` without wrapping the
builder in a branch.
