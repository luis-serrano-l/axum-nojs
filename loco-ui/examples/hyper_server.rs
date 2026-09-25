//! axum-nojs on raw hyper, no framework: `cargo run -p axum-nojs --example hyper_server --features http`,
//! then open http://127.0.0.1:3002.
//!
//! Everything the Axum glue does is wired here by hand, in a few lines each: `Ui` (caps, theme
//! and state) from path + query + the `Cookie:` header, the beacon route from
//! `caps::beacon_cookie`, Post/Redirect/Get from `ui.redirect`, the enhancement script from
//! `enhance::served()`. Three components: a dialog, tabs, and a counter kept in a cookie.
//!
//! **HTTP/2.** The connection builder speaks HTTP/1.1 and HTTP/2 on the same port and picks
//! by the client's preface, so a first visit's beacon images, the script and the page share
//! one connection instead of opening six: `curl --http2-prior-knowledge http://127.0.0.1:3002/`.
//! Browsers only speak HTTP/2 over TLS, so in production put TLS (and HTTP/3) in front: a
//! proxy such as Caddy or nginx terminates `h2`/`h3` and forwards here as `h2c` or HTTP/1.1.
//! `docs/caps.md` explains why the beacons cost nothing after the first visit.

use std::convert::Infallible;
use std::net::SocketAddr;

use axum_nojs::{Ui, caps, enhance};
use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, header};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use maud::html;
use tokio::net::TcpListener;

type Reply = Response<Full<Bytes>>;

fn html_reply(set_cookies: Vec<String>, body: String) -> Reply {
    let mut res = Response::builder().header(header::CONTENT_TYPE, "text/html; charset=utf-8");
    for c in set_cookies {
        res = res.header(header::SET_COOKIE, c);
    }
    res.body(Full::new(Bytes::from(body))).unwrap()
}

fn cookie<'a>(cookies: &'a str, name: &str) -> Option<&'a str> {
    cookies
        .split(';')
        .filter_map(|p| p.trim().split_once('='))
        .find(|(k, _)| *k == name)
        .map(|(_, v)| v)
}

async fn handle(req: Request<Incoming>) -> Result<Reply, Infallible> {
    let cookies = req
        .headers()
        .get_all(header::COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .collect::<Vec<_>>()
        .join("; ");
    let (path, query) = (
        req.uri().path().to_string(),
        req.uri().query().unwrap_or("").to_string(),
    );
    // Caps (`?caps=` first, then the cookies), theme and UI state: the whole of the Axum extractor.
    let ui = Ui::from_request(&path, &query, &cookies);
    let count: i64 = cookie(&cookies, "count")
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let reply = match (req.method(), path.as_str()) {
        (&Method::GET, "/") => {
            let page = ui.page("axum-nojs on hyper", html! {
                h1 { "axum-nojs on hyper" }
                p { "This browser supports: " @for n in ui.caps.names() { code { (n) } " " } }
                (ui.dialog("Open dialog").body(html! { p { "Closed by the platform, not by script." } }))
                h2 { "Tabs" }
                (ui.tabs("demo")
                    .tab("First", html! { p { "Tab state lives in the URL and a cookie." } })
                    .tab("Second", html! { p { "Reload, leave, come back: still here." } }))
                h2 { "Counter" }
                (ui.flash())
                (ui.counter("/counter", count))
            });
            html_reply(page.set_cookies().to_vec(), page.into_string())
        }
        (&Method::POST, "/counter") => {
            let body = req
                .into_body()
                .collect()
                .await
                .map(|b| b.to_bytes())
                .unwrap_or_default();
            let op = cookie(&String::from_utf8_lossy(&body).replace('&', ";"), "op")
                .unwrap_or("")
                .to_string();
            let next = ui.counter("/counter", count).apply(&op, None);
            ui.redirect("/")
                .flash("Counted.")
                .cookie(format!("count={next}; Path=/; SameSite=Lax"))
                .into_http()
        }
        (&Method::GET, caps::BEACON_PATH) => {
            // The beacon route: 204 + Set-Cookie for a known flag, 404 otherwise, never cached.
            let mut res = Response::builder().header(header::CACHE_CONTROL, "no-store");
            res = match caps::beacon_cookie(&query) {
                Some(c) => res
                    .status(StatusCode::NO_CONTENT)
                    .header(header::SET_COOKIE, c),
                None => res.status(StatusCode::NOT_FOUND),
            };
            res.body(Full::default()).unwrap()
        }
        (&Method::GET, enhance::SCRIPT_PATH) => Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .body(Full::new(Bytes::from_static(enhance::served().as_bytes())))
            .unwrap(),
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::default())
            .unwrap(),
    };
    Ok(reply)
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3002));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("http://{addr}");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            // HTTP/1.1 or HTTP/2 (h2c), chosen per connection by what the client sends first.
            let builder = auto::Builder::new(TokioExecutor::new());
            if let Err(e) = builder
                .serve_connection(TokioIo::new(stream), service_fn(handle))
                .await
            {
                eprintln!("connection error: {e}");
            }
        });
    }
}
