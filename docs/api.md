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
| `body` | the main markup, what a `lui!` block holds; on Form, markup placed among the fields | Card, Alert, Dialog, Drawer, ContextMenu, Button, SelectOption, EmptyState, Stack, Cluster, Grid, the blocks, Form (an item) |
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

## Markup in `lui!`

A block is markup wherever a value or an item can take one, so a page needs no
`html! { .. }` or `lui! { .. }` inside `lui!`:

- **A layout's block is its body**: `Stack gap=6 { .. }`, `Cluster between { .. }`,
  `Grid("20rem") { .. }` (`ui.stack().gap(6).body(html! { .. })`).
- **An attribute takes a block**: `footer={ Button("Save") primary; }` is
  `.footer(html! { (ui.button("Save").primary()) })`, the block itself `lui!`.
- **`body { .. }` among items** is `.body(..)` with that markup: an empty state's text beside
  its link, a form's controls between its fields.
- **An item's arguments are values.** `stat Stat("Revenue", "$1");` is an error that says
  what to write instead: `stat (ui.stat("Revenue", "$1"))`.

```rust
use loco_ui::prelude::*;
let ui = Ui::from_request("/", "", "");
let a = lui! { Grid("20rem") {
    Card title="Plan" footer={ LinkButton("Invoices", "/invoices"); Button("Upgrade") primary; } {
        p { "12 of 20 seats used." }
    }
    EmptyState("No invoices") { body { "They show here once a month." } link "Billing" "/billing"; }
} };
let b = html! { (ui.grid("20rem").body(html! {
    (ui.card().title("Plan")
        .footer(html! { (ui.link_button("Invoices", "/invoices")) (ui.button("Upgrade").primary()) })
        .body(html! { p { "12 of 20 seats used." } }))
    (ui.empty_state("No invoices").body(html! { "They show here once a month." }).link("Billing", "/billing"))
})) };
assert_eq!(a.into_string(), b.into_string());
```

A card's footer packs its buttons at its end, as a dialog's do, so it needs no cluster.

## Forms hold any control

A form lists its fields with adders (`text`, `email`, `select`, `checkbox`, `switch`, ..) and
takes any other control with `.body(..)` at that point in the list: a date picker, a range, a
toggle group, a select built with its own setters. Everything sits inside the one swap root,
under the error summary, before the button. `.get()` makes it a filter or a search whose
answer is the same page with the choices in its URL.

```rust
use loco_ui::prelude::*;
let ui = Ui::from_request("/orders", "status=paid", "");
let form = lui! { Form("/orders") get submit="Filter" {
    text "q" "Customer";
    switch "late" "Late only";
    body { Select("status", "Status") options=([("paid", "Paid"), ("open", "Open")]); }
} };
let html = form.into_string();
assert!(html.contains(r#"method="get""#) && html.contains(r#"<option value="paid" selected>"#));
```

## Answering a post

A POST handler returns `Result<Redirect, Page>`: `Ok` for Post/Redirect/Get, `Err` for the
form again, with `.invalid()` so the answer is `422 Unprocessable Content`. `Posted` reads the
body by name whether it came urlencoded or as `multipart/form-data` (a form with a file
field); `posted.pairs()` gives the form back what was sent.

```rust
use loco_ui::prelude::*;

fn signup(ui: &Ui, posted: &[(String, String)], errors: &[(&str, &str)]) -> Page {
    ui.page("Sign up", lui! {
        Form("/signup") values=(posted) errors=(errors) { email "email" "Email" required; }
    })
}

async fn submit(ui: Ui, posted: Posted) -> Result<Redirect, Page> {
    if posted.get("email").ends_with("@example.com") {
        let errors = [("email", "example.com addresses are not accepted.")];
        return Err(signup(&ui, posted.pairs(), &errors).invalid());
    }
    Ok(ui.redirect("/signup").ok("Signed up."))
}

let ui = Ui::from_request("/signup", "", "");
let posted = Posted::from_pairs(vec![("email".into(), "a@example.com".into())]);
let page = signup(&ui, posted.pairs(), &[("email", "Taken.")]).invalid();
assert_eq!(page.status(), 422);
```

A wizard's post is a `Posted` too: `posted.step()` and `posted.skip()`.

## The flash shows itself

`ui.page(..)` puts a flash that a redirect left at the top of the body. Place `(ui.flash())`
yourself only to put it elsewhere or give it setters (`Flash dismiss auto_hide;`), or show it
as `Toasts`; the page then leaves it where it is.

## What repeated in the demo, and what became of it

| Pattern | Decision |
|---|---|
| a field with its label, help and error | kept: one adder or one `Input(name, label)` line each, the caller's own words |
| a form with its button, around controls that are not fields | a generator: `Form` holds any control (`body { .. }`, `switch`), `.get()` for filters; the demo writes no `form class="lui-form"` by hand |
| a card with a title and actions | a better default: the footer packs its buttons at its end |
| layouts wrapped around markup | a block: `Stack gap=6 { .. }` |
| `(ui.flash())` on every page | a better default: `ui.page` shows it |
| a POST parsed by hand, a 422 built by hand | a new type and a new prop: `Posted`, `Page::invalid()` |
| a page that is its component, its title and a note | demo only: `PAGES` (href, component, note) and one handler, the title from the index |
| a table fed from a slice | see "Tables" |

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

