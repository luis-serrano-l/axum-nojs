# Changelog

All notable changes to `loco-ui` and `loco-ui-caps`. Both crates share a version. The format
follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/); the project uses
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### `loco-ui`

- M33: a table's query keys carry its id, `<key>.<id>` like `per.<id>` and `edit.<id>`:
  `q.files`, `sort.files`, `dir.files`, `page.files`, `cols.files`, so two tables on one page
  sort, filter and page on their own. `table::Keys::new(id)` spells them. The bare `q`, `sort`,
  `dir`, `page` and `cols` are deprecated and still read, for one release, when the table's own
  key is absent. The load-more `Pager` keeps its bare `page`.
- M32, less code per page: `Form` holds any control (`.body(markup)` among its fields,
  `.switch(name, label)`) and `.get()` makes it a filter; `Page::invalid()` answers 422, so a
  POST handler returns `Result<Redirect, Page>`; `Posted` (feature `axum`) reads an urlencoded
  or multipart post by name (`get`, `all`, `pairs`, `files`, and a wizard's `step`/`skip`;
  `wizard::Posted` is deprecated); `ui.page` shows a pending flash unless the body already
  does; a card's footer packs at its end. `lui!` takes a markup block as an attribute value
  (`footer={ .. }`) and `body { .. }` among items, and names the fix when an item is given a
  component. **Breaking:** `ui.stack()`, `ui.cluster()` and `ui.grid(min)` take their content
  as `.body(..)` (`Stack gap=6 { .. }` in `lui!`).
- M32, one word per concept (`docs/api.md` has the table): `multiple` (was `multi`),
  `description` (`summary`, `note`, `message`), `help` (`hint`), `body` (`content`, EmptyState
  `text`), `action` (EmptyState `post`), `group` (Menu `heading`), `option` (ToggleGroup
  `item`), `link`/`links` (Palette `command`/`commands`), `accesskey` (Palette `key`, now a
  `&str`), `icon_only` and `aria_label` (Button `icon()` and `label`), `hide_progress()`
  (Wizard `progress(bool)`), `disabled_dates` (Calendar and DatePicker `disabled`). The old
  names are `#[deprecated]` for one release, and so is `layout_with` (use `Page::tokens`).
  `.icon(..)` takes an `Icon` or a glyph everywhere; `.badge(..)` any `Display`.
- **Breaking:** `ui.select(name, label)`, `ui.color(name, label)`, `ui.range(name, label)` and
  `ui.range_pair(name, label)` read their value from the query, with `.value(..)` (`.values(..)`
  for a pair) for a saved one; their `.label(..)` is gone. Select's id is `f-<name>` like a
  form field's, so the error summary's link reaches it, and it takes `.error(..)`.
- ContextMenu takes Menu's adders and modifiers (`.items(..)` is deprecated); `.id(..)` on
  Meter, Progress, Sidebar and NavMenu; `ErrorSummary::field(name, label)` writes "Label:
  message" as a form's summary does. `props_follow_the_conventions` keeps the API to the rules.
- Latency: `stylesheet()` built once and minified (`minify_css`); `enhance::slim` middleware
  answers an enhanced request without the inline stylesheet and sets `Vary: Lui-Enhance, Cookie`
  (`enhance::slim_html` without a framework); the script sends `Accept: text/html`, fetches
  user actions with `priority: "high"`, prefetches links under `data-lui-prefetch` on hover or
  focus and reuses the answer for a click within five seconds; `enhance::served()` is the
  script without comment lines and indentation, and the 10 KB budget applies to it.
- Enhancement script: `data-lui-target`, `data-lui-swap`, `data-lui-oob`, the `Lui-Enhance: 1`
  request header, busy state (`data-lui-busy`, `aria-busy`, disabled submit buttons,
  `data-lui-indicator`, `--lui-busy`), failed requests fall back to a navigation,
  `data-lui-push="false"`, `data-lui-replace`, Back/Forward restore from history state,
  `lui:swap` event. Size limit raised to 10 KB.
- `dialog`: `title` with a close control, `size` (`DialogSize::Sm|Md|Lg`), `danger`,
  `confirm(label, action)` footer as a real `<form method="post">`, `returns_to`,
  `cancel_label`, `closedby`. `button.lui-danger` in the base styles.
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
- `counter` takes `CounterOptions` (`min`, `max`, `step`, `typed`): buttons disabled at the
  bounds, a number field posting `op=set`, and `CounterOptions::apply` for the handler.
- `range::range_pair` (two thumbs on one track, `<name>_min`/`<name>_max`) and `range::order`.
- `color` takes `ColorOptions` (`presets` posting `<name>-preset`, `alpha` slider posting
  `<name>-alpha`); `color::hex_alpha`. The swatch paints through `--lui-color-value`.
- `select` takes `&[select::Group]` of `SelectOption` (`icon`, `content`) and `SelectOptions`
  (`search(action, query)`, `search_over`): optgroups, icons, and a GET filter box over 15
  options. The root is now a `span.lui-select` around the `<select id=name>`.
