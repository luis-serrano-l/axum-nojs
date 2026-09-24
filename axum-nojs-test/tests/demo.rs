//! Every demo route through Blitz: layout assertions plus a PNG under `tests/shots/`.

use axum_nojs_test::Page;

const MODERN: &str = "nojs-cap-probed=1; nojs-cap-invokers=1; nojs-cap-anchor=1; nojs-cap-details_content=1; nojs-cap-view_transitions=1; nojs-cap-popover=1; nojs-cap-light_dark=1; nojs-cap-streaming_dsd=1; nojs-cap-base_select=1";
const OLD: &str = "nojs-cap-probed=1";

fn shot(page: &mut Page, name: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/shots");
    page.screenshot(format!("{dir}/{name}.png")).unwrap();
}

/// The same index under the second palette from `docs/theming.md`: `index-alt.png` next to
/// `index-modern.png` is the proof that a theme is a value passed to `layout_with`.
#[tokio::test]
async fn index_under_another_palette() {
    let mut page = Page::render(demo::router(), "/?palette=linen", MODERN).await;
    assert!(
        page.exists("style.nojs-tokens"),
        "layout_with emits the token overrides"
    );
    assert!(page.is_visible("h1"));
    shot(&mut page, "index-alt");
}

/// Screenshot every route twice: as a modern browser and as one that supports nothing.
#[tokio::test]
async fn every_route_renders_and_is_captured() {
    for path in demo::PATHS {
        let name = path
            .trim_start_matches('/')
            .replace(['/', '?', '=', '.'], "-");
        let name = if name.is_empty() {
            "index".to_string()
        } else {
            name
        };
        for (suffix, cookie) in [("modern", MODERN), ("fallback", OLD)] {
            let mut page = Page::render(demo::router(), path, cookie).await;
            // Blitz has no declarative shadow DOM: see `blitz_has_no_declarative_shadow_dom`.
            let inside_shadow_root = path == "/stream" && suffix == "modern";
            if !inside_shadow_root {
                assert!(page.is_visible("h1"), "{path} ({suffix}) has no visible h1");
                assert!(
                    page.is_visible(".nojs-header"),
                    "{path} ({suffix}) has no header"
                );
            }
            shot(&mut page, &format!("{name}-{suffix}"));
        }
    }
}

#[tokio::test]
async fn dialog_variants_follow_caps() {
    let page = Page::render(demo::router(), "/dialog", MODERN).await;
    assert!(page.is_visible("button[command=show-modal]"));
    assert!(!page.exists(".nojs-dialog-open"));
    assert!(!page.is_visible("dialog"), "closed dialog must not render");

    let page = Page::render(demo::router(), "/dialog", OLD).await;
    assert!(page.is_visible(".nojs-dialog-open"));
    assert!(!page.exists("button[command]"));

    let page = Page::render(demo::router(), "/dialog?dialog=confirm", OLD).await;
    assert!(
        page.is_visible("dialog[open]"),
        "server-opened dialog renders"
    );
    assert_eq!(page.text("dialog h2").as_deref(), Some("Delete account?"));
    assert!(page.exists("dialog.nojs-dialog-sm[aria-labelledby='confirm-title']"));
    assert!(
        page.is_visible(
            ".nojs-dialog-danger form[method=post][action='/dialog/delete'] button.nojs-button-danger"
        ),
        "confirm is a real form"
    );
    assert!(page.exists("form input[type=hidden][name=returns_to][value='/dialog']"));
    assert!(
        page.is_visible("a.nojs-dialog-cancel[href='#']")
            && page.is_visible("a.nojs-dialog-close[href='#']"),
        "fallback closes through links"
    );
    let page = Page::render(demo::router(), "/dialog?dialog=confirm", MODERN).await;
    assert!(
        page.is_visible("button[command=close][commandfor=confirm]"),
        "cancel is an invoker"
    );
    assert!(
        page.exists("dialog > .nojs-dialog-close:last-child"),
        "close control is last so focus lands on the field"
    );
}

