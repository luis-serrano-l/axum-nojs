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
//! `wo:swap`.
//!
//! **Fallback:** none needed. Without the script every form and link is a normal navigation;
//! the Blitz test suite renders every route with no script engine at all.
//!
//! **Contract:** an element with an `id` and `data-wo="swap"` is a swap root. Submitting a
//! form or following a same-origin link inside it fetches the response, parses it, and
//! replaces the root with the element of the same `id` from the new document. The flash
//! message, `<title>` and `data-theme` are synced too, and the URL follows the response.
//! A form or link anywhere may name its root instead with `data-wo-target="#id"`, and
//! `data-wo-swap="outer|inner|append|prepend"` (default `outer`) says how the new element
//! lands: replace the root, replace its children, or add them at the end or the start. The
//! request is the same either way, so the server may answer an enhanced request (header
//! `Wo-Enhance: 1`) with only the fragment it needs to; without the script it is a full
//! navigation to the same URL. A response may also carry elements marked `data-wo-oob`
//! (out of band): each replaces the element of the same `id` anywhere in the page, in the
//! mode the attribute names (`outer` by default), and is dropped from the main swap; the
//! full page without the script already shows them in place. While a request is in flight
//! the root and the form carry `data-wo-busy` and `aria-busy="true"`, the form's submit
//! buttons are disabled, and an element named by `data-wo-indicator="#id"` (authored with
//! `hidden`) is shown; the `--wo-busy` property sets how much the busy root fades. A request
//! that fails becomes the plain navigation the browser would have made, so the server's
//! answer is always seen. The URL follows the response (links push a history entry, forms
//! replace it); `data-wo-push="false"` keeps the URL as it is and `data-wo-replace` always
//! replaces. Every swap stores a copy of the roots in the history entry, so Back and Forward
//! restore them without a request. After every swap the root dispatches a bubbling `wo:swap`
//! event with `{ id, url, mode }` in `detail`.
//! Rapid actions on one root are queued, so a counter clicked five times counts five. The
//! script also mirrors `<input type=range>` and `type=color` values while they move, counts
//! characters into the `<output for>` of a field with `maxlength`, sends multipart forms as
//! `FormData` so files survive, opens
//! the `:target` dialog fallback as a real modal, moves through an open popover menu with the
//! arrow keys, searches a combobox as you type and walks its results with the arrow keys.
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

/// The whole enhancement script. Plain ES2020, no build step, under 10 KB.
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
  var wo = {};
  document.querySelectorAll(roots).forEach(function (r) { if (r.id) wo[r.id] = r.outerHTML; });
  return { wo: wo };
}
function swapped(el, url, mode) {
  history.replaceState(snapshot(), "", location.href);
  el.dispatchEvent(new CustomEvent("wo:swap", { bubbles: true, detail: { id: el.id, url: url, mode: mode } }));
}

