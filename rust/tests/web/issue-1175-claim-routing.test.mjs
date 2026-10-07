// Issue #1175 R3, browser root: the claim rows of
// data/seed/capability-routing.lino are consulted before a handler runs
// (claimRouteAdmits in js/worker/formal_ai_worker_dispatch.js), as
// rust/tests/unit/issue_1175_claim_routing.rs pins for the native router.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function admits(handler, prompt) {
  await seeded;
  return plain(evaluate(worker, `claimRouteAdmits(${JSON.stringify(handler)}, ${JSON.stringify(prompt)})`));
}

test("the claim rows are read from the capability table", async () => {
  await seeded;
  assert.deepEqual(plain(evaluate(worker, "claimRouteRows()")), [
    { handler: "software_project", browserHandler: "trySoftwareProjectRequest", admitsOn: ["object_phrase_artifact", "approval_of_a_proposal"] },
    { handler: "terminal_command", browserHandler: "tryTerminalCommand", admitsOn: ["shell_command_shape", "semantic_shell_task"] },
    { handler: "repository_lineage", browserHandler: "", admitsOn: ["repository_subject"] },
  ]);
});

test("a surface word alone is not admitted", async () => {
  assert.equal(await admits("tryTerminalCommand", "Make a 3-day itinerary for a first visit to Rome."), false);
  assert.equal(await admits("tryTerminalCommand", "Find the bug: def average(xs): return sum(xs) / len(xs)"), false);
  assert.equal(await admits("trySoftwareProjectRequest", "Write a regex for a US ZIP code with an optional 4-digit extension"), false);
});

test("the structural evidence admits", async () => {
  assert.equal(await admits("tryTerminalCommand", "find . -name '*.log' -size +10M"), true);
  assert.equal(await admits("trySoftwareProjectRequest", "Build a web app for tracking habits"), true);
});

test("a handler with no claim row is admitted as before", async () => {
  assert.equal(await admits("tryConceptLookup", "What is a monad?"), true);
});
