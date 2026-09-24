//! # Enhancement script
//!
//! One small optional script that makes the same markup feel instant. Every page works
//! identically with it blocked, disabled or not yet loaded: it only intercepts what the
//! browser would otherwise do as a full navigation and does it in place.
//!
//! **Platform features:** `<script defer>` (baseline 2010), `fetch` (Chrome 42, Firefox 39,
//! Safari 10.1), `DOMParser` (baseline 2010), `history.pushState` (Chrome 5, Firefox 4,
//! Safari 5), `document.startViewTransition` where present (Chrome 111, Firefox 144,
//! Safari 18) so swapped parts morph, `CustomEvent` (Chrome 15, Firefox 11, Safari 6) for
//! `nojs:swap`.
//!
//! **Fallback:** none needed. Without the script every form and link is a normal navigation;
//! the Blitz test suite renders every route with no script engine at all.
//!
//! **Contract:** an element with an `id` and `data-nojs="swap"` is a swap root. Submitting a
//! form or following a same-origin link inside it fetches the response, parses it, and
//! replaces the root with the element of the same `id` from the new document. The flash
//! messages, the toast list, `<title>` and `data-theme` are synced too, and the URL follows the response.
//! A form or link anywhere may name its root instead with `data-nojs-target="#id"`, and
//! `data-nojs-swap="outer|inner|append|prepend"` (default `outer`) says how the new element
//! lands: replace the root, replace its children, or add them at the end or the start. The
//! request is the same either way, so the server may answer an enhanced request (headers
//! `Nojs-Enhance: 1` and `Accept: text/html`) with only the fragment it needs to; without the
//! script it is a full navigation to the same URL. [`slim`] does that for every page: an
//! enhanced request gets the page without its inline stylesheet, which the document already
//! has, and every HTML answer says `Vary: Nojs-Enhance, Cookie`. User actions fetch with
//! `priority: "high"`. Links inside an element marked `data-nojs-prefetch` are fetched at low
//! priority on hover or focus, and a click within five seconds reuses that answer. A response may also carry elements marked `data-nojs-oob`
//! (out of band): each replaces the element of the same `id` anywhere in the page, in the
//! mode the attribute names (`outer` by default), and is dropped from the main swap; the
//! full page without the script already shows them in place. While a request is in flight
//! the root and the form carry `data-nojs-busy` and `aria-busy="true"`, the form's submit
//! buttons are disabled, and an element named by `data-nojs-indicator="#id"` (authored with
//! `hidden`) is shown; the `--nojs-busy` property sets how much the busy root fades. A request
//! that fails becomes the plain navigation the browser would have made, so the server's
//! answer is always seen. The URL follows the response (links push a history entry, forms
//! replace it); `data-nojs-push="false"` keeps the URL as it is and `data-nojs-replace` always
//! replaces. Every swap stores a copy of the roots in the history entry, so Back and Forward
//! restore them without a request. After every swap the root dispatches a bubbling `nojs:swap`
//! event with `{ id, url, mode }` in `detail`.
//! Rapid actions on one root are queued, so a counter clicked five times counts five. The
//! script also mirrors `<input type=range>` and `type=color` values while they move, counts
//! characters into the `<output for>` of a field with `maxlength`, sends multipart forms as
//! `FormData` so files survive, opens
//! the `:target` dialog fallback as a real modal, moves through an open popover menu with the
//! arrow keys, searches a combobox as you type and walks its results with the arrow keys.
//! A tab title opens its panel and slides the underline on the click itself; the server's
//! answer replaces the strip quietly once the slide ends (a lazy panel fills in then).
//!
//! [`script_tag`] goes at the end of `<body>`; [`router`] serves the file with a content
//! hash in the URL so it caches forever. It is compatible with `script-src 'self'`.
//!
//! ```rust
//! use axum_nojs::enhance;
//! let tag = enhance::script_tag().into_string();
//! assert!(tag.starts_with("<script src=\"/nojs/enhance.js?v="));
//! ```

use maud::{Markup, html};

/// Path the script is served from. [`script_url`] appends a content hash.
pub const SCRIPT_PATH: &str = "/nojs/enhance.js";

