// serde_json float fidelity in the JavaScript server (js/server/json.mjs):
// floats print the way serde_json's writer (the `zmij` crate) prints them,
// in JSON bodies, SSE frames and Links Notation exports, and a JSON body the
// server reads and writes back keeps `1.0` a float. Also the request-scoped
// dialog id (`DialogScope`, js/server/dialog-log.mjs).

import assert from "node:assert/strict";
import { test } from "node:test";

import { conversationContextToLino, turnsInExchange } from "../../../js/server/conversations.mjs";
import { currentDialogId, withDialogScope } from "../../../js/server/dialog-log.mjs";
import { f32, f64, parseJson, renderFloat, sortedKeys, toCompactJson, toPrettyJson } from "../../../js/server/json.mjs";

test("f64 renders like zmij: fixed between 1e-5 and 1e16, exponent otherwise", () => {
  const cases = [
    [1, "1.0"], [0, "0.0"], [-0, "-0.0"], [0.5, "0.5"], [-2.5, "-2.5"], [100, "100.0"],
    [43210.1, "43210.1"], [-5942736479622170, "-5942736479622170.0"], [1e15, "1000000000000000.0"],
    [1e16, "1e+16"], [123456789012345680, "1.2345678901234568e+17"], [1e21, "1e+21"],
    [0.00001, "0.00001"], [0.000001, "1e-6"], [1.5e-7, "1.5e-7"], [9.78344173444751e-9, "9.78344173444751e-9"],
    [2.9802322387695312e-8, "2.9802322387695312e-8"], [6.62607015e-34, "6.62607015e-34"], [5e-324, "5e-324"],
    [0.1 + 0.2, "0.30000000000000004"], [Infinity, "null"], [NaN, "null"],
  ];
  for (const [value, expected] of cases) assert.equal(renderFloat(value), expected, String(value));
});

test("f32 renders its own shortest digits with the f32 thresholds", () => {
  const cases = [
    [0.85, "0.85"], [0.1, "0.1"], [43210.1, "43210.1"], [1.342178e8, "134217800.0"], [1.3421781e8, "134217810.0"],
    [1e12, "1000000000000.0"], [1e13, "1e+13"], [0.000001, "0.000001"], [1e-7, "1e-7"], [6.62607e-34, "6.62607e-34"],
  ];
  for (const [value, expected] of cases) assert.equal(renderFloat(Math.fround(value), 32), expected, String(value));
  assert.equal(toCompactJson({ confidence: f32(0.85) }), '{"confidence":0.85}');
  assert.equal(toCompactJson({ confidence: f64(Math.fround(0.85)) }), '{"confidence":0.8500000238418579}');
});

test("float markers render in pretty and compact bodies", () => {
  assert.equal(toPrettyJson({ a: f64(1), b: [f64(0), 2] }), '{\n  "a": 1.0,\n  "b": [\n    0.0,\n    2\n  ]\n}');
  assert.equal(toCompactJson({ context_used_fraction: f64(1e-6) }), '{"context_used_fraction":1e-6}');
});

test("parseJson keeps every number the kind serde_json reads it as", () => {
  const text = '{"a":1.0,"b":[2.50,3,-0,1e2,-5,18446744073709551615,18446744073709551616,9007199254740993],"c":"1.0 \\" q"}';
  assert.equal(
    toCompactJson(parseJson(text)),
    '{"a":1.0,"b":[2.5,3,-0.0,100.0,-5,18446744073709551615,1.8446744073709552e+19,9007199254740993],"c":"1.0 \\" q"}',
  );
  assert.deepEqual(parseJson("[1, 2]"), [1, 2]);
  assert.throws(() => parseJson("{"), SyntaxError);
});

test("a recorded exchange keeps request floats floats", () => {
  const request = JSON.stringify({ model: "formal-ai", temperature: 1.0, messages: [{ role: "user", content: "hi", weight: 1.0 }] })
    .replace('"weight":1', '"weight":1.0');
  const turns = turnsInExchange(request, '{"choices":[{"message":{"role":"assistant","content":"ok","score":2.0}}]}');
  assert.equal(toCompactJson(sortedKeys(turns)), '[{"content":"hi","role":"user","weight":1.0},{"content":"ok","role":"assistant","score":2.0}]');
  const lino = conversationContextToLino("d1", { messages: turns });
  assert.match(lino, /weight 1\.0/);
  assert.match(lino, /score 2\.0/);
});

test("the dialog scope holds the declared session id for one request", async () => {
  assert.equal(currentDialogId(), null);
  const seen = await withDialogScope([["X-Formal-AI-Dialog-ID", "session-1"]], async () => {
    await new Promise((resolve) => setTimeout(resolve, 1));
    const inner = withDialogScope([["x-formal-ai-dialog-id", "bad id!"]], () => currentDialogId());
    return [currentDialogId(), inner];
  });
  assert.deepEqual(seen, ["session-1", null]);
  assert.equal(currentDialogId(), null);
});
