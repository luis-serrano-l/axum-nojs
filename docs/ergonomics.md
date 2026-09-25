# Ergonomics: how a call site reads

> The M17 sections below describe the `name_with(&ui, …, XOptions)` API as it was. M18
> replaced it with builders on `ui`; the last section, "M18: every component starts from
> `ui`", is the current shape.

M17 is about call sites a newcomer reads once and understands. This page starts with the
audit (every public component signature and how the demo calls it), then the ten worst call
sites with a proposed rewrite each. Later M17 boxes turn the proposals into code and add a
before/after for every call site they change.

## Audit: public signatures

Arguments are counted after `caps`. "Setters" is the number of builder methods on the
component's options struct.

| Component | Signature after `caps` | Args | Options setters | Demo calls |
|---|---|---|---|---|
| `paged_table` | `id, href, columns, rows, total, options` | 6 | 6, plus `TableOptions` (10) nested | 1 |
| `table` | `id, href, columns, rows, options` | 5 | 10 | 0 (through `paged_table`) |
| `wizard` | `id, action, steps, state, options` | 5 | 4 | 1 |
| `drawer` | `id, label, nav, content, options` | 5 | 3 | 1 |
| `dialog` | `id, trigger, body, options` | 4 | 9 | 1 |
| `select` | `name, groups, selected, options` | 4 | 4 | 2 |
| `popover_menu` | `id, label, items, options` | 4 | 1 | 2 |
| `command_palette` | `id, action, commands, options` | 4 | 3 | 1 |
| `pager` | `href, items, total, options` | 4 | 2 | 1 |
| `layout_with` | `title, theme, tokens, body` | 4 | none | via `page_with` |
| `layout` | `title, theme, body` | 3 | none | via `page_with` |
| `tabs` | `name, tabs, options` | 3 | 3 | 3 |
| `accordion` | `group, items, options` | 3 | 3 | 2 |
| `form` | `action, groups, options` | 3 | 7 | 1 |
| `stat` | `label, value, options` | 3 | 4 | 4 |
| `counter` | `action, value, options` | 3 | 4 | 1 |
| `combobox` | `name, action, options` | 3 | 8 | 1 |
| `range` / `range_pair` | `name, value, options` | 3 | 3 | 1 each |
| `color` | `name, value, options` | 3 | 2 | 1 |
| `theme_toggle` | `action, current` | 2 | none | 1 |
| `flash` | `text, options` | 2 | 2 | 7 |
| `toasts` | `text, options` | 2 | 1 | 1 |
| `empty_state` | `title, options` | 2 | 4 | 1 |
| `skeleton` | `lines, options` | 2 | 2 | 1 |
| `slot` | `id, placeholder` | 2 | none | 1 |
| `breadcrumbs` | `trail` | 1 | none | 2 |

Patterns across the demo (21 GET routes):

- 16 routes call `page(&caps, &jar, title, body)`, and each reads the theme out of the
  cookie jar through `theme_of(&jar)`.
- 6 routes start their body with the same `flash(&caps, state.flash(), Default::default())`.
- 6 calls pass `.state(&state)` to hand a component the `UiState` the route already
  extracted.
- Every stateful component needs the reader to know the `UiState` key scheme (`tab.<name>`,
  `open.<group>`, `per.<table>`) to follow the deep links in the copy.

## The ten worst call sites

### 1. `/table`: `paged_table` with 6 arguments, 6 + 6 setters and 20 lines of plumbing

```rust
let sort = sort_from_query(&FILE_COLS, t.sort.as_deref(), t.dir.as_deref());
let cols = cols_from_query(&FILE_COLS, t.cols.as_deref());
let q = t.q.unwrap_or_default().to_lowercase();
let (per, pg) = (state.per_page("files").unwrap_or(10).clamp(1, 50), t.page.unwrap_or(1).max(1));
let rows: Vec<Row> = files.iter().skip((pg - 1) * per).take(per).map(/* … */).collect();
let options = TableOptions::default().cols(cols.as_deref()).choose_columns(true).bulk(/* … */).csv(/* … */).empty(/* … */).loading(/* … */);
(paged_table(&caps, "files", "/table", &FILE_COLS, &rows, files.len(),
    PagedTableOptions::default().sort(sort).filter(&q).page(pg).per_page(per).state(&state).table(options)))
```

