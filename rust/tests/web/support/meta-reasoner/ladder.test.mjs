import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";
import {captured, solve, FIXTURES} from "./runtime.mjs";

test("every rung of the task ladder is derived, never handled", async () => {
  const ladder = readFileSync(path.join(FIXTURES, "ladder.lino"), "utf8");
  const rungs = [...ladder.matchAll(/^ {2}rung "((?:[^"\\]|\\.)*)"\n {4}steps "([^"]*)"(?:\n {4}awaits (\S+))?/gmu)];
  assert.ok(rungs.length >= 10, `${rungs.length} rungs`);
  for (const [, quoted, steps, awaits] of rungs) {
    if (awaits && !captured(awaits)) continue;
    const prompt = JSON.parse(`"${quoted}"`);
    const answer = await solve(prompt);
    assert.match(answer.intent, /^meta_reasoned_program/u, `${prompt}: ${answer.intent}`);
    const derived = /\n {2}program "([^"]*)"/u.exec(answer.derivation);
    assert.equal(derived && derived[1].split(" ∘ ").join(" "), steps, prompt);
  }
});
