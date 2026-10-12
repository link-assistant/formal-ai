#!/usr/bin/env node
// Ask the JavaScript engine (the browser worker, booted in node) one prompt
// and print its answer, its reasoning steps and its derivation. This is how a
// task is put to Formal AI itself without compiling the crate (2026-10-06
// doctrine: JavaScript first, Rust in CI).
//
// Sources: dictionary lookups replay the pre-cached captures under
// data/cache/wiktionary/ and rust/tests/fixtures/meta-reasoner/captures/;
// with --online a miss is fetched live (and not written anywhere).
//
// Usage: node scripts/formal-ai-js.mjs [--trace] [--json] [--online] "<prompt>" ["<prompt>" ...]
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

import { createWorkerContext, evaluate } from "../rust/tests/web/support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "..");
const DICTIONARY = /^https:\/\/api\.dictionaryapi\.dev\/api\/v2\/entries\/([a-z]+)\/([^/?#]+)$/u;

/** A web-root asset, a cached dictionary capture, or (with --online) the network. */
async function replayFetch(url, online) {
  const target = String(url);
  const respond = (text) => ({ ok: true, status: 200, text: async () => text });
  const missing = { ok: false, status: 404, text: async () => "" };
  if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
    const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
    const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
    return existsSync(onDisk) ? respond(readFileSync(onDisk, "utf8")) : missing;
  }
  const dictionary = DICTIONARY.exec(target);
  if (dictionary) {
    const word = decodeURIComponent(dictionary[2]).toLowerCase();
    for (const candidate of [
      path.join(REPO_ROOT, "data/cache/wiktionary", dictionary[1], `${word}.json`),
      path.join(REPO_ROOT, "rust/tests/fixtures/meta-reasoner/captures", `${word}.json`),
    ]) {
      if (existsSync(candidate)) return respond(readFileSync(candidate, "utf8"));
    }
  }
  return online ? fetch(target) : missing;
}

const args = process.argv.slice(2);
const trace = args.includes("--trace");
const json = args.includes("--json");
const online = args.includes("--online");
const prompts = args.filter((arg) => !arg.startsWith("--"));
if (prompts.length === 0) {
  console.error('usage: node scripts/formal-ai-js.mjs [--trace] [--json] "<prompt>"');
  process.exit(2);
}
const worker = createWorkerContext({ fetch: (url) => replayFetch(url, online) });
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