#[tokio::test]
async fn tabs_strip_versus_accordion() {
    let modern = Page::render(demo::router(), "/tabs?tab.demo=1", MODERN).await;
    assert_eq!(
        modern.display(".nojs-tabs details").as_deref(),
        Some("contents")
    );
    let first = modern.bbox(".nojs-tabs summary").unwrap();
    let second = modern
        .bbox(".nojs-tabs details:nth-of-type(2) summary")
        .unwrap();
    assert!(
        (first.y - second.y).abs() < 1.0,
        "tab titles share a row: {first:?} {second:?}"
    );
    assert!(second.x > first.x + first.width - 1.0);
    let panel = modern
        .bbox(".nojs-tabs details[open] .nojs-tabs-panel")
        .unwrap();
    assert!(
        panel.y >= first.y + first.height - 2.0,
        "open panel sits below the strip: {panel:?} vs {first:?}"
    );
    assert!(
        !modern.is_visible(".nojs-tabs details:nth-of-type(1) .nojs-tabs-panel"),
        "closed panel hidden"
    );
    assert_eq!(
        modern
            .text("#nojs-tabs-demo details[open] .nojs-tabs-badge")
            .as_deref(),
        Some("3")
    );
    assert!(
        modern.exists("#nojs-tabs-demo details:nth-of-type(3) .nojs-tabs-lazy"),
        "lazy tab has no body until opened"
    );
    assert!(modern.exists("#nojs-tabs-demo details[open] summary .nojs-tabs-mark[style*='view-transition-name: nojs-tabs-demo']"), "only the chip carries the name");
    let mark = modern
        .bbox("#nojs-tabs-demo details[open] .nojs-tabs-mark")
        .unwrap();
    let open_title = modern
        .bbox("#nojs-tabs-demo details[open] summary")
        .unwrap();
    // The mark is shadcn's active-tab chip: the open title inset by the pill's 3px padding.
    assert!(
        (mark.width - (open_title.width - 6.0)).abs() < 1.0
            && (mark.height - (open_title.height - 6.0)).abs() < 1.0
            && (mark.x - open_title.x - 3.0).abs() < 1.0,
        "chip fills the open title inside the pill: {mark:?} {open_title:?}"
    );
    assert!(modern.exists("#nojs-tabs-demo form.nojs-tabs-select select[name='tab.demo'] option[value='1'][selected]") && !modern.is_visible(".nojs-tabs-select"), "select is there but hidden on a wide screen");
    let lazy = Page::render(demo::router(), "/tabs?tab.demo=2", MODERN).await;
    assert!(
        lazy.is_visible("#nojs-tabs-demo details:nth-of-type(3) .nojs-tabs-panel p"),
        "lazy tab rendered once open"
    );
    let side_first = modern.bbox("#nojs-tabs-side summary").unwrap();
    let side_second = modern
        .bbox("#nojs-tabs-side details:nth-of-type(2) summary")
        .unwrap();
    let side_panel = modern
        .bbox("#nojs-tabs-side details[open] .nojs-tabs-panel")
        .unwrap();
    assert!(
        side_second.y > side_first.y + side_first.height - 1.0,
        "vertical titles stack"
    );
    assert!(
        side_panel.x >= side_first.x + side_first.width - 1.0
            && (side_panel.y - side_first.y).abs() < 2.0,
        "vertical panel sits beside the titles: {side_panel:?} {side_first:?}"
    );

    let old = Page::render(demo::router(), "/tabs?tab.demo=1", OLD).await;
    assert_eq!(
        old.display(".nojs-tabs details").as_deref(),
        Some("block/flow")
    );
    let first = old.bbox(".nojs-tabs summary").unwrap();
    let second = old
        .bbox(".nojs-tabs details:nth-of-type(2) summary")
        .unwrap();
    assert!(
        second.y > first.y + first.height - 1.0,
        "accordion titles stack"
    );
}

#[tokio::test]
async fn accordion_multi_open_with_controls_and_nesting() {
    let page = Page::render(
        demo::router(),
        "/accordion?open.faq=0,2&open.faq-more=0",
        MODERN,
    )
    .await;
    assert_eq!(
        page.count("#nojs-accordion-faq > details[open]"),
        2,
        "two sections open at once"
    );
    assert!(
        page.exists("#nojs-accordion-faq > details:not([name])"),
        "multi group has no name attribute"
    );
    assert!(
        page.exists("#nojs-accordion-faq-more > details[name='faq-more'][open]"),
        "nested group is exclusive and open"
    );
    assert!(
        page.is_visible("#nojs-accordion-faq-more"),
        "nested accordion sits inside the open third section"
    );
    assert_eq!(
        page.text(".nojs-accordion-controls a:first-child")
            .as_deref(),
        Some("Expand all")
    );
    assert!(
        page.exists(
            ".nojs-accordion-controls a[href='/accordion?open.faq=0%2C1%2C2&open.faq-more=0']"
        ),
        "expand all links to every index and keeps the nested key"
    );
    assert!(
        page.exists(".nojs-accordion-controls a[href='/accordion?open.faq=&open.faq-more=0']"),
        "collapse all sets the key to nothing, explicitly, so the cookie cannot reopen them"
    );
    assert!(page.exists("#nojs-accordion-faq > details:nth-of-type(2) summary a[href='/accordion?open.faq=0%2C1%2C2&open.faq-more=0']"), "a closed title adds itself to the list");
    assert!(page.exists("#nojs-accordion-faq > details:nth-of-type(1) summary a[href='/accordion?open.faq=2&open.faq-more=0']"), "an open title removes itself");
    assert!(
        page.is_visible("#nojs-accordion-faq > details:nth-of-type(2) .nojs-accordion-summary"),
        "summary line shows on a closed section"
    );
    assert!(
        !page.is_visible(
            "#nojs-accordion-faq > details:nth-of-type(1) > summary .nojs-accordion-summary"
        ),
        "summary line hidden once open"
    );
    assert!(page.is_visible("#nojs-accordion-faq > details:nth-of-type(1) .nojs-accordion-icon"));
    let outer = page
        .bbox("#nojs-accordion-faq > details:nth-of-type(3) > summary")
        .unwrap();
    let inner = page
        .bbox("#nojs-accordion-faq-more > details:nth-of-type(1) > summary")
        .unwrap();
    assert!(
        inner.x > outer.x + 8.0 && inner.y > outer.y + outer.height,
        "nested titles are indented below the outer title: {inner:?} {outer:?}"
    );
}