A reader must know which query keys exist, that the rows passed in are the current page only,
and how the two options structs nest. **Proposed:** a `table::Query` parsed once from the raw
query string (sort, dir, filter, page, cols) that the component reads itself, and the component
slicing the page out of all rows:

```rust
(paged_table(&caps, "files", &FILE_COLS, &all_rows, PagedTableOptions::from(&query)
    .state(&state).bulk("/table/bulk", &BULK).csv("/table.csv").empty("No files match this filter.")))
```

### 2. `/inputs`: building `select` options takes three `map`/`zip` chains

```rust
let sizes: Vec<SelectOption> = SIZES.iter().map(|(v, l, i)| SelectOption::new(v, l).icon(i)).collect();
let countries: Vec<Vec<SelectOption>> = COUNTRIES.iter().map(|(_, cs)| cs.iter().map(/* same */).collect()).collect();
let groups: Vec<SelectGroup> = COUNTRIES.iter().zip(&countries).map(|((g, _), cs)| SelectGroup::new(g, cs)).collect();
(select(&caps, "size", &[SelectGroup::flat(&sizes)], v.size.as_deref().unwrap_or("m"), Default::default()))
```

**Proposed:** `From<(&str, &str)>` and `From<(&str, &str, &str)>` for `SelectOption`, and a
`select` that accepts plain options when there are no groups:
`(select(&caps, "size", &SIZES, current, Default::default()))`.

### 3. `/form`: every `Field` repeats its name for the value and the error

```rust
Field::new("email", "Email", FieldKind::Email).value(v.get("email")).required(true).error(err("email")),
```

Eight fields, each naming itself up to three times. **Proposed:** short constructors per kind
(`Field::email("email", "Email")`) and `FormOptions::values(&v).errors(&errors)` that fills each
field by name, so the field list only says what the form asks for.

### 4. `/wizard`: a hand-rolled field closure re-implements the form component

`wizard_steps` builds its own `label` + `input` + `aria-invalid` + `aria-describedby` + error
paragraph closure (9 lines) because a wizard step takes raw `Markup`. **Proposed:** a step's body
can be a `&[Field]`, rendered by the same code as `form`, with values and errors filled by name.

### 5. Every route: `page(&caps, &jar, title, body)` and `theme_of(&jar)`

Each route extracts `Caps`, a `CookieJar` and often `UiState`, then threads all three through.
**Proposed:** one `Page` extractor carrying caps, theme, UI state and flash, so a route reads
`async fn dialog_page(p: Page) -> Markup { p.render("Dialog", html! { … }) }`.

### 6. Six routes: `flash(&caps, state.flash(), Default::default())`

The same line opens six pages. **Proposed:** the page shell renders the flash when there is
one (the `Page` extractor above holds it), with `flash` kept for pages that place it elsewhere
or need `.dismiss()`.

### 7. `/dialog`: the trigger and the title say the same thing twice

```rust
(dialog(&caps, "confirm", "Delete account", body, DialogOptions::default().title("Delete account?")
    .size(DialogSize::Sm).danger(true).confirm("Delete account", "/dialog/delete")
    .returns_to("/dialog").cancel_label("Keep it").open(state.dialog() == Some("confirm"))))
```

Four positional arguments and seven setters; `open` needs the reader to know the `dialog` state
key. **Proposed:** `DialogOptions::state(&state)` reading `?dialog=<id>` itself (as `tabs` does),
the title defaulting to the trigger text, and `returns_to` defaulting to the current path.

### 8. `/settings`: forms inside tabs written by hand

Two raw `<form>`s with hidden inputs carrying the open tab and the other tab's value. **Proposed:**
`form` with `FormOptions::hidden(&[("tab", …)])` and `FieldKind::Checkbox`, so the tab bodies are
two `form` calls.

### 9. `/dashboard`: `stat` makes the reader pick a `Trend` that the delta already says

