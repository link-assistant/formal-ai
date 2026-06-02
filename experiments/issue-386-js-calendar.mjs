// Issue #386 — parity for the web runtime's calendar reasoning after it was
// rewritten to query the self-describing `calendar_*` meanings instead of
// hardcoded marker arrays.
//
// Mirrors the Rust specification tests in
// tests/unit/specification/reasoning_paths.rs (the weekday-successor,
// current-day-across-languages, predecessor/successor-variations, and
// hindi/chinese-relations cases). It drives the worker's own
// `tryCalendarReasoning(prompt, normalizePrompt(prompt))` — the same function
// the solver dispatches to — and asserts the lexicon-driven detection routes,
// computes, and *localizes* exactly like the Rust handler.
// Run: `node experiments/issue-386-js-calendar.mjs`.

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const src = fs.readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => { throw new Error("no importScripts in node"); };
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => { throw new Error("no fetch"); };
sandbox.location = { search: "", origin: "http://localhost" };
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename: "formal_ai_worker.js" });

const fail = [];
function check(name, cond, extra) {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}${extra ? " :: " + extra : ""}`);
  if (!cond) fail.push(name);
}

function calendar(prompt) {
  return sandbox.tryCalendarReasoning(prompt, sandbox.normalizePrompt(prompt), {});
}
function hasEvidencePrefix(result, prefix) {
  return Array.isArray(result.evidence) && result.evidence.some((e) => String(e).startsWith(prefix));
}

// --- 1: Russian weekday successor (mirror of the Rust pinned test) -----------
console.log("=== russian weekday successor ===");
{
  const r = calendar("какой день недели наступает после вторника");
  check("ru successor routes to calendar_weekday_relation", r && r.intent === "calendar_weekday_relation", r && r.intent);
  check("ru successor computes среда (Wednesday)", r && r.content.toLowerCase().includes("среда"), r && r.content);
  check("ru successor exposes calendar:operation:next evidence", r && hasEvidencePrefix(r, "calendar:operation:next"), r && JSON.stringify(r.evidence));
}

// --- 2: current-day questions across all four supported languages ------------
console.log("\n=== current-day questions across supported languages ===");
for (const [prompt, fragment, langTag] of [
  ["What day is today?", "Today is", "language:en"],
  ["Какой сегодня день?", "Сегодня", "language:ru"],
  ["आज कौन सा दिन है?", "आज", "language:hi"],
  ["今天是星期几?", "今天", "language:zh"],
]) {
  const r = calendar(prompt);
  check(`current-day "${prompt}" routes to calendar_current_day`, r && r.intent === "calendar_current_day", r && r.intent);
  check(`current-day "${prompt}" is localized`, r && r.content.includes(fragment), r && r.content);
  check(`current-day "${prompt}" records calendar:today`, r && hasEvidencePrefix(r, "calendar:today"), r && JSON.stringify(r.evidence));
  check(`current-day "${prompt}" records calendar:weekday`, r && hasEvidencePrefix(r, "calendar:weekday"), r && JSON.stringify(r.evidence));
  check(`current-day "${prompt}" records ${langTag}`, r && Array.isArray(r.evidence) && r.evidence.includes(langTag), r && JSON.stringify(r.evidence));
}

// --- 3: predecessor / successor phrasing variations (en + ru) ----------------
console.log("\n=== predecessor / successor variations ===");
for (const [prompt, expected] of [
  ["What day of the week comes after Tuesday?", "wednesday"],
  ["What day comes before Monday?", "sunday"],
  ["какой день недели перед средой", "вторник"],
  ["следующий день после воскресенья", "понедельник"],
]) {
  const r = calendar(prompt);
  check(`"${prompt}" routes to calendar_weekday_relation`, r && r.intent === "calendar_weekday_relation", r && r.intent);
  check(`"${prompt}" mentions ${expected}`, r && r.content.toLowerCase().includes(expected), r && r.content);
}

// --- 4: weekday relations in Hindi and Chinese (issue #386 broadening) -------
console.log("\n=== weekday relations in hindi and chinese ===");
for (const [prompt, expected] of [
  ["सोमवार के बाद कौन सा दिन आता है", "मंगलवार"], // after Monday → Tuesday
  ["सोमवार से पहले कौन सा दिन आता है", "रविवार"], // before Monday → Sunday
  ["星期一之后是星期几", "星期二"],               // after Monday → Tuesday
  ["星期三之前是星期几", "星期二"],               // before Wednesday → Tuesday
]) {
  const r = calendar(prompt);
  check(`"${prompt}" routes to calendar_weekday_relation`, r && r.intent === "calendar_weekday_relation", r && r.intent);
  check(`"${prompt}" mentions ${expected} (localized)`, r && r.content.includes(expected), r && r.content);
}

console.log("");
if (fail.length) {
  console.error(`FAILED ${fail.length} check(s):\n  ${fail.join("\n  ")}`);
  process.exit(1);
}
console.log("ALL CHECKS PASSED");