#[tokio::test]
async fn combobox_chips_results_and_create_row() {
    let page = Page::render(demo::router(), "/combobox?q=ru&sel=Zig", MODERN).await;
    assert_eq!(
        page.count(".nojs-combobox-chip"),
        1,
        "one chip for the selection"
    );
    assert!(
        page.exists(".nojs-combobox-chip input[type=hidden][name=sel][value=Zig]"),
        "the chip rides along with the next search"
    );
    assert!(
        page.exists(".nojs-combobox-chip a[href='/combobox?q=ru'][aria-label='Remove Zig']"),
        "the chip's link removes it"
    );
    assert!(
        page.exists("datalist optgroup[label=Systems] option[value=Rust]"),
        "grouped suggestions"
    );
    assert_eq!(
        page.count("[role=listbox] [role=option]"),
        2,
        "Rust and Ruby match"
    );
    assert!(
        page.exists("[role=option] a[href='/combobox?q=ru&sel=Zig&sel=Rust']"),
        "a result adds itself after the selection"
    );
    assert!(page.exists("#q-results[aria-live=polite]"));
    assert_eq!(
        page.text(".nojs-combobox-status").as_deref(),
        Some("2 matches")
    );
    let chip = page.bbox(".nojs-combobox-chip").unwrap();
    let input = page.bbox("input[type=search]").unwrap();
    assert!(
        (chip.y + chip.height / 2.0 - (input.y + input.height / 2.0)).abs() < 6.0
            && chip.x < input.x,
        "chip sits on the input's row, before it: {chip:?} {input:?}"
    );

    let none = Page::render(demo::router(), "/combobox?q=elixir&sel=Zig", MODERN).await;
    assert_eq!(
        none.text(".nojs-combobox-status").as_deref(),
        Some("No matches.")
    );
    assert!(
        none.exists(
            "form.nojs-combobox-create[action='/combobox/new'] input[name=name][value=elixir]"
        ),
        "create row posts the text"
    );
    assert!(
        none.exists("form.nojs-combobox-create input[name=sel][value=Zig]"),
        "and keeps the selection"
    );
    assert!(none.is_visible("form.nojs-combobox-create button"));
    let picked = Page::render(demo::router(), "/combobox?q=zig&sel=Zig", MODERN).await;
    assert!(
        picked.exists("[role=option][aria-selected=true]") && !picked.exists("[role=option] a"),
        "an already selected result is not a link"
    );
}

#[tokio::test]
async fn popover_variants() {
    let modern = Page::render(demo::router(), "/popover", MODERN).await;
    assert!(modern.is_visible("button[popovertarget]"));
    assert!(
        !modern.is_visible("nav[popover]"),
        "closed popover is hidden"
    );
    assert_eq!(
        modern.count("#account [role=menuitem]"),
        7,
        "links, submenu button, action and the disabled link"
    );
    assert!(
        modern.exists("#account .nojs-popover-heading")
            && modern.count("#account .nojs-popover-sep") == 2
    );
    assert!(
        modern.exists("#account a[aria-disabled=true]:not([href])"),
        "disabled link has no href"
    );
    assert!(
        modern.exists(
            "#account form[method=post][action='/popover/signout'] button.nojs-popover-danger"
        ),
        "action is a post form"
    );
    assert!(
        modern.exists("#account nav#account-theme[popover]")
            && modern.exists("#account button[popovertarget=account-theme]"),
        "submenu is a nested popover"
    );
    assert!(modern.exists("#account kbd.nojs-popover-kbd"));
    assert!(
        modern.exists(".nojs-popover-end nav#more[popover]"),
        "placement class"
    );
    let old = Page::render(demo::router(), "/popover", OLD).await;
    assert!(old.is_visible(".nojs-popover-details summary"));
    assert!(
        !old.is_visible(".nojs-popover-details nav"),
        "closed details hides the menu"
    );
    assert!(
        old.exists("details#account details#account-theme"),
        "submenu is a nested details"
    );
}

#[tokio::test]
async fn settings_flash_and_form_values() {
    let cookie = format!(
        "{MODERN}; nojs-flash=ok%3ASettings%20saved.%0Awarn%3ANo%20releases.%0Adanger%3AReserved.; nojs-settings=name%3DAda%26notify%3Dtrue; nojs-ui=tab.settings=1"
    );
    let page = Page::render(demo::router(), "/settings", &cookie).await;
    assert_eq!(
        page.text(".nojs-flash-ok .nojs-flash-text").as_deref(),
        Some("Settings saved.")
    );
    assert!(page.is_visible(".nojs-flash"));
    // Stacked in order, one per level, danger announced as an alert, each with a dismiss link.
    let (ok, warn, danger) = (
        page.bbox(".nojs-flash-ok").unwrap(),
        page.bbox(".nojs-flash-warn").unwrap(),
        page.bbox(".nojs-flash-danger").unwrap(),
    );
    assert!(
        ok.y + ok.height <= warn.y + 1.0 && warn.y + warn.height <= danger.y + 1.0,
        "messages stack top to bottom"
    );
    assert!(
        page.exists(".nojs-flash-danger[role=alert]") && page.exists(".nojs-flash-ok[role=status]")
    );
    assert!(
        page.exists(".nojs-flash-ok.nojs-flash-auto")
            && !page.exists(".nojs-flash-danger.nojs-flash-auto"),
        "only calm levels auto-hide"
    );
    assert!(page.exists(".nojs-flash-warn a.nojs-flash-dismiss[href='/settings']"));
    assert!(page.is_visible(".nojs-flash-dismiss"));
    assert!(
        page.is_visible("input[type=checkbox][name=notify]"),
        "notifications tab is open"
    );
    assert!(
        !page.is_visible("input[name=name][id]"),
        "profile tab is closed"
    );
    let flash = page.bbox(".nojs-flash").unwrap();
    let tabs = page.bbox(".nojs-tabs").unwrap();
    assert!(
        flash.y + flash.height <= tabs.y + 1.0,
        "flash sits above the tabs"
    );
}

#[tokio::test]
async fn caps_table_lists_every_flag() {
    let page = Page::render(demo::router(), "/caps", MODERN).await;
    assert_eq!(page.count(".nojs-caps-table tbody tr"), 9);
    assert_eq!(page.count(".nojs-yes"), 9);
    let old = Page::render(demo::router(), "/caps", OLD).await;
    assert_eq!(old.count(".nojs-no"), 8);
    assert!(!old.exists(".nojs-caps"), "probed browser gets no beacons");
    let fresh = Page::render(demo::router(), "/caps", "").await;
    assert_eq!(
        fresh.count(".nojs-cap"),
        9,
        "unknown browser gets one beacon per flag"
    );
}

