# webonsive

Zero-JavaScript interactive HTML components for Rust servers. Axum + Maud.

Every component is a plain function returning `Markup`. Interactivity comes from the HTML/CSS
platform and ordinary form round trips. No page ships a `<script>` tag, and a test enforces it.

```rust
use maud::html;
use webonsive::{layout, dialog, Theme};

let page = layout("Hello", Theme::Auto, html! {
    (dialog("hi", "Say hi", html! { p { "Hello from a <dialog>." } }))
});
```

## Run the demo

```sh
cargo run -p demo      # http://127.0.0.1:3000
cargo test             # includes: no route may contain "<script"
```

## How to read this crate

- One component = one file in `webonsive/src/`. Each starts with a `//!` header: what it does,
  the platform features it uses (with browser baseline), the fallback, a usage example.
- Signatures are uniform: `fn name(id, ...required, ...) -> Markup`. No macros beyond `html!`.
- Output HTML is semantic with one `wo-<component>` class per root. `curl` any page and read it.
- CSS lives beside its component as `const CSS`. Theming is via `--wo-*` custom properties only.

## Feature matrix

| Component | Platform feature | Baseline | Fallback | Needs JS? |
|---|---|---|---|---|
| Dialog | `<dialog>`, `command`/`commandfor` invokers, `closedby`, `<form method=dialog>` | dialog 2022; invokers Chrome 135 / Firefox 144 / Safari 26 | `:target` overlay via `href="#id"` link | No |
| Popover menu | `popover` + `popovertarget`, anchor positioning | popover 2024; anchors Chrome 125 / Safari 26, Firefox flag | UA-centred popover | No |
| Tabs | `<details name>`, `display: contents`, `::details-content` + `order` | details name 2024; ::details-content Chrome 131 / Firefox 138 / Safari 18.4 | plain accordion | No |
| Accordion | `<details name>`, `::details-content` transition | 2024 | non-exclusive `<details>` | No |
| Combobox | `<input list>` + `<datalist>`, `<search>` | 2020 / 2023 | none needed | Suggestions no; live results **yes** |
| Load-more list | links + `@view-transition { navigation: auto }` + `scroll-margin` | Chrome 126 / Safari 18.2, Firefox flag | plain navigation | Click-to-load no; scroll-to-load **yes** |
| Validated form | `required`/`pattern`/`min`, `:user-invalid`, Post/Redirect/Get | 2015 / 2023 | none needed | No |
| Counter | `<form method=post>`, `<button name value>`, cookie | forever | none needed | No |
| Theme toggle | `prefers-color-scheme`, `light-dark()`, cookie + `data-theme` | light-dark() 2024 | OS preference | No |

## Findings

**What works better than expected**
- Dialog, popover, tabs, accordion: fully interactive with zero round trips. Light dismiss,
  Escape, focus trapping, top layer, exclusivity: all free from the platform.
- View transitions make server round trips feel like in-place updates. The counter number
  morphs; the list grows without a flash.
- `:user-invalid` gives validation UX that used to need a library.

**What needs a fallback today**
- Invoker commands and anchor positioning are Chrome-first. Firefox 144+ has invokers; anchor
  positioning is still behind a flag there. The dialog's `:target` fallback covers it.
- CSS cannot feature-detect HTML attributes, so the dialog hides its fallback links behind
  `@supports (anchor-name: --x)` as a proxy. This is the ugliest thing in the crate.

**What is impossible without script**
- Filtering results as you type against server data. Datalist covers static suggestions only.
- Infinite scroll. Cumulative pages with one click per page is the ceiling.
- Optimistic UI, offline behaviour, undo without a round trip.
- Drag and drop, resizable panes, canvas or charts drawn from data.
- Keeping `<details>` state across navigations without a `?tab=` round trip.

**Verdict:** for content sites, admin panels, forms, settings pages and dashboards that refresh
per action, the platform is enough. For editors, real-time collaboration, and anything that
reacts per keystroke, it is not.

## Layout

```
webonsive/src/lib.rs        crate docs, re-exports, stylesheet()
webonsive/src/layout.rs     page shell + base CSS
webonsive/src/<name>.rs     one component each: dialog, popover, tabs, accordion,
                            combobox, pager, form, counter, theme
demo/src/main.rs            Axum routes, ≤15 lines each, plus the no-script test
```
