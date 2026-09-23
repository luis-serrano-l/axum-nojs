//! Every demo route through Blitz: layout assertions plus a PNG under `tests/shots/`.

use webonsive_test::Page;

const MODERN: &str = "wo-cap-probed=1; wo-cap-invokers=1; wo-cap-anchor=1; wo-cap-details_content=1; wo-cap-view_transitions=1; wo-cap-popover=1; wo-cap-light_dark=1; wo-cap-streaming_dsd=1";
const OLD: &str = "wo-cap-probed=1";

fn shot(page: &mut Page, name: &str) {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../tests/shots");
    page.screenshot(format!("{dir}/{name}.png")).unwrap();
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
    assert_eq!(page.count(".wo-caps-table tbody tr"), 8);
    assert_eq!(page.count(".wo-yes"), 8);
    let old = Page::render(demo::router(), "/caps", OLD).await;
    assert_eq!(old.count(".wo-no"), 7);
    assert!(!old.exists(".wo-caps"), "probed browser gets no beacons");
    let fresh = Page::render(demo::router(), "/caps", "").await;
    assert_eq!(fresh.count(".wo-cap"), 8, "unknown browser gets one beacon per flag");
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