/// Documented limitation (FINDINGS.md, M4): Blitz parses `<template>` as inert and does not
/// attach declarative shadow roots, so a DSD page has no rendered body. The fallback variant
/// of the same page renders fully. When this test fails, Blitz has gained DSD: drop the
/// exception in `every_route_renders_and_is_captured`.
#[tokio::test]
async fn blitz_has_no_declarative_shadow_dom() {
    let dsd = Page::render(demo::router(), "/stream", MODERN).await;
    assert!(dsd.html.contains("<template shadowrootmode=\"open\">"));
    assert!(
        !dsd.is_visible("h1"),
        "Blitz now renders declarative shadow DOM; update the tests"
    );
    let fallback = Page::render(demo::router(), "/stream", OLD).await;
    assert!(fallback.is_visible("h1"));
    assert_eq!(fallback.count(".nojs-stream-section"), 3);
    let boxes: Vec<_> = (1..=3)
        .map(|i| {
            fallback
                .bbox(&format!(".nojs-stream-section:nth-of-type({i})"))
                .unwrap()
        })
        .collect();
    assert!(
        boxes[0].y < boxes[1].y && boxes[1].y < boxes[2].y,
        "sections stack in document order"
    );
}

#[tokio::test]
async fn table_sort_links_and_pages() {
    let page = Page::render(
        demo::router(),
        "/table?sort=size&dir=desc&q=a&per.files=5&page=2&cols=name,size",
        MODERN,
    )
    .await;
    assert_eq!(
        page.count(".nojs-table thead th a"),
        2,
        "every visible column header is a sort link"
    );
    assert!(
        page.exists("th[aria-sort=descending] a[href*='sort=size'][href*='dir=asc']"),
        "sorted column flips direction"
    );
    assert!(
        page.exists(
            "th a[href*='sort=name'][href*='q=a'][href*='per.files=5'][href*='cols=name%2Csize']"
        ),
        "other links keep filter, page size and columns"
    );
    assert_eq!(page.count(".nojs-table tbody tr"), 5, "one page of rows");
    assert!(
        !page.exists("th a[href*='sort=kind']"),
        "the hidden column has no header"
    );
    assert!(
        page.exists(".nojs-table-cols a[aria-pressed=false][href*='cols=name%2Csize%2Ckind']"),
        "the chooser links to showing Kind again"
    );
    assert!(
        page.exists(".nojs-table-cols a[aria-pressed=true][href*='cols=size']"),
        "and to hiding Name"
    );
    assert!(page.exists("a.nojs-table-csv[download][href='/table.csv?sort=size&dir=desc&q=a&per.files=5&cols=name%2Csize']"), "CSV link carries the whole state");
    assert_eq!(
        page.count("tbody input[type=checkbox][name=row][form='nojs-table-files-bulk']"),
        5,
        "a checkbox per row, owned by the bulk form"
    );
    assert!(page.exists("form#nojs-table-files-bulk[method=post][action='/table/bulk'] button[name=action][value=archive]"));
    assert_eq!(
        page.count("tbody .nojs-table-menu .nojs-popover"),
        5,
        "a menu per row"
    );
    assert_eq!(
        page.count("tbody details.nojs-table-detail"),
        5,
        "a detail block per row"
    );
    assert!(
        !page.is_visible("tbody details.nojs-table-detail .nojs-table-detail-body"),
        "detail closed by default"
    );
    let size_head = page.bbox("th.nojs-table-num").unwrap();
    let size_cell = page.bbox("tbody tr td.nojs-table-num").unwrap();
    assert!(
        (size_head.x + size_head.width - (size_cell.x + size_cell.width)).abs() < 2.0,
        "numeric cells end where their header ends: {size_head:?} {size_cell:?}"
    );
    assert!(
        page.exists("colgroup col[style*='width: 7rem']"),
        "column width in the colgroup"
    );
    assert!(page.is_visible("a[aria-current=page]"));
    assert_eq!(page.text("a[aria-current=page]").as_deref(), Some("2"));
    assert!(
        page.exists("a[rel=prev][href*='page=1'][href*='cols=name%2Csize']")
            && page.exists("a[rel=next][href*='page=3']")
    );
    assert_eq!(
        page.text(".nojs-paged-table-range").as_deref(),
        Some("6–10 of 21")
    );
    assert!(page.exists("select[name='per.files'] option[value='5'][selected]"));
    assert!(
        page.exists(".nojs-paged-table-per input[name=cols][value='name,size']"),
        "the page-size form keeps the columns"
    );
    assert!(
        page.exists("a.nojs-paged-table-end[href*='page=1']")
            && page.exists("a.nojs-paged-table-end[href*='page=5']"),
        "first and last links"
    );
    assert!(
        page.exists(".nojs-paged-table-jump input[type=number][name=page][max='5'][value='2']"),
        "jump-to-page form"
    );
    assert!(
        page.exists(".nojs-paged-table-jump input[type=hidden][name='per.files'][value='5']"),
        "the jump keeps the page size"
    );
    assert!(page.is_visible(".nojs-paged-table-jump button"));

    // 36 files at 5 a page is 8 pages: page 5 numbers 1 … 4 5 6 7 8.
    let long = Page::render(
        demo::router(),
        "/table?page=5",
        "nojs-cap-probed=1; nojs-ui=per.files=5",
    )
    .await;
    assert_eq!(
        long.text(".nojs-paged-table-range").as_deref(),
        Some("21–25 of 36"),
        "page size from the nojs-ui cookie"
    );
    assert_eq!(
        long.count(".nojs-paged-table-gap"),
        1,
        "one ellipsis before the current page's neighbours"
    );
    assert!(
        long.exists("a.nojs-paged-table-end[href='/table?per.files=5&page=8']"),
        "Last names the remembered size"
    );
    let rows = page.bbox(".nojs-table tbody").unwrap();
    let nav = page.bbox(".nojs-paged-table-nav").unwrap();
    assert!(
        nav.y >= rows.y + rows.height - 1.0,
        "pager sits under the rows: {nav:?} vs {rows:?}"
    );
    // Blitz paints sticky header cells at the viewport top (FINDINGS.md); the row still exists.
    assert!(page.exists(".nojs-table thead th"));

    let empty = Page::render(demo::router(), "/table?q=zzz", MODERN).await;
    assert_eq!(
        empty.text(".nojs-table-empty").as_deref(),
        Some("No files match this filter.")
    );
    let loading = Page::render(demo::router(), "/table?loading=1", MODERN).await;
    assert!(
        loading.exists("tbody[aria-busy=true]") && loading.count("tr.nojs-table-skeleton") == 3,
        "loading body is marked busy and drawn as skeleton rows"
    );
}

