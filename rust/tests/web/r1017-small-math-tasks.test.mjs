// Small computed tasks put to Formal AI itself while building it (R1017).
//
// Each answer is computed, never looked up or answered with an algorithm:
// the weekday of a stated date (calendar_reasoning, counting days from
// 1970-01-01), the primality of one stated number (trial division, with a
// factor for a composite), a spelled function application ("the square root
// of 144" read as sqrt(144)) and a list transformation asked without code.
// The cue words live in the seed (calendar_weekday_query,
// number_property_prime, math_function_argument_marker, the operation
// vocabulary) and the prose in data/seed/multilingual-responses-quantities.lino.
// Twins: rust/tests/unit/r1017_small_math_tasks.rs.
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

for (const [prompt, weekday, date] of [
  ["What day of the week was 2024-02-29?", "Thursday", "2024-02-29"],
  ["What weekday is 2025-12-25?", "Thursday", "2025-12-25"],
  ["What day of the week was 1 January 2000?", "Saturday", "2000-01-01"],
  ["Which day of the week is July 4, 1776?", "Thursday", "1776-07-04"],
  ["What day of the week was 1969-12-31?", "Wednesday", "1969-12-31"],
]) {
  test(`R1017: the weekday of a stated date is computed (${prompt})`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "calendar_date_weekday", JSON.stringify(answer));
    assert.ok(String(answer.content).startsWith(`${date} is a ${weekday}.`), JSON.stringify(answer));
  });
}

test("R1017: the date weekday shows its day count from the epoch", async () => {
  const answer = await solve("What day of the week was 2024-02-29?");
  assert.equal(
    answer.content,
    "2024-02-29 is a Thursday. Counting from 1970-01-01, a Thursday, it is day 19782: 19782 = 7 × 2826 + 0, and Thursday + 0 days = Thursday.",
  );
  const russian = await solve("Какой день недели был 2024-02-29?");
  assert.ok(String(russian.content).startsWith("2024-02-29 — четверг."), JSON.stringify(russian));
});

test("R1017: an impossible date or an offset question is not read as a date weekday", async () => {
  for (const prompt of ["What day of the week is 2023-02-29?", "What day of the week is 3 days after 2024-02-29?"]) {
    const answer = await solve(prompt);
    assert.notEqual(answer.intent, "calendar_date_weekday", `${prompt}: ${JSON.stringify(answer)}`);
  }
});

test("R1017: primality of one stated number is decided by trial division", async () => {
  const prime = await solve("Is 97 a prime number?");
  assert.equal(prime.intent, "number_primality", JSON.stringify(prime));
  assert.equal(
    prime.content,
    "Yes, 97 is a prime number: no integer from 2 to ⌊√97⌋ = 9 divides it, so its only divisors are 1 and 97.",
  );
  const composite = await solve("Is 91 prime?");
  assert.equal(composite.intent, "number_primality", JSON.stringify(composite));
  assert.match(String(composite.content), /^No, 91 is not a prime number: 91 = 7 × 13/u);
  const one = await solve("Is 1 a prime number?");
  assert.match(String(one.content), /^No, 1 is not a prime number/u);
  const two = await solve("Is 2 prime?");
  assert.match(String(two.content), /^Yes, 2 is a prime number/u);
  const russian = await solve("Является ли 97 простым числом?");
  assert.match(String(russian.content), /^Да, 97 — простое число/u);
});

test("R1017: a prime question about a range or an ordinal is not read as one number", async () => {
  for (const prompt of ["What is the largest prime below 100?", "What is the 10th prime?", "Pick a prime number between 14 and 18"]) {
    const answer = await solve(prompt);
    assert.notEqual(answer.intent, "number_primality", `${prompt}: ${JSON.stringify(answer)}`);
  }
});

for (const [prompt, expression, value] of [
  ["What is the square root of 144?", "sqrt(144)", "12"],
  ["Calculate the square root of 81", "sqrt(81)", "9"],
  ["Сколько будет квадратный корень из 144?", "sqrt(144)", "12"],
  ["What is sqrt(144)?", "sqrt(144)", "12"],
  ["What is the square root of 2?", "sqrt(2)", "1.4142135624"],
]) {
  test(`R1017: a spelled function application is computed (${prompt})`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "calculation", JSON.stringify(answer));
    assert.equal(answer.content, `${expression} = ${value}`);
  });
}

test("R1017: a statement about a function value is not rewritten into a calculation", async () => {
  await ready;
  assert.equal(await evaluate(worker, `rewriteMathFunctionPhrases("the square root of 2 is irrational")`), null);
  assert.equal(await evaluate(worker, `rewriteMathFunctionPhrases("the square root of 144")`), "sqrt(144)");
  assert.equal(await evaluate(worker, `rewriteMathFunctionPhrases("2 * square root of 9")`), "2 * sqrt(9)");
});

test("R1017: a list transformation without code answers with the result", async () => {
  const ascending = await solve("Sort the numbers 5, 2, 9, 1");
  assert.equal(ascending.intent, "numeric_list_result", JSON.stringify(ascending));
  assert.equal(ascending.content, "5, 2, 9, 1 sorted in ascending order: 1, 2, 5, 9.");
  const descending = await solve("Sort the numbers 5, 2, 9, 1 in descending order");
  assert.equal(descending.content, "5, 2, 9, 1 sorted in descending order: 9, 5, 2, 1.");
  const russian = await solve("Отсортируй числа 5, 2, 9, 1");
  assert.equal(russian.content, "5, 2, 9, 1 по возрастанию: 1, 2, 5, 9.");
  const coded = await solve("Sort the numbers 5, 2, 9, 1 in JavaScript");
  assert.equal(coded.intent, "write_program", JSON.stringify(coded));
  const algorithm = await solve("Write me a sorting algorithm in Rust");
  assert.equal(algorithm.intent, "algorithm_sort_rust", JSON.stringify(algorithm));
});
