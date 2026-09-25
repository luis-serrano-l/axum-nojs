# axum-nojs on Loco

[Loco](https://loco.rs) is a Rails-style framework on Axum, so a Loco controller is an Axum
handler and takes `ui: Ui` like any other. The `loco` feature adds the one piece of wiring
Loco needs; the API reference is the `axum_nojs::loco` module docs, whose examples are compiled
and run by `cargo test -p axum-nojs --features loco --doc loco`.

## Views: Maud, not Tera

Loco's generators write Tera templates under `assets/views/` and render them with
`format::render().view(&v, "notes/list.html", data!({..}))`. With axum-nojs the views are Rust:
a `src/views/` module of plain functions that take `&Ui` and the data and return `Markup`. A
controller calls one and returns the page.

```rust
// src/views/notes.rs
use axum_nojs::{prelude::*, table::Row};
use crate::models::_entities::notes;

pub fn list(ui: &Ui, rows: &[notes::Model], total: usize) -> Markup {
    let table = ui.table("notes", "/notes").column("title", "Title").sortable();
    html! {
        (ui.flash())
        (table.rows(rows.iter().map(|n| Row::new([html! { a href={ "/notes/" (n.id) } { (n.title) } }]))).paged(total))
        (ui.link_button("New note", "/notes/new"))
    }
}

pub fn form(ui: &Ui, action: &str, title: &str, errors: &[(&str, &str)]) -> Markup {
    html! {
        (ui.form(action).text("title", "Title").required().value(title).errors(errors).submit("Save"))
    }
}
```

```rust
// src/controllers/notes.rs
async fn list(ui: Ui, State(ctx): State<AppContext>) -> Result<Page> {
    let (rows, total) = /* the paging query in the `loco` module docs */;
    Ok(ui.page("Notes", views::notes::list(&ui, &rows, total)))
}
```

Tera and Maud views live side by side: each controller picks how it renders, so an existing
app can move one controller at a time. The two need nothing from each other.

**No Tera bridge.** A Tera function such as `{{ nojs_button(label="Save") }}` was considered
and left out:

- A component starts from `ui`: the browser's capabilities, the query and the `nojs-ui` state
  (which tab is open, the page size, the flash). A Tera function gets only JSON arguments, so
  every template would have to pass that state through by hand, and a forgotten argument
  renders the wrong variant silently.
- Builders are typed chains (`.required()`, `.maxlength(80)`, `.column(..).sortable()`); as
  keyword arguments they lose the compiler's checks, and a typo is a runtime error on the page.
- Maud escapes by default and checks the markup at compile time; Tera output would have to be
  marked `| safe`.

A Tera page can still link to or embed a Maud-rendered fragment from its own route; there is
nothing to share at template level.