/// `/swap`: the controls that name a target sit outside every swap root, and the page is
/// still complete without the script (count shown, list rendered from the cookie).
#[tokio::test]
async fn swap_targets_render_without_script() {
    let page = Page::render(
        demo::router(),
        "/swap?n=3",
        "nojs-cap-probed=1; nojs-notes=note%3Done%26note%3Dtwo",
    )
    .await;
    assert_eq!(page.text("#count").as_deref(), Some("3"));
    assert_eq!(page.count("#log li"), 2);
    assert_eq!(
        page.text("#note-count").as_deref(),
        Some("2"),
        "the count is plain markup on the full page"
    );
    assert!(page.exists("a[data-nojs-target='#count']"));
    assert!(page.exists("form[data-nojs-target='#log'][data-nojs-swap='append']"));
    assert!(
        !page.exists("form [data-nojs=swap]"),
        "the form is not inside a root"
    );
    assert!(
        page.exists("a[data-nojs-push='false'][data-nojs-target='#count']"),
        "the quiet link is a plain link"
    );
    assert!(
        page.exists("form[data-nojs-indicator='#saving']") && !page.is_visible("#saving"),
        "indicator hidden without the script"
    );
    assert!(
        !page.exists("[data-nojs-busy], [aria-busy]"),
        "nothing is busy without the script"
    );
}

/// `/counter` and `/inputs`: bounds switch the stepper off, the pair shares one track, presets
/// and opacity sit beside the picker, and a long select gets a filter box and groups.
#[tokio::test]
async fn counter_and_inputs() {
    let top = Page::render(
        demo::router(),
        "/counter",
        &format!("{MODERN}; nojs-count=n%3D20"),
    )
    .await;
    assert!(
        top.exists(".nojs-counter button[value=inc][disabled]")
            && !top.exists(".nojs-counter button[value=dec][disabled]"),
        "+ is off at the maximum"
    );
    assert!(top.is_visible(
        ".nojs-counter input[type=number][name=value][min='0'][max='20'][step='2'][value='20']"
    ));
    assert_eq!(
        top.text(".nojs-counter-bounds").as_deref(),
        Some("0 to 20, in steps of 2")
    );

    let page = Page::render(demo::router(), "/inputs", &format!("{MODERN}; nojs-inputs=size%3Dl%26volume%3D40%26accent%3D%2523b3261e%26accent-alpha%3D60%26price_min%3D10%26price_max%3D90%26country%3Djp")).await;
    let (lo, hi) = (
        page.bbox("#f-price_min").unwrap(),
        page.bbox("#f-price_max").unwrap(),
    );
    assert!(
        (lo.x - hi.x).abs() < 1.0 && (lo.y - hi.y).abs() < 1.0 && (lo.width - hi.width).abs() < 1.0,
        "both thumbs on one track: {lo:?} {hi:?}"
    );
    assert_eq!(page.text("output[for=f-price_min]").as_deref(), Some("10"));
    assert_eq!(page.count(".nojs-color-presets button"), 5);
    assert!(
        page.exists(".nojs-color-presets button[aria-pressed=true][value='#b3261e']"),
        "the saved colour is the pressed preset"
    );
    assert_eq!(
        page.text(".nojs-color code").as_deref(),
        Some("#b3261e99"),
        "opacity 60% in the code"
    );
    assert!(page.exists(".nojs-color input[type=range][name=accent-alpha][value='60']"));
    assert_eq!(page.count("#country optgroup"), 3, "grouped countries");
    assert!(page.exists("#country option[value=jp][selected]"));
    assert!(
        page.is_visible(".nojs-select-search input[name=country-q]")
            && page.exists(".nojs-select-search button[formmethod=get][formaction='/inputs']"),
        "21 options: a filter box"
    );
    assert!(
        !page.exists(".nojs-select-search input[name=size-q]"),
        "3 options: none"
    );
    assert!(
        page.exists("#size option[value=l] .nojs-select-icon"),
        "icons in options"
    );

    let filtered = Page::render(demo::router(), "/inputs?country=es&country-q=arg", MODERN).await;
    assert!(
        filtered.exists("#country option[value=es]")
            && filtered.exists("#country option[value=ar]")
            && !filtered.exists("#country option[value=jp]"),
        "filter keeps matches and the selected option"
    );
}

/// `/form`: groups with legends, help and counters tied to their fields, a multipart form for
/// the file field, and labels beside the fields in the inline layout.
#[tokio::test]
async fn form_groups_help_counters_and_layouts() {
    let page = Page::render(demo::router(), "/form", MODERN).await;
    assert_eq!(
        page.count("form.nojs-form fieldset.nojs-form-group legend"),
        2,
        "two groups with legends"
    );
    assert!(page.exists("form.nojs-form[enctype='multipart/form-data'] input[type=file][accept='image/png,image/jpeg']"), "file field makes the form multipart");
    assert_eq!(
        page.text("output.nojs-field-count[for=f-bio]").as_deref(),
        Some("0 / 160")
    );
    assert!(
        page.exists("textarea#f-bio[maxlength='160'][aria-describedby='f-bio-help f-bio-count']")
    );
    assert!(
        page.exists("input[type=date][min='2026-01-01'][max='2027-12-31']")
            && page.exists("input[type=time][min='09:00'][max='17:00']")
    );
    let label = page.bbox("label[for=f-name]").unwrap();
    let input = page.bbox("#f-name").unwrap();
    assert!(
        input.y > label.y + label.height - 1.0,
        "stacked: label above the field: {label:?} {input:?}"
    );

    let inline = Page::render(demo::router(), "/form?layout=inline", MODERN).await;
    let label = inline.bbox("label[for=f-name]").unwrap();
    let input = inline.bbox("#f-name").unwrap();
    assert!(
        input.x >= label.x + label.width - 1.0 && (input.y - label.y).abs() < 20.0,
        "inline: label beside the field: {label:?} {input:?}"
    );
}

