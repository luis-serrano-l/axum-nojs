# Who this is for

axum-nojs makes one promise: every page works with JavaScript off, and CI proves it with a
renderer that has no script engine. Each audience below relies on a different part of that
promise; each section says which, and where the proof lives.

## Public services (GOV.UK-style)

A public service cannot choose its users' browsers, devices or connections, and the
people who most need it are often the ones on the oldest phone or the worst signal. Government
design guidance, the UK's service manual among it, asks services to build with progressive
enhancement: the page works as HTML first, and script only improves it.

**The guarantee it relies on:** every page, form and flow works as HTML and form posts.
Validation is done on the server and the messages come back beside the fields
(`/app/signin`). Multi-step forms keep their place in the URL and a cookie (`/wizard`). The
optional script only swaps the same markup in place.

**The proof:** Blitz renders every demo route with no script engine
(`axum-nojs-test/tests/demo.rs`), and `a_whole_app_flow_with_no_script` signs in, adds,
edits and deletes with no script at all.

## Strict Content-Security-Policy

Banks, health services, admin back offices and anything that handles payments often run under
a Content-Security-Policy that forbids inline script, `eval` and third-party origins. Many UI
kits need exceptions to it.

**The guarantee it relies on:** there is no inline script, no `on*` handler and no
`javascript:` URL anywhere. The one script is a same-origin file, so `script-src 'self'` is
enough, and `script-src 'none'` works for a page built with `Page::without_script()`.
`enhance::csp` sends the policy (README, "Strict Content-Security-Policy").

**The proof:** `pages_ship_only_the_enhancement_script` checks every route for a single
script tag and no inline code. `every_route_is_served_under_a_strict_csp` checks the header.
The Firefox check (`scripts/browser-check.mjs`) runs every enhanced interaction with the policy
in force.

## Low bandwidth and old devices

A page that has to download, parse and run a large bundle before it responds is slow on a
cheap phone and a 3G link, and the delay is spent before the first tap works.

**The guarantee it relies on:** a page is 12–13.5 KB gzipped with the whole stylesheet inlined,
and it needs no script to be usable. The optional script is 3.6 KB gzipped, cached forever, and
loaded with `defer`, so it never blocks the page. Features newer than a browser are detected
on the server (`Caps`), and that browser is sent the variant it can use, not a polyfill.

**The proof:** the README's "What a page weighs" gives the numbers. Two tests hold the budgets:
the stylesheet under 64 KB, and every demo page under 96 KB. `scripts/bench.sh` measures time to
first byte and first paint.

## Tor Browser "Safest", NoScript, blocked script

Tor Browser's "Safest" security level turns JavaScript off on every site. NoScript users,
locked-down corporate browsers and a script that failed to load all end up in the same place.

**The guarantee it relies on:** the same as the first section. "Works with script off" is
not a degraded mode here, because it is the only mode the core has. The script is an addition,
and a request it makes that fails falls back to the plain navigation the browser would have
made.

**The proof:** the Blitz suite in CI, and `?script=off` on any demo page, which serves it
without the script under `script-src 'none'`.

## Internal tools

Admin panels, back offices and dashboards are mostly tables, forms, filters and settings, and
the team that builds them is usually a backend team that would rather not run a frontend
build.

**The guarantee it relies on:** Rust on the server is the only language needed. There is no
bundler, no `node_modules`, and no client state to keep in sync. The components cover the
usual work: sortable, filterable, paged tables with bulk actions and rows edited in place;
validated forms; dialogs and menus; a calendar and a date picker; uploads; a kanban board.
State is URLs, cookies and Post/Redirect/Get, so every screen can be bookmarked, shared and
`curl`ed.

**The proof:** the demo is the catalogue (`cargo run -p demo`), and every page shows the code
that drew it. `docs/components.md` shows how to add your own in about 30 lines.

## Who it is not for

An app that reacts on every keystroke, such as a document editor, a live canvas or real-time
collaboration, needs client-side state. Leptos or Dioxus are better fits for those; see
`docs/comparison.md`.
