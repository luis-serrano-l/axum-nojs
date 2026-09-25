# The API: rules you can rely on

Every component is reached the same way and takes its options the same way. This page lists the
rules, the few places that break them on purpose (and why), and every rename from M32. The
rules are checked: `props_follow_the_conventions` in `loco-ui/src/lib.rs` reads
`loco_ui::props()` and fails on a deviation that is not on one of its allow-lists, each entry
with the one-line reason repeated under "Kept, because" below.

## Getting a component

`ui.<name>(required…)` returns a builder; chain setters, then put it in `html!` (it is `Render`).
In `lui!` the same component is `Name(required…)` with its setters as attributes:

```rust
use loco_ui::prelude::*;
let ui = Ui::from_request("/", "", "");
let a = html! { (ui.button("Save").primary().small()) };
let b = lui! { Button("Save") primary small; };
assert_eq!(a.into_string(), b.into_string());
```

## Constructor arguments

- **Text first.** What the component says or is called: `ui.button("Save")`,
  `ui.alert("Heads up")`, `ui.menu("Account")`, `ui.dialog("Delete")`, `ui.chart("Visitors")`,
  `ui.stat("Revenue", "$48k")`.
- **Form controls take `(name, label)`**: `ui.input("email", "Email")`,
  `ui.select("plan", "Plan")`, `ui.color("accent", "Accent")`, `ui.range("volume", "Volume")`,
  `ui.range_pair("price", "Price")`, `ui.toggle_group("align", "Alignment")`,
  `ui.date_picker("due", "Due date")`, `ui.input_otp("code", "Code")`,
  `ui.radio_group("size", "Size")` (the label is the group's legend). The label is visible,
  in a `div.lui-field` like a form field's.
- **A control's value comes from the request.** `ui.select("plan", "Plan")` shows the query's
  `plan` as chosen, `ui.range("volume", "Volume")` the query's `volume`; `.value(..)` gives a
  saved value instead (`.values(low, high)` for a range pair).
- **Everything else is a setter.**

## Ids

- An id you do not care about is derived from the label with `loco_ui::slug`:
  `ui.menu("Account")` is `#account`, `ui.meter(..).label("Disk")` is `#lui-meter-disk`.
- A form control's id is `f-<name>` (`#f-email`, `#f-plan`); the error summary links there.
- `.id(..)` overrides a derived id wherever there is one (Menu, ContextMenu, Palette, Dialog,
  Drawer, Tooltip, Chart, Card, Button, Input, Form, Meter, Progress, Sidebar, NavMenu). Two
  components with the same label on one page need it.

## Kinds of setter

Every setter is one of six kinds (`props::PropKind`), listed in each builder's `PROPS`, in
the spec JSON and in the demo's props tables:

| Kind | Takes | Example |
|---|---|---|
| value | text, markup, or several values together | `.description("One line.")`, `.body(html! { .. })` |
| number | one number | `.step(5)`, `.gap(3)` |
| switch | nothing: switches something on, off until called | `.primary()`, `.multiple()`, `.required()` |
| condition | one `bool`, for a route that decides from state | `.open(show)`, `.loading(busy)` |
| item | adds one entry to the component's list | `.tab("Use", body)`, `.link("Profile", "/profile")` |
| modifier | changes the entry added last | `.badge(3)`, `.icon(Icon::Mail)`, `.danger()` |

In `lui!` a switch is a bare attribute (`multiple`), a condition `open[show]`, an item a line
in the block (`link "Profile" "/profile" icon="@";`), and a block that is not items goes to
`.body(..)`:

```rust
use loco_ui::prelude::*;
let ui = Ui::from_request("/", "", "");
let menu = lui! { Menu("Account") {
    group "Signed in as Ada";
    link "Profile" "/profile" icon=(Icon::User) shortcut="g p";
    separator();
    action "Sign out" "/logout" danger;
} };
assert!(menu.into_string().contains(r#"action="/logout""#));
```

## One word per concept

| Word | Means | Where |
|---|---|---|
| `multiple` | several may be chosen or open at once (the HTML attribute) | Accordion, Combobox, ToggleGroup, Input, Upload, Form |
| `description` | a line of text under a title | Card, Alert, Chart, Accordion item, Stat, Kanban card, ErrorPage |
| `help` | small print under a field | Input, Form, RadioGroup, Upload |
| `body` | the main markup, what a `lui!` block holds | Card, Alert, Dialog, Drawer, ContextMenu, Button, SelectOption, EmptyState |
| `action` | a button posting to a URL | Menu, ContextMenu, EmptyState |
| `link` / `links` | a destination; `links` for many from data | Menu, ContextMenu, Breadcrumbs, Sidebar, NavMenu, Palette, EmptyState |
| `group` | a heading over the entries after it | Form, Menu, ContextMenu, Sidebar, Palette (Select and Combobox: an `<optgroup>`) |
| `option` | one choice with its value | ToggleGroup, RadioGroup |
| `accesskey` | the access key (the HTML attribute) | Button, Palette |
| `icon(..)` | an `Icon`, or a glyph or emoji as text | Alert, EmptyState, SelectOption, Accordion, Menu, ContextMenu, Sidebar, ToggleGroup |
| `icon_only` | a square button holding only an icon | Button |
| `aria_label` | the accessible name (the HTML attribute) | Button |
| `badge(..)` | a count or short text after an entry, any `Display` | Tabs, Sidebar |
| `value` / `error` | a control's value and its server message | Input, Select, Color, Range, RadioGroup, Form |
| `disabled` / `disabled_dates` | the element is not usable / which days cannot be picked | Button, menus / Calendar, DatePicker |
| `hide_progress` | a switch that turns off something shown by default | Wizard |

## Kept, because

The entries below are the allow-lists of `props_follow_the_conventions`, with the same
reasons, plus the meter's minimum, which no rule flags.

**Constructor order**

- `ui.upload(action, name)`: the action comes first like every form-posting widget
  (`ui.counter`, `ui.kanban`, `ui.palette`); the name of its file field second.
- `ui.meter(value, min, max)`: a `<meter>` has a minimum that matters (a temperature, a score
  from 1); `ui.progress(value, max)` has none, as `<progress>` has none.
- `ui.table(id, href)` and `ui.wizard(id, action)`: the id keys the component's query
  parameters (`sort.<id>`, `step.<id>`), so it is required: two tables on one page must not
  share them.
- `SelectOption::new(value, text)`: value then text, as `<option value>` and the
  `(value, text)` tuples every list of options takes.

**Lists in one call** (everything else has an adder per entry)

- `options`, `group`, `groups` (Select, Combobox), `select` (Form): options come from data,
  and tuples are options.
- `results` (Combobox): the server's matches for the query.
- `presets` (Color): a fixed palette.
- `values` and `errors` (Form, Wizard): the POST body and its validation errors are not in
  `Ui`, so the route passes them in; `.value(..)` and `.error(..)` do one field.
- `rows` (Table), `bulk` (Table), `menu` (Row): a table's rows come from a slice of records,
  and a row (itself one entry of the table) takes its menu whole.
- `submenu` (Menu) and `panel` (NavMenu): a nested menu is given whole as one entry of its
  parent.
- `links` (Palette): destinations from data, beside `.link(..)` for one.

**Same name, other shape**

- Form's field adders (`text`, `email`, `password`, `number`, `pattern`, `textarea`, `date`,
  `time`, `datetime`, `file`) take the field's name and label, then Input's arguments; Input's
  own are switches or values on one field.
