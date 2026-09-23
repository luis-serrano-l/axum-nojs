# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

`webonsive` is a zero-JavaScript component library for Rust servers (Axum + Maud). Every
component is a plain `fn name(id, ...) -> Markup`; interactivity comes from the HTML/CSS platform
(`<dialog>`, `popover`, invoker commands, `<details name>`, `<datalist>`, view transitions) and
ordinary form round trips. **No page may ever contain a `<script>` tag.** A test enforces this
(`demo/src/main.rs::tests::no_page_ships_script`). When something cannot be done without script,
the answer is a documented finding in README, not a script tag.

## Commands

```sh
cargo run -p demo                  # demo server at http://127.0.0.1:3000
cargo test                         # all tests, including the no-script test and doctests
cargo test -p demo no_page_ships_script   # the single enforcement test
cargo test -p webonsive --doc      # component doc examples
cargo clippy --all-targets         # must be clean before a roadmap milestone counts as done
```

## Workspace layout

- `webonsive/` – the library crate. Depends only on `maud`. No axum, no serde.
- `demo/` – Axum binary, one route per component. Handlers only parse input (query, form,
  cookie) and call `webonsive`; keep each route around 15 lines. The no-script test lives here
  and hits every route via `tower::oneshot`, so **add new demo routes to its path list**.

## Component conventions (follow exactly when adding one)

1. One component = one file `webonsive/src/<name>.rs`, registered in `lib.rs` as `pub mod` and
   re-exported with `pub use`.
2. File starts with a `//!` header: what it does, **Platform features** (with browser baseline
   versions), **Fallback**, and a runnable ```` ```rust ```` usage example (these are doctests).
3. CSS lives beside the component as `pub const CSS: &str` and must be appended to the array in
   `stylesheet()` in `lib.rs`; `layout()` inlines that once per page. Theming only through
   `--wo-*` custom properties defined in `layout.rs`.
4. Root element carries a single `wo-<component>` class; sub-parts use `wo-<component>-<part>`.
   Output should be readable via `curl`.
5. No macros beyond `html!`. Signature order: `id`, required args, then options.
6. Server-held state (theme, counter, active tab) travels via cookie or `?query=`; mutations use
   `<form method="post">` + redirect (Post/Redirect/Get), never GET side effects.
7. Update the README feature matrix and Findings when a component or its fallback changes.

## Roadmap

`ROADMAP.md` is the work queue: work top to bottom (M1 capability beacons → M2 streaming → M3
state model → M4 Blitz tests → M5 machine-readable spec → M6 release polish). A milestone is
done only when every box is ticked, clippy is clean, tests pass, and README is updated.