```rust
(stat(&caps, "Orders", if none { "0" } else { "3" }, StatOptions::default()
    .delta(if none { "-3" } else { "0" }, if none { Trend::Down } else { Trend::Flat })))
```

**Proposed:** `.delta("-3")` infers the trend from a leading `+`/`-` (zero is flat), with
`.trend()` for the rare override; the grid becomes four short lines.

### 10. `/tabs`: laziness spelled as a branch in the route

```rust
if open == 2 { Tab::new("Why", html! { … }) } else { Tab::lazy("Why") },
```

The reader must know which index is open and that `Tab::lazy` means "render nothing". **Proposed:**
`Tab::lazy_with("Why", || html! { … })`: the component calls the closure only for the open tab.

## Also noted

- `pager` (`/list`) makes the route build every row up to the current page; a `Fn(usize) ->
  Markup` item source would let the component ask only for what it shows.
- `accordion` nested inside another passes `AccordionOptions::default().state(&state)` twice;
  with the `Page` extractor the state could reach components without being threaded by hand.

## Done: two forms per component

Every component with options now comes in two forms, like `layout` / `layout_with`:
`dialog(&caps, "hi", "Say hi", body)` for the common case and
`dialog_with(&caps, "confirm", "Delete account", body, DialogOptions::default().danger(true))`
for the rest. `popover_menu` and `drawer` derive their id from the label
(`popover_menu(&caps, "Account", &items)` is `#account`). Before, every call ended in
`Default::default()` when it wanted nothing special:

```rust
// before
(dialog(&caps, "hi", "Say hi", html! { p { "Hello." } }, Default::default()))
(popover_menu(&caps, "account", "Account", &items, Default::default()))
(flash(&caps, state.flash(), Default::default()))
// after
(dialog(&caps, "hi", "Say hi", html! { p { "Hello." } }))
(popover_menu(&caps, "Account", &items))
(flash(&caps, state.flash()))
```

## Done: data from plain tuples

Items convert from tuples with `From`, and the builders stay for the rare setting:
`SelectOption` from `(value, label)` or `(value, label, icon)`, `select::Group` from a slice
or `(label, slice)`, `Column` from `(key, label)` (a plain column), `Row` from its cells,
`MenuItem` and `Command` from `(text, href)`, `Tab` and `AccordionItem` from
`(title, body)`, `Step` from `(title, body)`, `FieldGroup` from a slice or
`(legend, slice)`. `SelectOption::new`, `.icon` and `Group::new` / `flat` are now `const`.

```rust
// before: /inputs
let sizes: Vec<SelectOption> = SIZES.iter().map(|(v, l, i)| SelectOption::new(v, l).icon(i)).collect();
let countries: Vec<Vec<SelectOption>> = COUNTRIES.iter().map(|(_, cs)| cs.iter().map(|(v, l, i)| SelectOption::new(v, l).icon(i)).collect()).collect();
let groups: Vec<SelectGroup> = COUNTRIES.iter().zip(&countries).map(|((g, _), cs)| SelectGroup::new(g, cs)).collect();
// after
let sizes = SIZES.map(SelectOption::from);
let countries = COUNTRIES.map(|(group, cs)| (group, cs.map(SelectOption::from)));
let groups = countries.each_ref().map(|(group, cs)| SelectGroup::new(group, cs));
```

## Done: one extractor per page

`loco_ui::Ui` holds the caps, the theme from its cookie and the `UiState` with the flash. It
dereferences to `Caps`, so `&ui` goes wherever a component wants `&Caps`, and returning it
beside the page writes the state back. `ui.flash()` renders the flash banner and
`ui.layout(title, body)` wraps a page in the request's theme. Every demo page route now takes it.

```rust
// before
async fn dialog_page(caps: Caps, jar: CookieJar, state: UiState) -> Markup {
    page(&caps, &jar, "Dialog", html! {
        (flash(&caps, state.flash(), Default::default()))
        (dialog(&caps, "confirm", "Delete account", body, DialogOptions::default().open(state.dialog() == Some("confirm"))))
    })
}
// after
async fn dialog_page(ui: Ui) -> Markup {
    page(&ui, "Dialog", html! {
        (ui.flash())
        (dialog_with(&ui, "confirm", "Delete account", body, DialogOptions::default().open(ui.state.dialog() == Some("confirm"))))
    })
}
```

