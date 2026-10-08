// Issue #1173 R1173-3 browser twin, the fourth routing-probe pass: requests
// that need no outside knowledge reach their class handler, never another
// lane or the fallback, each read from seed data. Native pins:
// rust/tests/unit/issue_1173_route2_probes.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

const failure = (name) => "Execution status: failed in isolated sandbox.\n```python\n" + name
  + "()\n```\nTraceback (most recent call last):\n  File 'main.py', line 1, in <module>\nNameError: name '" + name
  + "' is not defined.\nThe failure trace is appended to the action log; see the trace link.";

test("R1173-3: a script calling a function nothing defines fails whatever the function is named", async () => {
  for (const name of ["missing_helper", "not_defined_anywhere", "undefined_helper", "undefined_function"]) {
    const answer = await solve(`Write a Python script that calls ${name}()`);
    assert.equal(answer.intent, "execution_failure", JSON.stringify(answer));
    assert.equal(answer.content, failure(name));
  }
  const operand = await evaluate(worker, 'JSON.stringify(claimOperandUndefinedCall("Write a Python script that calls missing_helper()"))');
  assert.equal(operand, '["missing_helper"]');
});

test("R1173-3: a builtin call, a method call, a defined name and a call without a program are no undefined call", async () => {
  for (const prompt of [
    "Write a Python script that calls print()",
    "Write a Python script that calls len()",
    "Write a script that calls items.sort()",
    "Write a Python script that defines helper and calls helper()",
    "How do I call foo() from bar?",
  ]) {
    assert.equal(await evaluate(worker, `JSON.stringify(claimOperandUndefinedCall(${JSON.stringify(prompt)}))`), "[]", prompt);
  }
  const builtin = await solve("Write a Python script that calls print()");
  assert.notEqual(builtin.intent, "execution_failure", JSON.stringify(builtin));
});

test("R1173-3: Convert to JSON over a JSON payload reads the source format from the payload", async () => {
  const answer = await solve('Convert to JSON: {"word_counts": {"lines": 4}}');
  assert.equal(answer.intent, "format_conversion", JSON.stringify(answer));
  assert.equal(answer.content, "The text is already JSON, so there is no other format to convert from; here it is parsed and re-emitted with two-space indentation.\n\n```json\n{\n  \"word_counts\": {\n    \"lines\": 4\n  }\n}\n```\nRound-trip check: parsing the emitted JSON back reproduced the same values and nesting as the input.");
});

test("R1173-3: the request line of a YAML conversion is not read as the document's first key", async () => {
  for (const prompt of ["Convert this YAML to JSON:\ncounts: 3\nlines: 4", "Convert to JSON:\ncounts: 3\nlines: 4"]) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "format_conversion", JSON.stringify(answer));
    assert.ok(answer.content.includes("```json\n{\n  \"counts\": 3,\n  \"lines\": 4\n}"), answer.content);
    assert.ok(!answer.content.includes('"Convert'), answer.content);
  }
});

test("R1173-3: a product request with no marketplace names the catalogued ones instead of guessing", async () => {
  const answer = await solve("Find me a laptop under 1000 dollars");
  assert.equal(answer.intent, "product_search", JSON.stringify(answer));
  assert.equal(answer.content, "I compose a product search only on a marketplace the request names, and this one names none, so I will not guess a store for laptop. Name one of the marketplaces I know (amazon in, app store ios, google play) and I will compose the exact search link.\nConstraints you asked for: (none stated)");
  const russian = await solve("Найди мне ноутбук до 1000 долларов");
  assert.equal(russian.intent, "product_search", JSON.stringify(russian));
  assert.ok(russian.content.startsWith("Я составляю поиск товара"), russian.content);
});

test("R1173-3: a learning directive naming a declared source reaches learn_from_source in the browser", async () => {
  for (const prompt of [
    "Отсюда ты узнаешь актуальные темы https://trends.google.com/trending?hl=ru&geo=DE",
    "Here you can learn trending topics https://trends.google.com/trending",
  ]) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "learn_from_source", JSON.stringify(answer));
    assert.ok(answer.content.includes("google trends"), answer.content);
  }
  const unrelated = await solve("Learn about cats");
  assert.notEqual(unrelated.intent, "learn_from_source");
});

test("R1173-3: a quantity between two units is a conversion, not a translation", async () => {
  const answer = await solve("Переведи 10 миль в километры");
  assert.equal(answer.intent, "unit_conversion", JSON.stringify(answer));
});
