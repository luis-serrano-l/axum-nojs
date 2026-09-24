//! webonsive on raw hyper, no framework: `cargo run -p webonsive --example hyper_server --features http`,
//! then open http://127.0.0.1:3002.
//!
//! Everything the Axum glue does is wired here by hand, in a few lines each: `Caps` from the
//! `Cookie:` header, `UiState` from path + query + cookies, the beacon route from
//! `caps::beacon_cookie`, Post/Redirect/Get from `prg`, the enhancement script from
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

use http_body_util::{BodyExt, Full};
use hyper::body::{Bytes, Incoming};
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, header};
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use maud::html;
use tokio::net::TcpListener;
use webonsive::{Caps, Theme, UiState, caps, counter, dialog, dialog::DialogOptions, enhance, layout, prg, tabs, tabs::{Tab, TabsOptions}};

type Reply = Response<Full<Bytes>>;

fn html_reply(body: String, set_cookies: Vec<String>) -> Reply {
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
    let (path, query) = (req.uri().path().to_string(), req.uri().query().unwrap_or("").to_string());
    // `?caps=` first, then the cookies: the whole of the Axum extractor.
    let caps = Caps::from_query(&query).unwrap_or_else(|| Caps::from_cookie_header(&cookies));
    let count: i64 = cookie(&cookies, "count").and_then(|v| v.parse().ok()).unwrap_or(0);

    let reply = match (req.method(), path.as_str()) {
        (&Method::GET, "/") => {
            let state = UiState::from_request(&path, &query, &cookies);
            let page = layout(&caps, "webonsive on hyper", Theme::Auto, html! {
                h1 { "webonsive on hyper" }
                p { "This browser supports: " @for n in caps.names() { code { (n) } " " } }
                (dialog(&caps, "d", "Open dialog", html! { p { "Closed by the platform, not by script." } }, DialogOptions::default().open(state.dialog() == Some("d"))))
                h2 { "Tabs" }
                (tabs(&caps, "demo", &[Tab::new("First", html! { p { "Tab state lives in the URL and a cookie." } }),
                                      Tab::new("Second", html! { p { "Reload, leave, come back: still here." } })], TabsOptions::default().state(&state)))
                h2 { "Counter" }
                (counter(&caps, "/counter", count, Default::default()))
            });
            html_reply(page.into_string(), state.set_cookies())
        }
        (&Method::POST, "/counter") => {
            let body = req.into_body().collect().await.map(|b| b.to_bytes()).unwrap_or_default();
            let op = cookie(&String::from_utf8_lossy(&body).replace('&', ";"), "op").unwrap_or("").to_string();
            let next = match op.as_str() { "inc" => count + 1, "dec" => count - 1, _ => 0 };
            let mut res: Reply = prg("/", Some("Counted."));
            res.headers_mut().append(header::SET_COOKIE, format!("count={next}; Path=/; SameSite=Lax").parse().unwrap());
            res
        }
        (&Method::GET, caps::BEACON_PATH) => {
            // The beacon route: 204 + Set-Cookie for a known flag, 404 otherwise, never cached.
            let mut res = Response::builder().header(header::CACHE_CONTROL, "no-store");
            res = match caps::beacon_cookie(&query) {
                Some(c) => res.status(StatusCode::NO_CONTENT).header(header::SET_COOKIE, c),
                None => res.status(StatusCode::NOT_FOUND),
            };
            res.body(Full::default()).unwrap()
        }
        (&Method::GET, enhance::SCRIPT_PATH) => Response::builder()
            .header(header::CONTENT_TYPE, "text/javascript; charset=utf-8")
            .header(header::CACHE_CONTROL, "public, max-age=31536000, immutable")
            .body(Full::new(Bytes::from_static(enhance::served().as_bytes())))
            .unwrap(),
        _ => Response::builder().status(StatusCode::NOT_FOUND).body(Full::default()).unwrap(),
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
            if let Err(e) = builder.serve_connection(TokioIo::new(stream), service_fn(handle)).await {
                eprintln!("connection error: {e}");
            }
        });
    }
}