## Done: work moved into the components

Where a route used to prepare data for a component, the component now does that work
itself. Each move has a test in the component's own file.

| Route | Before | After | Test |
|---|---|---|---|
| `/table` | parse five query keys by hand, clamp the remembered page size, slice the page out of the rows | `TableQuery::parse(raw)` once; `PagedTableOptions::query(&t).state(&ui.state)`; pass every row and the component slices | `paged_table::tests::the_query_state_and_all_rows_are_enough` |
| `/table.csv` | the same parsing again | `t.sort(&FILE_COLS)`, `t.cols(&FILE_COLS)` | |
| `/form` | each field repeats its name for `.value(v.get(..))` and `.error(err(..))` | fields list only what the form asks for; `FormOptions::values(&v).errors(&errors)` fills them by name | `form::tests::values_and_errors_fill_fields_by_name` |
| `/wizard` | a hand-written 9-line label + input + `aria-*` + error closure | `form::fields(&[FieldGroup::plain(&ACCOUNT)], FormOptions::default().values(data).errors(errors))` | the same test |
| `/settings` | two raw `<form>`s with hidden inputs and a hand-made checkbox | two `form_with` calls, using `FieldKind::Hidden`, `FieldKind::Checkbox` and `FormOptions::id` (two forms, one action) | `form::tests::checkbox_and_hidden_fields` |
| `/dialog` | `.returns_to("/dialog").open(ui.state.dialog() == Some("confirm"))` | `.state(&ui.state)` | `dialog::tests::state_opens_the_named_dialog_and_returns_to_its_page` |
| `/dashboard` | `.delta(if none { "-3" } else { "0" }, if none { Trend::Down } else { Trend::Flat })` | `.delta(if none { "-3" } else { "0" })`: the sign is the trend, `.trend()` overrides it | `stat::tests::the_sign_of_the_delta_is_the_trend` |
| `/tabs` | `if open == 2 { Tab::new(..) } else { Tab::lazy(..) }` | `Tab::lazy_with("Why", &\|\| html! { … })`, called only for the open tab | `tabs::tests::a_lazy_tab_renders_only_when_open` |

```rust
// before: /table
let sort = sort_from_query(&FILE_COLS, t.sort.as_deref(), t.dir.as_deref());
let cols = cols_from_query(&FILE_COLS, t.cols.as_deref());
let q = t.q.unwrap_or_default().to_lowercase();
let (per, pg) = (ui.state.per_page("files").unwrap_or(10).clamp(1, 50), t.page.unwrap_or(1).max(1));
let rows: Vec<Row> = files.iter().skip((pg - 1) * per).take(per).map(/* … */).collect();
let options = TableOptions::default().cols(cols.as_deref())/* … */;
(paged_table_with(&ui, "files", "/table", &FILE_COLS, &rows, files.len(),
    PagedTableOptions::default().sort(sort).filter(&q).page(pg).per_page(per).state(&ui.state).table(options)))
// after
let t = TableQuery::parse(raw.as_deref().unwrap_or(""));
let files = files(t.sort(&FILE_COLS), &t.filter.to_lowercase());
let rows: Vec<Row> = files.iter().map(/* … */).collect();
(paged_table_with(&ui, "files", "/table", &FILE_COLS, &rows, files.len(),
    PagedTableOptions::default().query(&t).state(&ui.state).table(options)))
```

Things that were changed or left alone:
- `Tab::lazy` is gone. `lazy_with` does the same job without the branch in the route.
- `StatOptions::delta` takes one argument now.
- The wizard's fields now have ids starting with `f-`, the same as the form component's
  (`scripts/browser-check.mjs` follows).
- `pager` is unchanged: its route builds rows up to the current page in three lines, so a
  closure-based source would not make the call shorter.

## Done: names that read like HTML

