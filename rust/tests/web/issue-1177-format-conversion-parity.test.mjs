// Issue #1177 browser parity: JSON↔YAML format conversion.
//
// The worker twin in js/worker/formal_ai_worker_format_conversion.js must
// convert the probe rust/tests/unit/web-engine-core/issue_1177_code_task_handlers.rs pins and
// feed its own YAML back to the same JSON value. R10: the round trip must be
// lossless on every probe — integer vs float numbers, u64 range, key order,
// strings that would otherwise read as YAML scalars, and empty containers —
// with number text following serde_json (JSON) and Rust Display (YAML).

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

function handle(prompt) {
  const answer = worker.handleFormatConversion(prompt, worker.normalizePrompt(prompt));
  assert.ok(answer, `the converter should answer: ${prompt}`);
  return answer;
}

function fenced(answer, tag) {
  const open = "```" + tag + "\n";
  const start = answer.indexOf(open);
  assert.notEqual(start, -1, `answer should carry a \`${tag}\` fence: ${answer}`);
  const end = answer.indexOf("```", start + open.length);
  return answer.slice(start + open.length, end);
}

const PROBE = "{\"name\": \"formal-ai\", \"tags\": [\"rust\", \"lino\"], \"counts\": {\"lines\": 42, \"passed\": 41}}";

test("engine and handler round-trip the issue probe", async () => {
  const toYaml = "Convert this JSON to YAML:\n```json\n" + PROBE + "\n```";
  const engine = await solve(toYaml);
  assert.equal(engine.intent, "format_conversion");
  assert.ok(engine.evidence.includes("response:format_conversion"));
  for (const answer of [engine.content, handle(toYaml).content]) {
    const yaml = fenced(answer, "yaml");
    assert.ok(yaml.includes("name: formal-ai"), answer);
    assert.ok(yaml.includes("- rust"), answer);
    assert.ok(yaml.includes("lines: 42"), answer);
    const toJson = "Convert this YAML to JSON:\n```yaml\n" + yaml + "\n```";
    const back = (await solve(toJson)).content;
    assert.deepEqual(JSON.parse(fenced(back, "json")), JSON.parse(PROBE), back);
  }
});

test("R10: every probe round-trips losslessly through YAML and back", () => {
  const probes = [
    PROBE,
    "{\"float\": 1.0, \"half\": 0.5, \"huge\": 1e21, \"tiny\": 1e-7, \"neg\": -3, \"u64\": 18446744073709551615}",
    "{\"b\": 1, \"a\": 2, \"10\": \"ten\", \"2\": \"two\"}",
    "{\"yes\": \"yes\", \"t\": \"true\", \"n\": \"1.5\", \"colon\": \"a: b\", \"empty\": \"\", \"null\": null}",
    "{\"list\": [], \"map\": {}, \"rows\": [{\"id\": 1, \"tags\": [\"x\"]}, {\"id\": 2}], \"nested\": [[1, 2], []]}",
    "[1, \"two\", false, null, {\"k\": \"v\"}]",
  ];
  for (const probe of probes) {
    const yamlAnswer = handle("Convert this JSON to YAML:\n```json\n" + probe + "\n```");
    assert.equal(yamlAnswer.confidence, 0.7, yamlAnswer.content);
    assert.ok(yamlAnswer.evidence.includes("format_conversion:reverse_roundtrip:json=ok"));
    const yaml = fenced(yamlAnswer.content, "yaml");
    const jsonAnswer = handle("Convert this YAML to JSON:\n```yaml\n" + yaml + "\n```");
    assert.equal(jsonAnswer.confidence, 0.7, jsonAnswer.content);
    const emitted = fenced(jsonAnswer.content, "json");
    const original = worker.formatJsonParse(probe);
    const reparsed = worker.formatJsonParse(emitted);
    assert.ok(original.ok && reparsed.ok, emitted);
    assert.ok(worker.formatValuesEqual(original.value, reparsed.value), `${probe}\n${yaml}\n${emitted}`);
  }
});

test("R10: number text follows serde_json for JSON and Rust Display for YAML", () => {
  const yaml = fenced(
    handle("Convert this JSON to YAML:\n```json\n{\"float\": 1.0, \"huge\": 1e21, \"tiny\": 1e-7}\n```").content,
    "yaml",
  );
  assert.equal(yaml, "float: 1.0\nhuge: 1000000000000000000000.0\ntiny: 0.0000001\n");
  const json = fenced(handle("Convert this YAML to JSON:\n```yaml\n" + yaml + "```").content, "json");
  assert.equal(json, "{\n  \"float\": 1.0,\n  \"huge\": 1e21,\n  \"tiny\": 1e-7\n}");
});

test("R10: insertion order is kept even for integer-like keys", () => {
  const yaml = fenced(
    handle("Convert this JSON to YAML:\n```json\n{\"b\": 1, \"10\": 2, \"2\": 3}\n```").content,
    "yaml",
  );
  assert.equal(yaml, "b: 1\n\"10\": 2\n\"2\": 3\n");
});

test("malformed JSON and out-of-subset YAML are refused by name", () => {
  const bad = handle("Convert this JSON to YAML:\n```json\n{\"a\": }\n```").content;
  assert.ok(bad.startsWith("I will not guess a conversion. The text was recognized as JSON but could not be parsed ("), bad);
  const flow = handle("Convert this YAML to JSON:\n```yaml\na: 1\n  b: [\n```").content;
  assert.ok(flow.includes("outside the supported YAML subset"), flow);
});

test("unrelated prompts are not claimed by the converter", () => {
  for (const prompt of ["Hello, how are you today?", "What is the capital of France?"]) {
    assert.equal(worker.handleFormatConversion(prompt, worker.normalizePrompt(prompt)), null, prompt);
  }
});
