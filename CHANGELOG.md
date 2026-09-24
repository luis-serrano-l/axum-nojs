# Changelog

All notable changes to `webonsive` and `wo-caps`. Both crates share a version. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### `webonsive`

- Enhancement script: `data-wo-target`, `data-wo-swap`, `data-wo-oob`, the `Wo-Enhance: 1`
  request header, busy state (`data-wo-busy`, `aria-busy`, disabled submit buttons,
  `data-wo-indicator`, `--wo-busy`), failed requests fall back to a navigation,
  `data-wo-push="false"`, `data-wo-replace`, Back/Forward restore from history state,
  `wo:swap` event. Size limit raised to 10 KB.
- `dialog`: `title` with a close control, `size` (`DialogSize::Sm|Md|Lg`), `danger`,
  `confirm(label, action)` footer as a real `<form method="post">`, `returns_to`,
  `cancel_label`, `closedby`. `button.wo-danger` in the base styles.
- `popover_menu` takes `&[MenuItem]` and `PopoverOptions`: links, post-form actions, headings,
  separators, nested submenus, icons, shortcut labels, disabled and danger items,
  `Placement::BottomStart|BottomEnd|Right`. The enhancement script walks an open menu with the
  arrow keys. **Breaking:** the `(text, href)` tuple form is gone.
- `tabs` takes `&[Tab]` and `TabsOptions` (`state`, `vertical`, `select_below`): badges,
  lazy tabs (`Tab::lazy`), a vertical strip, a `<select>` under 40rem, the open tab's underline carries
  `view-transition-name` (only the bar morphs, the title never moves). `UiState::path()`. **Breaking:** the `(title, body)` tuple form and
  the `Option<&UiState>` argument are gone.
- `accordion` takes `&[AccordionItem]` (`icon`, `summary` line) and `AccordionOptions`
  (`state`, `multi`, `controls`): several sections open at once through `?open.<group>=0,2`,
  "Expand all" / "Collapse all" links, nested accordions with their own key.
  `UiState::opens()` returns the list. **Breaking:** the `(title, body)` tuple form and the
  `Option<&UiState>` argument are gone.
- `combobox` takes `ComboboxOptions` (`query`, `suggestions` as `OptionGroup`s with
  `<optgroup>`, `results`, `selected`, `multi`, `create`, `label`, `placeholder`): the
  selection shows as removable chips and rides along as `sel` fields, results are links that
  select, a "Create" post row appears when nothing matches, results carry `aria-live`, and
  the enhancement script walks input and results with the arrow keys. **Breaking:** the
  `(action, name, options, value)` positional form is gone; `name` now comes before `action`.
- `UiState::link(key, "")` keeps `key=` in the URL so an explicit empty beats the cookie.
- `table` takes `&[Row]` (`key`, `detail`, `menu`) and more `TableOptions`: `cols` with
  `cols_from_query` and a "Columns" chooser, `bulk` (checkboxes owned by a post form through
  the `form` attribute), `csv`, `empty`, `loading` (skeleton rows, `aria-busy`);
  `Column::numeric` and `Column::width`. `PagedTableOptions::table` carries them through the
  pager. **Breaking:** rows were `Vec<Markup>`; `Column` has two more fields.
- `paged_table`: First and Last links, an ellipsis past seven pages, a jump-to-page form,
  counts with thousands separators (`paged_table::thousands`), and `PagedTableOptions::state`
  to remember the page size per table as `per.<id>` (`UiState::per_page`). The whole block is
  now the swap root, so the page links follow an in-place sort.

## [0.1.0] - 2026-09-23

First release. Nothing here is published to crates.io yet.

### `wo-caps`

- `Caps` / `Cap` bitset of what a browser supports, learned with no script: `@supports` beacons
  in the page load one image each, the `/wo/caps?flag=x` route sets one cookie per capability.
- Plain functions for any server: `Caps::from_cookie_header`, `Caps::from_query` (`?caps=a,b`
  forces a set), `beacon_cookie`, `beacons`, `beacon_css`.
- `axum` feature: `Caps` extractor and `router()` for the beacon route.
- `examples/hyper.rs`: the whole protocol on raw hyper.

### `webonsive`

- Components, each one file with a doc header (platform features with browser baselines,
  fallback, doctest) and its CSS beside it: `dialog`, `popover_menu`, `tabs`, `accordion`,
  `combobox`, `pager`, `form`, `counter`, `theme_toggle`, `flash`, `select`, `range`, `color`,
  `table`, `paged_table`, `wizard`, `slot` (streaming).
- Every page works with script disabled. One optional script, `enhance::JS` at
  `/wo/enhance.js`, makes forms and links inside a swap root (`id` + `data-wo="swap"`) update in
  place; a test proves no route ships any other `<script>`.
- `layout` page shell with cross-document view transitions, light/dark through
  `prefers-color-scheme` and `data-theme`; `layout::Tokens` (light and dark `Palette`, radius,
  space) and `layout_with` for another palette; every component colour is a `--wo-*` token.
- `UiState`: tabs, accordions, dialogs and wizard steps as `?tab.x=n`-style query keys mirrored
  in one `wo-ui` cookie; `prg_parts` / `prg` for Post/Redirect/Get with a one-shot flash.
- `Streamed`: out-of-order streaming through declarative shadow DOM slots, in-order fallback
  chosen from `Caps`.
- Options structs with `Default` and builder setters (`DialogOptions`, `PagerOptions`,
  `RangeOptions`, `TableOptions`, `PagedTableOptions`, `WizardOptions`).
- `spec::SPECS`: machine-readable component spec; `spec/components.json` and the README matrix
  are generated from it.
- Feature levels: none (Maud only), `http` (`prg` as `http::Response`, `Streamed` as a chunk
  stream), `axum` (extractors, `IntoResponse`, beacon route). `examples/hyper_server.rs` and
  `examples/axum_server.rs` show each.
- Docs: `docs/caps.md`, `docs/state.md`, `docs/theming.md`, `FINDINGS.md` (what works with no
  script, what needs a fallback, what is impossible, Blitz gaps with issue links).

### Test harness (not published)

- `webonsive-test`: every demo route rendered through Blitz, layout assertions and a PNG per
  route and capability level under `tests/shots/`.
- `scripts/browser-check.mjs`: headless Firefox through geckodriver proves the enhancement
  script does its job; `scripts/verify.sh` runs everything.