/// The whole enhancement script. Plain ES2020, no build step, under 10 KB as [`served`].
pub const JS: &str = r##"(function () {
"use strict";
var roots = "[data-nojs=swap]", queue = {}, cache = {};
// Every request says it is the script's (the server may answer with less) and wants HTML.
var init = function (priority) { return { credentials: "same-origin", priority: priority || "high", headers: { "Nojs-Enhance": "1", Accept: "text/html" } }; };
var load = function (url, req) {
  return fetch(url, req).then(function (res) { return res.text().then(function (html) { return { url: res.url, html: html }; }); });
};
var parse = function (html) { return new DOMParser().parseFromString(html, "text/html"); };
var reduced = matchMedia("(prefers-reduced-motion: reduce)").matches;

function focusState() {
  var a = document.activeElement;
  if (!a || a === document.body) return null;
  var key = a.id ? "#" + a.id : (a.name ? "[name='" + a.name + "']" : null);
  return key && { key: key, pos: a.selectionEnd };
}
function restoreFocus(f) {
  var el = f && document.querySelector(f.key);
  if (!el) return;
  el.focus({ preventScroll: true });
  if (f.pos != null && el.setSelectionRange) try { el.setSelectionRange(f.pos, f.pos); } catch (e) {}
}

function place(root, fresh, mode) {
  var kids = Array.prototype.slice.call(fresh.childNodes);
  if (mode === "inner") root.replaceChildren.apply(root, kids);
  else if (mode === "append") root.append.apply(root, kids);
  else if (mode === "prepend") root.prepend.apply(root, kids);
  else { root.replaceWith(fresh); return fresh; }
  return root;
}

// Copy of every swap root, kept in the history entry so Back and Forward restore in place.
function snapshot() {
  var nojs = {};
  document.querySelectorAll(roots).forEach(function (r) { if (r.id) nojs[r.id] = r.outerHTML; });
  return { nojs: nojs };
}
function swapped(el, url, mode) {
  history.replaceState(snapshot(), "", location.href);
  el.dispatchEvent(new CustomEvent("nojs:swap", { bubbles: true, detail: { id: el.id, url: url, mode: mode } }));
}

// hist: "push" adds a history entry, "replace" rewrites the current one, "none" keeps the URL.
function apply(doc, id, url, hist, mode, quiet) {
  var root = document.getElementById(id), fresh = doc.getElementById(id);
  if (!root || !fresh) { if (url !== location.href) location.href = url; else location.reload(); return; }
  var f = focusState();
  var swap = function () {
    doc.querySelectorAll("[data-nojs-oob]").forEach(function (el) {
      var here = el.id && el !== fresh && document.getElementById(el.id), how = el.getAttribute("data-nojs-oob");
      el.remove(); el.removeAttribute("data-nojs-oob");
      if (here) place(here, el, how || "outer");
    });
    var anchor = place(root, fresh, mode);
    [".nojs-flash", ".nojs-toasts"].forEach(function (s, i) {
      var o = document.querySelector(s), n = doc.querySelector(s);
      if (o && n) o.replaceWith(n); else if (o) o.remove(); else if (n) i ? document.body.append(n) : anchor.before(n);
    });
    if (doc.title) document.title = doc.title;
    var theme = doc.documentElement.getAttribute("data-theme");
    if (theme) document.documentElement.setAttribute("data-theme", theme);
    restoreFocus(f);
    swapped(anchor, url, mode);
  };
  if (hist !== "none" && url !== location.href) {
    if (hist === "push") { history.replaceState(snapshot(), "", location.href); history.pushState(null, "", url); }
    else history.replaceState(null, "", url);
  }
  if (!quiet && document.startViewTransition && !reduced) document.startViewTransition(swap); else swap();
}

// The root an element acts on: the one named by data-nojs-target on it or an ancestor, else
// the closest swap root. data-nojs-swap picks the mode; data-nojs-push="false" and
// data-nojs-replace pick what happens to the URL (links push by default, forms replace).
function target(el, hist) {
  var t = el.closest("[data-nojs-target]");
  var root = t ? document.querySelector(t.getAttribute("data-nojs-target")) : el.closest(roots);
  if (!root || !root.id) return null;
  var m = el.closest("[data-nojs-swap]"), p = el.closest("[data-nojs-push]");
  if (p && p.getAttribute("data-nojs-push") === "false") hist = "none";
  else if (el.closest("[data-nojs-replace]")) hist = "replace";
  return { id: root.id, mode: m ? m.getAttribute("data-nojs-swap") : "outer", hist: hist };
}

