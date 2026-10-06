// Issue #1177 browser parity: the code-task handlers that read pasted code —
// debugging, explanation, review, test generation and refactoring.
//
// The worker twins in js/worker/formal_ai_worker_code_tasks.js must answer the
// prompts rust/tests/unit/issue_1177_code_task_handlers.rs pins, both through
// the full dispatcher (engine level) and through each `handle*` function with
// the normalized prompt (handler level). The refactoring twin goes beyond Rust
// (R7): it runs the chain and its rewrite in the worker and compares traces.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

function handle(name, prompt) {
  const answer = worker[name](prompt, worker.normalizePrompt(prompt));
  assert.ok(answer, `${name} should answer: ${prompt}`);
  return answer.content;
}

function fenced(answer, tag) {
  const open = "```" + tag + "\n";
  const start = answer.indexOf(open);
  assert.notEqual(start, -1, `answer should carry a \`${tag}\` fence`);
  const end = answer.indexOf("```", start + open.length);
  assert.notEqual(end, -1, `the \`${tag}\` fence should be closed`);
  return answer.slice(start + open.length, end);
}

const DEBUG_PROMPT = "What's wrong with `def average(xs): return sum(xs) / len(xs) - 1`?";
const EXPLAIN_PROMPT = "Explain this code: `def average(xs): return sum(xs) / len(xs)`";
const REVIEW_PROMPT =
  "Review this code:\n```python\ndef load(path):\n    try:\n        f = open(path)\n    except:\n        pass\n```";
const TESTS_PROMPT = "Write tests for `is_palindrome(s)` ignoring case, spaces, and punctuation";
const REFACTOR_PROMPT =
  "Refactor this promise chain with async/await:\n```javascript\nfetch(url)\n  .then(r => r.json())\n  .then(d => render(d))\n  .catch(e => log(e));\n```";

test("engine answers the code-debugging request with the shifted quotient", async () => {
  const answer = await solve(DEBUG_PROMPT);
  assert.equal(answer.intent, "code_debugging");
  assert.ok(answer.evidence.includes("response:code_debugging"));
  assert.ok(answer.content.includes("- 1"), answer.content);
  assert.ok(answer.content.includes("average"), answer.content);
  assert.ok(answer.content.includes("No code was executed"), answer.content);
});

test("handler reports the shifted quotient with both repairs", () => {
  const answer = handle("handleCodeDebugging", DEBUG_PROMPT);
  assert.ok(answer.includes("- 1"), answer);
  assert.ok(answer.includes("sum(xs)"), answer);
  assert.ok(answer.includes("    sum(xs) / len(xs)\n"), "the drop-shift repair is shown");
  assert.ok(answer.includes("    sum(xs) / (len(xs) - 1)\n"), "the parenthesized repair is shown");
  assert.ok(answer.includes("No code was executed"), answer);
});

test("a comparison after a division is not a shifted quotient", () => {
  const answer = handle("handleCodeDebugging", "Find the bug: `def average(xs): return sum(xs) / len(xs) == x - 1`");
  assert.ok(answer.startsWith("No recognized defect pattern."), answer);
});

test("engine and handler explain the function and its promise", async () => {
  const engine = await solve(EXPLAIN_PROMPT);
  assert.equal(engine.intent, "code_explanation");
  for (const answer of [engine.content, handle("handleCodeExplanation", EXPLAIN_PROMPT)]) {
    assert.ok(answer.includes("function"), answer);
    assert.ok(answer.includes("average"), answer);
    assert.ok(answer.includes("sum(xs) / len(xs)"), answer);
    assert.ok(answer.includes("No code was executed"), answer);
  }
});

test("engine and handler review the bare except with its upstream source", async () => {
  const engine = await solve(REVIEW_PROMPT);
  assert.equal(engine.intent, "code_review");
  for (const answer of [engine.content, handle("handleCodeReview", REVIEW_PROMPT)]) {
    assert.ok(answer.includes("except:"), answer);
    assert.ok(answer.includes("docs.python.org"), answer);
    assert.ok(answer.includes("no linter was run"), answer);
  }
});

test("engine and handler build the palindrome pytest suite", async () => {
  const engine = await solve(TESTS_PROMPT);
  assert.equal(engine.intent, "test_generation");
  for (const answer of [engine.content, handle("handleTestGeneration", TESTS_PROMPT)]) {
    assert.ok(answer.includes("def test_"), answer);
    assert.ok(answer.includes("RaceCar"), answer);
    assert.ok(answer.includes("never odd or even"), answer);
    assert.ok(answer.includes("def normalize(value):"), answer);
    assert.ok(answer.includes("is_palindrome(normalize("), answer);
    assert.ok(answer.includes("NOT executed"), answer);
  }
});

test("engine and handler rewrite the promise chain with async/await", async () => {
  const engine = await solve(REFACTOR_PROMPT);
  assert.equal(engine.intent, "code_refactoring");
  for (const answer of [engine.content, handle("handleCodeRefactoring", REFACTOR_PROMPT)]) {
    const code = fenced(answer, "javascript");
    assert.ok(code.includes("async function run()"), answer);
    assert.ok(code.includes("await fetch(url)"), answer);
    assert.ok(code.includes("await r.json()"), answer);
    assert.ok(code.includes("await render(d)"), answer);
    assert.ok(code.includes("catch (e)"), answer);
    assert.ok(answer.includes("No code was executed"), answer);
  }
});

test("R7: the worker runs the chain and its rewrite and the call traces match", () => {
  const answer = handle("handleCodeRefactoring", REFACTOR_PROMPT);
  assert.ok(
    answer.includes(
      "in 4 scenarios (every call fulfilled, then each of the 3 calls rejecting in turn), and the recorded call traces matched",
    ),
    answer,
  );
  assert.ok(answer.includes("Fulfilled trace: fetch(url); fetch(url).json(); render(fetch(url).json())."), answer);
});

test("R7: a rewrite that loses the catch clause is caught by execution", () => {
  const chain = worker.codeRefactoringParseChain(
    "fetch(url) .then(r => r.json()) .catch(e => log(e));",
  );
  assert.ok(chain);
  const broken = { head: chain.head, thens: chain.thens, catchHandler: null };
  const flat = "fetch(url) .then(r => r.json()) .catch(e => log(e));";
  const twin = worker.codeRefactoringRenderAsync(broken, true) + "\ntry { run(); } catch (__unhandled) {}";
  assert.notDeepEqual(
    Array.from(worker.codeRefactoringTraceRun(flat, 1)),
    Array.from(worker.codeRefactoringTraceRun(twin, 1)),
    "a rejected fetch reaches log(e) only in the original",
  );
});

test("unrelated prompts are not claimed by any code-task handler", () => {
  for (const prompt of ["Hello, how are you today?", "What is the capital of France?"]) {
    for (const name of [
      "handleCodeDebugging",
      "handleCodeExplanation",
      "handleCodeReview",
      "handleTestGeneration",
      "handleCodeRefactoring",
    ]) {
      assert.equal(worker[name](prompt, worker.normalizePrompt(prompt)), null, `${name} must not claim: ${prompt}`);
    }
  }
});
