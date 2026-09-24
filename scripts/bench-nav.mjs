#!/usr/bin/env node
// Navigation timing from headless Firefox for the routes given as arguments, against a demo
// already running on 3001 (scripts/bench.sh starts it). Plain WebDriver over HTTP, no packages.
import { spawn } from "node:child_process";
import { setTimeout as sleep } from "node:timers/promises";

const DEMO = "http://127.0.0.1:3001";
const DRIVER = "http://127.0.0.1:4445";
const driver = spawn("geckodriver", ["--port", "4445"], { stdio: "ignore" });
async function wd(method, path, body) {
  const res = await fetch(DRIVER + path, { method, headers: { "content-type": "application/json" }, body: body && JSON.stringify(body) });
  const json = await res.json();
  if (json.value && json.value.error) throw new Error(`${method} ${path}: ${json.value.message}`);
  return json.value;
}
for (let i = 0; i < 50; i++) { try { await fetch(DRIVER + "/status"); break; } catch { await sleep(100); } }
const { sessionId } = await wd("POST", "/session", { capabilities: { alwaysMatch: { browserName: "firefox", "moz:firefoxOptions": { args: ["-headless"] } } } });
const S = "/session/" + sessionId;
const median = (xs) => xs.sort((a, b) => a - b)[Math.floor(xs.length / 2)];
try {
  for (const route of process.argv.slice(2)) {
    const runs = { ttfb: [], dcl: [], load: [] };
    for (let i = 0; i < 5; i++) {
      await wd("POST", S + "/url", { url: DEMO + route });
      const n = await wd("POST", S + "/execute/sync", { script: "const n = performance.getEntriesByType('navigation')[0]; return [n.responseStart, n.domContentLoadedEventEnd, n.loadEventEnd]", args: [] });
      runs.ttfb.push(n[0]); runs.dcl.push(n[1]); runs.load.push(n[2]);
    }
    const f = (x) => median(x).toFixed(0).padStart(5);
    console.log(`${route.padEnd(8)} responseStart ${f(runs.ttfb)}  DOMContentLoaded ${f(runs.dcl)}  load ${f(runs.load)} ms`);
  }
} finally {
  await wd("DELETE", S).catch(() => {});
  driver.kill();
}
