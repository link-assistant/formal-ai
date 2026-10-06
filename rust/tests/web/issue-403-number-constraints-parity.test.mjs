// Issues #403 and #890 browser parity: hidden-number interval riddles.
//
// The worker twin (js/worker/formal_ai_worker_number_constraints.js) of
// rust/src/number_constraints.rs must formalize the bounds, render the
// canonical proof statement from data/seed/proof-program-templates.lino and
// discharge the interval through the linear decision procedure exactly as
// the native presenter does. Prompts come from rust/tests/unit/issue_403.rs
// and rust/tests/unit/issue_890.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("a Russian interval riddle returns formal reasoning, not the unknown fallback", async () => {
  const answer = await solve("Я загадал число больше 1 но меньше 3. что это за число?");
  assert.equal(answer.intent, "number_constraint_reasoning");
  assert.equal(
    answer.content,
    [
      "Если это задача про целое число, единственный ответ: 2.",
      "",
      "Формализация над целыми: x in Z, x > 1, x < 3. Проверяемая форма для решателя: `x > 1 and x < 3 is satisfiable over integers`.",
      "",
      "Если разрешены вещественные числа, ответ не единственный: например, x = 1.5 тоже подходит.",
      "",
      "Формальная проверка relative-meta-logic / SMT:",
      "Как я понял запрос: трактуем запрос как формальное утверждение «x > 1 and x < 3 is satisfiable» и доказываем методом «процедура разрешения relative-meta-logic / SMT» в relative-meta-logic.",
      "",
      "Доказательство (метод: процедура разрешения relative-meta-logic / SMT).",
      "",
      "Утверждение: x > 1 and x < 3 is satisfiable",
      "",
      "1. Определение: Delegate the normalized claim to the relative-meta-logic / SMT decision procedure for quantifier-free linear real arithmetic.",
      "2. Определение: Constraints: x > 1 and x < 3.",
      "3. Вывод: The constraints reduce to x: > 1 and < 3.",
      "4. Вывод: Witness found: x = 2.",
      "Therefore the constraint system is satisfiable. ∎",
    ].join("\n"),
  );
  assert.ok(!answer.content.includes("не удаётся сопоставить"));
  assert.ok(answer.content.includes("x > 1") && answer.content.includes("x < 3"));
  assert.ok(answer.content.includes("цел") && answer.content.includes("веществен"));
  assert.ok(answer.content.includes("relative-meta-logic"));
});

test("the solved statement is the seed's canonical proof statement", async () => {
  const answer = await solve("Я загадал число больше 1 но меньше 3. что это за число?");
  assert.equal(answer.content.split("`")[1], "x > 1 and x < 3 is satisfiable over integers");
  assert.ok(
    answer.evidence.includes("formalization:linear_constraint:x > 1 and x < 3 is satisfiable over integers"),
  );
});

test("an exclusive bound at i64::MAX has no solution and says so", async () => {
  const answer = await solve(
    "I chose a number greater than 9223372036854775807 and less than or equal to 9223372036854775807. What is the number?",
  );
  assert.equal(answer.intent, "number_constraint_reasoning");
  assert.ok(answer.content.includes("there is no solution"), answer.content);
  assert.ok(
    answer.content.includes(
      "x > 9223372036854775807 and x <= 9223372036854775807 is unsatisfiable over integers",
    ),
    answer.content,
  );
  // The disproof asks one follow-up question; the question budget of
  // rust/src/question_necessity.rs drops the second and compacts the body.
  assert.ok(answer.content.endsWith(
    "Clarifying questions:\n1. do you want to weaken the claim into a checkable form (e.g. replace equality with an inequality, or restrict the domain)?\n2.",
  ), answer.content);
  assert.ok(!answer.content.includes("\n\n\n"));
});
