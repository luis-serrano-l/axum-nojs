//! Every page of the app as a browser without script sees it: the account pages (sign up,
//! verify, sign in, forgot and reset password, magic link), then the scaffolded notes pages, through the router Loco boots (its middleware, `auth::JWT`, the loco-ui
//! initializer). Each GET page ships only the enhancement script, and Blitz (no script
//! engine) renders it into `tests/shots/loco-*.png`.

use axum::{
    Router,
    body::Body,
    http::{Request, Response, StatusCode, header},
};
use loco_app::{app::App, models::users};
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

/// Ada's row, for the tokens her mails would carry.
async fn ada(db: &sea_orm::DatabaseConnection) -> users::Model {
    users::Model::find_by_email(db, "ada@example.com")
        .await
        .unwrap()
}

#[tokio::test]
#[serial]
async fn every_page_works_without_script() {
    let boot = boot_test::<App>().await.unwrap();
    let db = boot.app_context.db.clone();
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
    assert_eq!(location(&res), "/");
    let auth = auth_cookie(&res);

    // The welcome mail's link verifies the address once; a made-up one is refused.
    let token = ada(&db)
        .await
        .email_verification_token
        .expect("sign-up mails a link");
    let res = send(&router, "GET", &format!("/verify/{token}"), "", "").await;
    assert_eq!(location(&res), "/");
    assert!(ada(&db).await.email_verified_at.is_some());
    let res = send(&router, "GET", "/verify/made-up", "", "").await;
    assert_eq!(location(&res), "/signin");

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
    assert_eq!(location(&res), "/");

    // Forgot password: the same answer for any email; the mailed link sets a new password
    // once, and the old one stops working.
    let res = send(&router, "POST", "/forgot", "", "email=nobody%40example.com").await;
    assert_eq!(location(&res), "/signin");
    let res = send(&router, "POST", "/forgot", "", "email=ada%40example.com").await;
    assert_eq!(location(&res), "/signin");
    let reset = format!(
        "/reset/{}",
        ada(&db).await.reset_token.expect("a reset link")
    );
    let res = send(&router, "POST", &reset, "", "password=").await;
    assert!(text(res).await.contains("This field is required."));
    let res = send(&router, "POST", &reset, "", "password=better").await;
    assert_eq!(location(&res), "/signin");
    let res = send(
        &router,
        "POST",
        "/signin",
        "",
        "email=ada%40example.com&password=secret",
    )
    .await;
    assert!(text(res).await.contains("Wrong email or password."));
    let res = send(
        &router,
        "POST",
        "/signin",
        "",
        "email=ada%40example.com&password=better",
    )
    .await;
    assert_eq!(location(&res), "/");
    assert!(
        text(send(&router, "GET", &reset, "", "").await)
            .await
            .contains("Link expired")
    );

    // Magic link: signs in once, then the link is spent.
    let res = send(
        &router,
        "POST",
        "/magic-link",
        "",
        "email=ada%40example.com",
    )
    .await;
    assert_eq!(location(&res), "/signin");
    let magic = format!(
        "/magic-link/{}",
        ada(&db).await.magic_link_token.expect("a sign-in link")
    );
    let res = send(&router, "GET", &magic, "", "").await;
    assert_eq!(location(&res), "/");
    auth_cookie(&res);
    assert!(
        text(send(&router, "GET", &magic, "", "").await)
            .await
            .contains("Link expired")
    );

    // A reset link for the pages below.
    send(&router, "POST", "/forgot", "", "email=ada%40example.com").await;
    let reset = format!("/reset/{}", ada(&db).await.reset_token.unwrap());

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

    // Every field kind the scaffold knows, on `tasks`: the reference is a select of users by
    // name, date-times are `datetime-local` (posted without seconds), the decimal a pattern.
    let html = text(send(&router, "GET", "/tasks/new", &auth, "").await).await;
    assert!(
        html.contains(r#"<option value="1">Ada</option>"#),
        "user select: {html}"
    );
    assert!(html.contains(r#"type="datetime-local""#) && html.contains(r#"type="number""#));
    let res = send(
        &router,
        "POST",
        "/tasks",
        &auth,
        "title=&size=many&starts_at=soon",
    )
    .await;
    let html = text(res).await;
    for message in [
        "Title: This field is required.",
        "Size: Check this field.",
        "Starts at: Check this field.",
    ] {
        assert!(html.contains(message), "no {message} in {html}");
    }
    let task = "title=Ship&user_id=1&done=true&due_on=2026-10-01&starts_at=2026-10-01T09%3A30\
                &remind_at=2026-09-30T18%3A00&price=12.50&status=doing&size=3";
    let res = send(&router, "POST", "/tasks", &auth, task).await;
    assert_eq!(location(&res), "/tasks/1");
    let html = text(send(&router, "GET", "/tasks/1/edit", &auth, "").await).await;
    for filled in [
        r#"value="2026-10-01T09:30""#,
        r#"value="2026-09-30T18:00""#,
        r#"value="12.5""#, // SQLite keeps a decimal as a real
        r#"<option value="1" selected>Ada</option>"#,
        r#"<option value="doing" selected>"#,
    ] {
        assert!(
            html.contains(filled),
            "task edit form: no {filled} in {html}"
        );
    }

    let pages = [
        ("/", "index"),
        ("/signin", "signin"),
        ("/signup", "signup"),
        ("/forgot", "forgot"),
        (reset.as_str(), "reset"),
        ("/reset/made-up", "link-expired"),
        ("/magic-link", "magic-link"),
        ("/notes", "notes"),
        ("/notes/new", "notes-new"),
        ("/notes/1", "notes-show"),
        ("/notes/1/edit", "notes-edit"),
        ("/tasks/new", "tasks-new"),
        ("/tasks/1/edit", "tasks-edit"),
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

    // A path no route answers is loco-ui's 404 page, not Loco's plain one.
    let res = send(&router, "GET", "/no-such-page", "", "").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let html = text(res).await;
    assert!(html.contains("Page not found"), "{html}");

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
