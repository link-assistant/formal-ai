// Tasks put to Formal AI itself while building it (R1017).
//
// A task it could not derive is made to work by a general mechanism in the
// seed data, never a rule written for that one request. "Reverse the order of
// words" failed while "reverse the words" worked: the English reverse_words row
// had literal phrases only and gained the "order of" phrasings.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const REPO_ROOT = path.resolve(import.meta.dirname, "../../..");

const worker = createWorkerContext({
  fetch: async (url) => {
    const target = String(url);
    if (!target.startsWith("http") || target.startsWith("http://localhost/")) {
      const relative = new URL(target, "http://localhost/").pathname.replace(/^\/+/u, "");
      const onDisk = relative.startsWith("seed/") ? path.join(REPO_ROOT, "data", relative) : path.join(REPO_ROOT, "js", relative);
      try {
        const text = readFileSync(onDisk, "utf8");
        return { ok: true, status: 200, text: async () => text };
      } catch {
        return { ok: false, status: 404, text: async () => "" };
      }
    }
    // Offline: every external provider is unreachable.
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}

for (const prompt of [
  "Reverse the order of words: 'green CI release'",
  "Reverse the order of the words in 'green CI release'",
  "Reverse words: 'green CI release'",
]) {
  test(`R1017: a reworded reverse-words request derives (${prompt})`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "text_manipulation", JSON.stringify(answer));
    assert.match(String(answer.content), /release CI green/u, JSON.stringify(answer));
  });
}

// A chain runs in the order the request names its steps, whatever order the
// vocabulary declares the operations in; a count still ends the chain.
test("R1017: text operations chain in the order the request states them", async () => {
  const sortedFirst = await solve("Sort words and then reverse words: 'b a c'");
  assert.match(String(sortedFirst.content), /^c b a$/u, JSON.stringify(sortedFirst));
  const reversedFirst = await solve("Reverse words and then sort words: 'b a c'");
  assert.match(String(reversedFirst.content), /^a b c$/u, JSON.stringify(reversedFirst));
});

// Prose names no function: "a function that reverses" has no signature, so
// `that` is never taken as the function's name (Rust's parse_signature).
test("R1017: a function described in prose is not named after its relative pronoun", async () => {
  const answer = await solve("Write a Python function that reverses a string");
  assert.doesNotMatch(String(answer.content), /function `that`/u, JSON.stringify(answer));
  await ready;
  assert.equal(await evaluate(worker, `extractPythonFunctionName("Implement Python function count_vowels(text: str) -> int.")`), "count_vowels");
  assert.equal(await evaluate(worker, `extractPythonFunctionName("Write a Python function that reverses a string")`), "");
});