- Enhancement script: honours a submitter's `formmethod` and `formaction`; a search box
  followed by a submit button filters through it.
- **Breaking:** `counter`, `color` and `select` signatures changed.
- `flash` takes `FlashOptions` (`dismiss(href)`, `auto_hide`): levels `info|ok|warn|danger`
  from a `level:` prefix per line (`flash::stack`, `flash::parse`, `flash::Level`), several
  messages stacked, `role="alert"` for danger, a CSS fade for info and ok that reduced motion
  turns off. `--lui-warn` token (`Palette::warn`). **Breaking:** `flash` gained an argument;
  `Palette` gained a field.
- New components: `toasts` (`ToastOptions`; the flash cookie in a fixed corner stack that
  fades, pauses on hover, danger stays), `breadcrumbs` (folds the middle of a long trail
  into `<details>`), `skeleton` (`SkeletonOptions`; shimmer bars with `aria-busy`, used as the
  streamed slot placeholder in the demo), `empty_state` (`EmptyOptions`: icon, text, link,
  post), `stat` (`StatOptions`, `Trend`; `.lui-stat-grid`), `drawer` (`DrawerOptions`;
  sidebar above 60rem, modal drawer below, `:target` fallback) and `command_palette`
  (`palette::Command`, `PaletteOptions`, `palette::exact`, `palette::matches`; popover +
  datalist + GET, `accesskey`).
- Enhancement script: carries the toast list across a swap like the flash.
- `form` takes `&[FieldGroup]` (a `<fieldset>` and `<legend>` each, or `FieldGroup::plain`)
  and `FormOptions` (`submit`, `layout: FormLayout::Stacked|Inline`). `Field::new` with
  `value`, `error`, `required`, `help` (tied by `aria-describedby`) and `max_len` (a
  `maxlength` with an `<output>` counter). New `FieldKind`s: `Textarea` (grows with
  `field-sizing: content`), `File` (`accept`, `multiple`; the form becomes multipart), `Date`,
  `Time`. **Breaking:** the signature and `Field` changed.
- Enhancement script: an `<output for>` of a field with `maxlength` counts its characters;
  multipart forms are sent as `FormData`, so files survive an in-place submit.
- `wizard`: `Step::new` with `optional` (a `formnovalidate` Skip button posting `skip=1`) and
  `error` (marked in the step list and on the fieldset), a `<progress>` bar
  (`WizardOptions::progress`, on by default), a "Picked up where you left off" notice with
  Start over when the step came from the cookie (`UiState::remembered`), and
  `wizard::summary` for a review whose values link back to their steps. **Breaking:** `Step`
  has two more fields; build it with `Step::new`.
- `paged_table`: First and Last links, an ellipsis past seven pages, a jump-to-page form,
  counts with thousands separators (`paged_table::thousands`), and `PagedTableOptions::state`
  to remember the page size per table as `per.<id>` (`UiState::per_page`). The whole block is
  now the swap root, so the page links follow an in-place sort.

## [0.1.0] - 2026-09-23

First release. Nothing here is published to crates.io yet.

### `loco-ui-caps`

- `Caps` / `Cap` bitset of what a browser supports, learned with no script: `@supports` beacons
  in the page load one image each, the `/lui/caps?flag=x` route sets one cookie per capability.
- Plain functions for any server: `Caps::from_cookie_header`, `Caps::from_query` (`?caps=a,b`
  forces a set), `beacon_cookie`, `beacons`, `beacon_css`.
- `axum` feature: `Caps` extractor and `router()` for the beacon route.
- `examples/hyper.rs`: the whole protocol on raw hyper.

### `loco-ui`

- Components, each one file with a doc header (platform features with browser baselines,
  fallback, doctest) and its CSS beside it: `dialog`, `popover_menu`, `tabs`, `accordion`,
  `combobox`, `pager`, `form`, `counter`, `theme_toggle`, `flash`, `select`, `range`, `color`,
  `table`, `paged_table`, `wizard`, `slot` (streaming).
- Every page works with script disabled. One optional script, `enhance::JS` at
  `/lui/enhance.js`, makes forms and links inside a swap root (`id` + `data-lui="swap"`) update in
  place; a test proves no route ships any other `<script>`.
- `layout` page shell with cross-document view transitions, light/dark through
  `prefers-color-scheme` and `data-theme`; `layout::Tokens` (light and dark `Palette`, radius,
  space) and `layout_with` for another palette; every component colour is a `--lui-*` token.
- `UiState`: tabs, accordions, dialogs and wizard steps as `?tab.x=n`-style query keys mirrored
  in one `lui-ui` cookie; `prg_parts` / `prg` for Post/Redirect/Get with a one-shot flash.
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

- `loco-ui-test`: every demo route rendered through Blitz, layout assertions and a PNG per
  route and capability level under `tests/shots/`.
- `scripts/browser-check.mjs`: headless Firefox through geckodriver proves the enhancement
  script does its job; `scripts/verify.sh` runs everything.
