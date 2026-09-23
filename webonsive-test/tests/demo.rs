//! Every demo route through Blitz: layout assertions plus a PNG under `tests/shots/`.

use webonsive_test::Page;

const MODERN: &str = "wo-cap-probed=1; wo-cap-invokers=1; wo-cap-anchor=1; wo-cap-details_content=1; wo-cap-view_transitions=1; wo-cap-popover=1; wo-cap-light_dark=1; wo-cap-streaming_dsd=1; wo-cap-base_select=1";
const OLD: &str = "wo-cap-probed=1";

fn shot(page: &mut Page, name: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/shots");
    page.screenshot(format!("{dir}/{name}.png")).unwrap();
}

/// The same index under the second palette from `docs/theming.md`: `index-alt.png` next to
/// `index-modern.png` is the proof that a theme is a value passed to `layout_with`.
#[tokio::test]
async fn index_under_another_palette() {
    let mut page = Page::render(demo::router(), "/?palette=linen", MODERN).await;
    assert!(page.exists("style.wo-tokens"), "layout_with emits the token overrides");
    assert!(page.is_visible("h1"));
    shot(&mut page, "index-alt");
}

/// Screenshot every route twice: as a modern browser and as one that supports nothing.
#[tokio::test]
async fn every_route_renders_and_is_captured() {
    for path in demo::PATHS {
        let name = path.trim_start_matches('/').replace(['/', '?', '=', '.'], "-");
        let name = if name.is_empty() { "index".to_string() } else { name };
        for (suffix, cookie) in [("modern", MODERN), ("fallback", OLD)] {
            let mut page = Page::render(demo::router(), path, cookie).await;
            // Blitz has no declarative shadow DOM: see `blitz_has_no_declarative_shadow_dom`.
            let inside_shadow_root = path == "/stream" && suffix == "modern";
            if !inside_shadow_root {
                assert!(page.is_visible("h1"), "{path} ({suffix}) has no visible h1");
                assert!(page.is_visible(".wo-header"), "{path} ({suffix}) has no header");
            }
            shot(&mut page, &format!("{name}-{suffix}"));
        }
    }
}

#[tokio::test]
async fn dialog_variants_follow_caps() {
    let page = Page::render(demo::router(), "/dialog", MODERN).await;
    assert!(page.is_visible("button[command=show-modal]"));
    assert!(!page.exists(".wo-dialog-open"));
    assert!(!page.is_visible("dialog"), "closed dialog must not render");

    let page = Page::render(demo::router(), "/dialog", OLD).await;
    assert!(page.is_visible(".wo-dialog-open"));
    assert!(!page.exists("button[command]"));

    let page = Page::render(demo::router(), "/dialog?dialog=confirm", OLD).await;
    assert!(page.is_visible("dialog[open]"), "server-opened dialog renders");
    assert_eq!(page.text("dialog h2").as_deref(), Some("Delete account?"));
}

#[tokio::test]
async fn tabs_strip_versus_accordion() {
    let modern = Page::render(demo::router(), "/tabs?tab.demo=1", MODERN).await;
    assert_eq!(modern.display(".wo-tabs details").as_deref(), Some("contents"));
    let first = modern.bbox(".wo-tabs summary").unwrap();
    let second = modern.bbox(".wo-tabs details:nth-child(2) summary").unwrap();
    assert!((first.y - second.y).abs() < 1.0, "tab titles share a row: {first:?} {second:?}");
    assert!(second.x > first.x + first.width - 1.0);
    let panel = modern.bbox(".wo-tabs details[open] .wo-tabs-panel").unwrap();
    assert!(panel.y >= first.y + first.height - 2.0, "open panel sits below the strip: {panel:?} vs {first:?}");
    assert!(!modern.is_visible(".wo-tabs details:nth-child(1) .wo-tabs-panel"), "closed panel hidden");

    let old = Page::render(demo::router(), "/tabs?tab.demo=1", OLD).await;
    assert_eq!(old.display(".wo-tabs details").as_deref(), Some("block/flow"));
    let first = old.bbox(".wo-tabs summary").unwrap();
    let second = old.bbox(".wo-tabs details:nth-child(2) summary").unwrap();
    assert!(second.y > first.y + first.height - 1.0, "accordion titles stack");
}

#[tokio::test]
async fn popover_variants() {
    let modern = Page::render(demo::router(), "/popover", MODERN).await;
    assert!(modern.is_visible("button[popovertarget]"));
    assert!(!modern.is_visible("nav[popover]"), "closed popover is hidden");
    let old = Page::render(demo::router(), "/popover", OLD).await;
    assert!(old.is_visible(".wo-popover-details summary"));
    assert!(!old.is_visible(".wo-popover-details nav"), "closed details hides the menu");
}

#[tokio::test]
async fn settings_flash_and_form_values() {
    let cookie = format!("{MODERN}; wo-flash=Settings%20saved.; settings=Ada%7C1; wo-ui=tab.settings=1");
    let page = Page::render(demo::router(), "/settings", &cookie).await;
    assert_eq!(page.text(".wo-flash").as_deref(), Some("Settings saved."));
    assert!(page.is_visible(".wo-flash"));
    assert!(page.is_visible("input[name=notify]"), "notifications tab is open");
    assert!(!page.is_visible("input[name=name][id]"), "profile tab is closed");
    let flash = page.bbox(".wo-flash").unwrap();
    let tabs = page.bbox(".wo-tabs").unwrap();
    assert!(flash.y + flash.height <= tabs.y + 1.0, "flash sits above the tabs");
}

