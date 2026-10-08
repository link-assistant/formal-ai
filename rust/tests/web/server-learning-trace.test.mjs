// The JavaScript server's self-improvement port (js/server/self-improvement.mjs,
// rust/src/self_improvement.rs): the `learning_trace` a chat completion
// carries when the solver verified a synthesized rule, and the rule proposals
// the conversation `learn` route stages from a recorded dialog.

import assert from "node:assert/strict";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";

import { learnFromConversation } from "../../../js/server/conversations.mjs";
import { stableId } from "../../../js/server/ids.mjs";
import {
  BenchmarkGateReport,
  eventPayload,
  learnFromReportedConversation,
  learnRulesFromUnknownTraces,
  learningRunLinksNotation,
  learningTraceFromSymbolicAnswer,
  unknownTrace,
} from "../../../js/server/self-improvement.mjs";

const PROMPT = "Сделай сортировку результатов в обратном порядке";
const CANDIDATE = [
  "rule_synthesis_candidate",
  "  id reverse_sort_list_files_arg",
  "  source constructed_from_operation_vocabulary",
  "  base_task list_files_arg",
  "  modifier reverse_sort",
  "  operation sort",
  "  operation_modifier descending",
  "  target program:last.output_order",
  "  resolved_task list_files_arg_reverse_sort",
].join("\n");
const VERIFICATION = [
  "rule_verification",
  "  candidate reverse_sort_list_files_arg",
  "  fixture list_files_output_order",
  "  input a.txt,b.txt,c.txt",
  "  expected_order c.txt,b.txt,a.txt",
  "  lowering_check passed",
  "  render_check passed",
  "  status passed",
].join("\n");
// The trace the Rust server (v0.350.0) attaches to this turn's completion.
const RUST_TRACE = {
  events: [
    { kind: "selected_rule", payload: "initial unknown reason no_seed_route next try_rule_synthesis" },
    { kind: "rule_synthesis_candidate", payload: CANDIDATE },
    { kind: "rule_verification", payload: VERIFICATION },
  ],
  prompt: PROMPT,
};

// The worker's trace for the same turn (js/worker/formal_ai_worker_write_program_and_research.js
// `writeProgramDiagnosticBundle`).
const WORKER_STEPS = [
  { step: "impulse", detail: PROMPT },
  { step: "route_attempt", detail: "selected_rule initial unknown reason no_seed_route next try_rule_synthesis" },
  { step: "modifier_detection", detail: "rule_synthesis_operation_vocabulary\n  reverse_sort" },
  { step: "rule_construction", detail: `rule_synthesis_request\n  base_task list_files_arg\n  modifier reverse_sort\n  source_text ${PROMPT}\n${CANDIDATE}` },
  { step: "rule_verification", detail: VERIFICATION },
  { step: "program_plan", detail: "write_program_plan\n  task list_files_arg_reverse_sort" },
];

test("a verified candidate in the worker trace yields the Rust learning_trace", () => {
  assert.deepEqual(learningTraceFromSymbolicAnswer(PROMPT, { worker_steps: WORKER_STEPS }), RUST_TRACE);
  assert.deepEqual(learningTraceFromSymbolicAnswer(PROMPT, { thinking_steps: WORKER_STEPS }), RUST_TRACE);
});

test("the native solver event log is read verbatim once it carries the candidate", () => {
  const solverEvents = [
    { kind: "impulse", payload: PROMPT },
    { kind: "selected_rule", payload: "initial unknown reason no_seed_route next try_rule_synthesis" },
    { kind: "rule_synthesis_candidate", payload: CANDIDATE },
    { kind: "rule_verification", payload: VERIFICATION },
  ];
  assert.deepEqual(learningTraceFromSymbolicAnswer(PROMPT, { solver_events: solverEvents, thinking_steps: [] }), RUST_TRACE);
});

test("no learning_trace without both a candidate and a verification", () => {
  assert.equal(learningTraceFromSymbolicAnswer("x", { thinking_steps: [{ step: "impulse", detail: "x" }] }), null);
  const noVerification = WORKER_STEPS.filter((step) => step.step !== "rule_verification");
  assert.equal(learningTraceFromSymbolicAnswer(PROMPT, { worker_steps: noVerification }), null);
});

test("event_payload stops at the next step and keeps only known fields", () => {
  const links = "step_0 selected_rule write_program reason seed_route; step_1 rule_verification rule_verification\\n  fixture f\\n  noise x\\n  status passed; step_2 response r";
  assert.equal(eventPayload(links, "selected_rule"), "write_program reason seed_route");
  assert.equal(eventPayload(links, "rule_verification"), "rule_verification\n  fixture f\n  noise x\n  status passed");
  assert.equal(eventPayload(links, "rule_synthesis_candidate"), null);
});

