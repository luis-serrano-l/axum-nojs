# How the server learns what a browser can do, with no script

`axum-nojs-caps` is server-side feature detection. The page carries a few empty elements; CSS
`@supports` rules give each one a background image only when the browser understands the
feature; loading that image hits one tiny route that sets a cookie. From the next request on,
the server knows and every component emits only the markup that browser needs.

## 1. The beacons

`caps::beacons(&caps)` puts this at the end of `<body>` while the browser is still unknown:

```html
<div class="nojs-caps" aria-hidden="true">
  <i class="nojs-cap nojs-cap-probed"></i>
  <i class="nojs-cap nojs-cap-invokers"></i>
  …one per flag…
</div>
```

`caps::beacon_css()` is part of the stylesheet. The container is a fixed 1×1 px box with
`opacity: 0` and `pointer-events: none`; each beacon is a 1×1 block, so the browser must lay it
out and fetch its background:

```css
.nojs-cap-probed { background-image: url("/nojs/caps?flag=probed"); }
@supports selector(:popover-open) {
  .nojs-cap-popover { background-image: url("/nojs/caps?flag=popover"); }
}
```

`probed` has no `@supports` around it: it fires on every browser that loads CSS images at
all, and says "the answers below are complete". Once `probed` is in the cookie jar the beacon
elements are not emitted any more, so a known browser pays nothing.

## 2. The beacon route

`GET /nojs/caps?flag=<name>` answers `204 No Content` with `Cache-Control: no-store` and

```
Set-Cookie: nojs-cap-<name>=1; Path=/; Max-Age=2592000; SameSite=Lax
```

An unknown flag answers `404`. `caps::beacon_cookie(query)` is that logic as a function of the
raw query string; the `axum` feature wraps it in `caps::router()`, and
`axum-nojs/examples/hyper_server.rs` wires it by hand in six lines.

## 3. The cookie format

One cookie per flag, `nojs-cap-<name>=1`, never a list. The beacons load in parallel: seven
responses each writing `nojs-caps=<old list + me>` would overwrite one another and keep one flag.
Separate names cannot race. There is no negative cache: an unsupported feature has no cookie,
and `probed` says the beacons ran. Cookies last 30 days so a browser upgrade is re-detected.

`Caps::from_cookie_header(header)` reads the whole `Cookie:` value and returns the set.
Without `nojs-cap-probed=1` it returns `Caps::ASSUMED` instead, whatever else it finds.

## 4. The first-view problem

The first page a browser ever loads has no cookie. The beacons fire *while* it loads, so the
tailored markup only arrives on the second view. A `<meta http-equiv=refresh>` could force a
reload, but it would double the first-load cost for `curl` and crawlers, so it is not done.

Instead an unprobed browser gets `Caps::ASSUMED`: the features at baseline in every engine for
over two years (today: `popover` only). Everything younger waits for the cookie, so the first
and the second view differ only where the fallback is harmless. Browsers that never load CSS
images (`curl`, readers, some mail clients) stay on fallbacks for ever, which is right.

`?caps=popover,anchor` on any URL forces a set (`Caps::from_query`), so a page can be viewed
as any browser without touching cookies: useful for tests, screenshots and bug reports. A
forced set counts as probed.

## 5. What the beacons cost

**After the first visit, nothing.** Once the cookies are set the server sees `Probed` and
`beacons()` renders an empty string: no `.nojs-cap-*` elements, no rules, no image requests. The
cookies last 30 days (`Max-Age=2592000`); a browser whose cookies lapsed is simply probed again.

**On the first visit, one small request per supported flag**, made by the CSS engine after the
stylesheet parses, at image priority, after the page's own resources. Each answer is a
`204` with one `Set-Cookie` and no body. Two things keep that cheap:

- **Serve them from the page's own origin.** `BEACON_PATH` is a path, not a URL, so the
  requests reuse the page's connection, send its cookies and set cookies the page can read.
  Mount the route on the same server (`caps::router()` in Axum, or match `caps::BEACON_PATH`
  by hand as `examples/hyper_server.rs` does). A CDN in front must pass `/nojs/caps` through
  uncached: the answers are `Cache-Control: no-store` because they set a cookie.
- **Speak HTTP/2 or HTTP/3.** Over HTTP/1.1 a browser opens up to six connections per origin
  and queues the rest; over HTTP/2 or HTTP/3 every beacon is a stream on the one connection
  the page already opened. `examples/hyper_server.rs` serves HTTP/1.1 and HTTP/2 on one port
  (checked with `curl --http2-prior-knowledge`: page, script and a beacon on one connection).
  Browsers only use HTTP/2 over TLS and HTTP/3 over QUIC, so in production the TLS proxy in
  front (Caddy, nginx, a load balancer) terminates `h2`/`h3` and forwards to the app as `h2c`
  or HTTP/1.1; the app needs no QUIC stack of its own.

## 6. What each flag tests

| Flag | `@supports` test | Real feature | Shipped |
|---|---|---|---|
| `probed` | none, always fires | the beacons loaded at all | |
| `invokers` | `selector(::picker(select))` or `(-moz-appearance: none) and (view-transition-class: x)` | `<button command commandfor>` | Chrome 135, Firefox 144, Safari 26.2 |
| `anchor` | `(anchor-name: --x)` | CSS anchor positioning | Chrome 125, Firefox 147, Safari 26 |
| `details_content` | `selector(::details-content)` | `::details-content` | Chrome 131, Firefox 143, Safari 18.4 |
| `view_transitions` | `(view-transition-class: x)` | same-document `view-transition-name` | Chrome 125, Firefox 144, Safari 18.2 |
| `popover` | `selector(:popover-open)` | `popover` attribute | Chrome 114, Firefox 125, Safari 17 |
| `light_dark` | `(color: light-dark(#000, #fff))` | `light-dark()` (informational) | Chrome 123, Firefox 120, Safari 17.5 |
| `streaming_dsd` | `selector(:popover-open)` | `<template shadowrootmode>` | Chrome 111, Firefox 123, Safari 16.4 |
| `base_select` | `selector(::picker(select))` | `appearance: base-select`, `<selectedcontent>` | Chrome 135, Safari 27 |

CSS cannot test HTML attributes, so `invokers` and `streaming_dsd` are **proxies**: CSS
features that shipped in the same release or later. Both err on the safe side and never
claim a feature the browser lacks; the error bands are in `FINDINGS.md` under M1.

## 7. Adding a flag

1. Add a variant to `Cap` in `axum-nojs-caps/src/lib.rs` and to `Cap::ALL` (display order).
2. Give it a `name()` (snake case; it becomes the cookie, the URL and the class name), a
   `supports()` test (`None` only for `probed`) and a one-line `description()`.
3. If the test is a proxy, say so in the doc header and add the error band to `FINDINGS.md`.
4. Branch on `caps.has(Cap::New)` in the component, emitting one variant only.
5. `cargo test -p axum-nojs-caps`: the round-trip and one-rule-per-flag tests cover the new flag
   without further changes. The `/caps` page of the demo lists it automatically.

`Caps` is a `u16` bitset, so up to 16 flags fit; past that, widen the field.
