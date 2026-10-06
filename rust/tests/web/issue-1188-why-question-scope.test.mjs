// PR #1188: a fronted rationale lead ("why …", "почему …") only claims the
// meta_explanation lane when the question addresses the assistant itself
// (an assistant_self_reference surface); a why-question about the user's
// code belongs to code_debugging. Mirrors addresses_assistant in
// rust/src/solver_handlers/meta_explanation.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("why-questions about the assistant stay meta explanations", async () => {
  for (const prompt of ["Why did you do that?", "why did you answer that?", "Почему ты так ответил?", "你为什么这样回答?"]) {
    assert.ok(worker.tryMetaExplanation(prompt), prompt);
    assert.equal((await solve(prompt)).intent, "meta_explanation", prompt);
  }
});

test("a why-question about the user's code is not a meta explanation", async () => {
  const prompt = "Why does this fail: def f(x): return x +";
  assert.equal(worker.tryMetaExplanation(prompt), null);
  assert.equal(worker.tryMetaExplanation("Why is the sky blue?"), null);
  assert.equal((await solve(prompt)).intent, "code_debugging");
});
