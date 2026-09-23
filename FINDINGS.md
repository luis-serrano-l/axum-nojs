# Findings

What the HTML/CSS platform can and cannot do for a server-rendered app with no script. Each
entry names the milestone that produced it. README keeps the short version; this file keeps the
reasons.

## M1 · Capability beacons

**Works.** `@supports` rules that give a hidden element a `background-image` are a reliable,
script-free feature probe: the image request only happens when the condition holds, and the
server records it in a cookie. Verified in Firefox 155 (all flags set on the second view) and
Chrome 109 (only `probed` set; every component rendered its fallback).

**One cookie per flag, not one list cookie.** The beacons load in parallel. Seven responses each
setting `wo-caps=<old list + me>` would overwrite one another and keep one flag. Separate
cookies `wo-cap-<name>=1` cannot race. The roadmap's "sets/extends a `wo-caps` cookie" is
implemented as this prefix family.

**CSS cannot test HTML attributes.** Two flags are proxies for CSS features that shipped in the
same release (versions from MDN browser-compat-data, September 2026):

| Flag | Real feature | Proxy `@supports` test | Error band |
|---|---|---|---|
| `invokers` | `button[command]`: Chrome 135, Firefox 144, Safari 26.2 | `selector(::picker(select))` (Chrome 135, Safari 27) or `(-moz-appearance: none) and (view-transition-class: x)` (Firefox 144) | Safari 26.2 reported as lacking invokers, gets the `:target` dialog (still works) |
| `streaming_dsd` | `<template shadowrootmode>`: Chrome 111, Firefox 123, Safari 16.4 | `selector(:popover-open)`: Chrome 114, Firefox 125, Safari 17 | Chrome 111-113, Firefox 123-124, Safari 16.4-16.x reported as lacking DSD, get in-order streaming |

Both proxies err on the conservative side: they never claim a feature the browser lacks.

**`view_transitions` means same-document names.** `@view-transition` for cross-document
navigations is Chrome 126 / Safari 18.2 and not in Firefox; `@supports at-rule(@view-transition)`
does not exist in Firefox either. The flag tests `view-transition-class`, and the layout always
emits the at-rule, which browsers without it ignore.

**`light_dark` is informational.** The theme is switched by `prefers-color-scheme` and
`data-theme`, which is one media query more CSS than `light-dark()` and works in Chrome 109.
Before M1 the theme variables used `light-dark()` and Chrome 109 rendered unstyled buttons.

**The first view is always the fallback.** Beacons fire while the first page loads; the cookie
is available from the second request. A `<meta http-equiv=refresh>` could force a reload but
would double the first-load cost for `curl` and crawlers too, so it is not done.

**Cookie lifetime is 30 days.** A browser upgrade is re-detected when the cookies expire. There
is no negative cache: an unsupported feature simply has no cookie, and `probed` says the beacons
ran. Once `probed` is set the beacon elements are no longer emitted.

## Impossible without script (from the prototype)

- Filtering results as you type against server data. `<datalist>` covers static suggestions.
- Infinite scroll. Cumulative pages with one click per page is the ceiling.
- Optimistic UI, offline behaviour, undo without a round trip.
- Drag and drop, resizable panes, canvas or charts drawn from data.
- Keeping `<details>` state across navigations without a `?tab=` round trip.