#[tokio::test]
async fn wizard_marks_steps() {
    let page = Page::render(demo::router(), "/wizard?step.signup=1", MODERN).await;
    assert_eq!(page.count(".nojs-wizard-steps li"), 3);
    assert_eq!(
        page.text("li[aria-current=step]").as_deref(),
        Some("Newsletter (optional)")
    );
    assert!(
        page.exists(".nojs-wizard-done a[href='/wizard?step.signup=0']"),
        "done step links back"
    );
    assert!(
        !page.exists(".nojs-wizard-steps li:nth-child(3) a"),
        "future step is not a link"
    );
    assert!(
        page.is_visible("input[type=hidden][name=step][value='1'] ~ fieldset")
            || page.is_visible(".nojs-wizard-form fieldset")
    );
    assert!(page.exists("a.nojs-wizard-back[href='/wizard?step.signup=0']"));
    let steps: Vec<_> = (1..=3)
        .map(|i| {
            page.bbox(&format!(".nojs-wizard-steps li:nth-child({i})"))
                .unwrap()
        })
        .collect();
    assert!(
        steps[0].x < steps[1].x && steps[1].x < steps[2].x,
        "steps lay out in a row"
    );
    let first = Page::render(demo::router(), "/wizard", MODERN).await;
    assert_eq!(
        first.text("li[aria-current=step]").as_deref(),
        Some("Account")
    );
    assert!(
        !first.exists(".nojs-wizard-back"),
        "no Back on the first step"
    );
    assert!(
        first.is_visible("progress.nojs-wizard-progress[value='0'][max='2']"),
        "progress bar"
    );
    assert!(!first.exists(".nojs-wizard-resume"), "nothing to resume");
    assert!(
        page.exists(".nojs-wizard-steps li:nth-child(2) small"),
        "the optional step says so"
    );
    assert!(
        page.is_visible("button[name=skip][value='1'][formnovalidate]"),
        "and can be skipped"
    );

    // Coming back to the bare path: the cookie resumes at the review, every value links back.
    let back = Page::render(
        demo::router(),
        "/wizard",
        &format!(
            "{MODERN}; nojs-ui=step.signup=2; wizard=name=Ada&email=ada@example.org&digest=weekly"
        ),
    )
    .await;
    assert_eq!(
        back.text("li[aria-current=step]").as_deref(),
        Some("Review")
    );
    assert!(
        back.is_visible(".nojs-wizard-resume a[href='/wizard?step.signup=0']"),
        "resume notice with Start over"
    );
    assert_eq!(
        back.count(".nojs-wizard-review dd a.nojs-wizard-edit"),
        4,
        "an Edit link per value"
    );
    assert!(
        back.exists("a.nojs-wizard-edit[aria-label='Edit Topics'][href='/wizard?step.signup=1']")
    );
    assert_eq!(
        back.text(".nojs-wizard-review dd:nth-of-type(4) .nojs-note")
            .as_deref(),
        Some("(skipped)")
    );
}

#[tokio::test]
async fn toasts_stack_in_the_corner() {
    let cookie = format!("{MODERN}; nojs-flash=ok%3AInvite%20sent.%0Adanger%3ASync%20failed.");
    let page = Page::render(demo::router(), "/toast", &cookie).await;
    assert_eq!(page.count(".nojs-toast"), 2);
    assert!(
        page.exists(".nojs-toast-danger[role=alert]") && page.exists(".nojs-toast-ok[role=status]")
    );
    assert!(page.exists(".nojs-toast a.nojs-toast-close[href='/toast']"));
    let (list, h1) = (page.bbox(".nojs-toasts").unwrap(), page.bbox("h1").unwrap());
    assert!(
        list.x > h1.x + 100.0,
        "toasts sit at the right edge, out of the flow"
    );
    let empty = Page::render(demo::router(), "/toast", MODERN).await;
    assert!(!empty.exists(".nojs-toasts"));
}

#[tokio::test]
async fn drawer_is_a_sidebar_when_wide_and_breadcrumbs_fold() {
    let page = Page::render(demo::router(), "/nav", MODERN).await;
    assert!(page.exists("button.nojs-drawer-open[command=show-modal][commandfor=site]"));
    assert!(
        page.is_visible(".nojs-drawer-panel nav a[aria-current=page]"),
        "wide viewport: the closed dialog shows as a sidebar"
    );
    assert!(
        !page.is_visible(".nojs-drawer-open"),
        "wide viewport: no menu button"
    );
    let (side, content) = (
        page.bbox(".nojs-drawer-panel").unwrap(),
        page.bbox(".nojs-drawer-content").unwrap(),
    );
    assert!(
        side.x + side.width <= content.x + 1.0,
        "sidebar left of the content"
    );
    assert_eq!(page.count(".nojs-breadcrumbs"), 2);
    assert!(page.exists(".nojs-breadcrumbs li:last-child [aria-current=page]"));
    assert!(
        page.exists(".nojs-breadcrumbs-fold details"),
        "the long trail folds"
    );
    assert!(
        !page.is_visible(".nojs-breadcrumbs-fold ol"),
        "folded middle is closed"
    );
    let old = Page::render(demo::router(), "/nav", OLD).await;
    assert!(
        old.exists("a.nojs-drawer-open[href='#site']"),
        "fallback opener is a :target link"
    );
}

