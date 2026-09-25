# Blocked

Questions only the owner can answer. The name, hosting and Loco questions were answered on
2026-09-24 (below). What is still with the owner: the publish (M13, on hold), posting the
announcement (M26), and turning on GitHub Pages and pushing once the snapshot exists.

## Name, license and repository (M13, box 1), answered

Answered 2026-09-24: **license is MIT**. `license = "MIT"` is set in both manifests, and
`LICENSE` sits at the root with `luis-serrano-l` (the git user name) as the copyright holder.
Edit that line if you want your full name there.

Answered 2026-09-24: the name was `axum-nojs`. **Renamed 2026-09-25 to `loco-ui`**: "nojs"
promised no JavaScript while an optional script ships. The crates are `loco-ui`,
`loco-ui-caps`, `loco-ui-macros` and `loco-ui-test` (not published); classes, tokens, cookies,
attributes and routes use the `lui-` prefix (`.lui-dialog`, `--lui-accent`, `lui-ui`,
`data-lui`, `/lui/enhance.js`), and the macro is `lui!`. `loco-ui` was free on crates.io on
2026-09-25. Yours: rename the GitHub repository to `loco-ui` (GitHub redirects the old URL)
before the next push, since the manifests and README already point there.

Answered 2026-09-24: **the repository is https://github.com/luis-serrano-l/loco-ui**
(public), set as `repository` in both manifests.

## The publish itself (M13, box 4), on hold

On hold at the owner's request (2026-09-24). The rename is done and `repository` is set; publish only on the owner's yes. Never run on your behalf. M25 puts it after M26, whose naming box (below) may still change the crate names.

## Rename to `nojs-ui`? (M26, box 1), answered

Answered 2026-09-24: keep `axum-nojs`. Reversed 2026-09-25: renamed to `loco-ui` (above).

## Launch: host the demo, post the announcement (M26, last box), partly answered

Hosting answered 2026-09-24: **a static snapshot on GitHub Pages.** Pages serves only static
files, so the snapshot shows every page as it renders, and `<dialog>`, `popover`, `<details>`
and tooltips still work. Forms, cookies and paging need the server, and a banner on every page
says so. The export (`scripts/snapshot.sh`) and the workflow (`.github/workflows/pages.yml`) are done.
Yours: Settings → Pages → Source "GitHub Actions", then push `main`; the site lands at
https://luis-serrano-l.github.io/loco-ui/ (put that link in `docs/launch-post.md`).

Posting stays with you and waits for the publish: the draft is `docs/launch-post.md`, and the
suggested order is crate first, then r/rust and This Week in Rust, with the Pages link filled in.

Done locally: the crates.io keywords are `no-js`, `maud`, `ssr`, `components` and `axum`.
`progressive-enhancement` is longer than crates.io's 20-character limit, so the phrase is in
the description instead.

## Make Loco the primary target? (M27, box 1), answered

Answered 2026-09-24: **yes, as a `loco` feature on `loco-ui`**, not a separate crate. The rest
of M27 is unblocked. The Loco API names in ROADMAP M27 are from memory and get checked against
loco-rs's source before anything is built on them.

## Reply to Blitz issue #923

Posted on 2026-09-23 after the owner's approval:
https://github.com/DioxusLabs/blitz/issues/923#issuecomment-5800768284
