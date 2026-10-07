// Issue #1176 R2: word problems formalized as quantity relations in the
// browser worker (JavaScript first). The stated numbers are joined by the
// seeded gain, loss, group and share relation words of
// data/seed/meanings-statistics.lino, and the derivation is shown.

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
    return { ok: false, status: 404, text: async () => "" };
  },
});
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return plain(await worker.solve(prompt, [], {}, {}, [], {}));
}

async function relation(prompt, language) {
  await ready;
  return plain(await evaluate(
    worker,
    `tryRelationWordProblem(${JSON.stringify(prompt)}, ${JSON.stringify(prompt.toLowerCase())}, ${JSON.stringify(language)})`,
  ));
}

for (const [prompt, derivation] of [
  ["Tom has 5 apples and buys 3 more. How many apples does he have?", "5 + 3 = 8"],
  ["Anna had 20 dollars and spent 7 dollars. How much money does she have left?", "20 - 7 = 13"],
  ["A box holds 6 eggs. How many eggs are in 4 boxes?", "6 × 4 = 24"],
  ["There are 24 cookies shared equally among 6 children. How many cookies does each child get?", "24 ÷ 6 = 4"],
  ["Lena had 12 stickers, got 5 more and gave away 4. How many stickers does she have now?", "12 + 5 - 4 = 13"],
]) {
  test(`#1176 R2: ${derivation} is derived from the relation words`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "word_problem_relation", JSON.stringify(answer));
    assert.ok(String(answer.content).includes(derivation), JSON.stringify(answer));
    assert.ok(answer.evidence.some((line) => String(line).startsWith("word_problem:relation:")), JSON.stringify(answer));
  });
}

test("#1176 R2: the relation words are seed data in every language (Russian loss)", async () => {
  const answer = await relation("У Маши было 10 яблок, она съела 3. Сколько яблок осталось?", "ru");
  assert.equal(answer.intent, "word_problem_relation", JSON.stringify(answer));
  assert.ok(String(answer.content).includes("10 - 3 = 7"), JSON.stringify(answer));
});

test("#1176 R2: numbers no relation word joins are declined, not guessed", async () => {
  assert.equal(await relation("I have 5 apples and 3 pears. How many fruits?", "en"), null);
  assert.equal(await relation("Tom has 5 apples. How many apples does he have?", "en"), null);
});

test("#1176 R2: the price-times-count pattern keeps its own answer", async () => {
  const answer = await solve("Pens cost 4 dollars each. Sam buys 3 pens and pays with 20 dollars. How much change does he get?");
  assert.equal(answer.intent, "word_problem_change", JSON.stringify(answer));
});
