// Counting tasks put to Formal AI itself while building it (R1017).
//
// "How many words are in '...'?" was answered "unknown": the count rows of
// data/seed/operation-vocabulary.lino only knew imperative phrasings. Three
// general mechanisms fix it, never a rule for one request: a seeded
// `counting_cue` block plus per-operation `unit` nouns, asked of the
// request's framing outside its quoted payload (so payload words never name
// an operation); a `filter_lines_by_prefix` row whose phrase carries its
// argument, which composes with the counts through the chain; and a seeded
// character class (`vowels` declares its `members`) that a count names.
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

for (const [prompt, expected] of [
  // A counting cue plus a unit noun in the framing asks for that count.
  ["How many words are in 'the quick brown fox'?", "4"],
  ["How many lines are in this text: 'a\nb'", "2"],
  ["How many characters are in 'abc'?", "3"],
  ["How many unique words are in 'a b a c'?", "3"],
  ["Сколько слов в 'раз два три'?", "3"],
  // Payload words are content: "words" inside the quote is not the unit.
  ["How many characters are in 'two words'?", "9"],
  // The line filter reads its argument, bare or quoted, after (en, ru) or
  // before (hi) its phrase, and a count named alongside ends the chain.
  ["Keep only the lines that start with x: 'x1\ny2\nx3'", "x1\nx3"],
  ["How many lines start with x in this text: 'x1\ny2\nx3'", "2"],
  ["Keep the lines starting with '#': '#a\nb\n#c'", "#a\n#c"],
  ["Сколько строк начинается с x в тексте: 'x1\ny2\nx3'", "2"],
  ["x से शुरू होने वाली पंक्तियाँ रखें: 'x1\ny2\nx3'", "x1\nx3"],
  // A seeded character class counted counts its members, any case.
  ["Count the vowels in 'formal'", "2"],
  ["How many vowels are in 'Formal Area'?", "5"],
]) {
  test(`R1017: ${JSON.stringify(prompt)} derives ${JSON.stringify(expected)}`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "text_manipulation", JSON.stringify(answer));
    assert.equal(String(answer.content), expected, JSON.stringify(answer));
  });
}

test("R1017: a counting question without a quoted payload is not a text count", async () => {
  const answer = await solve("How many words does the English language have?");
  assert.notEqual(answer.intent, "text_manipulation", JSON.stringify(answer));
});

test("R1017: 'many' without a counting cue still sorts", async () => {
  const answer = await solve("Sort these many words: pear apple");
  assert.equal(answer.intent, "text_manipulation", JSON.stringify(answer));
  assert.equal(String(answer.content), "apple pear", JSON.stringify(answer));
});

test("R1017: format conversion keeps payload keys that name count units", async () => {
  const answer = await solve('Convert this JSON to YAML: \'{"counts": 1, "lines": 2}\'');
  assert.equal(answer.intent, "format_conversion", JSON.stringify(answer));
});
