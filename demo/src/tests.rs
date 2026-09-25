//! The demo's own tests: one script only, strict CSP, snippets cut from the source, state
//! round trips, and markup that follows the capabilities.

use crate::code::{SOURCES, highlight, snippet};
use crate::site::{COMPONENTS, LAYERS};
use crate::{PATHS, router};
use axum::body::Body;
use axum::http::Request;
use loco_ui::prelude::*;
use tower::ServiceExt;

/// Every route answers with the strict CSP (`script-src 'self'`, nothing inline), and
/// `?script=off` with `script-src 'none'` and no script tag at all.
#[tokio::test]
async fn every_route_is_served_under_a_strict_csp() {
    let policy = |path: &'static str| async move {
        let req = Request::get(path).body(Body::empty()).unwrap();
        let res = router().oneshot(req).await.unwrap();
        let csp = res.headers()["content-security-policy"]
            .to_str()
            .unwrap()
            .to_string();
        let html = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        (csp, String::from_utf8(html.to_vec()).unwrap())
    };
    for path in PATHS {
        let (csp, _) = policy(path).await;
        assert!(
            csp.contains("script-src 'self';") && csp.contains("object-src 'none'"),
            "{path}: {csp}"
        );
    }
    let (csp, html) = policy("/dialog?script=off").await;
    assert!(csp.contains("script-src 'none'"), "{csp}");
    assert!(
        html.matches("<script").count() == 0,
        "a script-less page has no script tag"
    );
}

#[test]
fn every_index_entry_sits_in_a_layer() {
    for (href, _, group, ..) in COMPONENTS {
        assert!(
            LAYERS.iter().any(|l| l.2.contains(&group)),
            "{href}: group {group} is in no layer"
        );
    }
}

/// The only script on any page is the one optional enhancement tag: no inline script,
/// no handlers, no `javascript:` URLs. Blitz (no script engine) proves the pages work
/// without it.
#[tokio::test]
async fn pages_ship_only_the_enhancement_script() {
    let tag = loco_ui::enhance::script_tag().into_string();
    for path in PATHS {
        let modern = Cap::ALL
            .map(|c| format!("lui-cap-{}=1", c.name()))
            .join("; ");
        for cookie in ["", modern.as_str()] {
            let req = Request::get(path)
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap();
            let res = router().oneshot(req).await.unwrap();
            assert_eq!(res.status(), 200, "{path}");
            let body = axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap();
            let html = String::from_utf8(body.to_vec()).unwrap();
            // The whole page, inline stylesheet included (README: "What a page weighs").
            // A streamed page with declarative shadow DOM carries the stylesheet twice
            // (styles do not cross into the shadow root), so it gets its own budget.
            let budget = if html.contains("shadowrootmode") {
                128
            } else {
                96
            } * 1024;
            assert!(
                html.len() < budget,
                "{path}: {} bytes, over the {} KB page budget",
                html.len(),
                budget / 1024
            );
            assert_eq!(
                html.matches("<script").count(),
                1,
                "{path}: exactly one script tag"
            );
            assert!(
                html.contains(&tag),
                "{path}: the tag is the enhancement script"
            );
            assert!(!html.contains("javascript:"), "{path}");
            let inline_handler = html.split('<').any(|tag| {
                tag.split_whitespace()
                    .any(|a| a.starts_with("on") && a.contains('='))
            });
            assert!(!inline_handler, "{path}: inline event handler");
        }
    }
}

