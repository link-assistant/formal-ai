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
    { handler: "page_query_text", browserHandler: "tryPageQueryText", admitsOn: ["supplied_page"] },
    { handler: "javascript_execution", browserHandler: "tryJavaScriptExecution", admitsOn: ["javascript_program"] },
    { handler: "incompatible_units", browserHandler: "tryIncompatibleUnits", admitsOn: ["incompatible_unit_pair"] },
    { handler: "http_fetch", browserHandler: "", admitsOn: ["fetch_url"] },
    { handler: "url_navigate", browserHandler: "", admitsOn: ["navigation_url"] },
    { handler: "calendar_create_event", browserHandler: "", admitsOn: ["calendar_date_signal"] },
  ]);
});

test("every row names browser functions and evidence kinds the worker knows", async () => {
  await seeded;
  const unknown = plain(evaluate(worker, `claimRouteRows().flatMap((row) => [
    ...row.admitsOn.filter((kind) => typeof CLAIM_EVIDENCE[kind] !== "function"),
    ...(row.browserHandler && typeof self[row.browserHandler] !== "function" ? [row.browserHandler] : []),
  ])`));
  assert.deepEqual(unknown, []);
});

async function evidence(kind, prompt) {
  await seeded;
  return plain(evaluate(worker, `CLAIM_EVIDENCE[${JSON.stringify(kind)}](${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`));
}

test("the native-only rows read the same structure in the worker", async () => {
  assert.equal(await evidence("fetch_url", "Fetch https://example.com"), true);
  assert.equal(await evidence("fetch_url", "Fetch me a summary of the news"), false);
  assert.equal(await evidence("navigation_url", "Navigate to github.com"), true);
  assert.equal(await evidence("navigation_url", "Navigate the menu to the settings screen"), false);
  assert.equal(await evidence("calendar_date_signal", "Schedule a meeting with Anna tomorrow at 3pm"), true);
  assert.equal(await evidence("calendar_date_signal", "Explain how a calendar works."), false);
});

test("a surface word alone is not admitted", async () => {
  assert.equal(await admits("tryTerminalCommand", "Make a 3-day itinerary for a first visit to Rome."), false);
  assert.equal(await admits("tryTerminalCommand", "Find the bug: def average(xs): return sum(xs) / len(xs)"), false);
  assert.equal(await admits("trySoftwareProjectRequest", "Write a regex for a US ZIP code with an optional 4-digit extension"), false);
  assert.equal(await admits("tryPageQueryText", "What command builds the jar on this page?"), false);
  assert.equal(await admits("tryJavaScriptExecution", "Run this JavaScript"), false);
  assert.equal(await admits("tryIncompatibleUnits", "What does a unit test check?"), false);
});

test("the structural evidence admits", async () => {
  assert.equal(await admits("tryTerminalCommand", "find . -name '*.log' -size +10M"), true);
  assert.equal(await admits("trySoftwareProjectRequest", "Build a web app for tracking habits"), true);
  assert.equal(await admits("tryPageQueryText", "What command builds the jar?\nRun kotlinc hello.kt -include-runtime -d hello.jar."), true);
  assert.equal(await admits("tryJavaScriptExecution", "Run this JavaScript: console.log(1 + 2)"), true);
  assert.equal(await admits("tryIncompatibleUnits", "How many meters are in a kilobyte?"), true);
});

test("a handler with no claim row is admitted as before", async () => {
  assert.equal(await admits("tryConceptLookup", "What is a monad?"), true);
});