- **Flags take no argument.** A setter that was always called with `true` now just switches
  its setting on:
  - `Field::required()`
  - `DialogOptions::danger()`, `MenuItem::danger()` and `MenuItem::disabled()`
  - the `multi()` setters on the combobox and the accordion
  - `AccordionOptions::controls()`
  - `TabsOptions::vertical()` and `TabsOptions::select_below()`
  - `TableOptions::choose_columns()`
  - `FlashOptions::auto_hide()`
  - `StatOptions::down_is_good()`
  - `SkeletonOptions::heading()`
  - `CounterOptions::typed()`
  - `DrawerOptions::sidebar()`
  - `Step::optional()`
- **A `bool` stays where a route passes a condition.** These are `open(..)`, `loading(..)`,
  a wizard step's `error(..)` and `progress(..)`.
- **`Field::max_len` is now `Field::maxlength`**, the attribute it sets.
- **`Field::error` takes the message** (`.error("Too short.")`) rather than an `Option`.
  Messages that come from the server go through `FormOptions::errors` instead.
- **Doc examples** say in a comment what a literal means where the reader would otherwise
  have to look in another file: `(key, descending)`, `(label, value, step)`, `open.faq=0,2`.

```rust
// before
Field::new("handle", "Handle", kind).value(v.get("handle")).required(true).error(err("handle")),
Field::new("bio", "Bio", FieldKind::Textarea { rows: 3 }).value(v.get("bio")).max_len(160),
(dialog_with(&ui, "confirm", "Delete account", body, DialogOptions::default().danger(true)
    .confirm("Delete account", "/dialog/delete").returns_to("/dialog").open(ui.state.dialog() == Some("confirm"))))
// after
Field::new("handle", "Handle", kind).required(),
Field::new("bio", "Bio", FieldKind::Textarea { rows: 3 }).maxlength(160),
(dialog_with(&ui, "confirm", "Delete account", body, DialogOptions::default().danger()
    .confirm("Delete account", "/dialog/delete").state(&ui.state)))
```

## Done: README's first example

The first example used to build a page from `Caps::all()` and `layout(&caps, "Hello",
Theme::Auto, …)`, with no route and no server around it. Now it is a whole Axum route:
`async fn hello(ui: Ui) -> Markup { ui.layout("Hello", html! { (dialog(&ui, …)) }) }`, followed
by the router with the beacon and script routes. That is five lines a reader can paste, and
none of them is plumbing. `loco-ui/examples/axum_server.rs` follows the same shape.

## M18: every component starts from `ui`

After M17 the owner read the demo and found it still heavy: a 70-name import line,
`x_with(&ui, …, XOptions::default()…)` on every call, `.state(&ui.state)` repeating `ui`, and
cookies parsed by hand in half the handlers. M18 makes every component a method on `Ui` that
returns a builder rendering inside `html!`, with one `use loco_ui::prelude::*`.

```rust
// before
use loco_ui::{Ui, dialog_with, dialog::{DialogOptions, DialogSize}, prg, /* 60 more */};
async fn dialog_page(ui: Ui) -> Markup {
    page(&ui, "Dialog", html! {
        (dialog_with(&ui, "confirm", "Delete account", body, DialogOptions::default()
            .title("Delete account?").size(DialogSize::Sm).danger()
            .confirm("Delete account", "/dialog/delete").cancel_label("Keep it").state(&ui.state)))
    })
}
// after
use loco_ui::prelude::*;
async fn dialog_page(ui: Ui) -> Page {
    page(&ui, "Dialog", html! {
        (ui.dialog("Delete account").id("confirm").title("Delete account?").small().danger()
            .confirm("Delete account", "/dialog/delete").cancel("Keep it").body(body))
    })
}
```

Lists are built item by item, and a modifier applies to the item added last:

```rust
// before
const ACCOUNT: [Field; 2] = [Field::new("name", "Name", FieldKind::Text).required(), Field::new("email", "Email", FieldKind::Email).required()];
(form_with(&ui, "/signup", &[FieldGroup::plain(&ACCOUNT)], FormOptions::default().submit("Sign up")))
(tabs_with(&ui, "demo", &[Tab::new("Install", a), Tab::new("Use", b).badge(3)], TabsOptions::default().state(&ui.state)))
// after
(ui.form("/signup").text("name", "Name").required().email("email", "Email").required().submit("Sign up"))
(ui.tabs("demo").tab("Install", a).tab("Use", b).badge(3))
```

