// Browser parity for the remaining native-only coding and policy rows:
// `algorithm`, `execution_failure`, `document_generation_plan`,
// `source_conflict` and `shell_refusal`
// (js/worker/formal_ai_worker_code_plans.js).
//
// Prompts and expectations are the ones the Rust specs pin:
// rust/tests/unit/specification/chat_surface.rs (#258 sorting snippet),
// rust/tests/unit/specification/code_generation/single_turn.rs and
// agent_isolation.rs (execution failure), rust/tests/unit/issue_425.rs
// (document plan), rust/tests/unit/specification/source_cache.rs (source
// conflict) and rust/tests/unit/issue_1085_rule_interpreter.rs (shell refusal).

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("a sorting-algorithm request gets the reviewable snippet with its execution status", async () => {
  const answer = await solve("Write me a sorting algorithm in Rust");
  assert.equal(answer.intent, "algorithm_sort_rust");
  assert.equal(
    answer.content,
    "Here is a reviewable sorting algorithm in rust:\n\n```rust\nfn sort(values: &mut Vec<i32>) {\n    values.sort();\n}\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable.",
  );
  assert.ok(answer.evidence.includes("execution_status:unavailable"));
});

test("the sorting snippet carries a test when one is requested", () => {
  const answer = worker.handleAlgorithm("x", "write a sorting algorithm in python with tests");
  assert.equal(answer.intent, "algorithm_sort_python");
  assert.ok(answer.content.includes("Tests:\n```python\ndef test_sort_ascending():"), answer.content);
});

test("execution failures are reported with the full trace", async () => {
  const chat = await solve("Write a Python script that calls undefined_function()");
  assert.equal(chat.intent, "execution_failure");
  assert.ok(chat.content.includes("Execution status: failed"), chat.content);
  assert.ok(chat.evidence.some((link) => link.startsWith("trace:execution_failure")));
  const agent = await solve("[agent] Run a Python script that calls undefined_function()");
  assert.ok(agent.content.includes("Execution status: failed"), agent.content);
  assert.ok(agent.evidence.some((link) => link.startsWith("trace:execution_failure")));
  assert.ok(agent.evidence.some((link) => link.startsWith("agent_mode:opted_in:")));
});

test("an English PDF request returns the document plan", async () => {
  const answer = await solve(
    "Make me a PDF document listing countries with food subsidies for low-income people.",
  );
  assert.equal(answer.intent, "document_generation_plan");
  assert.ok(answer.content.includes("document-generation request"), answer.content);
  assert.ok(answer.content.includes("PDF"), answer.content);
  assert.ok(answer.content.includes("link-foundation/meta-language"), answer.content);
  assert.ok(answer.content.includes("txt, Markdown, HTML, PDF, and DOCX"), answer.content);
});

test("a Russian PDF request returns the Russian plan", async () => {
  const answer = await solve(
    "Сделай мне пдф файл со списком стран, где есть пособия/скидки на еду для " +
      "малоимущих, как в виде прямых денежных дотаций, так и в косвенной форме, " +
      "например, талоны.",
  );
  assert.equal(answer.intent, "document_generation_plan");
  assert.ok(!answer.content.includes("Я не смог определить"), answer.content);
  assert.ok(answer.content.includes("PDF"), answer.content);
  assert.ok(answer.content.includes("план") || answer.content.includes("План"), answer.content);
  assert.ok(answer.content.includes("Экспортировать"), answer.content);
});

test("a software artifact defers the document plan to the project handler", () => {
  const prompt = "Make me an app that exports a PDF report";
  assert.equal(worker.handleDocumentGenerationPlan(prompt, prompt.toLowerCase()), null);
});

test("conflicting sources are surfaced instead of silently resolved", async () => {
  const answer = await solve("The sources conflict: Wikipedia says X was born in 1880, but Britannica says 1881.");
  assert.equal(answer.intent, "source_conflict");
  assert.ok(answer.evidence.some((link) => link.startsWith("conflict:source_disagreement")));
});

test("a destructive shell request without agent mode is refused by the seed rule", async () => {
  const answer = await solve("Please run `rm -rf /tmp/cache` on my behalf");
  assert.equal(answer.intent, "policy_bounded_autonomy");
  assert.equal(
    answer.content,
    "I can only respond with a chat reply. Running shell commands on your behalf is not allowed without explicit agent mode opt-in, and even then only inside an isolated sandbox.",
  );
  assert.ok(answer.evidence.some((link) => link.startsWith("policy:chat_bounded_autonomy")));
  assert.equal(worker.tryShellRefusal("[agent] Please run `rm -rf /tmp/cache` on my behalf"), null);
});
