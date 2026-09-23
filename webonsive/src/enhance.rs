//! # Enhancement script
//!
//! One small optional script that makes the same markup feel instant. Every page works
//! identically with it blocked, disabled or not yet loaded: it only intercepts what the
//! browser would otherwise do as a full navigation and does it in place.
//!
//! **Platform features:** `<script defer>` (baseline 2010), `fetch` (Chrome 42, Firefox 39,
//! Safari 10.1), `DOMParser` (baseline 2010), `history.pushState` (Chrome 5, Firefox 4,
//! Safari 5), `document.startViewTransition` where present (Chrome 111, Firefox 144,
//! Safari 18) so swapped parts morph.
//!
//! **Fallback:** none needed. Without the script every form and link is a normal navigation;
//! the Blitz test suite renders every route with no script engine at all.
//!
//! **Contract:** an element with an `id` and `data-wo="swap"` is a swap root. Submitting a
//! form or following a same-origin link inside it fetches the response, parses it, and
//! replaces the root with the element of the same `id` from the new document. The flash
//! message, `<title>` and `data-theme` are synced too, and the URL follows the response.
//! Rapid actions on one root are queued, so a counter clicked five times counts five. The
//! script also mirrors `<input type=range>` and `type=color` values while they move, opens
//! the `:target` dialog fallback as a real modal, and searches a combobox as you type.
//!
//! [`script_tag`] goes at the end of `<body>`; [`router`] serves the file with a content
//! hash in the URL so it caches forever. It is compatible with `script-src 'self'`.
//!
//! ```rust
//! use webonsive::enhance;
//! let tag = enhance::script_tag().into_string();
//! assert!(tag.starts_with("<script src=\"/wo/enhance.js?v="));
//! ```

use maud::{Markup, html};

/// Path the script is served from. [`script_url`] appends a content hash.
pub const SCRIPT_PATH: &str = "/wo/enhance.js";

/// The whole enhancement script. Plain ES2020, no build step, under 6 KB.
pub const JS: &str = r##"(function () {
"use strict";
var roots = "[data-wo=swap]", queue = {};
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

function apply(doc, id, url, push) {
  var root = document.getElementById(id), fresh = doc.getElementById(id);
  if (!root || !fresh) { if (url !== location.href) location.href = url; else location.reload(); return; }
  var f = focusState();
  var swap = function () {
    root.replaceWith(fresh);
    var oldFlash = document.querySelector(".wo-flash"), newFlash = doc.querySelector(".wo-flash");
    if (oldFlash && newFlash) oldFlash.replaceWith(newFlash);
    else if (oldFlash) oldFlash.remove();
    else if (newFlash) fresh.before(newFlash);
    document.title = doc.title;
    var theme = doc.documentElement.getAttribute("data-theme");
    if (theme) document.documentElement.setAttribute("data-theme", theme);
    restoreFocus(f);
  };
  if (url !== location.href) history[push ? "pushState" : "replaceState"](null, "", url);
  if (document.startViewTransition && !reduced) document.startViewTransition(swap); else swap();
}

function request(id, url, init, push) {
  var run = function () {
    return fetch(url, init).then(function (res) {
      return res.text().then(function (html) { apply(parse(html), id, res.url, push); });
    }).catch(function () { location.reload(); });
  };
  queue[id] = (queue[id] || Promise.resolve()).then(run, run);
  return queue[id];
}

function submit(form, submitter) {
  var root = form.closest(roots);
  if (!root) return false;
  var data = new FormData(form);
  if (submitter && submitter.name) data.append(submitter.name, submitter.value);
  var url = new URL(form.getAttribute("action") || location.href, location.href);
  var init = { credentials: "same-origin", headers: { "Wo-Enhance": "1" } };
  var params = new URLSearchParams(data);
  if ((form.method || "get").toLowerCase() === "post") { init.method = "POST"; init.body = params; }
  else url.search = params.toString();
  request(root.id, url.href, init, false);
  return true;
}