- `submit`: Button's switch sets `type="submit"`; Form's value is its button's text.
- `search`: Input's switch sets `type="search"`; Select's value is its filter's action.
- `min`, `max`: a date on the date controls, a number on the numeric ones.
- `step`: a wizard's step, or the `step` of a numeric control.
- `rows`: Table's rows from data; Pager's closure renders one page of them.
- `value`: text on the text controls, a number on the slider. `values`: the POST body on
  Form and Wizard, a row's cells as text, a range pair's two numbers.
- `item`: what the component lists (a titled section, a term and its detail, a marquee entry).
- `link`, `action`: EmptyState holds one of each; the menus add one per call.
- `field`: RecordPage adds a field (label, value); ErrorSummary names one (name, label).

**Setter named otherwise than its attribute**

- `submit`, `reset` (Button), `email`, `password`, `search` (Input), `multiple` (ToggleGroup):
  named after the `type` they set.
- `pressed`, `current` (Button): `.pressed(on)` and `.current(on)` read better than
  `aria_pressed`, and say what `aria-pressed` and `aria-current` mean.
- `label` (Combobox): a form control's name is its label; here it is visually hidden, so it is
  `aria-label`.
- `length` (InputOtp): the number of digits, which sets `maxlength`.
- `duration` (Marquee): a custom property in `style`.
- `link` (Sidebar, NavMenu), `home` (ErrorPage), `edit`, `back`, `delete` (RecordPage): a link's
  `href` or a form's `action`.
