# Blocked

Questions only the owner can answer. Everything else in ROADMAP.md is done: what is left is
the name (M26), hosting and posting (M26), publishing (M13) and the Loco decision (M27), which
the rest of M27 waits on.

## Name, license and repository (M13, box 1), answered

Answered 2026-09-24: **license is MIT**. `license = "MIT"` is set in both manifests, and
`LICENSE` sits at the root with `luis-serrano-l` (the git user name) as the copyright holder.
Edit that line if you want your full name there.

Answered 2026-09-24: **the name is `axum-nojs`**. The crates are `axum-nojs`, `axum-nojs-caps`
and `axum-nojs-test` (not published); classes, tokens, cookies, attributes and routes use the
`nojs-` prefix (`.nojs-dialog`, `--nojs-accent`, `nojs-ui`, `data-nojs`, `/nojs/enhance.js`).
Both names were free on crates.io on 2026-09-24.

Answered 2026-09-24: **the repository is https://github.com/luis-serrano-l/axum-nojs**
(public), set as `repository` in both manifests.

## The publish itself (M13, box 4), on hold

On hold at the owner's request (2026-09-24). The rename is done and `repository` is set; publish only on the owner's yes. Never run on your behalf. M25 puts it after M26, whose naming box (below) may still change the crate names.

## Rename to `nojs-ui`? (M26, box 1), open

M26 asks whether the name should carry the differentiator more plainly now that `maud-ui`
exists: rename to **`nojs-ui`** (`nojs-ui-caps`, `nojs-ui-test`; Axum stays the `axum`
feature), or keep **`axum-nojs`**. The `nojs-` classes, `--nojs-*` tokens and `/nojs/` routes
stay either way, so only crate names, imports and docs change.

Suggested answer: **keep `axum-nojs` for now, and decide together with M27.** "nojs" is
already in the name, and the name you chose on 2026-09-24 is set in the manifests, the
repository and the docs. If Loco becomes the primary target (M27's first box), `nojs-ui` fits
better, because Axum would no longer be the only host (a `loco` feature beside `axum`); make
both decisions at once so the crates are renamed at most once, before the first publish.

## Launch: host the demo, post the announcement (M26, last box), open

These are outward actions, so they are yours to take or to approve one by one:

1. **Host the live demo.** Suggested: a small VM or a free-tier container host running
   `cargo run --release -p demo` behind a reverse proxy with HTTPS. The demo keeps uploads
   in memory, capped at 3 × 200 KB per visitor for 100 visitors, and sends a strict CSP. Nothing
   else needs configuring.
2. **Post the announcement.** The draft is in `docs/launch-post.md`, ready to edit. Suggested
   order: publish the crate first (M13, on hold above), then r/rust and This Week in Rust's
   call for submissions, with the demo link filled in.

Done locally: the crates.io keywords are `no-js`, `maud`, `ssr`, `components` and `axum`.
`progressive-enhancement` is longer than crates.io's 20-character limit, so the phrase is in
the description instead.

## Make Loco the primary target? (M27, box 1), open

You said Loco looks like the best fit and asked for M27 to be ready "if we go with that
decision". Every other M27 box depends on this one: a `loco` feature with an `Initializer`
that mounts `/nojs/enhance.js` and the caps beacon, handlers returning Loco's
`Result<Response>`, `validator` errors on the right fields, SeaORM's paginator behind the paged
table, a scaffold that generates Maud views, an example app and `docs/loco.md`.

Suggested answer: **yes, as a feature (`loco`) on the same crate, not a separate crate**, and
decide the crate name at the same time (the M26 question above). Loco controllers are Axum
handlers, so `Ui`, `Page`, `Redirect` and `Saved<T>` should work there already; M27 is wiring,
a generator and docs, not a rewrite. The Loco API names in ROADMAP M27 are from memory and must
be checked against loco-rs's source before building on them. A no keeps Axum (and any server
through the plain-string API) as the target, and M27 is dropped.

## Reply to Blitz issue #923

Posted on 2026-09-23 after the owner's approval:
https://github.com/DioxusLabs/blitz/issues/923#issuecomment-5800768284
