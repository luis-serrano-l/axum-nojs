#!/usr/bin/env node
// Drives headless Firefox through geckodriver (plain WebDriver over HTTP, no packages) to
// prove the enhancement script does its job: actions happen in place, with no navigation.
// Usage: node scripts/browser-check.mjs   (needs target/debug/demo built, geckodriver, firefox)
import { spawn } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";

// Own ports, so a demo you are looking at on 3000 is left alone.
const DEMO = "http://127.0.0.1:3001";
const DRIVER = "http://127.0.0.1:4445";
const server = spawn("target/debug/demo", { stdio: "ignore", env: { ...process.env, PORT: "3001" } });
const driver = spawn("geckodriver", ["--port", "4445"], { stdio: "ignore" });
const quit = (code) => { server.kill(); driver.kill(); process.exit(code); };

async function wd(method, path, body) {
  const res = await fetch(DRIVER + path, { method, headers: { "content-type": "application/json" }, body: body && JSON.stringify(body) });
  const json = await res.json();
  if (json.value && json.value.error) throw new Error(`${method} ${path}: ${json.value.message}`);
  return json.value;
}
async function ready(url) {
  for (let i = 0; i < 50; i++) { try { await fetch(url); return; } catch { await sleep(100); } }
  throw new Error("not reachable: " + url);
}
const assert = (ok, msg) => { if (!ok) { console.error("FAIL: " + msg); quit(1); } console.log("ok   " + msg); };

await ready(DEMO + "/");
await ready(DRIVER + "/status");
const { sessionId } = await wd("POST", "/session", { capabilities: { alwaysMatch: { browserName: "firefox", "moz:firefoxOptions": { args: ["-headless"] } } } });
const S = "/session/" + sessionId;
const go = (path) => wd("POST", S + "/url", { url: DEMO + path });
const js = (script, ...args) => wd("POST", S + "/execute/sync", { script, args });
const find = (css) => wd("POST", S + "/element", { using: "css selector", value: css }).then((e) => Object.values(e)[0]);
// A swap can land between find and click; a stale reference just means "find it again".
const click = async (css) => wd("POST", S + `/element/${await find(css)}/click`, {})
  .catch((e) => (/stale/.test(e.message) ? click(css) : Promise.reject(e)));
const type = async (css, text) => wd("POST", S + `/element/${await find(css)}/value`, { text });
const navigations = () => js("return performance.getEntriesByType('navigation').length");
async function until(fn, what, ms = 3000) {
  for (let t = 0; t < ms; t += 50) { if (await fn()) return; await sleep(50); }
  throw new Error("timed out waiting for " + what);
}
const text = (css) => js("return document.querySelector(arguments[0]).textContent.trim()", css);

