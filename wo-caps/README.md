# wo-caps

Server-side browser feature detection with no JavaScript.

Your server wants to know whether the browser understands `popover`, `<dialog>` invoker
commands, anchor positioning, `::details-content`, declarative shadow DOM or a customisable
`<select>`, so it can send the modern markup to browsers that have it and a fallback to the
rest, never both. User-agent sniffing guesses; a script is what you were avoiding. `wo-caps`
asks CSS instead.

## How it works

1. The page carries a few empty 1×1 px elements, one per capability.
2. `@supports` rules give each one a background image only when the browser understands
   the feature: `@supports selector(:popover-open) { .wo-cap-popover { background-image:
   url("/wo/caps?flag=popover") } }`.
3. Loading that image hits `/wo/caps?flag=popover`, which answers `204` with a cookie
   `wo-cap-popover=1`.
4. On the next request the server reads the cookies into a `Caps` bitset and branches on
   `caps.has(Cap::Popover)`.

The first page view has no cookies yet: it gets `Caps::ASSUMED` (features at baseline in
every engine for over two years), and the second view is tailored. A browser that never
loads CSS images (`curl`, a reader) stays on fallbacks, which is what it needs.

## Use

```rust
use wo_caps::{Cap, Caps, beacon_cookie, beacon_css, beacons};

// Reading a request: the query wins (`?caps=popover,anchor` forces a set), then the cookies.
let caps = Caps::from_query("page=2")
    .unwrap_or_else(|| Caps::from_cookie_header("wo-cap-probed=1; wo-cap-popover=1"));
assert!(caps.has(Cap::Popover) && !caps.has(Cap::Anchor));

// Rendering: `beacon_css()` goes in the stylesheet, `beacons(&caps)` at the end of <body>
// (empty once the browser is known).
let css = beacon_css();
let html = beacons(&caps).into_string();

// The beacon route, `GET /wo/caps?flag=<name>`: 204 with this cookie, or 404.
assert!(beacon_cookie("flag=anchor").unwrap().starts_with("wo-cap-anchor=1"));
assert_eq!(beacon_cookie("flag=nope"), None);
```

That is the whole protocol: three functions on strings, plus two that render. Any server can
use it; `examples/hyper.rs` does it on raw hyper in forty lines. With the `axum` feature,
`Caps` is an extractor and `wo_caps::router()` serves the beacon route.

Markup is rendered with [maud](https://crates.io/crates/maud); `.into_string()` hands it to
anything else.

## Flags

| Flag | Detects | Shipped |
|---|---|---|
| `probed` | the beacons loaded at all | |
| `invokers` | `<button command commandfor>` (proxy, see below) | Chrome 135, Firefox 144, Safari 26.2 |
| `anchor` | CSS anchor positioning | Chrome 125, Firefox 147, Safari 26 |
| `details_content` | `::details-content` | Chrome 131, Firefox 143, Safari 18.4 |
| `view_transitions` | same-document view transitions | Chrome 125, Firefox 144, Safari 18.2 |
| `popover` | `popover` attribute | Chrome 114, Firefox 125, Safari 17 |
| `light_dark` | `light-dark()` | Chrome 123, Firefox 120, Safari 17.5 |
| `streaming_dsd` | `<template shadowrootmode>` (proxy) | Chrome 111, Firefox 123, Safari 16.4 |
| `base_select` | `appearance: base-select`, `<selectedcontent>` | Chrome 135, Safari 27 |

CSS cannot test HTML attributes, so `invokers` and `streaming_dsd` test CSS features that
shipped in the same release or later; they never claim a feature the browser lacks.

## Part of webonsive

`wo-caps` is the detection half of [webonsive](https://crates.io/crates/webonsive), a
no-JavaScript component library for Rust servers, and is re-exported there as
`webonsive::caps`. Its `docs/caps.md` has the long version: cookie format, the first-view
problem, error bands, how to add a flag.
