// Issue #1188 JS parity for product search (issues #800 and #872): the browser
// worker composes the same site-scoped search the native handler does, for the
// prompts rust/tests/unit/issue_800_product_search.rs and
// issue_872_app_store_search.rs pin.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function handled(prompt) {
  await seeded;
  const literal = JSON.stringify(prompt);
  return plain(evaluate(worker, `tryProductSearch(${literal}, normalizePrompt(${literal}))`));
}

const AMAZON = "Найди мне зарядку для ноутбука Acer Aspire 3 A325-45 на amazon.in";
const APP_STORE =
  "игры для малышей с открытым исходным кодом (абсолютно и полностью бесплатные) в App Store (iOS)";

test("the reported amazon.in prompt composes the search through the worker", async () => {
  await seeded;
  const answer = await worker.solve(AMAZON, [], {}, {}, [], {});
  assert.equal(answer.intent, "product_search");
  assert.ok(answer.content.includes("amazon.in/s?k="), answer.content);
  assert.ok(answer.content.includes("A325-45"), answer.content);
  assert.ok(answer.content.includes("not fetched"), answer.content);
  assert.ok(answer.content.includes("wattage"), answer.content);
});

test("a shopping request without a marketplace is not claimed", async () => {
  assert.equal(await handled("Найди мне хороший подарок сестре"), null);
});

test("a non-shopping prompt is not claimed", async () => {
  assert.equal(await handled("What is the variance of 2, 4, 4, 4, 5"), null);
});

test("the reported App Store prompt composes the search", async () => {
  const answer = await handled(APP_STORE);
  assert.ok(answer, "the bare noun phrase with a marketplace must be handled");
  assert.ok(answer.content.includes("apple.com/us/search/"), answer.content);
  assert.ok(answer.content.includes("%D0%B8%D0%B3%D1%80%D1%8B"), answer.content);
});

test("both qualifiers survive as stated constraints", async () => {
  const answer = await handled(APP_STORE);
  assert.ok(answer.content.includes("fully_free") && answer.content.includes("open_source"), answer.content);
  assert.ok(answer.content.includes("in-app purchases") || answer.content.includes("source repository"));
});

test("the toddler advice names age band and permissions", async () => {
  const answer = await handled(APP_STORE);
  assert.ok(answer.content.includes("age band") && answer.content.includes("permissions"), answer.content);
});

test("a bare noun phrase without a marketplace still declines", async () => {
  assert.equal(await handled("игры для малышей с открытым исходным кодом"), null);
});
