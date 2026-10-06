// The JavaScript server answers from the memory store ahead of the solver,
// as rust/src/protocol.rs does through rust/src/protocol_memory.rs
// (`answer_from_memory_if_requested`) and rust/src/dreaming_application.rs
// (`solve_with_standing_requirements`, `apply_retained_amendments`): recall
// reports, memory inspection, and retained standing requirements, with the
// thinking steps the native `finalize_simple` event log curates.

import assert from "node:assert/strict";
import { test } from "node:test";

import { createChatCompletion } from "../../../js/server/openai.mjs";
import { createResponse } from "../../../js/server/responses.mjs";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { EventLog, memoryEventToLinkRecord } from "../../../js/server/memory-answer.mjs";
import {
  answerFromPrelearnedCache,
  retainedAmendments,
  topicMatches,
} from "../../../js/server/standing-requirements.mjs";
import { stableId } from "../../../js/server/ids.mjs";

const event = (fields) => ({
  id: "",
  kind: null,
  role: null,
  intent: null,
  tool: null,
  inputs: null,
  outputs: null,
  content: null,
  sent_at: null,
  demo_label: null,
  conversation_id: null,
  conversation_title: null,
  evidence: [],
  unknown_fields: [],
  access_count: 0,
  write_count: 1,
  ...fields,
});

const EVENTS = [
  event({
    id: "recall_1",
    kind: "message",
    role: "user",
    content: "Zebra stripes are my favourite pattern",
    conversation_id: "parity-recall",
    conversation_title: "Parity recall",
  }),
  event({ id: "recall_2", kind: "message", role: "user", content: "Quokka smiles make my day" }),
  event({
    id: "amendment_1",
    kind: "meta_algorithm_amendment",
    inputs: "topic=quokka",
    outputs: "rule=Mention that quokkas live on Rottnest Island.",
  }),
  event({
    id: "amendment_2",
    kind: "meta_algorithm_amendment",
    inputs: "topic=arithmetic",
    outputs: "rule=Show the operands before the result.",
  }),
];

const worker = new WorkerHost();
const ctx = { worker, agentMode: false, memory: { events: () => EVENTS } };

const chat = (content, history = []) =>
  createChatCompletion(ctx, {
    model: "formal-ai",
    messages: [...history, { role: "user", content }],
    tools: [],
    functions: [],
    tool_choice: null,
    function_call: null,
  });

test("a recall prompt reports every matching field and link, grouped by conversation", async () => {
  const completion = await chat("Did I mention zebra stripes?");
  const message = completion.choices[0].message;
  assert.equal(
    message.content,
    [
      'Found 2 mention(s) of "zebra stripes" across 1 conversation(s) in memory.',
      "- conversation Parity recall (parity-recall)",
      "  - user: Zebra stripes are my favourite pattern",
      "  - link: field:content -> value:Zebra stripes are my favourite pattern",
    ].join("\n"),
  );
  assert.deepEqual(
    message.thinking_steps.map((step) => `${step.step}|${step.source_event}`),
    ["impulse|impulse", "dispatch_handler|intent", "rule_verification|validation", "deformalize|response"],
  );
  assert.equal(message.thinking_steps[1].detail, "conversation_recall");
});

test("a recall with no match says so, and request history joins the searched memory", async () => {
  const missing = await chat("Did I mention platypus eggs?");
  assert.equal(missing.choices[0].message.content, 'No mentions of "platypus eggs" found in memory.');
  const withHistory = await chat("Did I mention platypus eggs?", [
    { role: "user", content: "Platypus eggs are leathery" },
    { role: "assistant", content: "Noted." },
  ]);
  const content = withHistory.choices[0].message.content;
  assert.match(content, /^Found 2 mention\(s\) of "platypus eggs" across 1 conversation\(s\) in memory\./);
  assert.match(content, /- conversation Current request \(request_history\)\n {2}- user: Platypus eggs are leathery/);
});

test("a recall answer carries a matching retained requirement and its evidence", async () => {
  const response = await createResponse(ctx, { model: "formal-ai", input: "Did I mention quokka smiles?" });
  const text = response.output.filter((item) => item.type === "message")[0].content[0].text;
  assert.ok(text.endsWith("\n\nLearned standing requirement (quokka): Mention that quokkas live on Rottnest Island."));
  assert.ok(response.evidence_links.includes("meta_algorithm_amendment:amendment_1"));
  assert.ok(response.evidence_links.includes("intent:conversation_recall"));
  assert.ok(response.evidence_links.includes("response:conversation_recall"));
  assert.equal(response.evidence_links[0], `prompt:${stableId("prompt", "Did I mention quokka smiles?")}`);
});

test("memory inspection counts the projected links of the Rust record graph", async () => {
  const completion = await chat("How many links are in your memory?");
  const links = EVENTS.reduce((total, item, index) => total + memoryEventToLinkRecord(item, index).links.length, 0);
  assert.match(completion.choices[0].message.content, new RegExp(`\\b${EVENTS.length}\\b[\\s\\S]*\\b${links}\\b`));
});

test("a solved task restates a matching standing requirement", async () => {
  const completion = await chat("Arithmetic: what is 2 + 2?");
  assert.ok(
    completion.choices[0].message.content.endsWith(
      "\n\nLearned standing requirement (arithmetic): Show the operands before the result.",
    ),
  );
  const unrelated = await chat("What is 2 + 2?");
  assert.ok(!unrelated.choices[0].message.content.includes("standing requirement"));
});

test("topic matching and amendment reading mirror dreaming_application.rs", () => {
  assert.ok(topicMatches("format this LaTeX table with proper formatting", "latex formatting"));
  assert.ok(!topicMatches("formatter", "format"));
  assert.ok(topicMatches("Arithmetic: 2 + 2", "arithmetic"));
  const amendments = retainedAmendments([
    ...EVENTS,
    event({ id: "amendment_0", kind: "meta_algorithm_amendment", inputs: "topic=quokka", outputs: "rule=Mention that quokkas live on Rottnest Island." }),
    event({ id: "legacy", kind: "meta_algorithm_amendment", demo_label: "legacy", content: "Old rule" }),
  ]);
  assert.deepEqual(
    amendments.map((item) => `${item.topic}:${item.id}`),
    ["arithmetic:amendment_2", "legacy:legacy", "quokka:amendment_0"],
  );
});

test("the pre-learned cache answers an unexpired anticipation alias", () => {
  const normalize = (text) => String(text).toLowerCase().replace(/[^\p{L}\p{N}]+/gu, " ").trim();
  const source = event({
    id: "anticipation_1",
    kind: "anticipation_source",
    inputs: "Weather today?",
    outputs: "Sunny.",
    content: 'expires_at 200\nprediction "weather"\nsource_url "https://example.org"',
  });
  const answer = answerFromPrelearnedCache(normalize, "weather today", [source], 100);
  assert.equal(answer.intent, "anticipation_cache");
  assert.equal(answer.answer, "Sunny.");
  assert.ok(answer.evidence_links.includes("cache_hit:https://example.org"));
  assert.equal(answerFromPrelearnedCache(normalize, "weather today", [source], 300), null);
});

test("event ids follow EventLog::append", () => {
  const log = new EventLog();
  const id = log.append("impulse", "hi");
  assert.equal(id, stableId("impulse", "impulse:0:hi"));
});
