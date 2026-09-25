//! loco-ui-caps on raw hyper, nothing else: `cargo run -p loco-ui-caps --example hyper`, then open
//! http://127.0.0.1:3003 twice. The first view says "unknown" for every flag while the
//! beacons fire; the second view has one line per flag with a yes or a no.

use std::convert::Infallible;
use std::net::SocketAddr;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Method, Request, Response, StatusCode, header};
use hyper_util::rt::TokioIo;
use loco_ui_caps::{BEACON_PATH, Cap, Caps, beacon_cookie, beacon_css, beacons};
use tokio::net::TcpListener;

async fn handle(req: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let query = req.uri().query().unwrap_or("");
    let reply = match (req.method(), req.uri().path()) {
        // The beacon route: 204 + the flag's cookie, or 404, never cached.
        (&Method::GET, BEACON_PATH) => {
            let res = Response::builder().header(header::CACHE_CONTROL, "no-store");
            match beacon_cookie(query) {
                Some(c) => res
                    .status(StatusCode::NO_CONTENT)
                    .header(header::SET_COOKIE, c),
                None => res.status(StatusCode::NOT_FOUND),
            }
            .body(Full::default())
            .unwrap()
        }
        (&Method::GET, "/") => {
            // The cookies say what the browser can do; `?caps=a,b` forces a set.
            let cookies = req
                .headers()
                .get_all(header::COOKIE)
                .iter()
                .filter_map(|v| v.to_str().ok())
                .collect::<Vec<_>>()
                .join("; ");
            let caps =
                Caps::from_query(query).unwrap_or_else(|| Caps::from_cookie_header(&cookies));
            let probed = caps.has(Cap::Probed);
            let mut lines = String::new();
            for cap in Cap::ALL {
                let answer = if caps.has(cap) {
                    "yes"
                } else if probed {
                    "no"
                } else {
                    "unknown"
                };
                lines += &format!("<li><code>{}</code>: {answer}</li>", cap.name());
            }
            let page = format!(
                "<!DOCTYPE html><meta charset=utf-8><title>loco-ui-caps</title><style>{}</style>\
                 <h1>What this browser can do</h1><p>{}</p><ul>{lines}</ul>{}",
                beacon_css(),
                if probed {
                    "Beacons have fired."
                } else {
                    "First view: reload once the beacons have fired."
                },
                beacons(&caps).into_string()
            );
            Response::builder()
                .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
                .body(Full::new(Bytes::from(page)))
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::default())
            .unwrap(),
    };
    Ok(reply)
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3003));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("http://{addr}");
    loop {
        let (stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            if let Err(e) = http1::Builder::new()
                .serve_connection(TokioIo::new(stream), service_fn(handle))
                .await
            {
                eprintln!("connection error: {e}");
            }
        });
    }
}