- `option`, `value` (ToggleGroup): the option's `value`; the option with this value is
  `checked`.

**Ids**

- Table: its id is the constructor's first argument. Kanban: the root's id comes from the
  action, the columns' from their keys. SettingsPage: section anchors from their titles,
  linked from its own navigation.

**Other names**

- `Ui::link_with(key, value)`: this page's URL with one parameter set, the pair of
  `link_without`; not a `_with` twin of a plainer function.
- `Table::paged_from(&PagerMeta)` (feature `loco`): takes Loco's own pager metadata as it
  comes from a paginated query, beside `.paged(total)` for anything else.

## Before and after (M32)

The old names still compile for one release, marked `#[deprecated]` with the new name in the
warning. The three constructors changed outright (the crate is unpublished).

| Before | After | On |
|---|---|---|
| `.multi()` | `.multiple()` | Accordion, Combobox, ToggleGroup |
| `.icon()` (no argument) | `.icon_only()` | Button |
| `.label("More")` | `.aria_label("More")` | Button |
| `.content(markup)` | `.body(markup)` | Button, SelectOption |
| `.text(markup)` | `.body(markup)` | EmptyState |
| `.summary(text)` | `.description(text)` | Accordion |
| `.note(text)` | `.description(text)` | Stat, Kanban card |
| `.message(text)` | `.description(text)` | ErrorPage |
| `.hint(text)` | `.help(text)` | Upload |
| `.post(label, action)` | `.action(label, action)` | EmptyState |
| `.heading(text)` | `.group(text)` | Menu |
| `.item(value, text)` | `.option(value, text)` | ToggleGroup |
| `.key('k')` | `.accesskey("k")` | Palette |
| `.command(label, href)`, `.commands(list)` | `.link(label, href)`, `.links(list)` | Palette |
| `.progress(false)` | `.hide_progress()` | Wizard |
| `.disabled(predicate)` | `.disabled_dates(predicate)` | Calendar, DatePicker |
| `.items([MenuItem::link(..), ..])` | `.link(..)`, `.action(..)`, `.group(..)`, `.separator()` and the modifiers | ContextMenu |
| `layout_with(&caps, title, theme, &tokens, body)` | `ui.page(title, body).tokens(&tokens)` | layout |
| `.icon(Icon)` here, `.icon("📦")` there | `.icon(..)` takes either everywhere | Alert, EmptyState, SelectOption, Accordion, Menu, Sidebar, ToggleGroup |
| `.badge(3)` on Tabs, `.badge("12")` on Sidebar | `.badge(..)` takes any `Display` on both | Tabs, Sidebar |
| `ui.select("plan", "pro").label("Plan")` | `ui.select("plan", "Plan").value("pro")` | Select (breaking) |
| `ui.color("accent", "#2f5bea").label("Accent")` | `ui.color("accent", "Accent").value("#2f5bea")` | Color (breaking) |
| `ui.range("volume", 40).label("Volume")` | `ui.range("volume", "Volume").value(40)` | Range (breaking) |
| `ui.range_pair("price", (20, 80)).label("Price")` | `ui.range_pair("price", "Price").values(20, 80)` | Range (breaking) |
| Select's id `plan`, while the error summary linked `#f-plan` | `f-plan`, with `.error(..)` like a field's | Select |
| a loose error summary listed bare messages | `.field("email", "Email")` writes "Email: …" as a form's does | ErrorSummary |
| no way to set the id of a meter, bar, sidebar or nav menu | `.id(..)` | Meter, Progress, Sidebar, NavMenu |

## Tables

A table route does three things: read what the visitor asked for, fetch and sort the rows, and
write the table. `ui.table_query(id, sortable)` does the first before any markup, so the table
itself can be written in `lui!` without holding the builder in a variable:

