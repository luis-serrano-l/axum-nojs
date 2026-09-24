//! Render costs: the inline stylesheet, a whole page, a 1 000-row table and a paged table,
//! and parsing UI state. `cargo bench -p axum-nojs`; numbers go in FINDINGS.md (M16).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use axum_nojs::{UiState, prelude::*, stylesheet, table::Row};

fn rows(n: usize) -> Vec<Row<'static>> {
    (0..n).map(|i| Row::new([html! { "file-" (i) ".txt" }, html! { (i * 17) " KB" }, html! { "—" }])).collect()
}

fn files(ui: &Ui) -> axum_nojs::table::Table<'_> {
    ui.table("t", "/t").column("name", "Name").sortable().column("size", "Size").sortable().numeric().column("note", "Note")
}

fn bench(c: &mut Criterion) {
    let ui = Ui::from(Caps::all());
    c.bench_function("stylesheet", |b| b.iter(|| black_box(stylesheet()).len()));
    c.bench_function("layout", |b| b.iter(|| ui.page("Title", html! { p { "body" } }).into_string().len()));
    let mut sorted = Ui::from_request("/t", "sort=name", "");
    sorted.caps = Caps::all();
    let thousand = rows(1000);
    c.bench_function("table 1000 rows", |b| {
        b.iter(|| files(&sorted).rows(black_box(thousand.clone())).render().into_string().len())
    });
    let mut paged = Ui::from_request("/t", "page=20&q=file", "nojs-ui=per.t=25");
    paged.caps = Caps::all();
    let page = rows(25);
    c.bench_function("paged_table 25 of 1000", |b| {
        b.iter(|| files(&paged).rows(page.clone()).paged(1000).render().into_string().len())
    });
    c.bench_function("UiState::from_request", |b| {
        b.iter(|| {
            UiState::from_request(
                black_box("/settings"),
                black_box("tab.settings=1&page=3&sort=name&q=hello+world"),
                black_box("theme=dark; nojs-ui=open.faq=2%2C3&per.files=25; nojs-flash=Saved."),
            )
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