#[tokio::test]
async fn stats_and_empty_state() {
    let page = Page::render(demo::router(), "/dashboard?orders=none", MODERN).await;
    assert_eq!(page.count(".nojs-stat"), 4);
    let (a, b) = (
        page.bbox(".nojs-stat-grid > :nth-child(1)").unwrap(),
        page.bbox(".nojs-stat-grid > :nth-child(2)").unwrap(),
    );
    assert!(
        (a.y - b.y).abs() < 1.0 && b.x > a.x,
        "cards sit side by side when there is room"
    );
    assert!(
        page.exists(".nojs-stat-good") && page.exists(".nojs-stat-bad"),
        "down_is_good flips the colour"
    );
    assert!(
        page.exists("a.nojs-stat[href='/table']"),
        "a card can be a link"
    );
    assert!(page.is_visible(".nojs-empty-title"));
    assert!(page.exists(".nojs-empty-actions a[href='/dashboard']"));
    let full = Page::render(demo::router(), "/dashboard", MODERN).await;
    assert!(!full.exists(".nojs-empty"));
}

#[tokio::test]
async fn command_palette_suggests_and_lists_matches() {
    let page = Page::render(demo::router(), "/palette?q=ta", MODERN).await;
    assert!(page.exists("button.nojs-palette-open[popovertarget=cmd][accesskey=k]"));
    assert!(page.exists("#cmd[popover] input[type=search][list=cmd-list][autofocus]"));
    assert!(page.count("datalist#cmd-list option") >= 19);
    assert!(!page.is_visible("#cmd"), "the popover is closed on arrival");
    assert!(
        page.is_visible(".nojs-palette-results a[href='/table']"),
        "results for a partial query"
    );
    let old = Page::render(demo::router(), "/palette?q=ta", OLD).await;
    assert!(
        old.exists("details#cmd[open] input[name=q]"),
        "fallback is an open disclosure after a search"
    );
}

#[tokio::test]
async fn table_row_edits_in_place() {
    let page = Page::render(
        demo::router(),
        "/table?per.files=5&edit.files=src/build.rs",
        MODERN,
    )
    .await;
    assert!(page.is_visible(
        ".nojs-table-editing .nojs-table-edit-input[name=kind][form='nojs-table-files-edit']"
    ));
    assert!(page.exists("form#nojs-table-files-edit[method=post][action='/table/edit'] input[name=key][value='src/build.rs']"));
    assert_eq!(
        page.count(".nojs-table-edit-input"),
        1,
        "only the edited row, only its editable column"
    );
    let row = page.bbox(".nojs-table-editing").unwrap();
    let other = page.bbox("tbody tr:first-child").unwrap();
    assert!(
        (row.width - other.width).abs() < 1.0,
        "the edited row keeps the table's columns"
    );
    assert!(
        page.exists("tbody tr:not(.nojs-table-editing) a.nojs-button[href*='edit.files=']"),
        "other rows link to their edit"
    );
}