#[tokio::test]
async fn caps_table_lists_every_flag() {
    let page = Page::render(demo::router(), "/caps", MODERN).await;
    assert_eq!(page.count(".wo-caps-table tbody tr"), 9);
    assert_eq!(page.count(".wo-yes"), 9);
    let old = Page::render(demo::router(), "/caps", OLD).await;
    assert_eq!(old.count(".wo-no"), 8);
    assert!(!old.exists(".wo-caps"), "probed browser gets no beacons");
    let fresh = Page::render(demo::router(), "/caps", "").await;
    assert_eq!(fresh.count(".wo-cap"), 9, "unknown browser gets one beacon per flag");
}

/// Documented limitation (FINDINGS.md, M4): Blitz parses `<template>` as inert and does not
/// attach declarative shadow roots, so a DSD page has no rendered body. The fallback variant
/// of the same page renders fully. When this test fails, Blitz has gained DSD: drop the
/// exception in `every_route_renders_and_is_captured`.
#[tokio::test]
async fn blitz_has_no_declarative_shadow_dom() {
    let dsd = Page::render(demo::router(), "/stream", MODERN).await;
    assert!(dsd.html.contains("<template shadowrootmode=\"open\">"));
    assert!(!dsd.is_visible("h1"), "Blitz now renders declarative shadow DOM; update the tests");
    let fallback = Page::render(demo::router(), "/stream", OLD).await;
    assert!(fallback.is_visible("h1"));
    assert_eq!(fallback.count(".wo-stream-section"), 3);
    let boxes: Vec<_> = (1..=3).map(|i| fallback.bbox(&format!(".wo-stream-section:nth-of-type({i})")).unwrap()).collect();
    assert!(boxes[0].y < boxes[1].y && boxes[1].y < boxes[2].y, "sections stack in document order");
}

#[tokio::test]
async fn table_sort_links_and_pages() {
    let page = Page::render(demo::router(), "/table?sort=size&dir=desc&q=a&per=5&page=2", MODERN).await;
    assert_eq!(page.count(".wo-table thead th a"), 3, "every column header is a sort link");
    assert!(page.exists("th[aria-sort=descending] a[href*='sort=size'][href*='dir=asc']"), "sorted column flips direction");
    assert!(page.exists("th a[href*='sort=name'][href*='q=a'][href*='per=5']"), "other links keep filter and page size");
    assert_eq!(page.count(".wo-table tbody tr"), 5, "one page of rows");
    assert!(page.is_visible("a[aria-current=page]"));
    assert_eq!(page.text("a[aria-current=page]").as_deref(), Some("2"));
    assert!(page.exists("a[rel=prev][href*='page=1']") && page.exists("a[rel=next][href*='page=3']"));
    assert_eq!(page.text(".wo-paged-table-range").as_deref(), Some("6–10 of 21"));
    assert!(page.exists("select[name=per] option[value='5'][selected]"));
    let rows = page.bbox(".wo-table tbody").unwrap();
    let nav = page.bbox(".wo-paged-table-nav").unwrap();
    assert!(nav.y >= rows.y + rows.height - 1.0, "pager sits under the rows: {nav:?} vs {rows:?}");
    // Blitz paints sticky header cells at the viewport top (FINDINGS.md); the row still exists.
    assert!(page.exists(".wo-table thead th"));
}

/// `/swap`: the controls that name a target sit outside every swap root, and the page is
/// still complete without the script (count shown, list rendered from the cookie).
#[tokio::test]
async fn swap_targets_render_without_script() {
    let page = Page::render(demo::router(), "/swap?n=3", "wo-cap-probed=1; notes=one|two").await;
    assert_eq!(page.text("#count").as_deref(), Some("3"));
    assert_eq!(page.count("#log li"), 2);
    assert!(page.exists("a[data-wo-target='#count']"));
    assert!(page.exists("form[data-wo-target='#log'][data-wo-swap='append']"));
    assert!(!page.exists("form [data-wo=swap]"), "the form is not inside a root");
}

#[tokio::test]
async fn wizard_marks_steps() {
    let page = Page::render(demo::router(), "/wizard?step.signup=1", MODERN).await;
    assert_eq!(page.count(".wo-wizard-steps li"), 3);
    assert_eq!(page.text("li[aria-current=step]").as_deref(), Some("Preferences"));
    assert!(page.exists(".wo-wizard-done a[href='/wizard?step.signup=0']"), "done step links back");
    assert!(!page.exists(".wo-wizard-steps li:nth-child(3) a"), "future step is not a link");
    assert!(page.is_visible("input[type=hidden][name=step][value='1'] ~ fieldset") || page.is_visible(".wo-wizard-form fieldset"));
    assert!(page.exists("a.wo-wizard-back[href='/wizard?step.signup=0']"));
    let steps: Vec<_> = (1..=3).map(|i| page.bbox(&format!(".wo-wizard-steps li:nth-child({i})")).unwrap()).collect();
    assert!(steps[0].x < steps[1].x && steps[1].x < steps[2].x, "steps lay out in a row");
    let first = Page::render(demo::router(), "/wizard", MODERN).await;
    assert_eq!(first.text("li[aria-current=step]").as_deref(), Some("Account"));
    assert!(!first.exists(".wo-wizard-back"), "no Back on the first step");
}
