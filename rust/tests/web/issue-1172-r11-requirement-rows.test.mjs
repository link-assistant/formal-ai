// Issue #1172 R1172-11: the #127 rows R173, R174 and R177 may stand as
// "Implemented" only while each one records the substring alias defect the
// Australia measurement exposed and its word-boundary correction, with issue
// #1172 cited. Pinned by substance (the defect, the correction, the citation),
// not by one exact sentence, so the assembler may reword the register.
//
// The behaviour the rows describe is pinned separately: an unseeded subject
// is never answered from another record (issue-1172-factual-qa-subject-gate).

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const shard = readFileSync(
  fileURLToPath(
    new URL("../../../docs/requirements/issue-0127-structured-fact-query-reasoning-requirements.md", import.meta.url),
  ),
  "utf8",
);

function row(id) {
  const line = shard.split("\n").find((candidate) => candidate.startsWith(`| ${id} |`));
  assert.ok(line, `${id} has a row in the #127 shard`);
  return line.split(" | ").slice(2).join(" | ");
}

for (const id of ["R173", "R174", "R177"]) {
  test(`R1172-11: ${id} records the substring defect and its correction`, () => {
    const status = row(id);
    assert.match(status, /\bissue #1172\b/u, `${id} cites issue #1172`);
    assert.match(status, /"us"[^|]*"australia"/u, `${id} names the alias that matched inside "australia"`);
    assert.match(status, /substring/u, `${id} names the substring defect`);
    assert.match(status, /word[- ]boundar/u, `${id} names the word-boundary correction`);
  });
}

test("R1172-11: the correction the rows cite holds — Australia is not answered from the USA record", async () => {
  const facts = readFileSync(fileURLToPath(new URL("../../src/seed/facts.rs", import.meta.url)), "utf8");
  assert.match(facts, /pub fn contains_word_sequence\b/u, "the Rust matcher tokenizes aliases");
  const worker = createWorkerContext();
  await evaluate(worker, "loadSeed()");
  const answer = await worker.solve("What is the capital of Australia?", [], {}, {}, [], {});
  assert.notEqual(answer.intent, "fact_lookup", JSON.stringify(answer));
  assert.doesNotMatch(String(answer.content), /Washington/u);
  const usa = await worker.solve("What is the capital of the USA?", [], {}, {}, [], {});
  assert.equal(usa.intent, "fact_lookup", JSON.stringify(usa));
  assert.match(String(usa.content), /Washington/u);
});
