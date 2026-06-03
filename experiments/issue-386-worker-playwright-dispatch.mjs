// Issue #386 / Issue #135 regression repro: the browser worker must route the
// reported Russian prompt "Можешь написать мне Playright скрипт?" to the
// Playwright starter-script handler, NOT to the generic write_program handler.
//
// The Issue #386 generalization of writeProgramParameters (program_kind +
// program_request) made "написать … скрипт" look like a bare write_program
// request, and because tryWriteProgram was dispatched BEFORE tryPlaywrightScript
// in the worker (the reverse of the canonical Rust order, where
// try_playwright_script runs ahead of the SPECIALIZED_HANDLERS group), the
// generic handler shadowed the specific one and returned the "unsupported /
// language missing / task missing" clarification.
//
// This harness loads the working-tree worker into a vm sandbox and calls the
// full async solve() dispatch on the reported prompt plus a couple of controls,
// asserting the Playwright handler wins. Run:
//   node experiments/issue-386-worker-playwright-dispatch.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

function loadWorker(source, label) {
  const sandbox = {
    self: { location: { search: "" } },
    importScripts: () => {
      throw new Error("no importScripts in node harness");
    },
    postMessage: () => {},
    console,
    TextEncoder,
    TextDecoder,
    WebAssembly,
    fetch: () => Promise.reject(new Error("offline")),
    setTimeout,
    clearTimeout,
  };
  sandbox.globalThis = sandbox;
  vm.createContext(sandbox);
  vm.runInContext(source, sandbox, { filename: `formal_ai_worker.js (${label})` });
  return sandbox;
}

const source = readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");
const worker = loadWorker(source, "working tree");
if (typeof worker.solve !== "function") throw new Error("worker missing solve()");

const CASES = [
  {
    prompt: "Можешь написать мне Playright скрипт?",
    wantIntent: "playwright_script",
    mustContain: ["Playwright", "@playwright/test", "https://playwright.dev/docs/writing-tests"],
  },
  {
    prompt: "Can you write me a Playwright script?",
    wantIntent: "playwright_script",
    mustContain: ["Playwright", "@playwright/test"],
  },
  {
    // control: a genuine bare write-program request stays write_program(_unsupported)
    prompt: "напиши программу",
    wantIntentOneOf: ["write_program", "write_program_unsupported"],
  },
];

let failures = 0;
for (const c of CASES) {
  // eslint-disable-next-line no-await-in-loop
  const result = await worker.solve(c.prompt, [], {}, {});
  const intent = result && result.intent;
  const content = (result && result.content) || "";
  const okIntent = c.wantIntent
    ? intent === c.wantIntent
    : c.wantIntentOneOf.includes(intent);
  const missing = (c.mustContain || []).filter((s) => !content.includes(s));
  if (!okIntent || missing.length) {
    failures += 1;
    console.error(`FAIL  "${c.prompt}"`);
    console.error(`      intent=${JSON.stringify(intent)} want=${JSON.stringify(c.wantIntent || c.wantIntentOneOf)}`);
    if (missing.length) console.error(`      missing substrings: ${JSON.stringify(missing)}`);
    console.error(`      content: ${JSON.stringify(content.slice(0, 160))}`);
  } else {
    console.log(`ok    "${c.prompt}" -> ${intent}`);
  }
}

if (failures) {
  console.error(`\n${failures}/${CASES.length} dispatch case(s) FAILED`);
  process.exit(1);
}
console.log(`\nall ${CASES.length} worker playwright-dispatch cases passed`);