document.addEventListener("submit", function (e) {
  if (e.defaultPrevented || e.target.method === "dialog") return;
  if (submit(e.target, e.submitter)) e.preventDefault();
});

document.addEventListener("click", function (e) {
  var a = e.target.closest("a[href]");
  if (!a || e.button || e.metaKey || e.ctrlKey || e.shiftKey || a.target) return;
  var d = a.closest(".wo-dialog");
  if (d && a.hash && a.hash.length > 1 && a.hash === "#" + (d.querySelector("dialog") || {}).id) {
    var dialog = d.querySelector("dialog");
    if (dialog.showModal) { e.preventDefault(); dialog.showModal(); }
    return;
  }
  if (d && a.getAttribute("href") === "#" && d.querySelector("dialog[open]")) {
    e.preventDefault(); d.querySelector("dialog").close(); return;
  }
  var root = a.closest(roots);
  if (!root || a.origin !== location.origin) return;
  e.preventDefault();
  request(root.id, a.href, { credentials: "same-origin", headers: { "Wo-Enhance": "1" } }, true);
});

var typing;
document.addEventListener("input", function (e) {
  var t = e.target;
  if (t.type === "range") {
    var out = t.id && document.querySelector("output[for='" + t.id + "']");
    if (out) out.textContent = t.value;
  } else if (t.type === "color") {
    var box = t.closest(".wo-color");
    if (box) {
      var sw = box.querySelector(".wo-color-swatch"), code = box.querySelector("code");
      if (sw) sw.style.background = t.value;
      if (code) code.textContent = t.value;
    }
  } else if (t.type === "search" && t.form && t.closest(roots)) {
    clearTimeout(typing);
    typing = setTimeout(function () { submit(t.form, null); }, 150);
  }
});

document.addEventListener("click", function (e) {
  document.querySelectorAll("details.wo-popover-details[open]").forEach(function (d) {
    if (!d.contains(e.target)) d.removeAttribute("open");
  });
});

addEventListener("popstate", function () {
  fetch(location.href, { credentials: "same-origin", headers: { "Wo-Enhance": "1" } })
    .then(function (r) { return r.text(); })
    .then(function (html) {
      var doc = parse(html);
      document.querySelectorAll(roots).forEach(function (r) { apply(doc, r.id, location.href, false); });
    });
});
})();
"##;

/// FNV-1a hash of [`JS`]: the cache-busting version in [`script_url`].
fn version() -> String {
    let hash = JS.bytes().fold(0xcbf2_9ce4_8422_2325_u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3));
    format!("{hash:016x}")
}

/// `/wo/enhance.js?v=<hash>`: changes whenever the script does, so it may cache forever.
pub fn script_url() -> String {
    format!("{SCRIPT_PATH}?v={}", version())
}

/// `<script src=… defer>` for the end of `<body>`. [`crate::layout`] includes it.
pub fn script_tag() -> Markup {
    html! { script src=(script_url()) defer {} }
}

/// Build a swap-root id from a component prefix and a key such as a form action.
/// `swap_id("wo-counter", "/counter")` is `wo-counter--counter`.
pub fn swap_id(prefix: &str, key: &str) -> String {
    let key: String = key.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    format!("{prefix}-{key}")
}

#[cfg(feature = "axum")]
mod axum_glue {
    use super::{JS, SCRIPT_PATH};
    use axum::{Router, http::header, routing::get};

    /// Serves [`JS`] at [`SCRIPT_PATH`], immutable for a year (the URL carries a hash).
    pub fn router() -> Router {
        Router::new().route(SCRIPT_PATH, get(|| async {
            (
                [
                    (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
                    (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
                ],
                JS,
            )
        }))
    }
}
#[cfg(feature = "axum")]
pub use axum_glue::router;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_is_small_and_plain() {
        assert!(JS.len() < 6144, "enhance.js is {} bytes", JS.len());
        assert!(!JS.contains("eval(") && !JS.contains("innerHTML"));
        assert!(script_url().starts_with("/wo/enhance.js?v="));
        assert_eq!(swap_id("wo-form", "/sign-up"), "wo-form--sign-up");
    }
}