```rust
use loco_ui::prelude::*;
let ui = Ui::from_request("/orders", "sort=total&dir=desc&status=paid", "");
let orders = [(1, "Ada", "paid", 12.5_f64), (2, "Grace", "pending", 30.0), (3, "Ken", "paid", 20.0)];
let q = ui.table_query("orders", &["id", "total"]);
let status = ui.param("status").unwrap_or("");
let mut found: Vec<_> = orders.iter().filter(|o| (status.is_empty() || o.2 == status) && q.matches(o.1)).collect();
q.sort_by(&mut found, |a, b, key| match key { "total" => a.3.total_cmp(&b.3), _ => a.0.cmp(&b.0) });
let (page, total) = q.page_of(&found);
let html = lui! { Table("orders", "/orders") paged=(total) {
    column "id" "Order" sortable numeric;
    column "customer" "Customer";
    column "status" "Status";
    column "total" "Total" sortable numeric;
    filter_select "status" "Status" ([("", "All"), ("paid", "Paid"), ("pending", "Pending")]);
    rows (page.iter().map(|o| (o.0, o.1, ui.badge(o.2), format!("${:.2}", o.3))));
} };
assert!(html.into_string().contains("<td>Ken</td>"));
```

- **`TableQuery`** (`ui.table_query(id, sortable)`): `.sort()` as `(key, descending)`, only for
  a key in `sortable`; `.filter()` the search text, trimmed; `.page()` 1-based; `.per_page()`
  the size the visitor picked (`ui.state.per_page(id)`, at most 50) or **10**. Defaults:
  unsorted, no filter, page 1, 10 rows.
- **Data helpers** on it: `.matches(text)` (case-insensitive, true with no search),
  `.sort_by(&mut v, |a, b, key| ..)` (ascending comparator; a descending sort reverses it;
  nothing happens without a sort), `.page_of(&v)` (the page's slice and the total for
  `.paged(total)`; past the end gives the last page, as the pager does), and `.visible(&keys)`
  for a CSV of the shown columns.
- **Rows** are a `Row` (in the prelude) when they need a key, a detail, a menu or in-place
  values, or else a tuple of up to eight cells of anything `Render`:
  `(o.id, o.customer, ui.badge(o.status))`.
- **A filter of the page's own** travels with the table: `.keep("status")` carries the
  request's `status` on every sort link, the search form, the columns chooser, the pager and
  the CSV link; `.filter_select(name, label, options)` puts a `<select>` for it in the search
  form and keeps it. A query in `href` (`/orders?status=paid`) is kept the same way.
- **The search box** is on unless `.hide_search()`; with no sortable column, no search, no
  pager and no chooser a table has no links, so `href` may be `""` for a display-only table
  (links, if any, are then relative: `?sort=..`, forms post to this page).
- **Chart** values from data in one call: `.points(SIGNUPS)` beside `.point(label, value)`.

| Before | After |
|---|---|
| a `Table` in a variable, asked `t.sort()`, `t.filter()`, `t.page()`, `t.per_page()` before its rows | `let q = ui.table_query(id, &sortable)`, then `lui! { Table(..) { .. } }` |
| `if let Some((key, desc)) = t.sort() { v.sort_by(..); if desc { v.reverse() } }` | `q.sort_by(&mut v, \|a, b, key\| ..)` |
| `v.iter().skip((page - 1) * per).take(per)` and `let total = v.len()` | `let (page, total) = q.page_of(&v)` |
| `n.to_lowercase().contains(&t.filter().to_lowercase())` | `q.matches(n)` |
| `Row::new([html! { (a) }, html! { (b) }])` | `(a, b)`, or `Row::from((a, b)).key(..)` |
| `use loco_ui::{prelude::*, table::Row}` | `use loco_ui::prelude::*` |
| `?status=` in the path (`/orders/{status}`) because links dropped it; `href="/o?status=paid"` built `/o?status=paid?sort=..` | `.keep("status")` or `.filter_select(..)`; a query in `href` is kept |
| a search box on every table | `.hide_search()`, and `href` may be `""` for a display-only table |
| `SIGNUPS.iter().fold(ui.chart(..), \|c, (d, n)\| c.point(d, *n))` | `ui.chart(..).points(SIGNUPS)` |

Kept, because: `.rows(..)`, `.filter_select(..)` and `Chart::points` take a list in one call
(rows and options come from data; `.point(..)` still adds one value); `ui.table(id, href)` keeps
its order (the id keys the query parameters). `Table::sort`, `filter`, `page` and `per_page`
stay for routes that do hold the builder.
