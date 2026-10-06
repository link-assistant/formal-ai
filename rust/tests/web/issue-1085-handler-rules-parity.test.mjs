// Issue #1085 D1.3: the rule-backed precedence rows (clarification,
// punctuation_only_prompt, ill_formed, physical_action_question,
// opinion_question) answer in the browser through the same
// data/seed/handler-rules.lino the native rule interpreter walks. Prompts and
// expected wording are the ones rust/tests/unit/issue_1085_rule_interpreter.rs
// asserts against the native engine.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("english-only policy rules keep their intents and wording", async () => {
  const opinion = await solve("What do you think about pineapple on pizza?");
  assert.equal(opinion.intent, "opinion_question");
  assert.equal(
    opinion.content,
    "I am a deterministic symbolic AI. I do not hold opinions, beliefs, or feelings — every answer I give is derived from an explicit Links Notation rule. If you are looking for factual information on this topic, try asking \"what is <topic>\" and I will look it up in my knowledge base.",
  );
  const punctuation = await solve("???");
  assert.equal(punctuation.intent, "clarification");
  assert.equal(
    punctuation.content,
    "I received only punctuation (`???`). What would you like me to do next?",
  );
  const physical = await solve("So, did you suck at it yesterday?");
  assert.equal(physical.intent, "physical_action_question");
  assert.equal(physical.content, "No. I do not have a physical body.");
});

test("clarification requests are recognized in every seeded language", async () => {
  for (const prompt of [
    "Sorry, I don't understand this at all",
    "Я не понимаю тебя",
    "मुझे समझ नहीं आया",
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "clarification", `${prompt}: ${response.content}`);
    assert.ok(response.evidence.includes("handler:clarification"), prompt);
  }
});

test("an unbalanced teach-this-fact link is ill formed and answers unknown", async () => {
  await ready;
  const hit = evaluate(worker, 'tryIllFormed("teach this fact (a b", "teach this fact (a b")');
  assert.equal(hit.intent, "unknown");
  assert.equal(hit.confidence, 0);
  assert.ok(hit.evidence.includes("error:unbalanced links notation"));
  const balanced = evaluate(worker, 'tryIllFormed("teach this fact (a b)", "teach this fact (a b)")');
  assert.equal(balanced, null);
});

test("an identity question is not swallowed by the opinion row", async () => {
  const response = await solve("Who are you?");
  assert.equal(response.intent, "identity");
});

test("a prompt without the role surface falls through every rule row", async () => {
  await ready;
  for (const name of ["tryClarification", "tryPhysicalActionQuestion", "tryOpinionQuestion"]) {
    assert.equal(
      evaluate(worker, `${name}("What is the capital of France?", "what is the capital of france?")`),
      null,
      name,
    );
  }
});