#[tokio::test]
async fn buttons_badges_and_icons() {
    let mut page = Page::render(demo::router(), "/button?loading=1", MODERN).await;
    shot(&mut page, "button-variants");
    let bg = |sel: &str| {
        let id = page.node(sel).unwrap();
        let style = page.doc().get_node(id).unwrap().primary_styles().unwrap();
        format!("{:?}", style.clone_background_color())
    };
    let outline = page
        .bbox(".nojs-cluster > .nojs-button:nth-child(2)")
        .unwrap();
    let primary = page.bbox(".nojs-button-primary").unwrap();
    assert!(
        (outline.height - 36.0).abs() < 1.0,
        "h-9 buttons: {outline:?}"
    );
    assert!(
        (primary.height - outline.height).abs() < 1.0,
        "same height in every tone"
    );
    assert_ne!(
        bg(".nojs-button-primary"),
        bg(".nojs-cluster > .nojs-button:nth-child(2)"),
        "primary is filled, outline is not"
    );
    assert_ne!(bg(".nojs-button-danger"), bg(".nojs-button-primary"));
    // The cluster stretches nothing (align-items: center), so small and icon keep their size.
    let small = page.bbox(".nojs-button-small").unwrap();
    assert!((small.height - 32.0).abs() < 1.0, "small is h-8: {small:?}");
    let icon = page.bbox(".nojs-button-icon").unwrap();
    assert!(
        (icon.width - icon.height).abs() < 1.0,
        "an icon button is square: {icon:?}"
    );
    assert!(
        page.is_visible(".nojs-button-primary .nojs-button-spinner"),
        "?loading=1 shows the spinner"
    );
    assert!(page.html.contains(r#"aria-busy="true" disabled"#));
    assert!(
        page.is_visible("a.nojs-button[href='/']"),
        "a link can look like a button"
    );
    assert_eq!(page.count(".nojs-badge"), 6);
    let svg = page.bbox("svg.nojs-icon").unwrap();
    assert!(
        (svg.width - 16.0).abs() < 1.0 && (svg.height - 16.0).abs() < 1.0,
        "icons are 1rem: {svg:?}"
    );
    assert_eq!(page.count("svg.nojs-icon"), 29);
}

#[tokio::test]
async fn fields_cards_and_layouts() {
    let mut page = Page::render(demo::router(), "/field?email=ada", MODERN).await;
    shot(&mut page, "field-error");
    assert!(
        page.is_visible("#f-email-error"),
        "the server's error shows under the field"
    );
    assert!(page.html.contains(r#"aria-describedby="f-email-error""#));
    assert!(page.is_visible("input.nojs-switch[role=switch]"));
    assert_eq!(page.count(".nojs-radio-group input[type=radio]"), 2);

    let mut page = Page::render(demo::router(), "/card", MODERN).await;
    shot(&mut page, "card");
    let (a, b) = (
        page.bbox(".nojs-grid > .nojs-card:nth-child(1)").unwrap(),
        page.bbox(".nojs-grid > .nojs-card:nth-child(2)").unwrap(),
    );
    assert!(
        (a.y - b.y).abs() < 1.0 && b.x > a.x,
        "two cards side by side at 1000px"
    );
    let action = page.bbox(".nojs-card-action").unwrap();
    let title = page.bbox(".nojs-card-title").unwrap();
    assert!(
        action.x > title.x + title.width,
        "the header action sits right of the title"
    );
    let avatar = page.bbox(".nojs-avatar").unwrap();
    assert!(
        (avatar.width - 32.0).abs() < 1.0 && page.text(".nojs-avatar").as_deref() == Some("AL")
    );

    let mut page = Page::render(demo::router(), "/layout", MODERN).await;
    shot(&mut page, "layout");
    let side = page.bbox(".nojs-split-side").unwrap();
    let main = page.bbox(".nojs-split-main").unwrap();
    assert!(
        (side.y - main.y).abs() < 1.0 && main.x > side.x && main.width > side.width,
        "split: side beside a wider main"
    );
    let first = page.bbox(".nojs-grid > :nth-child(1)").unwrap();
    let second = page.bbox(".nojs-grid > :nth-child(2)").unwrap();
    assert!(
        (second.x - (first.x + first.width) - 8.0).abs() < 1.0,
        "grid.gap(2) is 8px"
    );
}

#[tokio::test]
async fn calendar_date_picker_upload_and_kanban() {
    let page = Page::render(
        demo::router(),
        "/calendar?month.day=2026-09&day=2026-09-17",
        MODERN,
    )
    .await;
    assert_eq!(
        page.count("#nojs-calendar-day .nojs-calendar-grid thead th"),
        7
    );
    let (mo, tu) = (
        page.bbox("#nojs-calendar-day .nojs-calendar-grid tbody tr:first-child td:nth-child(1)")
            .unwrap(),
        page.bbox("#nojs-calendar-day .nojs-calendar-grid tbody tr:first-child td:nth-child(2)")
            .unwrap(),
    );
    assert!(
        (mo.y - tu.y).abs() < 1.0 && tu.x > mo.x,
        "a week is one row of seven cells"
    );
    assert!(
        page.is_visible("#nojs-calendar-day a.nojs-calendar-picked[href*='day=2026-09-17']"),
        "the picked day is drawn"
    );
    assert!(
        page.exists("#nojs-calendar-day span.nojs-calendar-off[aria-disabled=true]"),
        "weekends cannot be picked"
    );
    assert!(
        page.exists("#nojs-calendar-day a[aria-label='Next month'][href*='month.day=2026-10']")
    );
    assert!(
        page.is_visible("#f-due[popovertarget='f-due-calendar']"),
        "the date picker is a button with a popover"
    );
    assert!(
        !page.is_visible("#f-due-calendar"),
        "the popover starts closed"
    );

    let page = Page::render(demo::router(), "/calendar?month.due=2026-10", MODERN).await;
    assert!(
        page.is_visible(".nojs-date-picker > .nojs-calendar"),
        "after a month link the picker's calendar is in the page"
    );

    let page = Page::render(demo::router(), "/upload", MODERN).await;
    assert!(page.exists(
        "form[method=post][enctype='multipart/form-data'] input[type=file][name=file][multiple]"
    ));
    assert!(
        page.exists("progress[data-nojs-progress][hidden]"),
        "the progress bar waits for the script"
    );
    assert!(page.is_visible(".nojs-upload-drop"));

    let page = Page::render(demo::router(), "/kanban", MODERN).await;
    assert_eq!(page.count(".nojs-kanban-column"), 3);
    let (a, b) = (
        page.bbox(".nojs-kanban-column:nth-child(1)").unwrap(),
        page.bbox(".nojs-kanban-column:nth-child(2)").unwrap(),
    );
    assert!(
        (a.y - b.y).abs() < 1.0 && b.x > a.x + a.width,
        "columns side by side"
    );
    assert!(
        page.exists(".nojs-kanban-over"),
        "Doing is past its limit in the starting board"
    );
    assert!(
        page.exists(
            ".nojs-kanban-column:nth-child(1) .nojs-kanban-card button[name=to][value=doing]"
        )
    );
    assert!(
        !page.exists(".nojs-kanban-column:nth-child(1) .nojs-kanban-card button[value=todo]"),
        "no arrow off the board"
    );
}

#[tokio::test]
async fn a_component_written_outside_the_library() {
    let page = Page::render(demo::router(), "/pricing?billing=yearly", MODERN).await;
    assert_eq!(
        page.count(".demo-pricing .nojs-card"),
        3,
        "built from ui.card"
    );
    assert!(
        page.html.contains("<style class=\"nojs-user\">")
            && page.html.matches(".demo-pricing-cta{").count() == 1,
        "Page::css inlines its CSS once"
    );
    let (a, b) = (
        page.bbox("#demo-pricing-hobby").unwrap(),
        page.bbox("#demo-pricing-pro").unwrap(),
    );
    assert!(
        (a.y - b.y).abs() < 1.0 && (a.height - b.height).abs() < 1.0,
        "tiers side by side, equal height"
    );
    assert!(
        page.exists(".demo-pricing-featured a.nojs-button-primary"),
        "the featured tier's button is primary"
    );
    assert!(
        page.text("#demo-pricing-pro .demo-pricing-price")
            .unwrap()
            .contains("$120"),
        "yearly prices"
    );
    assert!(page.exists("a.nojs-button[aria-current=page][href='/pricing?billing=yearly']"));
}
