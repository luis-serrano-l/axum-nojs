# How loco-ui compares

Five ways to build the UI of a Rust web app, compared on what this library cares about: how
much JavaScript the page needs, whether it keeps working without it, whether that is proven,
what writing a component looks like, and what a Loco app gets. The maud-ui figures come from
its 0.20.3 release, as surveyed on 2026-09-23 and 2026-09-25 (`ROADMAP.md` M26 and M29); the
Loco column is Loco 1.x without loco-ui, from its generators reference and `loco new` 1.2.0;
the others describe how each project works in general. Corrections are welcome.

## At a glance

| | loco-ui | Loco 1.x on its own | maud-ui | htmx + hand-written Maud | Leptos, Dioxus |
|---|---|---|---|---|---|
| Where the UI runs | server | a React SPA in the browser (clientside apps), or Tera templates you write | server | server | the browser (Rust compiled to WebAssembly), rendered first on the server |
| JavaScript required | **none**; one optional 11 KB script (3.6 KB gzip) | the React bundle, for every scaffolded page | htmx plus an 89 KB script (24 KB gzip); its README lists 21 components that need it, among them Dialog, Popover, Menu, Select, Sheet and Data Table | htmx, for every `hx-` attribute | the WASM bundle and its JS glue, to hydrate |
| With script blocked | every page works | the scaffolded pages are empty | untested (its docs say the no-JS browser check "was not run") | the parts you wrote as plain links and forms | server-rendered HTML shows; interactivity stops |
| Proof | a script-less renderer (Blitz) renders every route in CI, a test allows exactly one optional script, and axe-core finds no serious or critical violation on any route | none published | none published | yours to write | yours to write |
| Strict CSP | `script-src 'self'` (or `'none'` without the script) out of the box, tested on every route | depends on the SPA build | depends on the widgets' scripts | `script-src` must allow htmx; `hx-on` attributes are evaluated as code, which a strict CSP forbids | `script-src` must allow the WASM loader (`'wasm-unsafe-eval'`) |
| Look | Linear / Magic UI: Radix-style 12-step oklch scales, layered shadows, gradient accents, CSS-only motion, opt-in effects (`.shimmer()`, `.beam()`, `.glow()`, `.gradient_border()`, `.reveal()`, `ui.marquee`) | whatever the React app uses | shadcn/ui | yours | from the ecosystem |
| CSS shipped | 82.6 KB (14.4 KB gzip), every component and block, inlined once | the SPA's | 313 KB (44 KB gzip) | yours | yours or a crate's |
| Components | 59 builders (primitives, components, widgets, six page blocks), each with its props listed, and a guide for your own | none | 84 components and 31 blocks | none | from the ecosystem |
| On Loco | a `loco` feature: initializer, `cargo lui install`, HTML scaffold templates, `cargo lui auth` account pages, `Valid<T>` forms, `paged_from(&PagerMeta)`, a 404 fallback | a JSON API and React pages from `cargo loco generate scaffold` (the `--html`/`--htmx` scaffolds were removed in 1.0) | no integration | yours to wire | server functions, no Loco integration |
| Languages | one table per language, from `Accept-Language` or a cookie, or from Loco's fluent files | fluent-templates for Tera | none built in | yours | from the ecosystem |
| Server state | URL, cookies and forms (Post/Redirect/Get), read by the components from `ui` | the SPA's | per component | on the server, swapped in as fragments | signals in the browser, server functions |
| API shape | builders on `ui`, or the same written as elements in `lui!` | JSX | `Props` structs | markup | components as functions |

## One button, two API shapes

loco-ui, as a builder or as an element in `lui!` (the macro expands to the builder):

```rust
(ui.button("Ship it").primary())
```

```rust
lui! { Button("Ship it") primary; }
```

maud-ui's `Props` style, schematically (field names illustrative):

```rust
(button::render(button::Props {
    label: "Ship it".into(),
    variant: button::Variant::Primary,
    ..Default::default()
}))
```

Both are fine Rust. The builder wins where this library lives:

- **Components read their input from `ui`.** A table knows its sort, filter, page and hidden
  columns from the request; a dialog knows whether the server opened it; every component
  knows which browser features it may use. A detached `Props` struct would have to be filled
  with all of that by the route.
- **List adders beat nested vectors.** `.tab("Install", body).badge(3)` adds an item and changes
  the item added last; the `Props` form is `tabs: vec![Tab { title, badge: Some(3), ..Default::default() }]`.
- **Short calls stay short inside `html!`.** The common case is one line, and setters carry
  meaning: a setter with no argument switches something on (`.danger()`), one that takes a
  `bool` is set from a condition (`.loading(busy)`), and ids come from the label.

`Props` puts a name on every value at the call site; `lui!` does too, as attributes
(`Dialog("Delete account") small danger confirm=("Delete", "/delete") { .. }`), while items
stay items (`tab "Use" badge=3 { .. }`, not a nested struct) and `@for`/`@if` build them from
data. A misspelled attribute is rustc's own error at that attribute, with the setter it meant.

Two things `Props` do better, the builders now do too: every builder is plain data (`Clone`
and `Debug`, so a route can keep one in a variable, build it in a loop or print it), and every
option is listed in one place, a "Setters" paragraph on the builder type grouping values and
items, switches and conditions. Two tests keep both true. The options are also data:
`loco_ui::props()` lists every component's setters with their kind, arguments, default, the
HTML attribute they set and a line of documentation. The same list is in
`spec/components.json` and is shown as a table on every demo page. We found no equivalent
in maud-ui's getting-started guide, where the `Props` struct and the compiler are the reference.

## When to pick which

- **loco-ui** for admin panels, dashboards, settings pages, forms, internal tools and content
  sites that refresh per action, and anywhere script is restricted: a strict CSP, Tor Browser's
  "Safest" level, public services that must work on anything.
- **loco-ui on Loco** when a Loco app wants server-rendered pages instead of Loco 1.x's React
  scaffold: `cargo lui install`, then `cargo loco generate scaffold` writes HTML controllers
  and Maud views, and `cargo lui auth` the account pages.
- **maud-ui** for the widest catalogue in the same stack, when requiring JavaScript is fine.
- **htmx + hand-written Maud** when you want full control of every fragment and do not need a
  component set.
- **Leptos or Dioxus** for an app that reacts on every keystroke: an editor, a live canvas,
  real-time collaboration.