var pending = {};
// Busy marks on the root and the source (form or link): attributes for CSS and assistive
// tech, submit buttons disabled, the named indicator shown. Everything is undone after.
function busy(t, src, on) {
  var root = document.getElementById(t.id);
  [root, src].forEach(function (el) {
    if (!el) return;
    if (on) { el.setAttribute("data-nojs-busy", ""); el.setAttribute("aria-busy", "true"); }
    else { el.removeAttribute("data-nojs-busy"); el.removeAttribute("aria-busy"); }
  });
  if (src.tagName === "FORM") src.querySelectorAll(on ? "button:not([type=button]):not(:disabled),input[type=submit]:not(:disabled)" : "[data-nojs-disabled]").forEach(function (b) {
    b.disabled = on; if (on) b.setAttribute("data-nojs-disabled", ""); else b.removeAttribute("data-nojs-disabled");
  });
  var i = src.closest("[data-nojs-indicator]"), ind = i && document.querySelector(i.getAttribute("data-nojs-indicator"));
  if (ind) ind.hidden = !on;
}

function request(t, src, url, req, fallback) {
  var id = t.id;
  pending[id] = (pending[id] || 0) + 1;
  busy(t, src, true);
  var run = function () {
    var hit = !req.method && cache[url];
    delete cache[url];
    return (hit && Date.now() - hit.at < 5000 ? hit.got : load(url, req)).then(function (r) {
      return Promise.resolve(t.quiet).then(function () { apply(parse(r.html), id, r.url, t.hist, t.mode, t.quiet); });
    }).then(function () { done(); }, function () { done(); fallback(); });
  };
  var done = function () { pending[id]--; busy(t, src, false); if (pending[id]) busy(t, src, true); };
  queue[id] = (queue[id] || Promise.resolve()).then(run, run);
  return queue[id];
}

// A tab title opens its panel and moves the underline before the request; the server's answer
// then replaces the strip, once the underline has slid, without a second transition (a lazy
// panel fills in at that point). Returns false, or a promise for the end of the slide.
function openTab(a) {
  var s = a.closest(".nojs-tabs summary"), d = s && s.parentElement, strip = d && d.parentElement;
  if (!d || d.open) return false;
  var mark = strip.querySelector(":scope > details > summary > .nojs-tabs-mark");
  var show = function () { d.open = true; if (mark) s.append(mark); };
  if (document.startViewTransition && !reduced) return document.startViewTransition(show).finished;
  show();
  return true;
}

function submit(form, submitter) {
  var t = target(form, "replace");
  if (!t) return false;
  var data = new FormData(form);
  if (submitter && submitter.name) data.append(submitter.name, submitter.value);
  var at = function (a) { return submitter && submitter.getAttribute("form" + a) || form.getAttribute(a); };
  var url = new URL(at("action") || location.href, location.href);
  var req = init();
  var params = new URLSearchParams(data);
  if ((at("method") || "get").toLowerCase() === "post") { req.method = "POST"; req.body = form.enctype === "multipart/form-data" ? data : params; }
  else url.search = params.toString();
  request(t, form, url.href, req, function () { HTMLFormElement.prototype.submit.call(form); });
  return true;
}

document.addEventListener("submit", function (e) {
  if (e.defaultPrevented || e.target.method === "dialog") return;
  if (submit(e.target, e.submitter)) e.preventDefault();
});

document.addEventListener("click", function (e) {
  var a = e.target.closest("a[href]");
  if (!a || e.button || e.metaKey || e.ctrlKey || e.shiftKey || a.target) return;
  var d = a.closest(".nojs-dialog");
  if (d && a.hash && a.hash.length > 1 && a.hash === "#" + (d.querySelector("dialog") || {}).id) {
    var dialog = d.querySelector("dialog");
    if (dialog.showModal) { e.preventDefault(); dialog.showModal(); }
    return;
  }
  if (d && a.getAttribute("href") === "#" && d.querySelector("dialog[open]")) {
    e.preventDefault(); d.querySelector("dialog").close(); return;
  }
  var t = target(a, "push");
  if (!t || a.origin !== location.origin) return;
  e.preventDefault();
  t.quiet = openTab(a);
  request(t, a, a.href, init(), function () { location.href = a.href; });
});

