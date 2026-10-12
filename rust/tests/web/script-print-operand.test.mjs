import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import test from "node:test";
import { WorkerHost } from "../../../js/server/worker-host.mjs";

const host = new WorkerHost();
const operand = (request) => host.run(`programTaskOperand(${JSON.stringify(request)})`);

async function emittedOutput(request) {
  const answer = await host.solve(request);
  assert.equal(answer.intent, "write_script_python");
  const code = answer.content.match(/```python\n([\s\S]*?)\n```/u)?.[1];
  assert.ok(code, "the answer contains the generated program");
  const run = spawnSync("python3", ["-I", "-c", code], {
    encoding: "utf8", timeout: 5000, maxBuffer: 1024 * 1024,
  });
  assert.equal(run.error, undefined);
  assert.equal(run.signal, null);
  assert.equal(run.status, 0, run.stderr);
  return { answer, output: run.stdout };
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

test("a script with no captured operand retains its original default route", async () => {
  const answer = await host.solve("Write a Python script");
  assert.equal(answer.intent, "write_script_python");
  assert.equal(answer.programExecution.task, "hello_world");
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