/// Every component page shows code cut from the route files, so the snippet cannot drift from
/// what runs; every `// code:` marker is closed, names a component page, and a page's markers
/// all sit in one file.
#[tokio::test]
async fn every_component_page_shows_its_code() {
    for (path, source) in SOURCES {
        let opens = source
            .lines()
            .filter(|l| l.trim().starts_with("// code: "))
            .count();
        let closes = source.lines().filter(|l| l.trim() == "// end code").count();
        assert_eq!(opens, closes, "{path}: every // code: has its // end code");
        for l in source
            .lines()
            .filter_map(|l| l.trim().strip_prefix("// code: "))
        {
            assert!(
                COMPONENTS.iter().any(|c| c.0 == l),
                "{path}: {l} is not a component page"
            );
        }
    }
    for (href, ..) in COMPONENTS {
        let open = format!("// code: {href}");
        let files = SOURCES
            .iter()
            .filter(|(_, s)| s.lines().any(|l| l.trim() == open))
            .count();
        assert_eq!(
            files, 1,
            "{href}: its markers are in {files} files, not one"
        );
        let (path, code) = snippet(href);
        let source = SOURCES.iter().find(|s| s.0 == path).unwrap().1;
        assert!(!code.trim().is_empty(), "{href}: no snippet");
        assert!(
            code.lines().all(|l| source.contains(l)),
            "{href}: snippet not cut from {path}"
        );
        let res = router()
            .oneshot(Request::get(href).body(Body::empty()).unwrap())
            .await
            .unwrap();
        let html = String::from_utf8(
            axum::body::to_bytes(res.into_body(), usize::MAX)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        // The highlighted block, tags stripped, is exactly the code.
        let pre = html
            .split("class=\"lui-snippet\"")
            .nth(1)
            .and_then(|s| s.split("<pre>").nth(1))
            .and_then(|s| s.split("</pre>").next());
        let text: String = pre
            .expect("{href}: no snippet box")
            .split('<')
            .map(|p| p.split_once('>').map_or(p, |(_, t)| t))
            .collect();
        assert_eq!(
            text,
            html! { (code) }.into_string(),
            "{href}: the box does not show its code"
        );
        assert!(
            pre.unwrap().contains("class=\"lui-hl-"),
            "{href}: not highlighted"
        );
    }
    let code = highlight(
        r#"let t = ui.tabs("demo").badge(3); // lazy
html! { @if x { Some(Page) } }"#,
    )
    .into_string();
    for part in [
        r#"hl-k">let<"#,
        r#"hl-f">tabs<"#,
        r#"hl-s">&quot;demo&quot;<"#,
        r#"hl-n">3<"#,
        r#"hl-c">// lazy"#,
        r#"hl-m">html!<"#,
        r#"hl-k">if<"#,
        r#"hl-t">Some<"#,
    ] {
        assert!(code.contains(part), "{part} in {code}");
    }
}

#[tokio::test]
async fn whole_pages_are_compressed_and_streams_are_not() {
    let gz = |path: &str| {
        Request::get(path)
            .header("accept-encoding", "gzip")
            .body(Body::empty())
            .unwrap()
    };
    let res = router().oneshot(gz("/table")).await.unwrap();
    assert_eq!(
        res.headers()["content-encoding"],
        "gzip",
        "a whole page is compressed"
    );
    let res = router().oneshot(gz("/stream")).await.unwrap();
    assert!(
        res.headers().get("content-encoding").is_none(),
        "a stream keeps its chunks"
    );
    let res = router()
        .oneshot(Request::get("/table").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert!(res.headers().get("content-encoding").is_none());
    assert!(
        axum::body::HttpBody::size_hint(res.body())
            .exact()
            .is_some(),
        "plain pages carry a Content-Length"
    );
}

#[tokio::test]
async fn enhanced_requests_get_the_page_without_its_stylesheet() {
    let get = |enhanced: bool| {
        let req = Request::get("/tabs?tab.demo=1");
        let req = if enhanced {
            req.header("lui-enhance", "1")
        } else {
            req
        };
        router().oneshot(req.body(Body::empty()).unwrap())
    };
    let full = get(false).await.unwrap();
    assert_eq!(full.headers()["vary"], "lui-enhance, cookie");
    let full = String::from_utf8(
        axum::body::to_bytes(full.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let slim = get(true).await.unwrap();
    assert_eq!(slim.headers()["vary"], "lui-enhance, cookie");
    let slim = String::from_utf8(
        axum::body::to_bytes(slim.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        full.contains("<style>") && !slim.contains("<style>"),
        "the stylesheet stays home"
    );
    assert!(
        slim.contains("id=\"lui-tabs-demo\"") && slim.contains("<title>"),
        "the swap root and title are still there"
    );
    assert!(
        slim.len() * 3 < full.len(),
        "slim is {} of {} bytes",
        slim.len(),
        full.len()
    );
}

#[tokio::test]
async fn enhancement_script_is_served_immutable() {
    let req = Request::get(loco_ui::enhance::script_url())
        .body(Body::empty())
        .unwrap();
    let res = router().oneshot(req).await.unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.headers()["content-type"],
        "text/javascript; charset=utf-8"
    );
    assert!(
        res.headers()["cache-control"]
            .to_str()
            .unwrap()
            .contains("immutable")
    );
    let body = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    assert_eq!(body, loco_ui::enhance::served().as_bytes());
}

#[tokio::test]
async fn caps_beacon_sets_one_cookie_per_flag() {
    let req = Request::get("/lui/caps?flag=popover")
        .body(Body::empty())
        .unwrap();
    let res = router().oneshot(req).await.unwrap();
    assert_eq!(res.status(), 204);
    let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(cookie.starts_with("lui-cap-popover=1;"), "{cookie}");
    let req = Request::get("/lui/caps?flag=nope")
        .body(Body::empty())
        .unwrap();
    assert_eq!(router().oneshot(req).await.unwrap().status(), 404);
}

/// Collect the body frames of `path` as they arrive.
async fn frames(path: &str, cookie: &str) -> Vec<String> {
    use http_body_util::BodyExt;
    let req = Request::get(path)
        .header("cookie", cookie)
        .body(Body::empty())
        .unwrap();
    let mut body = router().oneshot(req).await.unwrap().into_body();
    let mut out = Vec::new();
    while let Some(frame) = body.frame().await {
        let frame = frame.unwrap();
        if let Some(data) = frame.data_ref() {
            out.push(String::from_utf8(data.to_vec()).unwrap());
        }
    }
    out
}

#[tokio::test]
async fn stream_is_chunked_in_completion_order() {
    let chunks = frames("/stream", "lui-cap-probed=1; lui-cap-streaming_dsd=1").await;
    assert!(
        chunks.len() >= 6,
        "expected head + shell + 3 fills + suffix, got {}",
        chunks.len()
    );
    assert!(
        chunks[0].ends_with("</head>") && chunks[0].contains("<style>"),
        "the head, stylesheet included, goes first"
    );
    assert!(chunks[1].contains("<template shadowrootmode=\"open\">"));
    assert!(chunks[1].contains("<slot name=\"slow\">"));
    let order: Vec<&str> = chunks[2..5]
        .iter()
        .map(|c| {
            c.split("slot=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap()
        })
        .collect();
    assert_eq!(order, ["fast", "medium", "slow"]);
    assert!(
        chunks.last().unwrap().ends_with("</script></body></html>"),
        "suffix carries the enhancement tag"
    );
}

#[tokio::test]
async fn stream_fallback_is_in_document_order() {
    let chunks = frames("/stream", "lui-cap-probed=1").await;
    let html = chunks.concat();
    assert!(!html.contains("<template") && !html.contains("<slot"));
    let pos = |s: &str| html.find(s).unwrap();
    assert!(
        pos("slow</strong>") < pos("medium</strong>")
            && pos("medium</strong>") < pos("fast</strong>")
    );
    assert!(
        chunks.len() >= 5,
        "streamed in pieces, got {}",
        chunks.len()
    );
    assert!(chunks[0].ends_with("</head>"), "the head goes first");
}

#[tokio::test]
async fn palette_exact_name_redirects_and_toast_posts_stack() {
    let res = router()
        .oneshot(
            Request::get("/palette?q=largest%20FILES")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), 303);
    assert_eq!(
        res.headers().get("location").unwrap(),
        "/table?sort=size&dir=desc"
    );
    let res = router()
        .oneshot(Request::get("/palette?q=zzz").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let req = Request::post("/toast")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("kind=all"))
        .unwrap();
    let res = router().oneshot(req).await.unwrap();
    let cookie = res.headers().get("set-cookie").unwrap().to_str().unwrap();
    assert!(
        cookie.starts_with("lui-flash=ok%3AInvite")
            && cookie.contains("%0Awarn%3A")
            && cookie.contains("%0Adanger%3A"),
        "{cookie}"
    );
}

#[tokio::test]
async fn state_round_trip_through_prg_and_cookies() {
    // POST → 303 with a flash cookie and the values saved; the tab is in the lui-ui cookie.
    let req = Request::post("/settings")
        .header("content-type", "application/x-www-form-urlencoded")
        .body(Body::from("name=Ada&notify=true"))
        .unwrap();
    let res = router().oneshot(req).await.unwrap();
    assert_eq!(res.status(), 303);
    assert_eq!(res.headers().get("location").unwrap(), "/settings");
    let cookies: Vec<String> = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();
    assert!(
        cookies
            .iter()
            .any(|c| c.starts_with("lui-flash=ok%3ASettings%20saved.")),
        "{cookies:?}"
    );
    assert!(
        cookies
            .iter()
            .any(|c| c.starts_with("lui-settings=name%3DAda%26notify%3Dtrue;")),
        "{cookies:?}"
    );
    // GET the redirect target: flash shown and cleared, tab persisted to lui-ui, values filled in.
    let req = Request::get("/settings?tab.settings=1")
        .header(
            "cookie",
            "lui-flash=Settings%20saved.; lui-settings=name%3DAda%26notify%3Dtrue",
        )
        .body(Body::empty())
        .unwrap();
    let res = router().oneshot(req).await.unwrap();
    let cookies: Vec<String> = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap().to_string())
        .collect();
    assert!(
        cookies
            .iter()
            .any(|c| c.starts_with("lui-ui=tab.settings=1;")),
        "{cookies:?}"
    );
    assert!(
        cookies
            .iter()
            .any(|c| c.starts_with("lui-flash=; Path=/; Max-Age=0")),
        "{cookies:?}"
    );
    let html = String::from_utf8(
        axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(
        html.contains("Settings saved.")
            && html.contains("value=\"Ada\"")
            && html.contains("checked")
    );
    assert!(
        html.contains("<details name=\"settings\" open>")
            && html.contains("href=\"/settings?tab.settings=0\"")
    );
    // Coming back with only the cookie: the tab is still open, nothing is rewritten.
    let req = Request::get("/settings")
        .header("cookie", "lui-ui=tab.settings=1")
        .body(Body::empty())
        .unwrap();
    let res = router().oneshot(req).await.unwrap();
    assert!(res.headers().get("set-cookie").is_none());
    let html = String::from_utf8(
        axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(html.contains("<details name=\"settings\" open><summary><a href=\"/settings?tab.settings=1\">Notifications"));
}

#[tokio::test]
async fn markup_follows_caps() {
    async fn body(path: &str, cookie: &str) -> String {
        let req = Request::get(path)
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap();
        let res = router().oneshot(req).await.unwrap();
        let body = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        String::from_utf8(body.to_vec()).unwrap()
    }
    let old = body("/dialog", "").await;
    assert!(old.contains("href=\"#confirm\"") && !old.contains("commandfor"));
    assert!(old.contains("lui-caps"), "unknown browser gets beacons");
    let new = body("/dialog", "lui-cap-probed=1; lui-cap-invokers=1").await;
    assert!(new.contains("commandfor=\"confirm\"") && !new.contains("href=\"#confirm\""));
    assert!(
        !new.contains("class=\"lui-caps\""),
        "probed browser gets no beacons"
    );
    assert!(body("/popover", "").await.contains("lui-popover-details"));
    assert!(
        body("/tabs", "lui-cap-details_content=1")
            .await
            .contains("lui-tabs-panel")
    );
    assert!(body("/tabs", "").await.contains("lui-accordion-body"));
}

/// A component page lists what its builders accept under the snippet, from
/// `loco_ui::props()`: the tabs page has a `Tabs` table with every setter in it.
#[tokio::test]
async fn component_pages_show_their_props() {
    let res = router()
        .oneshot(Request::get("/tabs").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let html = String::from_utf8(
        axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    let section = html
        .split("class=\"lui-props\"")
        .nth(1)
        .expect("/tabs: no props section");
    assert!(section.contains("<summary><code>Tabs</code> <code>ui.tabs(name: &amp;str)</code>"));
    let tabs = loco_ui::props()
        .iter()
        .find(|c| c.builder == "Tabs")
        .unwrap();
    for p in tabs.props {
        assert!(
            section.contains(&format!("<td><code>{}</code></td>", p.name)),
            "/tabs: no row for {}",
            p.name
        );
    }
}
