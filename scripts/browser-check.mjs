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
  await until(async () => (await text(".wo-counter output")) === "0", "reset");
  for (let i = 0; i < 5; i++) await click("button[value=inc]");
  await until(async () => (await text(".wo-counter output")) === "10", "counter to reach 10 in steps of 2");
  assert(await navigations() === 1, "counter: 5 quick clicks counted in place, no reload");
  await js("document.querySelector('.wo-counter input[name=value]').value = 20");
  await click(".wo-counter button[value=set]");
  await until(async () => (await text(".wo-counter output")) === "20", "typed value set");
  assert(await js("return document.querySelector('.wo-counter button[value=inc]').disabled"), "counter: + switches off at the maximum");

  // Tabs: click a title, panel switches, URL updated, no reload.
  await go("/tabs");
  await click(".wo-tabs summary a[href*='tab.demo=1']");
  await until(async () => (await text(".wo-tabs details[open] summary")).startsWith("Use"), "tab switch");
  assert(await js("return location.search") === "?tab.demo=1", "tabs: URL follows the swap");
  assert(await navigations() === 1, "tabs: switched without a reload");
  assert(await js("return getComputedStyle(document.querySelector('.wo-tabs details[open] .wo-tabs-mark')).viewTransitionName") === "wo-tabs-demo", "tabs: the underline, not the title, carries the view-transition-name");
  const fetched = (q) => js("return performance.getEntriesByType('resource').filter((r) => r.name.endsWith(arguments[0])).length", q);
  await js("document.querySelector(\".wo-tabs summary a[href*='tab.demo=2']\").dispatchEvent(new MouseEvent('mouseover', { bubbles: true }))");
  await until(async () => (await fetched("?tab.demo=2")) === 1, "hover prefetches the tab");
  await click(".wo-tabs summary a[href*='tab.demo=2']");
  await until(async () => await js("return !!document.querySelector('#wo-tabs-demo details[open] .wo-tabs-panel p')"), "prefetched tab shown");
  assert(await fetched("?tab.demo=2") === 1 && await navigations() === 1, "tabs: the click reused the prefetched answer in place, no second request");
  await until(async () => await js("return !!document.querySelector('#wo-tabs-demo details[open] .wo-tabs-panel p')"), "lazy tab filled");
  assert(!(await js("return document.querySelector('#wo-tabs-demo details[open] .wo-tabs-lazy')")), "tabs: lazy panel rendered by the request that opened it");
  await wd("POST", S + "/window/rect", { width: 500, height: 800 });
  await until(async () => await js("return document.querySelector('#wo-tabs-demo .wo-tabs-select').offsetParent !== null"), "select shown on a narrow screen");
  assert(await js("return document.querySelector('#wo-tabs-demo summary').offsetParent === null"), "tabs: titles hidden when the select shows");
  await js("var s = document.querySelector('#wo-tabs-demo select'); s.value = '0'; s.dispatchEvent(new Event('change', { bubbles: true }))");
  await until(async () => (await js("return document.querySelector('#wo-tabs-demo select').value")) === "0" && (await text("#wo-tabs-demo details[open] summary")) === "Install", "select switched the tab");
  assert(await navigations() === 1 && await js("return location.search") === "?tab.demo=0", "tabs: select change swapped in place");
  await wd("POST", S + "/window/rect", { width: 1000, height: 700 });

  // Accordion: expand all, then one title, in place; several stay open.
  await go("/accordion");
  await click(".wo-accordion-controls a");
  await until(async () => (await js("return document.querySelectorAll('#wo-accordion-faq > details[open]').length")) === 3, "expand all");
  assert(await js("return location.search").then(q => q.includes("open.faq=0%2C1%2C2")), "accordion: expand all swapped in place with the list in the URL");
  await click("#wo-accordion-faq > details:nth-of-type(2) > summary a");
  await until(async () => (await js("return [...document.querySelectorAll('#wo-accordion-faq > details')].map(d => d.open ? 1 : 0).join('')")) === "101", "toggle one of three");
  await click("#wo-accordion-faq-more > details:nth-of-type(2) > summary a");
  await until(async () => (await js("return [...document.querySelectorAll('#wo-accordion-faq-more > details')].map(d => d.open ? 1 : 0).join('')")) === "01", "nested group toggles on its own key");
  assert(await js("return [...document.querySelectorAll('#wo-accordion-faq > details')].map(d => d.open ? 1 : 0).join('')") === "101", "accordion: nested toggle kept the outer sections");
  await click(".wo-accordion-controls a:last-child");
  await until(async () => (await js("return document.querySelectorAll('#wo-accordion-faq > details[open]').length")) === 0, "collapse all");
  assert(await navigations() === 1, "accordion: every toggle swapped without a reload");
  await go("/accordion");
  assert(await js("return document.querySelectorAll('#wo-accordion-faq > details[open]').length") === 0, "accordion: collapse all beat the cookie's memory on the next visit");

  // Combobox: results as you type, focus kept.
  await go("/combobox");
  await type("input[type=search]", "ru");
  await until(async () => (await js("return [...document.querySelectorAll('#langs [role=option]')].map(l => l.textContent).join()")) === "Rust,Ruby", "search results");
  assert(await js("return document.activeElement.name") === "q", "combobox: focus stays in the input");
  await js("document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))");
  assert(await js("return document.activeElement.textContent") === "Rust", "combobox: ArrowDown moves from the input to the first result");
  await js("document.activeElement.click()");
  await until(async () => (await js("return [...document.querySelectorAll('.wo-combobox-chip')].map(c => c.firstChild.textContent).join()")) === "Rust", "result became a chip");
  assert(await js("return location.search") === "?q=ru&sel=Rust", "combobox: the chip is in the URL");
  await click(".wo-combobox-chip a");
  await until(async () => (await js("return document.querySelectorAll('.wo-combobox-chip').length")) === 0, "chip removed");
  assert(await navigations() === 1, "combobox: searched, picked and removed without a reload");

  // Table: a sort link re-renders the rows in place, URL follows.
  await go("/table?per.files=5");
  await click(".wo-table th a[href*='sort=size']");
  await until(async () => (await js("return document.querySelector('.wo-table th[aria-sort]')?.textContent.trim()")) === "Size▲", "sort by size");
  assert(await js("return location.search") === "?sort=size&dir=asc&per.files=5", "table: URL follows the sort");
  assert(await navigations() === 1, "table: sorted without a reload");
  assert(await js("return document.querySelector('a[rel=next]').href.includes('sort=size')"), "table: the pager's links follow the sort swapped in place");
  await click(".wo-table-cols summary");
  await click(".wo-table-cols a[href*='cols=name%2Csize']");
  await until(async () => (await js("return document.querySelectorAll('.wo-table thead th a').length")) === 2, "column hidden");
  assert(await js("return location.search").then(q => q.includes("cols=name%2Csize")), "table: hidden column swapped in place with ?cols= in the URL");
  await js("document.querySelector('tbody tr:nth-child(2) input[type=checkbox]').click(); document.querySelector('#wo-table-files-bulk button[value=archive]').click()");
  await until(async () => (await js("return document.querySelector('.wo-flash')?.textContent || ''")).includes("archive: 1 file"), "bulk form posted and redirected with a flash");
  await js("document.querySelector('tbody tr:first-child .wo-table-detail summary').click()");
  assert(await js("return document.querySelector('tbody tr:first-child .wo-table-detail').open"), "table: a row's detail opens natively");
  // Paged table: the page size picked on one visit is remembered on the next (wo-ui cookie).
  await go("/table");
  assert(await text(".wo-paged-table-range") === "1–5 of 36", "table: page size remembered from the earlier visit");
  await js("const f = document.querySelector('.wo-paged-table-jump'); f.page.value = 7; f.requestSubmit()");
  await until(async () => (await js("return document.querySelector('.wo-paged-table-range')?.textContent")) === "31–35 of 36", "jump to page 7");
  assert(await js("return location.search").then(q => q.includes("page=7")), "table: the jump is in the URL");
  assert(await navigations() === 1, "table: jumped without a reload");

  // Wizard: Next posts, PRG lands on step 2 in place, Back keeps the value.
  await go("/wizard?step.signup=0");
  await type("input[name=name]", "Ada");
  await type("input[name=email]", "ada@example.com");
  await click(".wo-wizard button.wo-primary");
  await until(async () => (await js("return document.querySelector('#f-email-error')?.textContent || ''")).includes("example.com"), "server message beside the field");
  assert(await js("return document.querySelector('.wo-wizard-steps li[aria-current=step]').classList.contains('wo-wizard-error')"), "wizard: the step is marked in error");
  await js("const e = document.querySelector('input[name=email]'); e.value = ''");
  await type("input[name=email]", "ada@example.org");
  await click(".wo-wizard button.wo-primary");
  await until(async () => (await text(".wo-wizard li[aria-current=step]")) === "Newsletter (optional)", "wizard step 2");
  assert(await navigations() === 1, "wizard: advanced without a reload");
  await click(".wo-wizard-back");
  await until(async () => (await js("return document.querySelector('input[name=name]')?.value")) === "Ada", "wizard back keeps the name");
  await click(".wo-wizard button.wo-primary");
  await until(async () => (await text(".wo-wizard li[aria-current=step]")) === "Newsletter (optional)", "wizard step 2 again");
  await click(".wo-wizard button[name=skip]");
  await until(async () => (await js("return document.querySelector('.wo-wizard-review')?.textContent || ''")).includes("(skipped)"), "skipped straight to the review");
  await go("/wizard");
  assert(await js("return !!document.querySelector('.wo-wizard-resume')") && await text(".wo-wizard li[aria-current=step]") === "Review", "wizard: a new visit resumes at the review");

  // Form: the counter follows typing; a multipart post with a file lands as a flash in place.
  await go("/form");
  await type("#f-bio", "Hello there");
  assert(await text("output[for=f-bio]") === "11 / 160", "form: the counter follows typing");
  await type("#f-name", "Ada");
  await type("#f-email", "ada@example.org");
  await type("#f-age", "36");
  await type("#f-handle", "ada_l");
  await type("#f-avatar", process.cwd() + "/webonsive-test/tests/fixture.png");
  await click(".wo-form button[type=submit]");
  await until(async () => (await js("return document.querySelector('.wo-flash')?.textContent || ''")).includes("fixture.png"), "file posted as multipart and named in the flash");
  assert(await navigations() === 1, "form: submitted with a file without a reload");

  // Swap targets: a link outside any root updates only #count; a form appends to #log.
  await go("/swap");
  await click("a[data-wo-target='#count']");
  await until(async () => (await text("#count")) === "2", "count swapped by target");
  assert(await js("return location.search") === "?n=2", "swap: URL follows the targeted link");
  const before = await js("return document.querySelectorAll('#log li').length");
  await type("input[name=note]", "hello");
  await click("form[data-wo-target='#log'] button");
  await until(async () => (await js("return document.querySelectorAll('#log li').length")) === before + 1, "note appended");
  assert(await js("return document.querySelector('#log li:last-child').textContent") === "hello", "swap: appended the fragment only");
  assert((await text("#note-count")) === String(before + 1), "swap: out-of-band count updated outside the target");
  assert(!(await js("return document.querySelector('[data-wo-oob]')")), "swap: oob element not left in the page");
  assert(await navigations() === 1, "swap: target and append without a reload");
  // Busy state is observable synchronously right after the submit is dispatched.
  await type("input[name=note]", "again");
  const busy = await js(`var f = document.querySelector("form[data-wo-target='#log']"); f.requestSubmit();
    return [document.querySelector('#log').hasAttribute('data-wo-busy'), f.getAttribute('aria-busy'), f.querySelector('button').disabled, !document.querySelector('#saving').hidden]`);
  assert(busy.every(Boolean), "busy: root marked, form aria-busy, button disabled, indicator shown while pending");
  await until(async () => (await js("return document.querySelectorAll('#log li').length")) === before + 2, "second note appended");
  const idle = await js("return [!document.querySelector('[data-wo-busy],[aria-busy],[data-wo-disabled]'), !document.querySelector('form button').disabled, document.querySelector('#saving').hidden]");
  assert(idle.every(Boolean), "busy: everything restored after the swap");
  // A failed fetch becomes the navigation the browser would have made.
  await js("window.fetch = function () { return Promise.reject(new Error('down')); }");
  await click("a[data-wo-target='#count']");
  await until(async () => await js("return !/down/.test(String(window.fetch))"), "document reloaded after the failed fetch");
  assert(await js("return location.search") === "?n=2" && (await text("#count")) === "2", "busy: failed request fell back to a full navigation");
  // History: Back and Forward restore the root from the entry's copy, with no request; wo:swap fires.
  await js("window.__swaps = []; addEventListener('wo:swap', function (e) { window.__swaps.push(e.detail); })");
  await click("a[data-wo-target='#count']:not([data-wo-push])");
  await until(async () => (await text("#count")) === "3", "count pushed to 3");
  await js("window.__fetch = window.fetch; window.fetch = function () { return Promise.reject(new Error('down')); }; history.back()");
  await until(async () => (await text("#count")) === "2", "count restored by Back");
  assert(await js("return location.search") === "?n=2" && await navigations() === 1, "history: Back restored the root from the cached copy");
  await js("history.forward()");
  await until(async () => (await text("#count")) === "3", "count restored by Forward");
  assert(await js("return location.search") === "?n=3", "history: Forward restored the root too");
  assert(await js("return window.__swaps.filter(function (d) { return d.id === 'count'; }).length") === 3, "history: wo:swap fired for the swap and both restores");
  await js("window.fetch = window.__fetch");
  await click("a[data-wo-push='false']");
  await until(async () => (await text("#count")) === "12", "count swapped with the URL kept");
  assert(await js("return location.search") === "?n=3", "history: data-wo-push=false keeps the URL");
  const len = await js("var a = document.querySelector('a[data-wo-push]'); a.removeAttribute('data-wo-push'); a.setAttribute('data-wo-replace', ''); return history.length");
  const swapsBefore = await js("return window.__swaps.length");
  await click("a[data-wo-replace]");
  await until(async () => (await js("return window.__swaps.length")) === swapsBefore + 1, "swap with replace");
  assert(await js("return history.length") === len && await js("return location.search") === "?n=12", "history: data-wo-replace rewrites the entry");

  // Dialog: opens modal from the invoker, focus on the first field, Escape closes, confirm posts.
  await go("/dialog");
  await click("button[command=show-modal]");
  await until(async () => await js("return document.querySelector('dialog').open"), "dialog open");
  assert(await js("return document.activeElement.name") === "reason", "dialog: focus landed on the first field");
  await type("input[name=reason]", "\uE00C"); // Escape
  await until(async () => !(await js("return document.querySelector('dialog').open")), "dialog closed by Escape");
  await click("button[command=show-modal]");
  await until(async () => await js("return document.querySelector('dialog').open"), "dialog open again");
  await click("dialog button.wo-danger");
  await until(async () => (await js("return document.querySelector('.wo-flash')?.textContent")) || "").then(() => {}, () => {});
  await until(async () => /Account deleted/.test(await js("return document.querySelector('.wo-flash')?.textContent || ''")), "flash after confirm");
  assert(await js("return location.pathname") === "/dialog" && !(await js("return document.querySelector('dialog').open")), "dialog: confirm posted and the server came back");

  // Popover: arrow keys walk the items, submenu opens inside, an action posts and comes back.
  await go("/popover");
  await click("button[popovertarget=account]");
  await until(async () => await js("return document.querySelector('#account').matches(':popover-open')"), "menu open");
  await type("button[popovertarget=account]", ""); // ArrowDown from the button
  assert(await js("return document.activeElement.querySelector('.wo-popover-text').textContent") === "Profile", "popover: ArrowDown focuses the first item");
  await js("document.activeElement.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true }))");
  assert(await js("return document.activeElement.querySelector('.wo-popover-text').textContent") === "Settings", "popover: ArrowDown moves to the next item");
  await click("button[popovertarget=account-theme]");
  await until(async () => await js("return document.querySelector('#account-theme').matches(':popover-open') && document.querySelector('#account').matches(':popover-open')"), "submenu open with parent");
  await click("#account form button");
  await until(async () => /Signed out/.test(await js("return document.querySelector('.wo-flash')?.textContent || ''")), "flash after the action");

  // Flash: saving with notifications off stacks ok + warn; ok fades by CSS; dismiss clears both.
  await go("/settings?tab.settings=1");
  await click(".wo-tabs details[open] button[type=submit]");
  await until(async () => (await js("return document.querySelectorAll('.wo-flash-item').length")) === 2, "flash: ok and warn stacked");
  assert(await js("return getComputedStyle(document.querySelector('.wo-flash-ok')).animationName") === "wo-flash-hide", "flash: ok auto-hides by CSS animation");
  assert(await js("return getComputedStyle(document.querySelector('.wo-flash-warn')).animationName") === "none", "flash: warn stays");
  await click(".wo-flash-warn .wo-flash-dismiss");
  await until(async () => (await js("return document.querySelectorAll('.wo-flash-item').length")) === 0, "flash: dismiss link clears the stack");

  // Command palette: the popover opens with the caret in the box; an exact name redirects.
  await go("/palette");
  await click(".wo-palette-open");
  await until(async () => await js("return document.querySelector('#cmd').matches(':popover-open') && document.activeElement.name === 'q'"), "palette: opens focused");
  await type("#cmd input[name=q]", "Toasts\ue007"); // Enter
  await until(async () => (await js("return location.pathname")) === "/toast", "palette: exact name goes to its page");

  // Toasts: posted, stacked in the corner, calm ones fade, danger stays.
  await click("button[value=all]");
  await until(async () => (await js("return document.querySelectorAll('.wo-toast').length")) === 3, "toasts: three stacked");
  assert(await js("return getComputedStyle(document.querySelector('.wo-toasts')).position") === "fixed", "toasts: out of the flow");
  assert(await js("return getComputedStyle(document.querySelector('.wo-toast-ok')).animationName") === "wo-toast-out", "toasts: ok fades");
  assert(await js("return getComputedStyle(document.querySelector('.wo-toast-danger')).animationName") === "none", "toasts: danger stays");

  // Range: output mirrors while moving, before any submit.
  await go("/inputs");
  await type("#f-volume", ""); // ArrowRight
  assert((await text(".wo-range output")) !== "40", "range: output mirrors the slider live");
  await type("#f-price_max", ""); // ArrowLeft
  assert((await text("output[for=f-price_max]")) === "75", "range pair: the high thumb mirrors into its own output");
  // Select: typing in the filter re-renders the options through a GET, nothing is saved.
  await type("input[name=country-q]", "jap");
  await until(async () => (await js("return [...document.querySelectorAll('#country option')].map(o => o.value).join()")) === "es,jp", "filtered to Japan plus the selected Spain");
  assert(await js("return location.search").then(q => q.includes("country-q=jap")), "select: the filter is a GET in the URL");
  assert(await navigations() === 1, "select: filtered in place");
  await click(".wo-color-presets button[value='#b3261e']");
  await until(async () => (await js("return document.querySelector('.wo-color-presets button[aria-pressed=true]')?.value")) === "#b3261e", "preset saved");
  assert(/Inputs saved/.test(await js("return document.querySelector('.wo-flash')?.textContent || ''")), "color: a preset posts the form");

  // Theme: applied in place.
  await click(".wo-theme button[value=dark]");
  await until(async () => (await js("return document.documentElement.dataset.theme")) === "dark", "theme");
  assert(await navigations() === 1, "theme: switched without a reload");

  // Index prefetch: a component page opens from the cache with the current theme. (A copy
  // prefetched before a theme change is not reused: Vary: Cookie; checked by hand, see FINDINGS.)
  await go("/");
  await sleep(1000);
  await click(".wo-index a[href='/dialog']");
  await until(async () => (await js("return location.pathname")) === "/dialog", "dialog opened");
  assert(await js("return performance.getEntriesByType('navigation')[0].transferSize") === 0, "index: the prefetched page came from the cache");
  assert(await js("return document.documentElement.dataset.theme") === "dark", "index: the cached copy follows the theme cookie");
  await click(".wo-theme button[value=auto]");
  await until(async () => (await js("return document.documentElement.dataset.theme")) === "auto", "theme back");
} catch (e) {
  console.error("FAIL: " + e.message);
  await wd("DELETE", S).catch(() => {});
  quit(1);
}
await wd("DELETE", S);
console.log("browser check OK");
quit(0);
