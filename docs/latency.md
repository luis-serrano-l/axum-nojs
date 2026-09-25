# Latency: what moved the numbers

A loco-ui page is one server round trip, so the round trip is the whole experience. This
page collects what was measured in M16, what moved and what did not, and the order to apply it
to your own server. The raw numbers and the reasoning are in `FINDINGS.md` (M16 sections).

How it was measured: `scripts/bench.sh` starts the release demo on port 3001 and reports curl
p50/p95 time to first byte and full response, cold (new connection) and warm (kept alive),
plus Firefox navigation timing (`scripts/bench-nav.mjs`, median of five).
`cargo bench -p loco-ui` times the render functions with criterion. All on loopback: real
networks make the byte savings count for more, not less.

## What moved

| Change | Before | After |
|---|---|---|
| Stylesheet built once (`OnceLock`), release profile `lto = "fat"` | `/` warm TTFB 0.33 ms | 0.17 ms |
| gzip for bodies of known size (`tower-http` `CompressionLayer`) | `/` 48.9 KB | 10.5 KB on the wire |
| Stylesheet minified once per process (`minify_css`) | `/` 48.9 KB plain | 41.8 KB (8.8 KB gzipped) |
| Firefox DOMContentLoaded on `/` (the two above together) | 66 ms | 33 ms |
| `enhance::slim`: enhanced requests get the page without its stylesheet | tab swap 39.9 KB | 3.2 KB (1.2 KB gzipped) |
| `<link rel="prefetch">` on the index | `/dialog` 24–46 ms, 8.9 KB | from cache, 0 bytes |
| `data-lui-prefetch`: fetch on hover/focus, reuse on click | request starts on click | starts on hover; the click reuses it (one request, checked in Firefox) |
| Streamed pages flush `<head>` as the first chunk | head and shell in one chunk with the body | first paint 50 ms while the last slot lands at 2 s |
| `layout` sizes its buffer once | 15.6 µs per page | 11.9 µs |
| `UiState` borrows until it must decode | 884 ns per request | 596 ns |
| HTTP/2 in the hyper example | HTTP/1.1 only: up to six connections | page, script and beacons on one connection |

## What did not

- **Brotli** at `tower-http`'s default quality came out slightly *larger* than gzip on these
  pages. Both are offered; the browser picks.
- **Streaming `/table`**: its body awaits nothing, and a streamed body loses `Content-Length`
  and compression (51 KB on the wire instead of 9 KB). Stream only what awaits I/O.
- **A size hint on the 1 000-row table** made it 12 % slower: the estimate overshot, and a
  large growing `String` reallocates in place without copying. Maud's own hint (the size of
  the template's literals) is already right for components.
- **Lazy or low-priority beacons**: they are CSS background images, already fetched late and
  never blocking paint, and gone once the browser is probed. Nothing to gain.
- **One merged caps cookie**: the beacons answer in parallel, so merging flags server-side
  would race and lose one. Per-flag cookies stay.
- **Prerender**: only speculation rules can do it, and those are a `<script>`. Out of bounds.
- **The handler itself** was never the problem: every route answers in well under a
  millisecond. The browser parsing and painting the page is where the time goes.

## Apply them in this order

1. **Measure** your own routes first (curl `-w "%{time_starttransfer}"` and the Navigation
   Timing entry in the browser). A change that does not move a number is not worth keeping.
2. **Compress** whole responses: gzip or br for any body of known size. Pages are mostly
   inline stylesheet and shrink to a fifth. Skip streamed bodies, or their chunks are held.
3. **Mount `enhance::slim`** so enhanced swaps skip the stylesheet, and keep the
   `Vary: Lui-Enhance, Cookie` it sends: every page depends on cookies.
4. **Serve over HTTP/2 or HTTP/3** from the same origin as the page, usually by letting the
   TLS proxy in front terminate them (`docs/caps.md`, section 5).
5. **Stream** routes whose body awaits a database or another service: `Streamed` flushes the
   head first, `slot`s fill as they land. Leave in-memory pages whole.
6. **Prefetch** where the next click is predictable: `<link rel="prefetch">` for a small set
   of likely pages, `data-lui-prefetch` around a tab strip or a list of links.
7. **Build with** `lto = "fat"`, `codegen-units = 1`, `panic = "abort"`. The library already
   caches and minifies the stylesheet; nothing to do there.
