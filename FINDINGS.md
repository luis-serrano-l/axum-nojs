# Findings

What the HTML/CSS platform can and cannot do for a server-rendered app with no script, as
learned building and testing this crate. Three lists first; the milestone notes with the
reasons follow. The per-component feature matrix (with per-browser versions) is generated
from `webonsive::spec` into `spec/components.json` and README.

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
  Chrome and Safari; Firefox navigates normally.
- **Validation UX:** `required`/`pattern` block bad input; `:user-invalid` styles only after
  interaction. The server re-checks and re-renders with messages.
- **Testing without a browser:** Blitz renders every route headless with real layout, so
  "it has a box and sits below the strip" is a unit test, and every route has a PNG.

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
| First page view of any browser | everyone | the beacons have not fired yet; fallback markup |

## 3. What is impossible without script

- Filtering results as you type against server data. `<datalist>` covers static suggestions;
  results update per submit.
- Infinite scroll. Cumulative pages with one click per page is the ceiling.
- A modal opened on page load. `<dialog open>` is visible but not modal; only `showModal()`
  gives a backdrop and focus trap. `#id` + `:target` fakes the overlay.
- Persisting `<details>` toggles made by clicking the summary. Only the title link (a
  navigation) persists; the instant toggle is client-only.
- Optimistic UI, offline behaviour, undo without a round trip.
- Drag and drop, resizable panes, canvas or charts drawn from data.
- Feature-detecting HTML attributes. CSS can only test CSS; `invokers` and `streaming_dsd`
  use CSS features that shipped in the same releases as proxies (see M1 below).

**Verdict:** content sites, admin panels, forms, settings pages and dashboards that refresh per
action need no script. Editors, real-time collaboration and per-keystroke reactions do.

## Notes by milestone

### M1 · Capability beacons

**One cookie per flag, not one list cookie.** The beacons load in parallel. Seven responses
each setting `wo-caps=<old list + me>` would overwrite one another and keep one flag. Separate
cookies `wo-cap-<name>=1` cannot race. The roadmap's "sets/extends a `wo-caps` cookie" is
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

**The first view is always the fallback.** A `<meta http-equiv=refresh>` could force a reload
but would double the first-load cost for `curl` and crawlers too, so it is not done.

**Cookie lifetime is 30 days.** No negative cache: an unsupported feature simply has no cookie,
and `probed` says the beacons ran. Once `probed` is set the beacon elements are not emitted.

### M2 · Out-of-order streaming

**The host must be an ancestor of the late chunks.** Slots only match direct light-DOM
children of the host. A per-section `<wo-slot>` host cannot receive a chunk appended at the end
of the document, so the roadmap's per-slot host became one host: `<body>`. `slot()` emits a
plain `<slot name>` inside it.

**Shadow trees do not see document stylesheets.** The page inside the shadow root needs its
own `<style>`; the slotted chunks are light DOM and need the document one. The stylesheet is
inlined twice in DSD mode (about 12 KB extra). A `<link>` to a cached `/wo.css` in both places
would cost one fetch instead; not done because the layout's contract is one inline stylesheet.

**Fallback is still streaming.** Without DSD `slot()` leaves `<!--wo-slot:id-->` and the
response is spliced in document order: the bytes before the first marker leave immediately,
each section as soon as it and its predecessors resolve. A slow first section delays the rest.

### M3 · State model

**The `wo-ui` cookie is written on GET.** Preference state has no other trigger without script:
the tab link is a GET. It is idempotent and never touches application data, which stays
POST-only. `UiState` only emits `Set-Cookie` when the query actually changed something.

**Title links versus summary toggles.** A `<summary>` toggles on click without a round trip; a
link inside it navigates. The title is the persisted link, the padding around it the instant
toggle. The open accordion title links to `open.<group>=` (close), so it always toggles too.

**The cookie crate percent-encodes.** `axum-extra`'s `CookieJar` writes `Ada|1` as `Ada%7C1`
and decodes it on the way back. Tests that look at raw `Set-Cookie` headers must expect that.

### M4 · Blitz as the test engine

`webonsive-test` renders any demo route through `tower::oneshot`, parses it with `blitz-html`,
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
| Header reads "webonsive· zero": leading space of a span after an inline is trimmed | [#857](https://github.com/DioxusLabs/blitz/pull/857) (open PR, whitespace collapsing across spans) |

The DSD gap is pinned by a test (`blitz_has_no_declarative_shadow_dom`) that fails the day
Blitz gains it, so the exception in the screenshot loop gets removed then. Blitz's `svg`
feature is off: `usvg 0.48` wants `base64 ^0.23`, which the local index did not resolve.

### M5 · Machine-readable spec

`webonsive::spec::SPECS` is the single source: `spec/components.json` and the README matrix
are generated from it (`cargo run -p demo -- spec write`) and tests fail when they drift. A
test also reads every component's `//!` header and requires each spec feature name to appear
in it verbatim, so the header and the spec cannot disagree about which features a component
uses.
