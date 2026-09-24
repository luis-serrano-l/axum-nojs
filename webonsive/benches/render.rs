//! Render costs: the inline stylesheet, a whole page, a 1 000-row table and a paged table,
//! and parsing UI state. `cargo bench -p webonsive`; numbers go in FINDINGS.md (M16).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use maud::html;
use webonsive::table::{Column, Row, TableOptions};
use webonsive::{Caps, PagedTableOptions, Theme, UiState, layout, paged_table, stylesheet, table};

fn rows(n: usize) -> Vec<Row<'static>> {
    (0..n).map(|i| Row::new(vec![html! { "file-" (i) ".txt" }, html! { (i * 17) " KB" }, html! { "—" }])).collect()
}

const COLS: [Column; 3] = [Column::sortable("name", "Name"), Column::numeric("size", "Size"), Column::plain("note", "Note")];

fn bench(c: &mut Criterion) {
    let caps = Caps::all();
    c.bench_function("stylesheet", |b| b.iter(|| black_box(stylesheet()).len()));
    c.bench_function("layout", |b| {
        b.iter(|| layout(&caps, "Title", Theme::Auto, html! { p { "body" } }).into_string().len())
    });
    let thousand = rows(1000);
    c.bench_function("table 1000 rows", |b| {
        b.iter(|| table(&caps, "t", "/t", &COLS, black_box(&thousand), TableOptions::default().sort(Some(("name", false)))).into_string().len())
    });
    let page = rows(25);
    c.bench_function("paged_table 25 of 1000", |b| {
        b.iter(|| {
            let opts = PagedTableOptions::default().page(black_box(20)).per_page(25).filter("file");
            paged_table(&caps, "t", "/t", &COLS, &page, 1000, opts).into_string().len()
        })
    });
    c.bench_function("UiState::from_request", |b| {
        b.iter(|| {
            UiState::from_request(
                black_box("/settings"),
                black_box("tab.settings=1&page=3&sort=name&q=hello+world"),
                black_box("theme=dark; wo-ui=open.faq=2%2C3&per.files=25; wo-flash=Saved."),
            )
        })
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
