// Issue #1173 R1173-3 browser twin: a request that needs no outside
// knowledge reaches its class handler, never the fallback. Native pins:
// rust/tests/unit/solver/issue_1173_local_requests_route.rs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

function solve(prompt) {
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("R1173-3: a yearly compounding problem reaches the interest handler", async () => {
  const answer = await solve("invest $2500 at 6% annual interest compounded yearly for 10 years");
  assert.equal(answer.intent, "calculation", JSON.stringify(answer));
  assert.equal(answer.content, "Compound interest calculation\n\nFormula: A = P(1 + r/n)^(n*t)\nP = 2500 USD\nr = 0.06 (6% annual)\nn = 1 (annually)\nt = 10 years\n\nStep 1: periodic rate = r/n = 0.06/1 = 0.06\nStep 2: number of periods = n*t = 1*10 = 10\nStep 3: A = 2500 * (1 + 0.06)^10\nFinal amount: 4477.12 USD");
});

test("R1173-3: a CSV table asked for as JSON reaches the format converter", async () => {
  const answer = await solve("Convert this CSV to JSON: name,age\nAnn,30");
  assert.equal(answer.intent, "format_conversion", JSON.stringify(answer));
  assert.equal(answer.content, "Converted CSV to JSON. Data rows: 1; header columns: name, age. CSV carries no types, so every value stays a JSON string rather than a guessed number or boolean.\n\n```json\n[\n  {\n    \"name\": \"Ann\",\n    \"age\": \"30\"\n  }\n]\n```\nRound-trip check: parsing the emitted JSON back gave every row the same value under every header column as the CSV it came from.");
});