try {
  // Counter: five fast clicks, all counted, no page load.
  await go("/counter");
  await click("button[value=reset]");
  await until(async () => (await text(".lui-counter output")) === "0", "reset");
  for (let i = 0; i < 5; i++) await click("button[value=inc]");
  await until(async () => (await text(".lui-counter output")) === "10", "counter to reach 10 in steps of 2");
  assert(await navigations() === 1, "counter: 5 quick clicks counted in place, no reload");
  await js("document.querySelector('.lui-counter input[name=value]').value = 20");
  await click(".lui-counter button[value=set]");
  await until(async () => (await text(".lui-counter output")) === "20", "typed value set");
  assert(await js("return document.querySelector('.lui-counter button[value=inc]').disabled"), "counter: + switches off at the maximum");

  // Tabs: click a title, panel switches, URL updated, no reload.
  await go("/tabs");
  // The toggle comes from the script, before the answer: the strip it fires in is still the original.
  assert(await wd("POST", S + "/execute/async", { args: [], script: "var cb = arguments[0], a = document.querySelector(\".lui-tabs summary a[href*='tab.demo=1']\"), d = a.closest('details'); d.addEventListener('toggle', function () { cb(d.open && a.isConnected && !!a.closest('summary').querySelector('.lui-tabs-mark')); }, { once: true }); a.click();" }), "tabs: the clicked tab opens and takes the underline before the answer");
  await until(async () => (await text(".lui-tabs details[open] summary")).startsWith("Use"), "tab switch");
  await until(async () => (await js("return location.search")) === "?tab.demo=1", "tabs: URL follows the swap");
  assert(true, "tabs: URL follows the swap");
  assert(await navigations() === 1, "tabs: switched without a reload");
  assert(await js("return getComputedStyle(document.querySelector('.lui-tabs details[open] .lui-tabs-mark')).viewTransitionName") === "lui-tabs-demo", "tabs: the underline, not the title, carries the view-transition-name");
  const fetched = (q) => js("return performance.getEntriesByType('resource').filter((r) => r.name.endsWith(arguments[0])).length", q);
  await js("document.querySelector(\".lui-tabs summary a[href*='tab.demo=2']\").dispatchEvent(new MouseEvent('mouseover', { bubbles: true }))");
  await until(async () => (await fetched("?tab.demo=2")) === 1, "hover prefetches the tab");
  await click(".lui-tabs summary a[href*='tab.demo=2']");
  await until(async () => await js("return !!document.querySelector('#lui-tabs-demo details[open] .lui-tabs-panel p')"), "prefetched tab shown");
  assert(await fetched("?tab.demo=2") === 1 && await navigations() === 1, "tabs: the click reused the prefetched answer in place, no second request");
  await until(async () => await js("return !!document.querySelector('#lui-tabs-demo details[open] .lui-tabs-panel p')"), "lazy tab filled");
  assert(!(await js("return document.querySelector('#lui-tabs-demo details[open] .lui-tabs-lazy')")), "tabs: lazy panel rendered by the request that opened it");
  await wd("POST", S + "/window/rect", { width: 500, height: 800 });
  await until(async () => await js("return document.querySelector('#lui-tabs-demo .lui-tabs-select').offsetParent !== null"), "select shown on a narrow screen");
  assert(await js("return document.querySelector('#lui-tabs-demo summary').offsetParent === null"), "tabs: titles hidden when the select shows");
  await js("var s = document.querySelector('#lui-tabs-demo select'); s.value = '0'; s.dispatchEvent(new Event('change', { bubbles: true }))");
  await until(async () => (await js("return document.querySelector('#lui-tabs-demo select').value")) === "0" && (await text("#lui-tabs-demo details[open] summary")) === "Install", "select switched the tab");
  assert(await navigations() === 1 && await js("return location.search") === "?tab.demo=0", "tabs: select change swapped in place");
  await wd("POST", S + "/window/rect", { width: 1000, height: 700 });

  // Accordion: expand all, then one title, in place; several stay open.
  await go("/accordion");
  await click(".lui-accordion-controls a");
  await until(async () => (await js("return document.querySelectorAll('#lui-accordion-faq > details[open]').length")) === 3, "expand all");
  assert(await js("return location.search").then(q => q.includes("open.faq=0%2C1%2C2")), "accordion: expand all swapped in place with the list in the URL");
  await click("#lui-accordion-faq > details:nth-of-type(2) > summary a");
  await until(async () => (await js("return [...document.querySelectorAll('#lui-accordion-faq > details')].map(d => d.open ? 1 : 0).join('')")) === "101", "toggle one of three");
  await click("#lui-accordion-faq-more > details:nth-of-type(2) > summary a");
  await until(async () => (await js("return [...document.querySelectorAll('#lui-accordion-faq-more > details')].map(d => d.open ? 1 : 0).join('')")) === "01", "nested group toggles on its own key");
  assert(await js("return [...document.querySelectorAll('#lui-accordion-faq > details')].map(d => d.open ? 1 : 0).join('')") === "101", "accordion: nested toggle kept the outer sections");
  await click(".lui-accordion-controls a:last-child");
  await until(async () => (await js("return document.querySelectorAll('#lui-accordion-faq > details[open]').length")) === 0, "collapse all");
  assert(await navigations() === 1, "accordion: every toggle swapped without a reload");
  await go("/accordion");
  assert(await js("return document.querySelectorAll('#lui-accordion-faq > details[open]').length") === 0, "accordion: collapse all beat the cookie's memory on the next visit");

  // Combobox: results as you type, focus kept.
  await go("/combobox");
  await type("input[type=search]", "ru");
  await until(async () => (await js("return [...document.querySelectorAll('#langs [role=option]')].map(l => l.textContent).join()")) === "Rust,Ruby", "search results");
  assert(await js("return document.activeElement.name") === "q", "combobox: focus stays in the input");
  await js("document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))");
  assert(await js("return document.activeElement.textContent") === "Rust", "combobox: ArrowDown moves from the input to the first result");
  await js("document.activeElement.click()");
  await until(async () => (await js("return [...document.querySelectorAll('.lui-combobox-chip')].map(c => c.firstChild.textContent).join()")) === "Rust", "result became a chip");
  assert(await js("return location.search") === "?q=ru&sel=Rust", "combobox: the chip is in the URL");
  await click(".lui-combobox-chip a");
  await until(async () => (await js("return document.querySelectorAll('.lui-combobox-chip').length")) === 0, "chip removed");
  assert(await navigations() === 1, "combobox: searched, picked and removed without a reload");

  // Table: a sort link re-renders the rows in place, URL follows.
  await go("/table?per.files=5");
  await click(".lui-table th a[href*='sort=size']");
  await until(async () => (await js("return document.querySelector('.lui-table th[aria-sort]')?.textContent.trim()")) === "Size▲", "sort by size");
  assert(await js("return location.search") === "?sort=size&dir=asc&per.files=5", "table: URL follows the sort");
  assert(await navigations() === 1, "table: sorted without a reload");
  assert(await js("return document.querySelector('a[rel=next]').href.includes('sort=size')"), "table: the pager's links follow the sort swapped in place");
  await click(".lui-table-cols summary");
  await click(".lui-table-cols a[href*='cols=name%2Csize']");
  await until(async () => (await js("return document.querySelectorAll('.lui-table thead th a').length")) === 2, "column hidden");
  assert(await js("return location.search").then(q => q.includes("cols=name%2Csize")), "table: hidden column swapped in place with ?cols= in the URL");
  await js("document.querySelector('tbody tr:nth-child(2) input[type=checkbox]').click(); document.querySelector('#lui-table-files-bulk button[value=archive]').click()");
  await until(async () => (await js("return document.querySelector('.lui-flash')?.textContent || ''")).includes("archive: 1 file"), "bulk form posted and redirected with a flash");
  await js("document.querySelector('tbody tr:first-child .lui-table-detail summary').click()");
  assert(await js("return document.querySelector('tbody tr:first-child .lui-table-detail').open"), "table: a row's detail opens natively");
  // Paged table: the page size picked on one visit is remembered on the next (lui-ui cookie).
  await go("/table");
  assert(await text(".lui-paged-table-range") === "1–5 of 36", "table: page size remembered from the earlier visit");
  await js("const f = document.querySelector('.lui-paged-table-jump'); f.page.value = 7; f.requestSubmit()");
  await until(async () => (await js("return document.querySelector('.lui-paged-table-range')?.textContent")) === "31–35 of 36", "jump to page 7");
  assert(await js("return location.search").then(q => q.includes("page=7")), "table: the jump is in the URL");
  assert(await navigations() === 1, "table: jumped without a reload");

  // Wizard: Next posts, PRG lands on step 2 in place, Back keeps the value.
  await go("/wizard?step.signup=0");
  await type("input[name=name]", "Ada");
  await type("input[name=email]", "ada@example.com");
  await click(".lui-wizard button.lui-button-primary");
  await until(async () => (await js("return document.querySelector('#f-email-error')?.textContent || ''")).includes("example.com"), "server message beside the field");
  assert(await js("return document.querySelector('.lui-wizard-steps li[aria-current=step]').classList.contains('lui-wizard-error')"), "wizard: the step is marked in error");
  await js("const e = document.querySelector('input[name=email]'); e.value = ''");
  await type("input[name=email]", "ada@example.org");
  await click(".lui-wizard button.lui-button-primary");
  await until(async () => (await text(".lui-wizard li[aria-current=step]")) === "Newsletter (optional)", "wizard step 2");
  assert(await navigations() === 1, "wizard: advanced without a reload");
  await click(".lui-wizard-back");
  await until(async () => (await js("return document.querySelector('input[name=name]')?.value")) === "Ada", "wizard back keeps the name");
  await click(".lui-wizard button.lui-button-primary");
  await until(async () => (await text(".lui-wizard li[aria-current=step]")) === "Newsletter (optional)", "wizard step 2 again");
  await click(".lui-wizard button[name=skip]");
  await until(async () => (await js("return document.querySelector('.lui-wizard-review')?.textContent || ''")).includes("(skipped)"), "skipped straight to the review");
  await go("/wizard");
  assert(await js("return !!document.querySelector('.lui-wizard-resume')") && await text(".lui-wizard li[aria-current=step]") === "Review", "wizard: a new visit resumes at the review");

  // Form: the counter follows typing; a multipart post with a file lands as a flash in place.
  await go("/form");
  await type("#f-bio", "Hello there");
  assert(await text("output[for=f-bio]") === "11 / 160", "form: the counter follows typing");
  await type("#f-name", "Ada");
  await type("#f-email", "ada@example.org");
  await type("#f-age", "36");
  await type("#f-handle", "ada_l");
  await type("#f-avatar", process.cwd() + "/loco-ui-test/tests/fixture.png");
  await click(".lui-form button[type=submit]");
  await until(async () => (await js("return document.querySelector('.lui-flash')?.textContent || ''")).includes("fixture.png"), "file posted as multipart and named in the flash");
  assert(await navigations() === 1, "form: submitted with a file without a reload");

  // Swap targets: a link outside any root updates only #count; a form appends to #log.
  await go("/swap");
  await click("a[data-lui-target='#count']");
  await until(async () => (await text("#count")) === "2", "count swapped by target");
  assert(await js("return location.search") === "?n=2", "swap: URL follows the targeted link");
  const before = await js("return document.querySelectorAll('#log li').length");
  await type("input[name=note]", "hello");
  await click("form[data-lui-target='#log'] button");
  await until(async () => (await js("return document.querySelectorAll('#log li').length")) === before + 1, "note appended");
  assert(await js("return document.querySelector('#log li:last-child').textContent") === "hello", "swap: appended the fragment only");
  assert((await text("#note-count")) === String(before + 1), "swap: out-of-band count updated outside the target");
  assert(!(await js("return document.querySelector('[data-lui-oob]')")), "swap: oob element not left in the page");
  assert(await navigations() === 1, "swap: target and append without a reload");
  // Busy state is observable synchronously right after the submit is dispatched.
  await type("input[name=note]", "again");
  const busy = await js(`var f = document.querySelector("form[data-lui-target='#log']"); f.requestSubmit();
    return [document.querySelector('#log').hasAttribute('data-lui-busy'), f.getAttribute('aria-busy'), f.querySelector('button').disabled, !document.querySelector('#saving').hidden]`);
  assert(busy.every(Boolean), "busy: root marked, form aria-busy, button disabled, indicator shown while pending");
  await until(async () => (await js("return document.querySelectorAll('#log li').length")) === before + 2, "second note appended");
  const idle = await js("return [!document.querySelector('[data-lui-busy],[aria-busy],[data-lui-disabled]'), !document.querySelector('form button').disabled, document.querySelector('#saving').hidden]");
  assert(idle.every(Boolean), "busy: everything restored after the swap");
  // A failed fetch becomes the navigation the browser would have made.
  await js("window.fetch = function () { return Promise.reject(new Error('down')); }");
  await click("a[data-lui-target='#count']");
  await until(async () => await js("return !/down/.test(String(window.fetch))"), "document reloaded after the failed fetch");
  assert(await js("return location.search") === "?n=2" && (await text("#count")) === "2", "busy: failed request fell back to a full navigation");
  // History: Back and Forward restore the root from the entry's copy, with no request; lui:swap fires.
  await js("window.__swaps = []; addEventListener('lui:swap', function (e) { window.__swaps.push(e.detail); })");
  await click("a[data-lui-target='#count']:not([data-lui-push])");
  await until(async () => (await text("#count")) === "3", "count pushed to 3");
  await js("window.__fetch = window.fetch; window.fetch = function () { return Promise.reject(new Error('down')); }; history.back()");
  await until(async () => (await text("#count")) === "2", "count restored by Back");
  assert(await js("return location.search") === "?n=2" && await navigations() === 1, "history: Back restored the root from the cached copy");
  await js("history.forward()");
  await until(async () => (await text("#count")) === "3", "count restored by Forward");
  assert(await js("return location.search") === "?n=3", "history: Forward restored the root too");
  assert(await js("return window.__swaps.filter(function (d) { return d.id === 'count'; }).length") === 3, "history: lui:swap fired for the swap and both restores");
  await js("window.fetch = window.__fetch");
  await click("a[data-lui-push='false']");
  await until(async () => (await text("#count")) === "12", "count swapped with the URL kept");
  assert(await js("return location.search") === "?n=3", "history: data-lui-push=false keeps the URL");
  const len = await js("var a = document.querySelector('a[data-lui-push]'); a.removeAttribute('data-lui-push'); a.setAttribute('data-lui-replace', ''); return history.length");
  const swapsBefore = await js("return window.__swaps.length");
  await click("a[data-lui-replace]");
  await until(async () => (await js("return window.__swaps.length")) === swapsBefore + 1, "swap with replace");
  assert(await js("return history.length") === len && await js("return location.search") === "?n=12", "history: data-lui-replace rewrites the entry");

  // Dialog: opens modal from the invoker, focus on the first field, Escape closes, confirm posts.
  await go("/dialog");
  await click("button[command=show-modal]");
  await until(async () => await js("return document.querySelector('dialog').open"), "dialog open");
  assert(await js("return document.activeElement.name") === "reason", "dialog: focus landed on the first field");
  await type("input[name=reason]", "\uE00C"); // Escape
  await until(async () => !(await js("return document.querySelector('dialog').open")), "dialog closed by Escape");
  await click("button[command=show-modal]");
  await until(async () => await js("return document.querySelector('dialog').open"), "dialog open again");
  await click("dialog button.lui-button-danger");
  await until(async () => (await js("return document.querySelector('.lui-flash')?.textContent")) || "").then(() => {}, () => {});
  await until(async () => /Account deleted/.test(await js("return document.querySelector('.lui-flash')?.textContent || ''")), "flash after confirm");
  assert(await js("return location.pathname") === "/dialog" && !(await js("return document.querySelector('dialog').open")), "dialog: confirm posted and the server came back");

  // Popover: arrow keys walk the items, submenu opens inside, an action posts and comes back.
  await go("/popover");
  await click("button[popovertarget=account]");
  await until(async () => await js("return document.querySelector('#account').matches(':popover-open')"), "menu open");
  await type("button[popovertarget=account]", ""); // ArrowDown from the button
  assert(await js("return document.activeElement.querySelector('.lui-popover-text').textContent") === "Profile", "popover: ArrowDown focuses the first item");
  await js("document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))");
  assert(await js("return document.activeElement.querySelector('.lui-popover-text').textContent") === "Settings", "popover: ArrowDown moves to the next item");
  await click("button[popovertarget=account-theme]");
  await until(async () => await js("return document.querySelector('#account-theme').matches(':popover-open') && document.querySelector('#account').matches(':popover-open')"), "submenu open with parent");
  await click("#account form button");
  await until(async () => /Signed out/.test(await js("return document.querySelector('.lui-flash')?.textContent || ''")), "flash after the action");

  // Flash: saving with notifications off stacks ok + warn; ok fades by CSS; dismiss clears both.
  await go("/settings?tab.settings=1");
  await click(".lui-tabs details[open] button[type=submit]");
  await until(async () => (await js("return document.querySelectorAll('.lui-flash-item').length")) === 2, "flash: ok and warn stacked");
  assert(await js("return getComputedStyle(document.querySelector('.lui-flash-ok')).animationName") === "lui-flash-hide", "flash: ok auto-hides by CSS animation");
  assert(await js("return getComputedStyle(document.querySelector('.lui-flash-warn')).animationName") === "none", "flash: warn stays");
  await click(".lui-flash-warn .lui-flash-dismiss");
  await until(async () => (await js("return document.querySelectorAll('.lui-flash-item').length")) === 0, "flash: dismiss link clears the stack");

  // Command palette: the popover opens with the caret in the box; an exact name redirects.
  await go("/palette");
  await click(".lui-palette-open");
  await until(async () => await js("return document.querySelector('#cmd').matches(':popover-open') && document.activeElement.name === 'q'"), "palette: opens focused");
  await type("#cmd input[name=q]", "Toasts\ue007"); // Enter
  await until(async () => (await js("return location.pathname")) === "/toast", "palette: exact name goes to its page");

  // Toasts: posted, stacked in the corner, calm ones fade, danger stays.
  await click("button[value=all]");
  await until(async () => (await js("return document.querySelectorAll('.lui-toast').length")) === 3, "toasts: three stacked");
  assert(await js("return getComputedStyle(document.querySelector('.lui-toasts')).position") === "fixed", "toasts: out of the flow");
  assert(await js("return getComputedStyle(document.querySelector('.lui-toast-ok')).animationName") === "lui-toast-out", "toasts: ok fades");
  assert(await js("return getComputedStyle(document.querySelector('.lui-toast-danger')).animationName") === "none", "toasts: danger stays");

  // Range: output mirrors while moving, before any submit.
  await go("/inputs");
  await type("#f-volume", ""); // ArrowRight
  assert((await text(".lui-range output")) !== "40", "range: output mirrors the slider live");
  await type("#f-price_max", ""); // ArrowLeft
  assert((await text("output[for=f-price_max]")) === "75", "range pair: the high thumb mirrors into its own output");
  // Select: typing in the filter re-renders the options through a GET, nothing is saved.
  await type("input[name=country-q]", "jap");
  await until(async () => (await js("return [...document.querySelectorAll('#country option')].map(o => o.value).join()")) === "es,jp", "filtered to Japan plus the selected Spain");
  assert(await js("return location.search").then(q => q.includes("country-q=jap")), "select: the filter is a GET in the URL");
  assert(await navigations() === 1, "select: filtered in place");
  await click(".lui-color-presets button[value='#b3261e']");
  await until(async () => (await js("return document.querySelector('.lui-color-presets button[aria-pressed=true]')?.value")) === "#b3261e", "preset saved");
  assert(/Inputs saved/.test(await js("return document.querySelector('.lui-flash')?.textContent || ''")), "color: a preset posts the form");

  // Button: Tab onto the primary button; the keyboard focus ring is a 3px outline (Blitz
  // cannot check this: :focus-visible never matches there, blitz#839).
  await go("/button");
  await js("document.querySelector('.lui-back').focus()");
  const tab = () => wd("POST", S + "/actions", { actions: [{ type: "key", id: "kb", actions: [{ type: "keyDown", value: "\ue004" }, { type: "keyUp", value: "\ue004" }] }] });
  for (let i = 0; i < 10 && !(await js("return document.activeElement.matches('.lui-button-primary')")); i++) await tab();
  assert(await js("return document.activeElement.matches('.lui-button-primary:focus-visible')"), "button: reached by Tab, :focus-visible");
  assert(await js("return getComputedStyle(document.activeElement).outlineWidth") === "3px", "button: the focus ring is a 3px outline");

  // Table: edit a row in place; Save posts, the redirect comes back with the new value.
  await go("/table?per.files=5&edit.files=src/build.rs");
  await js("const i = document.querySelector('.lui-table-edit-input'); i.value = 'rust'");
  await click(".lui-table-editing .lui-button-primary");
  await until(async () => /Saved src\/build.rs/.test(await js("return document.querySelector('.lui-flash')?.textContent || ''")), "table: row saved");
  assert(await js("return !document.querySelector('.lui-table-edit-input') && [...document.querySelectorAll('td')].some(td => td.textContent.trim() === 'rust')"), "table: an edited row is saved and shown");

  // Upload: a real file through the enhancement script (XMLHttpRequest, progress bar), in place.
  await go("/upload");
  await type(".lui-upload-input", process.cwd() + "/loco-ui/src/upload.rs");
  await click(".lui-upload-form .lui-button-primary");
  await until(async () => /Uploaded 1/.test(await js("return document.querySelector('.lui-flash')?.textContent || ''")), "upload: the file arrived");
  assert(await js("return [...document.querySelectorAll('.lui-upload-name')].some(n => n.textContent.trim() === 'upload.rs')"), "upload: the list shows what the server kept");
  assert(await navigations() === 1, "upload: sent in place, no reload");

  // Kanban: an arrow posts the move; the card lands in the next column in place.
  await go("/kanban");
  await click(".lui-kanban-card:has(input[value=docs]) button[value=doing]");
  await until(async () => await js("return !!document.querySelector('.lui-kanban-column:nth-child(2) input[value=docs]')"), "kanban: card moved");
  assert(await navigations() === 1, "kanban: moved in place, no reload");
  await sleep(600); // let the card's view transition finish before the next click
  await click(".lui-kanban-card:has(input[value=docs]) button[value=todo]");
  await until(async () => await js("return !!document.querySelector('.lui-kanban-column:nth-child(1) input[value=docs]')"), "kanban: card moved back");
  assert(true, "kanban: and back again");

  // Calendar: the next-month link swaps the calendar in place; picking a day follows.
  await go("/calendar?month.day=2026-09");
  await click("#lui-calendar-day a[aria-label='Next month']");
  await until(async () => (await text("#lui-calendar-day .lui-calendar-title")) === "October 2026", "calendar: next month");
  assert(await navigations() === 1 && (await js("return location.search")).includes("month.day=2026-10"), "calendar: month changed in place, URL follows");
  await click("#lui-calendar-day a[href*='day=2026-10-15']");
  await until(async () => await js("return !!document.querySelector('.lui-calendar-picked[href*=\"2026-10-15\"]')"), "calendar: day picked");
  assert(await navigations() === 1, "calendar: a day picked in place");

  // Tooltip: hidden until its trigger has focus (or the pointer), then shown; named by aria-describedby.
  await go("/feedback");
  assert(await js("return getComputedStyle(document.querySelector('.lui-tooltip-text')).visibility") === "hidden", "tooltip: hidden at rest");
  await js("document.querySelector('.lui-tooltip button').focus()");
  await until(async () => (await js("return getComputedStyle(document.querySelector('.lui-tooltip-text')).visibility")) === "visible", "tooltip shows on focus");
  assert(await js("return document.activeElement.getAttribute('aria-describedby') === document.querySelector('.lui-tooltip-text').id"), "tooltip: shown on focus and named by aria-describedby");

  // Date picker: the button opens the calendar popover; a day is a radio the form posts.
  await go("/calendar?due=2026-09-24");
  await click("#f-due");
  await until(async () => await js("return document.querySelector('#f-due-calendar').matches(':popover-open')"), "date picker: popover opens");
  assert(true, "date picker: the button opens the calendar");
  await click("#f-due-calendar label:has(input[value='2026-09-25'])");
  assert(await js("return document.querySelector(\"#f-due-calendar input[value='2026-09-25']\").checked"), "date picker: clicking a day checks its radio");

  // Theme: applied in place.
  await click(".lui-theme button[value=dark]");
  await until(async () => (await js("return document.documentElement.dataset.theme")) === "dark", "theme");
  assert(await navigations() === 1, "theme: switched without a reload");

  // Index prefetch: a component page opens from the cache with the current theme. (A copy
  // prefetched before a theme change is not reused: Vary: Cookie; checked by hand, see FINDINGS.)
  await go("/");
  await sleep(1000);
  await click(".lui-index a[href='/dialog']");
  await until(async () => (await js("return location.pathname")) === "/dialog", "dialog opened");
  assert(await js("return performance.getEntriesByType('navigation')[0].transferSize") === 0, "index: the prefetched page came from the cache");
  assert(await js("return document.documentElement.dataset.theme") === "dark", "index: the cached copy follows the theme cookie");
  await click(".lui-theme button[value=auto]");
  await until(async () => (await js("return document.documentElement.dataset.theme")) === "auto", "theme back");
} catch (e) {
  console.error("FAIL: " + e.message);
  await wd("DELETE", S).catch(() => {});
  quit(1);
}
await wd("DELETE", S);
console.log("browser check OK");
quit(0);