test("a reported conversation stages one proposal against an absent gate", () => {
  const staged = learnFromReportedConversation({
    messages: [{ role: "user", content: "ignored when the trace names its prompt" }],
    server_logs: [{ response_body: JSON.stringify({ learning_trace: RUST_TRACE }) }],
  });
  assert.equal(staged.awaiting_human_review, true);
  assert.equal(staged.promoted_ledger, null);
  assert.equal(staged.trace.prompt, PROMPT);
  assert.equal(staged.trace.events.length, 3);
  const [proposal] = staged.learning.proposals;
  assert.equal(staged.learning.proposals.length, 1);
  assert.equal(proposal.rule_id, "reverse_sort_list_files_arg");
  assert.equal(proposal.base_task, "list_files_arg");
  assert.equal(proposal.resolved_task, "list_files_arg_reverse_sort");
  assert.equal(proposal.fixture, "list_files_output_order");
  assert.equal(proposal.adoption, "blocked_by_benchmark");
  assert.equal(proposal.id, stableId("learned_rule", `${staged.trace.id}:reverse_sort_list_files_arg:list_files_arg:reverse_sort:list_files_arg_reverse_sort`));
  assert.equal(
    proposal.summary,
    "Learn `reverse_sort` for `list_files_arg` by rewriting to `list_files_arg_reverse_sort`; fixture `list_files_output_order` passed; benchmark `issue_362_multilingual_coding_modification` is absent (0/4).",
  );
  assert.match(proposal.seed_rule_lino, /^substitution_rules\n {2}id "learned_program_plan_rules"\n {2}rule "reverse_sort_list_files_arg"\n/);
  const lino = learningRunLinksNotation(staged.learning);
  assert.match(lino, /^self_improvement_run\n {2}id "self_improvement_run_[0-9a-f]{16}"\n {2}trace_count "1"\n/);
  assert.match(lino, /benchmark_status "absent"/);
  assert.match(lino, /\n {2}learned_rule\n {4}id "learned_rule_[0-9a-f]{16}"/);
});

test("a failed verification is a rejection, not a proposal", () => {
  const failed = VERIFICATION.replace("status passed", "status failed");
  const trace = unknownTrace(PROMPT, [
    { id: "a", kind: "rule_synthesis_candidate", payload: CANDIDATE },
    { id: "b", kind: "rule_verification", payload: failed },
  ]);
  const run = learnRulesFromUnknownTraces([trace], BenchmarkGateReport.issue362FromCounts(4, 0));
  assert.equal(run.proposals.length, 0);
  assert.deepEqual(run.rejections, [{ trace_id: trace.id, reason: "rule verification did not pass: failed" }]);
  const invalid = unknownTrace(PROMPT, [
    { id: "a", kind: "rule_synthesis_candidate", payload: CANDIDATE.replace("modifier reverse_sort", "modifier reverse sort!") },
    { id: "b", kind: "rule_verification", payload: VERIFICATION },
  ]);
  const gated = learnRulesFromUnknownTraces([invalid], BenchmarkGateReport.issue362FromCounts(4, 0));
  assert.deepEqual(gated.rejections.map((rejection) => rejection.reason), ["invalid modifier `reverse sort!`"]);
  const adoptable = learnRulesFromUnknownTraces([unknownTrace(PROMPT, [
    { id: "a", kind: "rule_synthesis_candidate", payload: CANDIDATE },
    { id: "b", kind: "rule_verification", payload: VERIFICATION },
  ])], BenchmarkGateReport.issue362FromCounts(4, 0));
  assert.equal(adoptable.proposals[0].adoption, "adoptable");
});

test("the learn route counts the proposals of a recorded dialog", () => {
  const home = mkdtempSync(path.join(os.tmpdir(), "formal-ai-learn-"));
  try {
    const dialogs = path.join(home, "dialogs");
    const env = { FORMAL_AI_DIALOG_LOG_DIR: dialogs, FORMAL_AI_MEMORY_PATH: path.join(home, "memory.lino") };
    const record = {
      timestamp_unix_ms: 1,
      dialog_id: "issue-822-learning",
      request_id: "request_1",
      method: "POST",
      path: "/v1/chat/completions",
      request_model: "formal-ai",
      request_tools: [],
      status: 200,
      response_model: null,
      response_tool_calls: [],
      response_content_preview: "",
      request_body: JSON.stringify({ model: "formal-ai", messages: [{ role: "user", content: PROMPT }] }),
      response_body: JSON.stringify({ choices: [], learning_trace: RUST_TRACE }),
    };
    mkdirSync(dialogs, { recursive: true });
    writeFileSync(path.join(dialogs, "issue-822-learning.jsonl"), `${JSON.stringify(record)}\n`);
    const learned = learnFromConversation("issue-822-learning", env);
    assert.equal(learned.learning_trace_found, true);
    assert.equal(learned.rule_proposals, 1);
    assert.equal(learned.awaiting_human_review, true);
    assert.equal(learned.promoted, false);
    assert.deepEqual(Object.keys(learned), [
      "awaiting_human_review", "dialog_id", "events_recorded", "learned", "learning_trace_found", "promoted", "rule_proposals",
    ]);
  } finally {
    rmSync(home, { recursive: true, force: true });
  }
});
