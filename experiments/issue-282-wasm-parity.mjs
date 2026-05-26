// Smoke test: Issue #282 Rust/WASM browser-worker parity helpers.
//
// Run with:
//   src/web/wasm-worker/build.sh
//   node experiments/issue-282-wasm-parity.mjs

import { readFile } from "node:fs/promises";

const bytes = await readFile(new URL("../src/web/formal_ai_worker.wasm", import.meta.url));
const module = await WebAssembly.instantiate(bytes, {});
const wasm = module.instance.exports;

const enc = new TextEncoder();
const dec = new TextDecoder();

function writeInput(text) {
  const data = enc.encode(text);
  const view = new Uint8Array(wasm.memory.buffer, wasm.input_ptr(), data.length);
  view.set(data);
  return data.length;
}

function readOutput(length) {
  if (length === 0) return "";
  const view = new Uint8Array(wasm.memory.buffer, wasm.output_ptr(), length);
  return dec.decode(view);
}

let failures = 0;
function check(label, actual, expected) {
  const ok = JSON.stringify(actual) === JSON.stringify(expected);
  if (!ok) {
    failures += 1;
    console.error(
      `FAIL ${label}\n  expected: ${JSON.stringify(expected)}\n  actual:   ${JSON.stringify(actual)}`,
    );
  } else {
    console.log(`ok   ${label}`);
  }
}

const stableIdLength = wasm.engine_stable_id(
  writeInput("unknown_opener\nневедомослово"),
);
check(
  "stable id hashes UTF-8 bytes",
  readOutput(stableIdLength),
  "unknown_opener_3f0af77ee5085861",
);

const openerCases = [
  ["en", "blorfblarf", "I'm not sure how to respond to that yet."],
  ["ru", "неведомослово", "Я ещё не научился отвечать на это."],
  ["hi", "अज्ञातशब्द", "मैं समझ नहीं पाया।"],
  ["zh", "未知词", "我不太明白你说的意思。"],
];
for (const [language, prompt, expected] of openerCases) {
  const openerLength = wasm.engine_select_unknown_opener(
    writeInput(`${language}\n${prompt}`),
  );
  check(
    `${language} unknown opener matches native Rust`,
    readOutput(openerLength),
    expected,
  );
}

const routePayload = [
  "who are you",
  "Who are you?",
  "P\twho are you",
  "C\twho\tyou",
].join("\n");
check(
  "intent route matcher handles phrases and combos",
  wasm.engine_match_intent_route(writeInput(routePayload)),
  1,
);

const missPayload = ["what is wikipedia", "What is Wikipedia?", "K\thello"].join("\n");
check(
  "intent route matcher rejects misses",
  wasm.engine_match_intent_route(writeInput(missPayload)),
  0,
);

if (failures > 0) {
  console.error(`\n${failures} failure(s).`);
  process.exit(1);
}
console.log("\nIssue #282 WASM parity checks passed.");
