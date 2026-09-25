#!/usr/bin/env node
// Click-to-paint of an in-place update: the enhancement script against htmx doing the same
// swap of the same server answer, in headless Firefox through geckodriver.
//
// The demo runs on 3002. A small proxy on 3003 serves the same pages with the enhancement
// script replaced by htmx 2 (`hx-boost`, `hx-target="this"`, `hx-select="#id"`,
// `hx-swap="outerHTML"` on every swap root) and forwards htmx's requests with
// `Nojs-Enhance: 1`, so both sides fetch the same bytes. For each case the page is loaded
// fresh, then a script clicks and times until the new state is on the page and two frames
// have run (the paint that shows it).
//
// Usage: node scripts/bench-swap.mjs [runs]   (needs a built demo, geckodriver, firefox,
// npm for htmx on first run). Uses target/release/demo when it exists.
import { spawn, execSync } from "node:child_process";
import { existsSync, readFileSync, mkdirSync } from "node:fs";
import http from "node:http";
import { setTimeout as sleep } from "node:timers/promises";

const RUNS = Number(process.argv[2] || 20);
const DEMO = "http://127.0.0.1:3002", PROXY = "http://127.0.0.1:3003", DRIVER = "http://127.0.0.1:4446";

const HTMX = "target/bench/htmx.min.js";
if (!existsSync(HTMX)) {
  mkdirSync("target/bench", { recursive: true });
  execSync("npm pack htmx.org@2 --silent && tar xzf htmx.org-*.tgz package/dist/htmx.min.js && mv package/dist/htmx.min.js .", { cwd: "target/bench", stdio: "ignore" });
}
const htmx = readFileSync(HTMX);
const bin = existsSync("target/release/demo") ? "target/release/demo" : "target/debug/demo";

const server = spawn(bin, { stdio: "ignore", env: { ...process.env, PORT: "3002" } });
const driver = spawn("geckodriver", ["--port", "4446"], { stdio: "ignore" });

// The htmx side: same pages, htmx instead of the enhancement script.
const proxy = http.createServer(async (req, res) => {
  if (req.url === "/htmx.min.js") { res.writeHead(200, { "content-type": "text/javascript" }); return res.end(htmx); }
  const headers = { ...req.headers };
  delete headers.host; delete headers["accept-encoding"];
  const fromHtmx = "hx-request" in headers;
  if (fromHtmx) headers["nojs-enhance"] = "1";
  const body = await new Promise((ok) => { const c = []; req.on("data", (d) => c.push(d)); req.on("end", () => ok(Buffer.concat(c))); });
  const up = await fetch(DEMO + req.url, { method: req.method, headers, body: body.length ? body : undefined, redirect: "manual" });
  const out = {};
  up.headers.forEach((v, k) => { if (!["content-length", "content-security-policy", "content-encoding"].includes(k)) out[k] = v; });
  let text = Buffer.from(await up.arrayBuffer());
  if ((out["content-type"] || "").startsWith("text/html") && !fromHtmx) {
    text = Buffer.from(text.toString()
      .replace(/<script src="\/nojs\/enhance\.js[^"]*"[^>]*><\/script>/, "")
      .replace("</body>", '<script src="/htmx.min.js"></script></body>')
      .replace(/id="([^"]+)" data-nojs="swap"/g, 'id="$1" data-nojs="swap" hx-boost="true" hx-target="this" hx-select="#$1" hx-swap="outerHTML"'));
  }
  res.writeHead(up.status, out);
  res.end(text);
});
proxy.listen(3003);
const quit = (code) => { server.kill(); driver.kill(); proxy.close(); process.exit(code); };

async function wd(method, path, body) {
  const r = await fetch(DRIVER + path, { method, headers: { "content-type": "application/json" }, body: body && JSON.stringify(body) });
  const json = await r.json();
  if (json.value && json.value.error) throw new Error(`${method} ${path}: ${json.value.message}`);
  return json.value;
}
async function ready(url) {
  for (let i = 0; i < 100; i++) { try { await fetch(url); return; } catch { await sleep(100); } }
  throw new Error("not reachable: " + url);
}

// Each case: the page, what to click, and the state that means the update is shown.
const CASES = [
  { name: "table sort", path: "/table", click: '#nojs-paged-table-files th a[href*="sort=size"]', root: "nojs-paged-table-files", shown: "replaced" },
  { name: "tab", path: "/tabs", click: "#nojs-tabs-demo details:nth-of-type(2) > summary a", root: "nojs-tabs-demo", shown: "tab" },
  { name: "pager", path: "/list", click: "#nojs-pager--list .nojs-pager-more", root: "nojs-pager--list", shown: "replaced" },
];

// Runs in the page: click, then resolve with the milliseconds until the new state is on the
// page and two animation frames have passed.
const MEASURE = `
const [sel, id, shown, done] = arguments;
const before = document.getElementById(id);
const ok = () => shown === "tab"
  ? (document.querySelectorAll("#" + id + " details")[1] || {}).open === true
  : document.getElementById(id) !== before && document.getElementById(id) !== null;
const t0 = performance.now();
let settled = false;
const finish = () => { if (settled || !ok()) return; settled = true; obs.disconnect();
  requestAnimationFrame(() => requestAnimationFrame(() => done(performance.now() - t0))); };
const obs = new MutationObserver(finish);
obs.observe(document.documentElement, { subtree: true, childList: true, attributes: true });
document.querySelector(sel).click();
finish();
setTimeout(() => { if (!settled) done(-1); }, 5000);
`;

const median = (xs) => { const s = [...xs].sort((a, b) => a - b); return s[Math.floor(s.length / 2)]; };
const p90 = (xs) => { const s = [...xs].sort((a, b) => a - b); return s[Math.min(s.length - 1, Math.floor(s.length * 0.9))]; };

try {
  await ready(DEMO + "/");
  await ready(DRIVER + "/status");
  const { sessionId } = await wd("POST", "/session", { capabilities: { alwaysMatch: { browserName: "firefox", "moz:firefoxOptions": { args: ["-headless"] } } } });
  const S = "/session/" + sessionId;
  await wd("POST", S + "/timeouts", { script: 10000 });
  const go = (url) => wd("POST", S + "/url", { url });
  // Two loads so the capability beacons have set their cookies (shared by both ports).
  await go(DEMO + "/tabs"); await go(DEMO + "/tabs");
  const rows = [];
  for (const c of CASES) {
    const result = {};
    for (const [side, base] of [["script", DEMO], ["htmx", PROXY]]) {
      const times = [];
      for (let i = 0; i < RUNS + 3; i++) {
        await go(base + c.path);
        await sleep(150);
        const ms = await wd("POST", S + "/execute/async", { script: MEASURE, args: [c.click, c.root, c.shown] });
        if (ms < 0) throw new Error(`${side} ${c.name}: the new state never showed`);
        if (i >= 3) times.push(ms);
      }
      result[side] = times;
    }
    rows.push([c.name, result]);
  }
  await wd("DELETE", S);
  console.log(`click to painted update, ms, ${RUNS} runs each (${bin}, Firefox headless)`);
  console.log("| Update | enhancement script p50 / p90 | htmx 2 p50 / p90 |");
  console.log("|---|---|---|");
  for (const [name, r] of rows) {
    const f = (xs) => `${median(xs).toFixed(0)} / ${p90(xs).toFixed(0)}`;
    console.log(`| ${name} | ${f(r.script)} | ${f(r.htmx)} |`);
  }
  quit(0);
} catch (e) {
  console.error(e.message);
  quit(1);
}
