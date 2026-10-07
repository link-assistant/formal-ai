// Server parity for the native solver log (R1013): js/server/solver-log.mjs
// splices the formalization, intent-formalization and meta-core records
// rust/src/solver.rs logs between the worker's prelude and its handler events.
//
// The ids below hash kind, position and payload (rust/src/event_log.rs
// `EventLog::append`), so each pins a payload byte for byte. They were printed
// by the Rust server (`formal-ai serve`, 0.347.0) for "What is 2 + 2?" on the
// records that have not changed since; the records added or reshaped on this
// branch (the need ledger's `planned` count, the obligation ledger, the method
// registry's heuristics and learned methods) are pinned by kind and order, and
// the CI server-parity job compares them against the branch binary.

import assert from "node:assert/strict";
import { before, test } from "node:test";

import { solveSymbolic } from "../../../js/server/solve.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";

let host;
before(async () => {
  host = new WorkerHost();
  await host.boot();
});

const PROMPT = "What is 2 + 2?";

const RUST_PINNED = [
  "prompt:prompt_b888e028816d6739",
  "impulse:impulse_e2d6cfb001a2899c",
  "language:en",
  "candidate:candidate_b5fdef2942a95b50",
  "statement_weight:formalization:0 weight=1.000000 subject=raw:2 + 2 unresolved=2 + 2",
  "policy:temperature_selection:policy:temperature_selection_ca0d529473261fbc",
  "formalization:formalization_434aad92a324c45c",
  "formalization:raw:raw:2 + 2",
  "formalization_unresolved:2 + 2",
  "intent_formalization_cache:miss:impulse_3a78a6188c5f0b7d",
  "intent_formalization:intent_formalization_c43e38ff15c773f8",
  "intent_formalization:kind:question",
  "intent_formalization:route:arithmetic",
  "intent_formalization:relevant:handler:arithmetic",
  "intent_formalization:relevant:handler:concept_lookup",
  "intent_formalization:relevant:route:arithmetic",
  "problem_frame:problem_frame_724851d4a510ccfe",
  "problem_frame:need_count:problem_frame:need_count_9ce5d7e9ff62db68",
  "problem_frame:need:problem_frame:need_0304b1dea6634cbc",
  "work_unit:work_unit_664497d430d9ceea",
  "work_unit:count:work_unit:count_3a5a47a25cd6063b",
  "work_unit:leaf_count:work_unit:leaf_count_81db8f51b10aa350",
  "work_unit:enter:work_unit:enter_86b32f9453c442c4",
  "work_unit:exit:work_unit:exit_a7b2a536e6af8ac2",
];

const RUST_PINNED_AT = {
  25: "need_ledger:accounted_for:need_ledger:accounted_for_8cffe07212ac9385",
  29: "work_unit_reasoning:work_unit_reasoning_20b6bf7c03b138b1",
  30: "work_unit_reasoning:steps:work_unit_reasoning:steps_7c676d567c04f3c5",
  31: "upward_construction:upward_construction_f1e115439e375b0c",
  32: "upward_construction:steps:upward_construction:steps_1e3865f4c6105652",
  34: "solution_evidence:accounted_for:solution_evidence:accounted_for_5a78271258f49634",
  35: "selection:selection_45ef4e0b02f4360b",
  37: "skill_ledger:promotable:skill_ledger:promotable_cc51405b5d05ff12",
  38: "reasoning_standard:reasoning_standard_86d5c1d3e3a0f779",
  39: "reasoning_standard:gates:reasoning_standard:gates_4381cef1224d27ab",
  40: "reasoning_standard:verdict:reasoning_standard:verdict_22f1b510c4974055",
};

const evidence = async (prompt) => (await solveSymbolic({ worker: host }, prompt, [])).evidence_links;
const linkKind = (link) => link.replace(/^(.+):\1_[0-9a-f]{16}$/u, "$1");

test("the formalization, intent and frame records match the Rust server's ids", async () => {
  const links = await evidence(PROMPT);
  assert.deepEqual(links.slice(0, RUST_PINNED.length), RUST_PINNED);
  for (const [index, link] of Object.entries(RUST_PINNED_AT)) assert.equal(links[Number(index)], link, `link ${index}`);
});

test("the meta core, rule selection, handler and method events follow in Rust's order", async () => {
  const links = await evidence(PROMPT);
  assert.deepEqual(links.slice(24).map(linkKind), [
    "need_ledger", "need_ledger:accounted_for", "need:status",
    "method_registry", "method_registry:count", "work_unit_reasoning", "work_unit_reasoning:steps",
    "upward_construction", "upward_construction:steps", "solution_evidence", "solution_evidence:accounted_for",
    "selection", "skill_ledger", "skill_ledger:promotable", "reasoning_standard", "reasoning_standard:gates",
    "reasoning_standard:verdict", "obligation_ledger", "obligation_ledger:discharged", "need_ledger:executed",
    "need_ledger:executed_satisfied", "search:local", "selected_rule", "calculation:request",
    "calculation:engine:link-calculator", "calculation:lino:(2 + 2)", "calculation:steps", "calculation",
    "intent:calculation", "validation", "response:calculation", "trace:simplification", "trace", "method", "derivation:answer_41e931da82c378e5",
    "data/cache/derivations/answer_41e931da82c378e5.lino",
  ]);
});

test("a server answer ends with the method and the derivation links", async () => {
  const answer = await solveSymbolic({ worker: host }, PROMPT, []);
  const tail = answer.evidence_links.slice(-3);
  assert.match(tail[0], /^method:method_[0-9a-f]{16}$/u);
  assert.match(tail[1], /^derivation:answer_[0-9a-f]{16}$/u);
  assert.ok(answer.thinking_steps.some((step) => step.step === "formalize" && step.detail === "arithmetic"));
});

test("a greeting routes to a seeded rule, so no rule synthesis is selected", async () => {
  const links = await evidence("hi");
  assert.ok(links.includes("intent_formalization:route:greeting"), links.join("\n"));
  assert.ok(links.some((link) => link.startsWith("problem_frame:")));
  assert.ok(!links.some((link) => link.startsWith("selected_rule:")));
});
