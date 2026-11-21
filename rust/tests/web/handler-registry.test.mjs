// Browser handler precedence is seed data, not a second JavaScript inventory
// (issue #1138 B9, plan 09).

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { test } from "node:test";

import {
  createBrowserContext,
  createWorkerContext,
  evaluate,
  loadBrowserScript,
  plain,
  REPO_ROOT,
} from "./support/browser-runtime.mjs";

test("reordering the seed reorders synchronous browser dispatch", () => {
  const context = createBrowserContext();
  loadBrowserScript(context, "js/seed_loader.js");
  loadBrowserScript(context, "js/worker/formal_ai_worker_dispatch.js");
  context.firstHandler = (prompt) => ({ intent: "first", content: prompt, evidence: [] });
  context.secondHandler = (prompt) => ({ intent: "second", content: prompt, evidence: [] });

  const parse = (source) => context.FormalAiSeed.extractBrowserHandlerPrecedence(
    context.FormalAiSeed.parse(source),
  );
  context.testContext = { prompt: "from seed" };
  context.testRegistry = parse(
    "browser-handler-precedence\n  handler firstHandler\n    argument_prompt\n  handler secondHandler\n    argument_prompt\n",
  );
  assert.deepEqual(
    plain(evaluate(context, "synchronousHandlerCandidates(testContext, testRegistry).map((entry) => entry.name)")),
    ["firstHandler", "secondHandler"],
  );
  assert.equal(
    evaluate(context, "synchronousHandlerCandidates(testContext, testRegistry)[0].run().intent"),
    "first",
  );

  context.testRegistry = parse(
    "browser-handler-precedence\n  handler secondHandler\n    argument_prompt\n  handler firstHandler\n    argument_prompt\n",
  );
  assert.deepEqual(
    plain(evaluate(context, "synchronousHandlerCandidates(testContext, testRegistry).map((entry) => entry.name)")),
    ["secondHandler", "firstHandler"],
  );
  assert.equal(
    evaluate(context, "synchronousHandlerCandidates(testContext, testRegistry)[0].run().intent"),
    "second",
  );
});

test("the shipped browser registry resolves every seed-declared implementation", async () => {
  const worker = createWorkerContext();
  await evaluate(worker, "init()");
  const source = readFileSync(
    path.join(REPO_ROOT, "data/seed/browser-handler-precedence.lino"),
    "utf8",
  );
  const declared = worker.FormalAiSeed.extractBrowserHandlerPrecedence(
    worker.FormalAiSeed.parse(source),
  );

  assert.ok(declared.length > 0, "the shipped seed declares browser handlers");
  assert.deepEqual(
    plain(evaluate(worker, "browserHandlerPrecedence().map((entry) => entry.name)")),
    plain(declared.map((entry) => entry.name)),
    "worker startup installs the seed order byte-for-byte",
  );
});

test("named argument lists preserve order, context values and inline fields", () => {
  const context = createBrowserContext();
  loadBrowserScript(context, "js/seed_loader.js");
  loadBrowserScript(context, "js/worker/formal_ai_worker_dispatch.js");
  context.firstHandler = (...values) => ({ intent: "first", content: values, evidence: [] });
  const parse = (source) => context.FormalAiSeed.extractBrowserHandlerPrecedence(context.FormalAiSeed.parse(source));
  const source = `browser-handler-precedence
  arguments calendar
    argument_prompt
    argument_normalized
    argument_userContext
  handler firstHandler
    arguments calendar
    result_intent_first
`;
  const before = parse(source.replace("    arguments calendar", "    argument_prompt\n    argument_normalized\n    argument_userContext"));
  context.testRegistry = parse(source);
  assert.deepEqual(plain(context.testRegistry), plain(before));
  context.testContext = { prompt: "request", normalized: "normalized", userContext: { timezone: "UTC" } };
  assert.deepEqual(plain(evaluate(context, "synchronousHandlerCandidates(testContext, testRegistry)[0].run().content")),
    ["request", "normalized", { timezone: "UTC" }]);
  assert.throws(() => parse(source.replace("    arguments calendar", "    arguments missing")), /unknown browser handler arguments: missing/);
});
