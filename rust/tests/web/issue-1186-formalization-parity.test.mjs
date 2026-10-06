// Issue #1188 JS parity for the issue #1186 user-facing formalization task:
// the browser worker parses the quantified clause, renders every seeded target
// grammar with the named one first, and deformalizes with a structural round
// trip, for the prompts rust/tests/unit/issue_1186_formalization_task.rs pins.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function handlerAnswer(prompt) {
  await seeded;
  const literal = JSON.stringify(prompt);
  const answer = plain(evaluate(worker, `tryFormalizationRequest(${literal}, normalizePrompt(${literal}))`));
  assert.ok(answer, `the formalization handler should answer: ${prompt}`);
  return answer.content;
}

function fenced(answer, tag) {
  const open = "```" + tag + "\n";
  const start = answer.indexOf(open);
  assert.ok(start !== -1, `answer should carry a \`${tag}\` fence: ${answer}`);
  const from = start + open.length;
  const end = answer.indexOf("```", from);
  assert.ok(end !== -1, `the \`${tag}\` fence should be closed: ${answer}`);
  return answer.slice(from, end);
}

test("the issue probe sentence formalizes into every target", async () => {
  const answer = await handlerAnswer(
    "Formalize in first-order logic: Every student who studies passes the exam.",
  );
  assert.ok(fenced(answer, "fol").startsWith("∀x (Student(x) ∧ Studies(x) → Passes(x"), answer);
  const lean = fenced(answer, "lean");
  assert.ok(lean.includes("∀ (x : U)") && lean.includes("Student x"), answer);
  assert.ok(fenced(answer, "rocq").includes("forall (x : U)"), answer);
  const lino = fenced(answer, "lino");
  assert.ok(lino.includes("formal_clause") && lino.includes("predicate Student"), answer);
  assert.ok(answer.includes("No theorem prover was invoked"), answer);
  assert.ok(answer.includes("Derivation:"), answer);
});

test("an objectless conditional formalizes exactly", async () => {
  const answer = await handlerAnswer("Formalize in FOL: Every student who studies passes");
  assert.equal(fenced(answer, "fol"), "∀x (Student(x) ∧ Studies(x) → Passes(x))\n");
});

test("a Russian conditional formalizes in Russian", async () => {
  const answer = await handlerAnswer(
    "Формализуй в логике первого порядка: Каждый студент, который учится, сдаёт экзамен",
  );
  assert.ok(fenced(answer, "fol").startsWith("∀x (студент(x) ∧ учится(x) → сдаёт(x"), answer);
  assert.ok(answer.includes("Формализованное утверждение:"), answer);
});

test("Hindi and Chinese conditionals formalize", async () => {
  const hindi = await handlerAnswer("औपचारिक बनाओ: हर छात्र जो पढ़ता है, परीक्षा पास करता है");
  assert.ok(fenced(hindi, "fol").startsWith("∀x (छात्र(x) ∧ पढ़ता(x) → "), hindi);
  const chinese = await handlerAnswer("形式化：每个学习的学生都通过考试");
  assert.ok(fenced(chinese, "fol").startsWith("∀x (学生(x) ∧ 学习(x) → "), chinese);
});

test("existential and negative readings are conjunctive", async () => {
  const some = await handlerAnswer("Formalize in first-order logic: Some bird that sings flies");
  assert.equal(fenced(some, "fol"), "∃x (Bird(x) ∧ Sings(x) ∧ Flies(x))\n");
  const none = await handlerAnswer("Formalize in FOL: No cat that sleeps hunts");
  assert.equal(fenced(none, "fol"), "¬∃x (Cat(x) ∧ Sleeps(x) ∧ Hunts(x))\n");
});

test("the named target renders first", async () => {
  const answer = await handlerAnswer("Formalize in Lean: Every student who studies passes");
  assert.ok(answer.indexOf("```lean") !== -1, answer);
  assert.ok(answer.indexOf("```lean") < answer.indexOf("```fol"), answer);
});

test("the deformalization round trip preserves structure", async () => {
  const fol = "∀x (Student(x) ∧ Studies(x) → Passes(x, exam))";
  const back = await handlerAnswer(`Deformalize in plain English: ${fol}`);
  assert.ok(back.includes("every student that studies passes the exam"), back);
  assert.ok(back.includes("structure preserved"), back);
  const again = await handlerAnswer(
    "Formalize in first-order logic: every student that studies passes the exam",
  );
  assert.equal(fenced(again, "fol"), `${fol}\n`);
});

test("deformalization localizes the wrapper prose", async () => {
  const answer = await handlerAnswer("Деформализуй: ∀x (Student(x) ∧ Studies(x) → Passes(x))");
  assert.ok(answer.includes("Прочтение на естественном языке"), answer);
  assert.ok(answer.includes("каждый") && answer.includes("который"), answer);
  assert.ok(answer.includes("studies"), answer);
});

test("an unparseable sentence gets an honest refusal", async () => {
  const answer = await handlerAnswer("Formalize in first-order logic: hello world");
  assert.ok(answer.includes("quantified clause"), answer);
  assert.ok(!answer.includes("Web search requested"), answer);
});

test("unrelated prompts are not claimed", async () => {
  await seeded;
  for (const prompt of [
    "Hello, how are you today?",
    "What is the capital of France?",
    "Write a regular expression that matches five digits",
  ]) {
    const literal = JSON.stringify(prompt);
    assert.equal(evaluate(worker, `tryFormalizationRequest(${literal}, normalizePrompt(${literal}))`), null);
  }
});

test("the worker answers the cued formalization request", async () => {
  await seeded;
  const answer = await worker.solve(
    "Formalize in first-order logic: Every student who studies passes the exam.",
    [], {}, {}, [], {},
  );
  assert.equal(answer.intent, "formalization");
  assert.ok(answer.content.includes("∀x"), answer.content);
  assert.ok(!answer.content.includes("Web search requested"), answer.content);
});
