// Issue #1177 browser parity: natural-language shell-command composition.
//
// The worker twin in js/worker/formal_ai_worker_shell_compose.js must compose
// the command rust/tests/unit/issue_1177_code_task_handlers.rs pins, explain
// every emitted flag from data/seed/manual-pages.lino, and never execute it.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

function handle(prompt) {
  const answer = worker.handleShellCommandCompose(prompt, worker.normalizePrompt(prompt));
  assert.ok(answer, `the composer should answer: ${prompt}`);
  return answer.content;
}

const FIND_PROMPT = "find .log files larger than 10 MB under /var";

test("engine and handler compose the find command", async () => {
  const engine = await solve(FIND_PROMPT);
  assert.equal(engine.intent, "shell_command_compose");
  assert.ok(engine.evidence.includes("response:shell_command_compose"));
  for (const answer of [engine.content, handle(FIND_PROMPT)]) {
    assert.ok(answer.includes("find /var"), answer);
    assert.ok(answer.includes("-name '*.log'"), answer);
    assert.ok(answer.includes("-size +10M"), answer);
    assert.ok(answer.includes("Not executed"), answer);
    assert.ok(answer.includes("https://www.gnu.org/software/findutils/manual/html_node/find.html"), answer);
  }
});

test("line slices compose head/tail with the manual meaning of -n", () => {
  const answer = handle("show the last 20 lines of app.log");
  assert.ok(answer.includes("    tail -n 20 app.log\n"), answer);
  assert.ok(answer.includes("`-n N`"), answer);
});

test("a relative path is the search root (R1017)", () => {
  const answer = handle("find .lino files under data/meta");
  assert.ok(answer.includes("    find data/meta -name '*.lino'\n"), answer);
});

test("content searches compose grep with the requested flags", () => {
  const answer = handle("search for TODO in files under /src ignoring case");
  assert.ok(answer.includes("    grep -r -i 'TODO' /src\n"), answer);
});

test("a more specific code-task cue makes the composer step aside", () => {
  const prompt = "Write a SQL query that selects all files";
  assert.equal(worker.handleShellCommandCompose(prompt, worker.normalizePrompt(prompt)), null);
});

test("unrelated prompts are not claimed by the composer", () => {
  for (const prompt of ["Hello, how are you today?", "What is the capital of France?"]) {
    assert.equal(worker.handleShellCommandCompose(prompt, worker.normalizePrompt(prompt)), null, prompt);
  }
});
