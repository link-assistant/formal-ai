#!/usr/bin/env node
// Ask the JavaScript engine (the browser worker, booted in node) one prompt
// and print its answer, its reasoning steps and its derivation. This is how a
// task is put to Formal AI itself without compiling the crate (2026-10-06
// doctrine: JavaScript first, Rust in CI).
//
// Usage: node scripts/formal-ai-js.mjs [--trace] [--json] "<prompt>" ["<prompt>" ...]
import { createWorkerContext, evaluate } from "../rust/tests/web/support/browser-runtime.mjs";

const args = process.argv.slice(2);
const trace = args.includes("--trace");
const json = args.includes("--json");
const prompts = args.filter((arg) => !arg.startsWith("--"));
if (prompts.length === 0) {
  console.error('usage: node scripts/formal-ai-js.mjs [--trace] [--json] "<prompt>"');
  process.exit(2);
}
const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");
for (const prompt of prompts) {
  const answer = await worker.solve(prompt, [], {}, {}, [], {});
  if (json) {
    console.log(JSON.stringify(answer, null, 2));
    continue;
  }
  console.log(`# ${prompt}`);
  console.log(`intent: ${answer.intent}  confidence: ${answer.confidence}`);
  console.log(answer.content);
  if (trace) {
    console.log("\n## steps");
    for (const step of answer.steps || []) console.log(`- ${step.step}: ${String(step.detail || "").slice(0, 160)}`);
    if (answer.derivation) console.log(`\n## derivation\n${answer.derivation}`);
  }
  console.log("");
}