// data-nojs-prefetch on a link or an ancestor: a hover or focus fetches the link's answer at
// low priority (not the page already shown, nor the tab already open), and a click within
// five seconds uses it instead of asking again.
["mouseover", "focusin"].forEach(function (type) {
  document.addEventListener(type, function (e) {
    var a = e.target.closest && e.target.closest("a[href]");
    if (!a || a.href === location.href || a.origin !== location.origin || !a.closest("[data-nojs-prefetch]") || a.closest(".nojs-tabs details[open] > summary") || !target(a, "push")) return;
    var hit = cache[a.href];
    if (hit && Date.now() - hit.at < 5000) return;
    var got = load(a.href, init("low"));
    got.catch(function () { delete cache[a.href]; });
    cache[a.href] = { at: Date.now(), got: got };
  });
});

// The narrow-screen tab select submits on change (its Go button stays for everyone else).
document.addEventListener("change", function (e) {
  if (e.target.matches(".nojs-tabs-select select") && e.target.form) submit(e.target.form, null);
});

var typing;
document.addEventListener("input", function (e) {
  var t = e.target, out = t.id && document.querySelector("output[for='" + t.id + "']");
  // Range: its value. Field with maxlength: length / limit.
  if (out) out.textContent = t.maxLength > 0 ? t.value.length + " / " + t.maxLength : t.value;
  if (t.type === "color") {
    var box = t.closest(".nojs-color");
    if (box) {
      var sw = box.querySelector(".nojs-color-swatch"), code = box.querySelector("code");
      if (sw) sw.style.setProperty("--nojs-color-value", t.value);
      if (code) code.textContent = t.value;
    }
  } else if (t.type === "search" && t.form && t.closest(roots)) {
    clearTimeout(typing);
    // Submit through an adjacent button (a select filter's formmethod=get).
    var b = t.nextElementSibling;
    typing = setTimeout(function () { submit(t.form, b && b.type === "submit" ? b : null); }, 150);
  }
});

// Arrow keys walk the items of the open menu the focus is in (or that this button opened),
// and the combobox input plus its results.
document.addEventListener("keydown", function (e) {
  if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
  var box = e.target.closest(".nojs-popover"), menu = e.target.closest(".nojs-popover nav"), items;
  if (!menu && box) menu = box.querySelector("nav:popover-open, details[open] > nav");
  var combo = e.target.closest(".nojs-combobox");
  if (combo) items = Array.prototype.slice.call(combo.querySelectorAll("input[type=search], [role=option] a[href]"));
  else if (menu) items = Array.prototype.filter.call(menu.querySelectorAll("a[href], button:not(:disabled), summary"), function (el) {
    return el.closest("nav") === menu;
  });
  else return;
  if (!items.length) return;
  e.preventDefault();
  var i = items.indexOf(e.target), n = items.length;
  items[i < 0 ? (e.key === "ArrowDown" ? 0 : n - 1) : (i + (e.key === "ArrowDown" ? 1 : n - 1)) % n].focus();
});

document.addEventListener("click", function (e) {
  document.querySelectorAll("details.nojs-popover-details[open]").forEach(function (d) {
    if (!d.contains(e.target)) d.removeAttribute("open");
  });
});

// Back and Forward: the entry's stored copy of each root when there is one, else a fetch.
addEventListener("popstate", function (e) {
  var nojs = e.state && e.state.nojs;
  if (nojs) {
    Object.keys(nojs).forEach(function (id) {
      var root = document.getElementById(id), fresh = parse(nojs[id]).getElementById(id);
      if (root && fresh) { root.replaceWith(fresh); swapped(fresh, location.href, "outer"); }
    });
    return;
  }
  load(location.href, init())
    .then(function (r) {
      var doc = parse(r.html);
      document.querySelectorAll(roots).forEach(function (r) { apply(doc, r.id, location.href, "none", "outer"); });
    });
});
})();
"##;

/// [`JS`] as served: comment lines and indentation dropped, nothing else touched. The budget
/// (10 KB) applies to this; the source keeps its comments for the reader.
pub fn served() -> &'static str {
    static SERVED: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    SERVED.get_or_init(|| {
        JS.lines().map(str::trim_start).filter(|l| !l.is_empty() && !l.starts_with("//")).collect::<Vec<_>>().join("\n")
    })
}

/// FNV-1a hash of [`JS`]: the cache-busting version in [`script_url`].
fn version() -> String {
    let hash = JS.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    format!("{hash:016x}")
}

