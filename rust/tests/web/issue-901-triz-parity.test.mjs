// Issue #901: the TRIZ/contradiction row answers in the browser from
// data/seed/triz-principles.lino exactly as rust/src/triz_solver.rs does.
// Prompts and expectations mirror rust/tests/unit/issue_901_triz_solver.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function handled(prompt) {
  await ready;
  return evaluate(worker, `tryTrizResolution(${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`);
}

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("the seed carries twelve families and twenty benchmark tasks", async () => {
  await ready;
  assert.ok(evaluate(worker, "trizFamilies().length") >= 12);
  assert.equal(evaluate(worker, "trizBenchmarkTasks().length"), 20);
  assert.ok(evaluate(worker, 'trizFamilies().some((family) => family.methodId === "family_range_selection")'));
});

test("a Russian contradiction prompt gets the family map", async () => {
  const answer = await solve("Как разрешить противоречие: деталь должна быть жёсткой и одновременно гибкой?");
  assert.equal(answer.intent, "triz_resolution");
  assert.ok(answer.content.includes("Range selection") && answer.content.includes("Dimension change"), answer.content);
  assert.ok(answer.content.includes("no default 50 %"), answer.content);
  assert.ok(answer.content.includes("methods: family_"), answer.content);
});

test("an umbrella prompt finds the umbrella precedent", async () => {
  const answer = await solve(
    "Inventive problem: an umbrella must be big enough in rain yet small enough in a crowded bus.",
  );
  assert.equal(answer.intent, "triz_resolution");
  assert.ok(answer.content.includes("umbrella") && answer.content.includes("crowded bus"), answer.content);
  assert.ok(answer.content.includes("family_space_separation"), answer.content);
});

test("the TRIZ acronym and English paradox cues trigger", async () => {
  assert.ok(await handled("Apply TRIZ to smartwatch battery vs thickness"));
  assert.ok(await handled("How do we resolve this paradox: transparency vs strength?"));
});

test("ordinary prompts decline", async () => {
  assert.equal(await handled("What is the capital of France?"), null);
  assert.equal(await handled("Напомни мне встречу в 20:00"), null);
});

test("the handler logs its cue and precedents", async () => {
  const answer = await handled("Engineering contradiction: coat tiny pills evenly without waste");
  assert.ok(answer.evidence.includes("triz_solver:cued:contradiction question"));
  assert.ok(answer.evidence.some((link) => link.startsWith("triz_solver:precedent:")));
  assert.ok(answer.content.includes("pill"), answer.content);
});
