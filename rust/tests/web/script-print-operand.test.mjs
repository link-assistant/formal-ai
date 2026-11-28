import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { startCli } from "./support/structured-log-fixture.mjs";

const host = new WorkerHost();
const operand = (request) => host.run(`programTaskOperand(${JSON.stringify(request)})`);

function runPython(code, input) {
  assert.ok(code, "the answer contains the generated Python program");
  const run = spawnSync("python3", ["-I", "-c", code], {
    input, timeout: 5000, maxBuffer: 1024 * 1024,
  });
  assert.equal(run.error, undefined);
  assert.equal(run.signal, null);
  assert.equal(run.status, 0, run.stderr.toString());
  return run.stdout;
}

async function emittedOutput(request) {
  const answer = await host.solve(request);
  assert.equal(answer.intent, "write_script_python");
  const code = answer.content.match(/```python\n([\s\S]*?)\n```/u)?.[1];
  assert.ok(code, "the answer contains the generated program");
  return { answer, output: runPython(code).toString() };
}

test("the original print-script request prints its requested operand", async () => {
  const original = readFileSync(new URL("./issue-1173-route3-probes.test.mjs", import.meta.url), "utf8");
  const request = original.match(/solve\("(Write a Python script that prints hello)"\)/u)?.[1];
  assert.ok(request, "the unchanged original request remains present");
  const result = await emittedOutput(request);
  assert.equal(result.output, "hello\n");
  assert.equal(result.answer.programExecution.output, "hello");
  assert.equal(result.answer.programExecution.task, "print_text");
});

test("literal whitespace, Unicode and quotes survive the actual emitter", async () => {
  for (const text of ["two  spaces", "e\u0301 😀", 'say "yes"']) {
    const result = await emittedOutput(`Write a Python script that prints '${text}'`);
    assert.equal(result.output, `${text}\n`);
    assert.equal(result.answer.programExecution.output, text);
  }
});

test("a code-shaped literal remains output data", async () => {
  const text = 'print(99); "other"';
  const result = await emittedOutput(`Write a Python script that prints '${text}'`);
  assert.equal(result.output, `${text}\n`);
});

test("only the complete seeded termination clause follows an owned literal", async () => {
  assert.equal(await operand("Write a program in Rust that prints 'Hello' and then exits"), "Hello");
  for (const request of [
    "Write a Python script that prints 'x'); print(99); #'",
    "Write a Python script that prints 'x' and deploy",
    "Write a Python script that prints 'x' and 'y'",
    "Write a Python script that prints 'x'. Then edit file a.txt",
  ]) assert.equal(await operand(request), null, request);
});

test("described values and multiline literal operands remain unresolved", async () => {
  for (const request of [
    "Write a program that prints the date",
    "Write a program that prints the sum of a and b",
    "Write a program that prints 'line one\nline two'",
  ]) assert.equal(await operand(request), null, request);
});

test("a script with no task asks for one in the requested language", async () => {
  for (const language of ["Bash", "Python"]) {
    const answer = await host.solve(`Write a ${language} script`);
    assert.equal(answer.intent, "write_program_skill_gap");
    assert.match(answer.content, new RegExp(`^What should the ${language.toLowerCase()} program do\\?`));
    assert.equal(answer.programExecution, undefined);
  }
});

test("a quoted language name is printed by the requested implementation", async () => {
  const result = await emittedOutput("Write a Python script that prints 'rust'");
  assert.equal(result.output, "rust\n");
});

test("a non-string operand request cannot invoke caller coercion", async () => {
  const result = await host.run(`(() => {
    let invoked = 0;
    const actual = programTaskOperand({ toString() { invoked += 1; return "Write a program that prints hello"; } });
    return { actual, invoked };
  })()`);
  assert.deepEqual(result, { actual: null, invoked: 0 });
});

