// Issue #1177 browser parity: natural-language shell-command composition.
//
// The worker twin in js/worker/formal_ai_worker_shell_compose.js must compose
// the command rust/tests/unit/web-engine-core/issue_1177_code_task_handlers.rs pins, explain
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

test("an ending suffix names the -name pattern under the shell-command framing", async () => {
  const prompt = "Write a shell command that finds files ending in .lino under data/meta";
  const engine = await solve(prompt);
  assert.equal(engine.intent, "shell_command_compose");
  for (const answer of [engine.content, handle(prompt)]) {
    assert.ok(answer.includes("    find data/meta -name '*.lino'\n"), answer);
    assert.ok(answer.includes("Not executed"), answer);
  }
});

test("a seeded counting cue pipes the find into wc -l", async () => {
  for (const prompt of [
    "count .lino files under data/meta",
    "Write a shell command that counts files ending in .lino under data/meta",
  ]) {
    const engine = await solve(prompt);
    assert.equal(engine.intent, "shell_command_compose", prompt);
    for (const answer of [engine.content, handle(prompt)]) {
      assert.ok(answer.includes("    find data/meta -name '*.lino' | wc -l\n"), answer);
      assert.ok(answer.includes("https://www.gnu.org/software/coreutils/manual/html_node/wc-invocation.html"), answer);
      assert.ok(answer.includes("`-l`"), answer);
      assert.ok(answer.includes("Not executed"), answer);
    }
  }
});

test("content searches compose grep with the requested flags", () => {
  const answer = handle("search for TODO in files under /src ignoring case");
  assert.ok(answer.includes("    grep -r -i 'TODO' /src\n"), answer);
});

test("a seeded Russian containment cue composes grep", () => {
  const answer = handle("найди файлы содержащие TODO в /src");
  assert.ok(answer.includes("    grep -r 'TODO' /src\n"), answer);
});

test("a more specific code-task cue makes the composer step aside", () => {
  const prompt = "Write a SQL query that selects all files";
  assert.equal(worker.handleShellCommandCompose(prompt, worker.normalizePrompt(prompt)), null);
});

test("a requested function or program is left to program synthesis", () => {
  const prompt = "Write a JavaScript function that counts the lines of a text";
  assert.equal(worker.handleShellCommandCompose(prompt, worker.normalizePrompt(prompt)), null);
});

test("unrelated prompts are not claimed by the composer", () => {
  for (const prompt of ["Hello, how are you today?", "What is the capital of France?"]) {
    assert.equal(worker.handleShellCommandCompose(prompt, worker.normalizePrompt(prompt)), null, prompt);
  }
});

test("a prose lead before a colon is not part of the looped command", async () => {
  const prompt = "Make this a single line loop: sleep 5m && cleanup -f";
  const expected = "while true; do sleep 5m && cleanup -f; done";
  const engine = await solve(prompt);
  assert.equal(engine.intent, "shell_command_transform");
  assert.equal(engine.content, expected);
  assert.equal(worker.tryShellCommandTransform(prompt, []).content, expected);
});

test("a substitution request composes a literal sed expression", async () => {
  const answer = handle("Replace 1.0 with 2.0 in version.txt");
  assert.ok(answer.includes("    sed -i 's/1\\.0/2.0/g' version.txt\n"), answer);
  assert.ok(answer.includes("https://www.gnu.org/software/sed/manual/sed.html"), answer);
  assert.ok(answer.includes("'Replace 1.0 with 2.0' -> s/1\\.0/2.0/g"), answer);
  const russian = await solve("Замени foo на bar в файле config.txt");
  assert.equal(russian.intent, "shell_command_compose");
  assert.ok(russian.content.includes("    sed -i 's/foo/bar/g' config.txt\n"), russian.content);
  const sentence = "Replace cat with dog in this sentence";
  assert.equal(worker.handleShellCommandCompose(sentence, worker.normalizePrompt(sentence)), null);
});
