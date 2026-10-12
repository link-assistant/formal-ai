// Issue #1176 browser parity: statistics and price-times-count word problems.
//
// The worker twin of rust/src/solver_handlers/statistics.rs
// (js/worker/formal_ai_worker_statistics.js) must answer the same prompts
// with the same text as the native handler. Prompts and expected fragments
// are the ones rust/tests/unit/issue_1176_quantities_dates.rs pins.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

function citesResponse(answer, link) {
  return Array.isArray(answer.evidence) && answer.evidence.includes(link);
}

test("mean and median are computed exactly with the arithmetic shown", async () => {
  const answer = await solve(
    "Given the values 4, 8, 15, 16, 23, 42, what are the mean and the median?",
  );
  assert.equal(answer.intent, "statistics");
  assert.ok(citesResponse(answer, "response:statistics"), "cites response:statistics");
  assert.equal(
    answer.content,
    "The values are 4, 8, 15, 16, 23, 42 (n = 6).\n" +
      "mean: 18 (108 / 6 = 18)\n" +
      "median: 15.5 ((15 + 16) / 2 = 15.5)",
  );
});

test("mode and range read their names from the seed vocabulary", async () => {
  const answer = await solve("For the values 2, 3, 3, 5, what are the mode and the range?");
  assert.equal(answer.intent, "statistics");
  assert.ok(answer.content.includes("mode: 3"), answer.content);
  assert.ok(answer.content.includes("range: 3 (5 - 2 = 3)"), answer.content);
});

test("variance and standard deviation derive from exact fractions", async () => {
  const answer = await solve(
    "What are the variance and the standard deviation of 4, 8, 15, 16, 23, 42?",
  );
  assert.equal(answer.intent, "statistics");
  assert.ok(
    answer.content.includes("variance: ≈151.6666667 (910 / 6 ≈ 151.6666667)"),
    answer.content,
  );
  assert.ok(answer.content.includes("standard deviation: ≈12.3153021"), answer.content);
});

test("statistics labels follow the prompt language", async () => {
  const answer = await solve("Каково среднее значение чисел 4, 8, 15, 16, 23, 42?");
  assert.equal(answer.intent, "statistics");
  assert.ok(answer.content.includes("среднее: 18 (108 / 6 = 18)"), answer.content);
});

test("the change word problem shows the arithmetic", async () => {
  const answer = await solve(
    "I buy 4 pens at 3 dollars each and pay with a 20 dollar note. How much change do I get?",
  );
  assert.equal(answer.intent, "word_problem_change");
  assert.ok(citesResponse(answer, "response:word_problem_change"));
  assert.equal(answer.content, "The change is 8: 20 - 4 × 3 = 8.");
});

test("the total word problem multiplies price by count", async () => {
  const answer = await solve("I buy 4 pens at 3 dollars each. What is the total?");
  assert.equal(answer.intent, "word_problem_total");
  assert.equal(answer.content, "The total is 12: 4 × 3 = 12.");
});

test("a Spanish change word problem reads the seed markers", async () => {
  const answer = await solve(
    "Compra 4 bolígrafos a 3 dólares cada uno y paga con 20. ¿Cuánto cambio recibe?",
  );
  assert.equal(answer.intent, "word_problem_change", answer.content);
  assert.ok(answer.content.includes("20 - 4 × 3 = 8"), answer.content);
});

test("the percentile interpolates between closest ranks with the derivation shown", async () => {
  const answer = await solve("What is the 90th percentile of 1, 2, 3, 4, 5?");
  assert.equal(answer.intent, "statistics");
  assert.equal(
    answer.content,
    "The values are 1, 2, 3, 4, 5 (n = 5).\n" +
      "percentile: 4.6 (p = 90; rank = 1 + (5 - 1) × 90 / 100 = 4.6; 4 + 0.6 × (5 - 4) = 4.6)",
  );
});

test("the percentile rank leaves the dataset and joins other operations", async () => {
  const answer = await solve(
    "What are the mean and the 25th percentile of 4, 8, 15, 16, 23, 42?",
  );
  assert.equal(answer.intent, "statistics");
  assert.ok(answer.content.startsWith("The values are 4, 8, 15, 16, 23, 42 (n = 6)."), answer.content);
  assert.ok(answer.content.includes("mean: 18 (108 / 6 = 18)"), answer.content);
  assert.ok(
    answer.content.includes(
      "percentile: 9.75 (p = 25; rank = 1 + (6 - 1) × 25 / 100 = 2.25; 8 + 0.25 × (15 - 8) = 9.75)",
    ),
    answer.content,
  );
});

test("percentile labels and derivations follow the prompt language", async () => {
  const russian = await solve("Каков 90-й перцентиль чисел 1, 2, 3, 4, 5?");
  assert.ok(
    russian.content.includes("перцентиль: 4.6 (p = 90; ранг = 1 + (5 - 1) × 90 / 100 = 4.6;"),
    russian.content,
  );
  const spanish = await solve("¿Cuál es el percentil 90 de 1, 2, 3, 4, 5?");
  assert.ok(spanish.content.includes("percentil: 4.6 (p = 90; posición ="), spanish.content);
  const chinese = await solve("1、2、3、4、5的第90百分位数是多少？");
  assert.ok(chinese.content.includes("百分位数: 4.6 (p = 90；秩 ="), chinese.content);
});

test("a percentile without a stated or valid rank is declined", async () => {
  for (const prompt of [
    "What is the percentile of 1, 2, 3?",
    "What is the 150th percentile of 1, 2, 3?",
  ]) {
    const answer = await solve(prompt);
    assert.notEqual(answer.intent, "statistics", `${prompt} -> ${answer.content}`);
  }
});

test("questions without a quantity operation are not claimed", async () => {
  for (const prompt of [
    "What is the deeper meaning of joy?",
    "What is 2 + 2?",
    "Who wrote The Master and Margarita?",
  ]) {
    const answer = await solve(prompt);
    for (const intent of ["statistics", "unit_conversion", "word_problem_change"]) {
      assert.notEqual(answer.intent, intent, `${prompt} -> ${answer.content}`);
    }
  }
});