test("documented escaping preserves raw backslashes and active quotes", async () => {
  const slash = String.fromCharCode(92);
  for (const text of ["C:" + slash + "temp" + slash + "file", "raw" + slash + "q", "tail" + slash]) {
    const result = await emittedOutput("Write a Python script that prints '" + text + "'");
    assert.equal(result.output, text + "\n");
  }
  const result = await emittedOutput('Write a Python script that prints "it\'s exact"');
  assert.equal(result.output, "it's exact\n");
});
test("unowned nominal suffix symbols remain unresolved", async () => {
  for (const tail of [" program 🧩", " program +"]) {
    assert.equal(await operand("Write a Python script that prints 'x'" + tail), null);
  }
});

test("an explicit refused print request cannot fall back to a greeting", async () => {
  for (const request of [
    "Write a Python script that prints the date",
    "Write a Python script that prints 'x' program opaque",
    "Write a Python script that prints 'x' and then exits 🧩",
    "Write a Python script that prints 'x' and then exits +",
    "Write a Python script that prints 'x' and do not write",
    "Write a Python script that prints 'x'. Then deploy production",
  ]) {
    assert.equal(await operand(request), null);
    const answer = await host.solve(request);
    assert.equal(/^write_script_/u.test(answer.intent ?? ""), false);
  }
});

const nativeCli = process.env.FORMAL_AI_TEST_NATIVE_CLI;
test("native program replies bind caller operands and report unobserved execution", { skip: !nativeCli, timeout: 30_000 }, async (t) => {
  const server = await startCli(t, { binary: nativeCli, silent: true, lifetime: 30_000, env: { FORMAL_AI_OFFLINE: "1" } });
  await server.ready();
  async function solve(prompt) {
    const response = await fetch(`${server.url}/v1/chat/completions`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: t.signal,
      body: JSON.stringify({ model: "formal-ai", messages: [{ role: "user", content: prompt }] }),
    });
    assert.ok(response.ok, await response.clone().text());
    return (await response.json()).choices[0].message.content;
  }
  assert.match(await solve("Write a Bash script"), /^What should the bash program do\?/u);
  const print = await solve("Write a Python script that prints 'rust'");
  assert.match(print, /not probed/u);
  assert.equal(runPython(print.match(/```python\n([\s\S]*?)\n```/u)?.[1]).toString(), "rust\n");
  const copy = await solve("Write a Python program that copies stdin to stdout");
  assert.match(copy, /not probed/u);
  assert.doesNotMatch(copy, /```text|printf/u);
  const input = Buffer.from([0, 37, 195, 169, 10, 255]);
  assert.deepEqual(runPython(copy.match(/```python\n([\s\S]*?)\n```/u)?.[1], input), input);
  for (const [task, expected] of [
    ["counts from -6 to -4", "-6\n-5\n-4"],
    ["implements FizzBuzz from 14 to 16", "14\nFizzBuzz\n16"],
    ["calculates the factorial of seven", "5040"],
    ["will reverse the string «cat 🧩»", "🧩 tac"],
    ["calculates the sum from 12 through 15", "54"],
    ["calculates the 13th Fibonacci number", "233"],
  ]) {
    const answer = await solve(`Write a Python program that ${task}`);
    assert.match(answer, /not probed/u);
    assert.equal(runPython(answer.match(/```python\n([\s\S]*?)\n```/u)?.[1]).toString(), expected + "\n");
  }
  const missing = await solve("Write a Python program that calculates the factorial");
  assert.match(missing, /^Which integer from 0 to 18/u);
  assert.doesNotMatch(missing, /```|Execution status/u);
  const missingRange = await solve("Write a Python program that implements FizzBuzz");
  assert.match(missingRange, /^Which integer range/u);
  assert.doesNotMatch(missingRange, /```|Execution status/u);
  const missingText = await solve("Write a Python program that reverses a string");
  assert.match(missingText, /^Which text should the program reverse\?/u);
  assert.doesNotMatch(missingText, /```|Execution status/u);
});