Handlers lose their cookie plumbing. `Page` writes back the UI state and clears a shown flash
(no more `(ui, markup)` tuples); `Redirect` is Post/Redirect/Get with stacked messages;
`Saved<T>` keeps a value in a cookie named after its type:

```rust
// before
async fn counter_submit(jar: CookieJar, Form(f): Form<CounterOp>) -> (CookieJar, Redirect) {
    let n: i64 = jar.get("count").and_then(|c| c.value().parse().ok()).unwrap_or(0);
    let n = COUNTER.apply(n, &f.op, f.value);
    (jar.add(Cookie::new("count", n.to_string())), Redirect::to("/counter"))
}
// after
async fn counter_submit(ui: Ui, Saved(c): Saved<Count>, Form(f): Form<CounterOp>) -> Redirect {
    let n = counter(&ui, c.n).apply(&f.op, f.value);
    ui.redirect("/counter").save(&Count { n })
}
```

Components read their own input from the request: the table its sort, filter, page and
columns (`t.sort()`, `t.filter()` tell the route how to fetch), the combobox `?q` and `?sel`,
the pager `?page`, the select its filter box, the palette whether `?q` names a command
(`.exact()`), the dialog and drawer `?dialog=`. The table and its CSV export share one
`files_table(&ui)`; the wizard's review step is generated from the steps' fields.

| | Before | After |
|---|---|---|
| `demo/src/lib.rs` | 1011 lines | 915 lines (the 180 lines of tests unchanged) |
| Import lines naming `loco_ui` items | 1 line of about 70 names | `prelude::*` plus 3 lines for types a helper names |
| Hand-parsed cookies (`settings`, `count`, `inputs`, `wizard`, `notes`) | 5 parsers | 0: `Saved<T>` |
| `(ui, markup)` tuples and `prg::<Body>(..)` calls | 9 and 12 | 0 and 0 |

## M28: components written like elements

The owner liked how maud-ui's `Props` put a name on every value, and wanted the call to read
like the Maud around it (`a href=".."`). `lui!` is `html!` with components as elements; it
expands to the builder chain, so both forms stay one code path and the dot form remains for a
builder a route keeps in a variable.

```rust
// before: the board built in a loop, outside the markup
let mut k = ui.kanban("/kanban");
for (lane, title) in LANES {
    k = k.column(lane, title);
    if lane == "doing" {
        k = k.limit(2);
    }
    for (key, _) in board.0.iter().filter(|(_, l)| l == lane) {
        if let Some((key, text, note)) = CARDS.iter().find(|c| c.0 == key) {
            k = k.card(key, text).description(note);
        }
    }
}
// after: items from data, inline
let k = lui! {
    Kanban("/kanban") {
        @for (lane, title) in LANES {
            column (lane) (title) limit=[(lane == "doing").then_some(2)] {
                @for (key, _) in board.0.iter().filter(|(_, l)| l == lane) {
                    @if let Some((key, text, note)) = CARDS.iter().find(|c| c.0 == key) {
                        card (key) (text) description=(note);
                    }
                }
            }
        }
    }
};
```

```rust
// before
(ui.dialog("Delete account").id("confirm").title("Delete account?").small().danger()
    .confirm("Delete account", "/dialog/delete").cancel("Keep it")
    .body(html! { p { "This cannot be undone." } }))
// after
Dialog("Delete account") id="confirm" title="Delete account?" small danger
    confirm=("Delete account", "/dialog/delete") cancel="Keep it" {
    p { "This cannot be undone." }
}
```

| | Before | After |
|---|---|---|
| Demo snippets in `lui!` | 0 | 41 blocks; the rest keep a builder in a variable |
| `let mut` builders mutated in a loop, in snippets | 2 (kanban, upload) | 0 |
| Route lines (`demo/src/routes/`) | 1791 | 1798: nested blocks take a line more, loops a few less |
| Where to read a component's options | its **Setters** paragraph | the same, plus `loco_ui::props()`, `spec/components.json` and a props table on each demo page |