/// `/nojs/enhance.js?v=<hash>`: changes whenever the script does, so it may cache forever.
pub fn script_url() -> String {
    format!("{SCRIPT_PATH}?v={}", version())
}

/// `<script src=… defer>` for the end of `<body>`. [`crate::Ui::page`] includes it.
pub fn script_tag() -> Markup {
    html! { script src=(script_url()) defer {} }
}

/// The page an enhanced request needs: `html` without the `<style>` elements in its `<head>`,
/// since the document making the request already has them. Everything the script reads
/// (swap roots, flash, toasts, out-of-band elements, `<title>`, `data-theme`) is kept.
pub fn slim_html(html: &str) -> String {
    let head_end = html.find("</head>").unwrap_or(0);
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    let mut done = 0;
    while let Some(start) = rest.find("<style") {
        if done + start >= head_end {
            break;
        }
        let Some(len) = rest[start..].find("</style>") else { break };
        out.push_str(&rest[..start]);
        let skip = start + len + "</style>".len();
        done += skip;
        rest = &rest[skip..];
    }
    out.push_str(rest);
    out
}

/// Build a swap-root id from a component prefix and a key such as a form action.
/// `swap_id("nojs-counter", "/counter")` is `nojs-counter--counter`.
pub fn swap_id(prefix: &str, key: &str) -> String {
    let key: String = key.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    format!("{prefix}-{key}")
}

#[cfg(feature = "axum")]
mod axum_glue {
    use super::{SCRIPT_PATH, served, slim_html};
    use axum::{
        Router,
        body::{Body, HttpBody},
        extract::Request,
        http::{HeaderValue, header},
        middleware::Next,
        response::Response,
        routing::get,
    };

    /// Middleware for the whole router (`.layer(axum::middleware::from_fn(enhance::slim))`):
    /// an HTML answer to an enhanced request (`Nojs-Enhance: 1`) loses its inline stylesheet
    /// ([`slim_html`]), and every HTML answer carries `Vary: Nojs-Enhance, Cookie`, so a cache or
    /// a `<link rel="prefetch">` never serves one variant, or one cookie's page, for another. Streamed bodies (no known size) pass through untouched.
    pub async fn slim(req: Request, next: Next) -> Response {
        let enhanced = req.headers().contains_key("nojs-enhance");
        let mut res = next.run(req).await;
        let html = res.headers().get(header::CONTENT_TYPE).is_some_and(|v| v.as_bytes().starts_with(b"text/html"));
        if !html {
            return res;
        }
        res.headers_mut().append(header::VARY, HeaderValue::from_static("nojs-enhance, cookie"));
        if !enhanced || res.body().size_hint().exact().is_none() {
            return res;
        }
        let (mut parts, body) = res.into_parts();
        let Ok(bytes) = axum::body::to_bytes(body, usize::MAX).await else {
            return Response::from_parts(parts, Body::empty());
        };
        let page = slim_html(&String::from_utf8_lossy(&bytes));
        parts.headers.remove(header::CONTENT_LENGTH);
        Response::from_parts(parts, Body::from(page))
    }

    /// Serves [`served`] at [`SCRIPT_PATH`], immutable for a year (the URL carries a hash).
    pub fn router() -> Router {
        Router::new().route(SCRIPT_PATH, get(|| async {
            (
                [
                    (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                ],
                served(),
            )
        }))
    }
}
#[cfg(feature = "axum")]
pub use axum_glue::{router, slim};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_is_small_and_plain() {
        assert!(served().len() < 10240, "enhance.js is {} bytes served", served().len());
        assert!(JS.len() < 12288, "enhance.js source is {} bytes; trim before adding comments", JS.len());
        assert!(!JS.contains("eval(") && !JS.contains("innerHTML"));
        assert!(script_url().starts_with("/nojs/enhance.js?v="));
        assert_eq!(swap_id("nojs-form", "/sign-up"), "nojs-form--sign-up");
    }

    #[test]
    fn slim_drops_head_styles_only() {
        let page = "<html><head><title>T</title><style>a{}</style><style class=\"nojs-tokens\">b{}</style></head><body><style>c{}</style><p id=x>hi</p></body></html>";
        assert_eq!(slim_html(page), "<html><head><title>T</title></head><body><style>c{}</style><p id=x>hi</p></body></html>");
        assert_eq!(slim_html("<p>no head</p>"), "<p>no head</p>");
    }
}
