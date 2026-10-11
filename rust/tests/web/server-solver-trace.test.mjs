// Server parity for the thinking trace (R1013, R1015): the worker records the
// events the native solver logs for a turn (`solverEvents`,
// js/worker/formal_ai_worker_solver_events.js) and the JavaScript server
// projects them through js/server/solver-trace.mjs, a port of
// `EventLog::thinking_steps_for_answer`. The expected values below are what
// the Rust server answers for the same prompts.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import { before, test } from "node:test";

import { REPO_ROOT, WorkerHost } from "../../../js/server/worker-host.mjs";
import { symbolicFromWorker } from "../../../js/server/solve.mjs";
import { curateThinkingEvent, thinkingStepsFromEvents } from "../../../js/server/solver-trace.mjs";
import { renderThinkingSteps } from "../../../js/server/thinking.mjs";

let host;
before(async () => {
  host = new WorkerHost();
  await host.boot();
});

const RUST_TRACE_2_PLUS_2 = [
  ["thinking_step_6de6c412b8eec16b", "impulse", "What is 2 + 2?", "high", "impulse", null],
  ["thinking_step_98d6c31b7eee9797", "detect_language", "en", "high", "language", null],
  ["thinking_step_607dae06d32b99b9", "formalize", "arithmetic", "high", "intent_formalization:route", null],
  ["thinking_step_42b3515d5c29cbf2", "compute", "2 + 2 = 4", "high", "calculation", null],
  ["thinking_step_a852e25ee0231f35", "compute_engine", "link-calculator", "detailed", "calculation:engine", "thinking_step_42b3515d5c29cbf2"],
  ["thinking_step_1c4660a93c99db11", "compute_expression", "(2 + 2)", "detailed", "calculation:lino", "thinking_step_42b3515d5c29cbf2"],
  ["thinking_step_285fee20b1b3bd4a", "compute_steps", "6", "detailed", "calculation:steps", "thinking_step_42b3515d5c29cbf2"],
  ["thinking_step_d864a35f66b0d864", "dispatch_handler", "calculation", "high", "intent", null],
  ["thinking_step_e8f877244b091572", "rule_verification", "", "detailed", "validation", null],
  ["thinking_step_526ba4577552410b", "deformalize", "2 + 2 = 4", "high", "response", null],
];

const RUST_REASONING_2_PLUS_2 = [
  "This was a calculation, so I worked it out step by step and checked the result.",
  'Read the request: "What is 2 + 2?".',
  "Detect the request language: English.",
  "Formalize the request as an arithmetic task.",
  "Compute 2 + 2 = 4.",
  "  ↳ Evaluate with the link calculator.",
  "  ↳ Reduce the expression (2 + 2).",
  "  ↳ Apply 6 reduction step(s).",
  "Route to the calculation handler.",
  "Verify the result against the rules.",
  'Compose the answer: "2 + 2 = 4".',
].join("\n");

test("the arithmetic turn projects the exact Rust thinking trace", async () => {
  const symbolic = symbolicFromWorker(await host.solve("What is 2 + 2?"));
  assert.deepEqual(
    symbolic.thinking_steps.map((step) => [step.id, step.step, step.detail, step.level, step.source_event, step.parent_id ?? null]),
    RUST_TRACE_2_PLUS_2,
  );
  assert.equal(renderThinkingSteps(symbolic.thinking_steps), RUST_REASONING_2_PLUS_2);
});

// Rust `link-calculator` `to_lino` and `evaluate_with_steps` lengths, as the
// native server reports them in `compute_expression` / `compute_steps`.
const CALCULATOR_CASES = [
  ["2 + 2", "(2 + 2)", 6],
  ["(3 + 4) * 2", "((3 + 4) * 2)", 10],
  ["2^3^2", "(2 ^ (3 ^ 2))", 9],
  ["-5 + 3", "((-5) + 3)", 7],
  ["10 / 4", "(10 / 4)", 6],
  ["7 % 3", "(7 % 3)", 6],
  ["1.50 + 2", "(1.5 + 2)", 6],
  ["-(2 + 3)", "(-(2 + 3))", 8],
  ["2 * (3 + (4 - 1))", "(2 * (3 + (4 - 1)))", 14],
  ["12 - 4 - 2", "((12 - 4) - 2)", 9],
];

test("the link-calculator mirror renders the crate's lino and step count", async () => {
  for (const [expression, lino, steps] of CALCULATOR_CASES) {
    assert.deepEqual(await host.run("linkCalculatorTrace(__expression)", { __expression: expression }), { lino, steps }, expression);
  }
  for (const outside of ["2 x 3", "5%", "sqrt(4)", "", "2 +"]) {
    assert.equal(await host.run("linkCalculatorTrace(__expression)", { __expression: outside }), null, outside);
  }
});

test("routes come from the intent table and the seeded handler promotions", async () => {
  assert.equal(await host.run("solverIntentRoute(__prompt)", { __prompt: "What is 2 + 2?" }), "arithmetic");
  assert.equal(await host.run("solverIntentRoute(__prompt)", { __prompt: "hi" }), "greeting");
});

test("every promotion names a method the native registry resolves", () => {
  const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");
  const promoted = [...read("data/seed/handler-promotions.lino").matchAll(/^ {2}promotion (\S+)$/gm)].map((m) => m[1]);
  const registered = new Set([
    ...[...read("data/seed/handler-precedence.lino").matchAll(/^ {2}handler (\S+)$/gm)].map((m) => m[1]),
    ...[...read("data/meta/route-method-aliases.lino").matchAll(/^ {2}route "([^"]+)"$/gm)].map((m) => m[1]),
  ]);
  const dispatch = read("rust/src/solver_dispatch.rs");
  for (const constant of ["CONTEXTUAL_HANDLER_NAMES", "PRELUDE_METHOD_NAMES"]) {
    const block = dispatch.slice(dispatch.indexOf(`pub const ${constant}`));
    for (const match of block.slice(0, block.indexOf("];")).matchAll(/"([a-z_]+)"/g)) registered.add(match[1]);
  }
  assert.ok(promoted.length > 0);
  for (const handler of promoted) assert.ok(registered.has(handler), handler);
});

test("the curated projection keeps Rust's allowlist and folds the calculator", () => {
  assert.equal(curateThinkingEvent("trace", "x", ""), null);
  assert.deepEqual(curateThinkingEvent("validation", "accepted_without_extra_constraints", "").detail, "");
  assert.equal(curateThinkingEvent("tool_fetch", "a", "").step, "invoke_tool");
  assert.equal(curateThinkingEvent("response", "response:x", "final").detail, "final");
  const steps = thinkingStepsFromEvents([
    { kind: "impulse", payload: "p" },
    { kind: "impulse", payload: "p" },
    { kind: "calculation:request", payload: "1/0" },
    { kind: "intent", payload: "calculation_error" },
  ]);
  assert.deepEqual(steps.map((step) => [step.order, step.step, step.detail]), [
    [0, "impulse", "p"],
    [1, "compute", "1/0"],
    [2, "dispatch_handler", "calculation_error"],
  ]);
});