**Not `(name, label)`**

- `ui.combobox(name, action)`: its label is not shown (the chips and the results around the
  box say what it is, so the label is the input's `aria-label`, `.label(..)`), while the
  action, the route that answers each search, is required. A `(name, label)` constructor
  would promise a visible label like a field's.
- `ui.calendar(name)`: a month of day links under the month's name, with no field and so no
  label to show. The form control is `ui.date_picker(name, label)`, which has one.
- `ui.split(side, main)`: two blocks of markup, where a `lui!` block can be only one, so it
  keeps both in the call; `ui.stack()`, `ui.cluster()` and `ui.grid(min)` take their one
  block as `.body(..)`.

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
- `body`: a component's main markup, set once; Form's is an item, markup placed among its
  fields, and a form may hold several.

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
warning. The constructors changed outright (the crate is unpublished, and Rust cannot keep
`ui.stack(content)` beside `ui.stack()`).

| Before | After | On |
|---|---|---|
| `ui.stack(html! { .. }).gap(6)`, `Stack(lui! { .. }) gap=6;` | `ui.stack().gap(6).body(html! { .. })`, `Stack gap=6 { .. }` | Stack, Cluster, Grid (breaking) |
| `footer=(lui! { .. })`, `body (html! { .. });`, `body() { .. }` | `footer={ .. }`, `body { .. }` | `lui!` |
| `.footer(html! { (ui.cluster(..).end()) })` | `.footer(html! { .. })`: the footer packs at its end | Card |
| `form id=.. data-lui="swap" class="lui-form" method=.. { .. (ui.button("Save").primary()) }` | `Form(action) get submit="Save" { body { .. } switch "x" "X"; }` | Form: `.body(..)`, `.switch(..)`, `.get()` |
| `(StatusCode::UNPROCESSABLE_ENTITY, view).into_response()` from a `Response` handler | `Err(view.invalid())` from a `Result<Redirect, Page>` handler | Page |
| a `Multipart` loop, or `Form<Vec<(String, String)>>` and a `get` closure | `posted: Posted`, `posted.get("email")`, `posted.files()`, `posted.pairs()` | Posted |
| `wizard::Posted::from_pairs(&pairs)`, `.step`, `.skip` | `posted.step()`, `posted.skip()` | Wizard |
| `(ui.flash())` at the top of every page | nothing: `ui.page` shows a pending flash | Page |
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
let ui = Ui::from_request("/orders", "sort.orders=total&dir.orders=desc&status=paid", "");
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
  (links, if any, are then relative: `?sort.<id>=..`, forms post to this page).
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

### A table's query keys (M33)

Every query key a table reads and writes carries its id, `<key>.<id>`, the way its page size
(`per.<id>`), its in-place edit (`edit.<id>`), a tab group (`tab.<id>`) and a wizard
(`step.<id>`) already did: `q.<id>` (search), `sort.<id>` and `dir.<id>` (sort), `page.<id>`
(page) and `cols.<id>` (shown columns). The key comes first so the whole family reads alike
and a key names what it holds before whose it is. Two tables on one page (the /table page and
its playground) now sort, filter and page on their own. `loco_ui::table::Keys::new(id)` spells
them for a hand-written link.

```rust
use loco_ui::prelude::*;
let ui = Ui::from_request("/p", "sort.files=size&dir.files=desc&q.notes=milk", "");
let (files, notes) = (ui.table_query("files", &["size"]), ui.table_query("notes", &["text"]));
assert_eq!((files.sort(), files.filter()), (Some(("size", true)), ""));
assert_eq!((notes.sort(), notes.filter()), (None, "milk"));
// Deprecated, read for one release: the bare keys, when the table's own are absent.
let old = Ui::from_request("/p", "sort=size&q=a&page=2", "");
let q = old.table_query("files", &["size"]);
assert_eq!((q.sort(), q.filter(), q.page()), (Some(("size", false)), "a", 2));
```

- **Deprecated:** the bare `q`, `sort`, `dir`, `page` and `cols`. A table still reads each one
  when its own `<key>.<id>` is absent, so old bookmarks and hand-written links keep working for
  one release; its own key wins, and every link and form it writes uses its own. Rewrite links
  such as `/table?sort=size&dir=desc` as `/table?sort.files=size&dir.files=desc`.
- **State or query:** only the page size is state (remembered in the `lui-ui` cookie); the
  others describe one view and stay in the URL, as before.
- **Kept bare:** the load-more `Pager`'s `page`. It has no id, and a page holds one; a paged
  table beside it pages by its own `page.<id>`.
- `.keep(..)`, `.filter_select(..)` and a query in `href` are the page's own parameters, carried
  as named (`status`), not prefixed.
