#!/usr/bin/env node
// Drives headless Firefox through geckodriver (plain WebDriver over HTTP, no packages) to
// prove the enhancement script does its job: actions happen in place, with no navigation.
// Usage: node scripts/browser-check.mjs   (needs target/debug/demo built, geckodriver, firefox)
import { spawn } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";

const DEMO = "http://127.0.0.1:3000";
const DRIVER = "http://127.0.0.1:4444";
const server = spawn("target/debug/demo", { stdio: "ignore" });
const driver = spawn("geckodriver", ["--port", "4444"], { stdio: "ignore" });
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
const click = async (css) => wd("POST", S + `/element/${await find(css)}/click`, {});
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
  await until(async () => (await text(".wo-counter output")) === "5", "counter to reach 5");
  assert(await navigations() === 1, "counter: 5 quick clicks counted in place, no reload");

  // Tabs: click a title, panel switches, URL updated, no reload.
  await go("/tabs");
  await click(".wo-tabs summary a[href*='tab.demo=1']");
  await until(async () => (await text(".wo-tabs details[open] summary")) === "Use", "tab switch");
  assert(await js("return location.search") === "?tab.demo=1", "tabs: URL follows the swap");
  assert(await navigations() === 1, "tabs: switched without a reload");

  // Combobox: results as you type, focus kept.
  await go("/combobox");
  await type("input[type=search]", "ru");
  await until(async () => (await js("return [...document.querySelectorAll('#langs li')].map(l => l.textContent).join()")) === "Rust,Ruby", "search results");
  assert(await js("return document.activeElement.name") === "q", "combobox: focus stays in the input");
  assert(await navigations() === 1, "combobox: searched without a reload");

  // Range: output mirrors while moving, before any submit.
  await go("/inputs");
  await type("input[type=range]", ""); // ArrowRight
  assert((await text(".wo-range output")) !== "40", "range: output mirrors the slider live");

  // Theme: applied in place.
  await click(".wo-theme button[value=dark]");
  await until(async () => (await js("return document.documentElement.dataset.theme")) === "dark", "theme");
  assert(await navigations() === 1, "theme: switched without a reload");
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