// hist: "push" adds a history entry, "replace" rewrites the current one, "none" keeps the URL.
function apply(doc, id, url, hist, mode) {
  var root = document.getElementById(id), fresh = doc.getElementById(id);
  if (!root || !fresh) { if (url !== location.href) location.href = url; else location.reload(); return; }
  var f = focusState();
  var swap = function () {
    doc.querySelectorAll("[data-wo-oob]").forEach(function (el) {
      var here = el.id && el !== fresh && document.getElementById(el.id), how = el.getAttribute("data-wo-oob");
      el.remove(); el.removeAttribute("data-wo-oob");
      if (here) place(here, el, how || "outer");
    });
    var anchor = place(root, fresh, mode);
    var oldFlash = document.querySelector(".wo-flash"), newFlash = doc.querySelector(".wo-flash");
    if (oldFlash && newFlash) oldFlash.replaceWith(newFlash);
    else if (oldFlash) oldFlash.remove();
    else if (newFlash) anchor.before(newFlash);
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
  if (document.startViewTransition && !reduced) document.startViewTransition(swap); else swap();
}

// The root an element acts on: the one named by data-wo-target on it or an ancestor, else
// the closest swap root. data-wo-swap picks the mode; data-wo-push="false" and
// data-wo-replace pick what happens to the URL (links push by default, forms replace).
function target(el, hist) {
  var t = el.closest("[data-wo-target]");
  var root = t ? document.querySelector(t.getAttribute("data-wo-target")) : el.closest(roots);
  if (!root || !root.id) return null;
  var m = el.closest("[data-wo-swap]"), p = el.closest("[data-wo-push]");
  if (p && p.getAttribute("data-wo-push") === "false") hist = "none";
  else if (el.closest("[data-wo-replace]")) hist = "replace";
  return { id: root.id, mode: m ? m.getAttribute("data-wo-swap") : "outer", hist: hist };
}

var pending = {};
// Busy marks on the root and the source (form or link): attributes for CSS and assistive
// tech, submit buttons disabled, the named indicator shown. Everything is undone after.
function busy(t, src, on) {
  var root = document.getElementById(t.id);
  [root, src].forEach(function (el) {
    if (!el) return;
    if (on) { el.setAttribute("data-wo-busy", ""); el.setAttribute("aria-busy", "true"); }
    else { el.removeAttribute("data-wo-busy"); el.removeAttribute("aria-busy"); }
  });
  if (src.tagName === "FORM") src.querySelectorAll(on ? "button:not([type=button]):not(:disabled),input[type=submit]:not(:disabled)" : "[data-wo-disabled]").forEach(function (b) {
    b.disabled = on; if (on) b.setAttribute("data-wo-disabled", ""); else b.removeAttribute("data-wo-disabled");
  });
  var i = src.closest("[data-wo-indicator]"), ind = i && document.querySelector(i.getAttribute("data-wo-indicator"));
  if (ind) ind.hidden = !on;
}

function request(t, src, url, init, fallback) {
  var id = t.id;
  pending[id] = (pending[id] || 0) + 1;
  busy(t, src, true);
  var run = function () {
    return fetch(url, init).then(function (res) {
      return res.text().then(function (html) { apply(parse(html), id, res.url, t.hist, t.mode); });
    }).then(function () { done(); }, function () { done(); fallback(); });
  };
  var done = function () { pending[id]--; busy(t, src, false); if (pending[id]) busy(t, src, true); };
  queue[id] = (queue[id] || Promise.resolve()).then(run, run);
  return queue[id];
}

function submit(form, submitter) {
  var t = target(form, "replace");
  if (!t) return false;
  var data = new FormData(form);
  if (submitter && submitter.name) data.append(submitter.name, submitter.value);
  var url = new URL(form.getAttribute("action") || location.href, location.href);
  var init = { credentials: "same-origin", headers: { "Wo-Enhance": "1" } };
  var params = new URLSearchParams(data);
  if ((form.method || "get").toLowerCase() === "post") { init.method = "POST"; init.body = form.enctype === "multipart/form-data" ? data : params; }
  else url.search = params.toString();
  request(t, form, url.href, init, function () { HTMLFormElement.prototype.submit.call(form); });
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
  var t = target(a, "push");
  if (!t || a.origin !== location.origin) return;
  e.preventDefault();
  request(t, a, a.href, { credentials: "same-origin", headers: { "Wo-Enhance": "1" } }, function () { location.href = a.href; });
});

// The narrow-screen tab select submits on change (its Go button stays for everyone else).
document.addEventListener("change", function (e) {
  if (e.target.matches(".wo-tabs-select select") && e.target.form) submit(e.target.form, null);
});

var typing;
document.addEventListener("input", function (e) {
  var t = e.target, out = t.id && document.querySelector("output[for='" + t.id + "']");
  // A range shows its value; a field with maxlength shows its length out of the limit.
  if (out) out.textContent = t.maxLength > 0 ? t.value.length + " / " + t.maxLength : t.value;
  if (t.type === "color") {
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

// Arrow keys walk the items of the open menu the focus is in (or that this button opened),
// and the combobox input plus its results.
document.addEventListener("keydown", function (e) {
  if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;
  var box = e.target.closest(".wo-popover"), menu = e.target.closest(".wo-popover nav"), items;
  if (!menu && box) menu = box.querySelector("nav:popover-open, details[open] > nav");
  var combo = e.target.closest(".wo-combobox");
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
  document.querySelectorAll("details.wo-popover-details[open]").forEach(function (d) {
    if (!d.contains(e.target)) d.removeAttribute("open");
  });
});

// Back and Forward: the entry's stored copy of each root when there is one, else a fetch.
addEventListener("popstate", function (e) {
  var wo = e.state && e.state.wo;
  if (wo) {
    Object.keys(wo).forEach(function (id) {
      var root = document.getElementById(id), fresh = parse(wo[id]).getElementById(id);
      if (root && fresh) { root.replaceWith(fresh); swapped(fresh, location.href, "outer"); }
    });
    return;
  }
  fetch(location.href, { credentials: "same-origin", headers: { "Wo-Enhance": "1" } })
    .then(function (r) { return r.text(); })
    .then(function (html) {
      var doc = parse(html);
      document.querySelectorAll(roots).forEach(function (r) { apply(doc, r.id, location.href, "none", "outer"); });
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

/// `<script src=… defer>` for the end of `<body>`. [`crate::layout()`] includes it.
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
        assert!(JS.len() < 10240, "enhance.js is {} bytes", JS.len());
        assert!(!JS.contains("eval(") && !JS.contains("innerHTML"));
        assert!(script_url().starts_with("/wo/enhance.js?v="));
        assert_eq!(swap_id("wo-form", "/sign-up"), "wo-form--sign-up");
    }
}
