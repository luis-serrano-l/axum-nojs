//! Every page of the app as a browser without script sees it: sign up, then the scaffolded
//! notes pages, through the router Loco boots (its middleware, `auth::JWT`, the loco-ui
//! initializer). Each GET page ships only the enhancement script, and Blitz (no script
//! engine) renders it into `tests/shots/loco-*.png`.

use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode, header},
};
use loco_app::app::App;
use loco_rs::testing::prelude::*;
use loco_ui_test::Page;
use serial_test::serial;
use tower::ServiceExt;

async fn send(
    router: &Router,
    method: &str,
    path: &str,
    cookie: &str,
    form: &str,
) -> Response<Body> {
    let req = Request::builder()
        .method(method)
        .uri(path)
        .header(header::COOKIE, cookie)
        .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
        .body(Body::from(form.to_string()))
        .unwrap();
    router.clone().oneshot(req).await.unwrap()
}

async fn text(res: Response<Body>) -> String {
    let bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
        .await
        .unwrap();
    String::from_utf8(bytes.to_vec()).unwrap()
}

fn location(res: &Response<Body>) -> &str {
    res.headers()[header::LOCATION].to_str().unwrap()
}

/// The `auth=…` pair from a response's `Set-Cookie` headers.
fn auth_cookie(res: &Response<Body>) -> String {
    res.headers()
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("auth="))
        .and_then(|v| v.split(';').next())
        .expect("sign-in sets the auth cookie")
        .to_string()
}

#[tokio::test]
#[serial]
async fn every_page_works_without_script() {
    let boot = boot_test::<App>().await.unwrap();
    let router = boot.router.unwrap();
    let tag = loco_ui::enhance::script_tag().into_string();

    // Signed out, the scaffold's routes are Loco's 401.
    let res = send(&router, "GET", "/notes", "", "").await;
    assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

    // A blank sign-up comes back with a message on each field, not a JSON error.
    let res = send(&router, "POST", "/signup", "", "name=&email=&password=").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(text(res).await.contains("This field is required."));

    let res = send(
        &router,
        "POST",
        "/signup",
        "",
        "name=Ada&email=ada%40example.com&password=secret",
    )
    .await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&res), "/notes");
    let auth = auth_cookie(&res);

    // A wrong password is a message on the form; the right one signs in.
    let res = send(
        &router,
        "POST",
        "/signin",
        "",
        "email=ada%40example.com&password=nope",
    )
    .await;
    assert!(text(res).await.contains("Wrong email or password."));
    let res = send(
        &router,
        "POST",
        "/signin",
        "",
        "email=ada%40example.com&password=secret",
    )
    .await;
    assert_eq!(location(&res), "/notes");

    // The generated create: a missing title re-renders the form with what was typed.
    let res = send(&router, "POST", "/notes", &auth, "title=&body=kept").await;
    assert_eq!(res.status(), StatusCode::OK);
    let html = text(res).await;
    assert!(
        html.contains("This field is required.") && html.contains("kept"),
        "{html}"
    );
    let res = send(
        &router,
        "POST",
        "/notes",
        &auth,
        "title=First&body=Hello&done=on&due=2026-10-01",
    )
    .await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_eq!(location(&res), "/notes/1");

    let pages = [
        ("/", "index"),
        ("/signin", "signin"),
        ("/signup", "signup"),
        ("/notes", "notes"),
        ("/notes/new", "notes-new"),
        ("/notes/1", "notes-show"),
        ("/notes/1/edit", "notes-edit"),
    ];
    for (path, name) in pages {
        let res = send(&router, "GET", path, &auth, "").await;
        assert_eq!(res.status(), StatusCode::OK, "{path}");
        let html = text(res).await;
        if path == "/notes/1/edit" {
            // The edit form starts from the saved note, every field filled in.
            for filled in [
                r#"value="First""#,
                ">Hello</textarea>",
                r#"value="2026-10-01""#,
                "checked",
            ] {
                assert!(html.contains(filled), "edit form: no {filled} in {html}");
            }
        }
        assert_eq!(html.matches("<script").count(), 1, "{path}: one script");
        assert!(
            html.contains(&tag),
            "{path}: and it is the enhancement script"
        );

        let mut page = Page::render(router.clone(), path, &auth).await;
        assert!(page.is_visible("h1"), "{path}: Blitz lays the page out");
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/shots");
        page.screenshot(format!("{dir}/loco-{name}.png")).unwrap();
    }

    // The script and the beacon the pages ask for are mounted.
    let res = send(&router, "GET", loco_ui::enhance::SCRIPT_PATH, "", "").await;
    assert_eq!(res.status(), StatusCode::OK);

    // Update and delete, each Post/Redirect/Get.
    let res = send(&router, "POST", "/notes/1", &auth, "title=Renamed").await;
    assert_eq!(location(&res), "/notes/1");
    assert!(
        text(send(&router, "GET", "/notes/1", &auth, "").await)
            .await
            .contains("Renamed")
    );
    let res = send(&router, "POST", "/notes/1/delete", &auth, "").await;
    assert_eq!(location(&res), "/notes");
    let res = send(&router, "GET", "/notes/1", &auth, "").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}
