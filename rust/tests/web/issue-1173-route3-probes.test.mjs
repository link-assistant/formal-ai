// Issue #1173 R1173-3 browser twin, the fifth routing-probe pass (ROUTE3).
// "a program that prints <text>" names the catalog task whose output is the
// request's operand (`print_text` in data/seed/hello-world-programs.lino): the
// program is the documented hello_world print procedure with the text bound
// into its output literal, never a stored program. And a handler the
// precedence seed marks `before-promotion` is asked in solve()'s fixed early
// phase, which the native dispatcher now mirrors (issue #1175 p133). Native
// pins: rust/tests/unit/seed/issue_1173_route3_probes.rs.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

const solve = async (prompt) => plain(await worker.solve(prompt, [], {}, {}, [], {}));
const operand = (prompt) => JSON.parse(evaluate(worker, `JSON.stringify(programTaskOperand(${JSON.stringify(prompt)}))`));

test("R1173-3: a program that prints a greeting is the print_text task, composed from the documented procedure", async () => {
  const answer = await solve("Write a program in Rust that prints hello");
  assert.equal(answer.intent, "write_program");
  assert.equal(answer.content, [
    "Here is a minimal Rust print text program:",
    "",
    "```rust",
    "fn main() {",
    "    println!(\"hello\");",
    "}",
    "```",
    "",
    "Execution status: not run - the browser sandbox cannot invoke a rust toolchain.",
    "",
    "Copy the snippet into a rust environment to verify.",
    "",
    "Expected output after verification:",
    "```text",
    "hello",
    "```",
    "",
    "How it works:",
    "The program performs the requested task and prints its result to standard output.",
    "",
    "How to test it yourself:",
    "1. Install the Rust toolchain from https://rustup.rs.",
    "2. Save the code above to a file named `main.rs`.",
    "3. Check that it compiles: `rustc main.rs -o main`.",
    "4. Run it: `./main`.",
    "5. Compare the output with the expected output shown above.",
  ].join("\n"));
  assert.equal(answer.programExecution.task, "print_text");
  assert.equal(answer.programExecution.rediscoveredFrom, "https://doc.rust-lang.org/book/ch01-02-hello-world.html");
  const russian = await solve("Напиши программу на Python, которая печатает привет");
  assert.equal(russian.intent, "write_program");
  assert.ok(russian.content.includes("```python\nprint('привет')\n```"), russian.content);
  assert.equal(russian.programExecution.output, "привет");
});

test("R1173-3: the operand is a quoted literal, an unquoted utterance or a greeting, in every script", () => {
  assert.equal(operand("Write a Java program that prints 'Good morning, team'"), "Good morning, team");
  assert.equal(operand("Write a C program to print Good Morning"), "Good Morning");
  assert.equal(operand("Write a program in Go that prints Banana in Go"), "Banana");
  assert.equal(operand("Write a program in Rust that prints 'Hello' and then exits"), "Hello");
  assert.equal(operand("Python में एक प्रोग्राम लिखो जो नमस्ते प्रिंट करे"), "नमस्ते");
  assert.equal(operand("用Python写一个打印“你好”的程序"), "你好");
});

test("R1173-3: a described value, two literals or no print verb name no operand", async () => {
  for (const prompt of [
    "Write a program in Rust that prints the date",
    "Write a program that prints prime numbers below 50",
    "Write a program that prints the sum of a and b",
    "Write a program that prints 'a' and 'b'",
    "Write a program in Rust",
  ]) {
    assert.equal(operand(prompt), null, prompt);
  }
  assert.equal((await solve("Write a program in Rust that prints the date")).intent, "write_program_skill_gap");
});

test("R1173-3: a script request stays the minimal-script lane, and no program is stored for print_text", async () => {
  assert.equal((await solve("Write a Python script that prints hello")).intent, "write_script_python");
  assert.equal(evaluate(worker, 'writeProgramTemplate("print_text", "rust")'), null);
  const seed = readFileSync(path.join(REPO_ROOT, "data/seed/hello-world-programs.lino"), "utf8");
  assert.ok(seed.includes("task_print_text\n  task print_text\n  label \"print text\"\n  output \"{operand}\"\n  procedure hello_world\n"));
  assert.ok(!/template_print_text_/u.test(seed), "the operand task stores no per-language program");
});

test("issue #1175 p133: every before-promotion row is asked in solve()'s early phase, before the promoted walk", async () => {
  const precedence = readFileSync(path.join(REPO_ROOT, "data/seed/handler-precedence.lino"), "utf8");
  const early = [...precedence.matchAll(/^ {2}handler (\S+)\n(?: {4}.*\n)*? {4}before[-_]promotion true$/gmu)].map((match) => match[1]);
  assert.deepEqual(early, ["github_repository_traffic"]);
  const claims = readFileSync(path.join(REPO_ROOT, "data/seed/capability-routing.lino"), "utf8");
  const solveSource = readFileSync(path.join(REPO_ROOT, "js/worker/formal_ai_worker_solve.js"), "utf8");
  const walk = solveSource.indexOf("synchronousHandlerCandidates({");
  for (const handler of early) {
    const browser = claims.match(new RegExp(`handler ${handler}\\n {4}browser_handler (\\S+)`, "u"))[1];
    const call = solveSource.indexOf(`claimRouteRun("${browser}"`);
    assert.ok(call !== -1 && call < walk, `${browser} runs before the promoted walk`);
  }
  assert.equal((await solve("Can I see who visited my GitHub repository?")).intent, "github_repository_traffic");
});
