// Issue #918 (E71, R914-6), browser root: `conversation_topic`,
// `source_refresh` and `source_conflict` answer through the same
// data/seed/handler-rules.lino rule sets the native interpreter walks, with
// the `role_slot` and `stable_id` value sources and the `evidence` condition
// in js/worker/formal_ai_worker_handler_rules.js. Prompts and expected
// wording are the ones rust/tests/unit/issue_918_handler_rules_batch.rs
// asserts against the native engine.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await ready;
  return worker.solve(prompt, [], {}, {}, [], {});
}

async function rule(name, prompt) {
  await ready;
  return plain(evaluate(worker,
    `runHandlerRuleSet(${JSON.stringify(name)}, ${JSON.stringify(prompt)}, ${JSON.stringify(prompt.toLowerCase())}, [])`));
}

const TOPIC_EN = "We can talk about existence. I can start with a short definition, context, or a specific question; when web search is available, public facts can be checked against an external source.";
const TOPIC_RU = "Можем. Тема: бытие. Я могу начать с краткого определения, контекста или конкретного вопроса; если веб-поиск доступен, публичные факты можно уточнить через внешний источник.";
const TOPIC_HI = "हम बात कर सकते हैं. विषय: गणित. मैं छोटी परिभाषा, संदर्भ, या किसी ठोस प्रश्न से शुरू कर सकता हूँ; web search उपलब्ध हो तो public facts बाहरी स्रोत से जाँचे जा सकते हैं.";
const TOPIC_ZH = "可以聊。主题: 音乐。我可以从简短定义、上下文或具体问题开始; 如果 web search 可用, 公开事实可以通过外部来源核对。";
const CONFLICT_EN = "Sources disagree on this question. The disagreement is recorded as a conflict:source_disagreement link in the network rather than silently resolved.";
const REFRESH_EN = "Cached source source_e2db54b48c90e140 has been queued for refresh against its origin URL. The refresh event is appended to the audit log and a fresh fetched_at timestamp will be recorded once the new copy is verified.";
const REFRESH_UNNAMED_EN = "Cached source source_2a324f9681a3e3bd has been queued for refresh against its origin URL. The refresh event is appended to the audit log and a fresh fetched_at timestamp will be recorded once the new copy is verified.";

const EXECUTION_FAILURE = "Execution status: failed in isolated sandbox.\n```python\nundefined_function()\n```\nTraceback (most recent call last):\n  File 'main.py', line 1, in <module>\nNameError: name 'undefined_function' is not defined.\nThe failure trace is appended to the action log; see the trace link.";
const UNITS_EN = "meters measures length; kilogram measures mass. These are different physical dimensions and cannot be converted into each other. The incompatibility is recorded as a `unit_incompatibility` link in the network.";
const UNITS_RU = "метр measures length; килограмм measures mass. These are different physical dimensions and cannot be converted into each other. The incompatibility is recorded as a `unit_incompatibility` link in the network.";
const KUPI_EN = "Buy an elephant is a well-known Russian children's word game. Whatever you reply, the answer comes back: everyone says that, but you buy an elephant! The traditional winning reply is: everyone has an elephant, but I do not.";
const KUPI_RU = "«Купи слона» — это известная русская детская фраза-игра. На любой ответ следует продолжение: «Все так говорят, а ты купи слона!» Правильный ответ по правилам игры: «У всех есть слон, а у меня нет».";

test("english and russian conversation topics answer unchanged", async () => {
  for (const [prompt, expected] of [
    ["Let's talk about existence", TOPIC_EN],
    ["Поговорим о бытие", TOPIC_RU],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "conversation_topic", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("hindi and chinese openers capture their slot from the seed rule", async () => {
  for (const [prompt, expected, topic] of [
    ["चलो बात करें गणित", TOPIC_HI, "गणित"],
    ["聊聊音乐", TOPIC_ZH, "音乐"],
  ]) {
    const response = await rule("conversation_topic", prompt);
    assert.equal(response.content, expected, prompt);
    assert.ok(response.evidence.includes(`conversation_topic:${topic}`), prompt);
  }
});

test("the topic claim is the rule and an empty slot is no topic", async () => {
  await ready;
  const claims = (prompt) => evaluate(worker,
    `CLASS_CLAIM_EVIDENCE.conversation_topic_subject(${JSON.stringify(prompt)})`);
  assert.equal(claims("Let's talk about existence"), true);
  assert.equal(claims("Let's talk about ?!"), false);
  assert.equal(claims("What is the capital of France?"), false);
});

test("a source conflict answers unchanged", async () => {
  const response = await solve("Was X born in 1880 or 1881?");
  assert.equal(response.intent, "source_conflict");
  assert.equal(response.content, CONFLICT_EN);
  assert.ok(response.evidence.some((link) => link.startsWith("conflict:source_disagreement")));
});

test("a source refresh answers unchanged and an unnamed one takes the refusal lane", async () => {
  const named = await solve("Refresh the cached page for example.com");
  assert.equal(named.intent, "source_refresh");
  assert.equal(named.content, REFRESH_EN);

  const unnamed = await rule("source_refresh", "Refresh the cache");
  assert.equal(unnamed.intent, "source_refresh");
  assert.equal(unnamed.content, REFRESH_UNNAMED_EN);
  assert.ok(unnamed.evidence.includes("source_refresh:refusal:no source named"));

  const url = await rule("source_refresh", "Refresh the cached page https://example.com/docs");
  assert.ok(!url.evidence.some((link) => link.startsWith("source_refresh:refusal")));
});

test("an execution failure answers unchanged in chat and agent mode", async () => {
  const chat = await solve("Write a Python script that calls undefined_function()");
  assert.equal(chat.intent, "execution_failure");
  assert.equal(chat.content, EXECUTION_FAILURE);
  const agent = await solve("[agent] Run a Python script that calls undefined_function()");
  assert.equal(agent.content, EXECUTION_FAILURE);
  assert.ok(agent.evidence.some((link) => link.startsWith("agent_mode:opted_in:")));
});

test("an incompatible unit pair answers unchanged from the seeded wording", async () => {
  for (const [prompt, expected] of [
    ["How many meters are in a kilogram?", UNITS_EN],
    ["Сколько метров в килограмме?", UNITS_RU],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "unit_incompatibility", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

test("the buy-an-elephant idiom answers through its seed rule in the prompt language", async () => {
  // The browser used to answer every language with the Russian text inline;
  // the rule renders the seeded response the native interpreter renders.
  for (const [prompt, expected] of [
    ["Hey, buy an elephant!", KUPI_EN],
    ["Ну купи слона, пожалуйста", KUPI_RU],
  ]) {
    const response = await solve(prompt);
    assert.equal(response.intent, "kupi_slona", prompt);
    assert.equal(response.content, expected, prompt);
  }
});

const RESEARCH_SEARCH = "Search for information about:\n1. Machine learning algorithms\n2. Deep learning vs traditional ML\n3. Neural networks basics";
const RESEARCH_TABLE = "Research comparison table (draft; verify claims against the source links from the preceding retrieval).\n\n| Topic | Key differences | Use cases | Advantages | Disadvantages |\n| --- | --- | --- | --- | --- |\n| Machine learning algorithms | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |\n| Deep learning vs traditional ML | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |\n| Neural networks basics | Extract from the preceding source captures what distinguishes this topic from the others. | Extract the practical settings in which the preceding sources apply this topic. | Extract strengths supported by the preceding source captures; leave unsupported claims unverified. | Extract limitations supported by the preceding source captures; leave unsupported claims unverified. |";
const RESEARCH_TASK = "Research task: What would be the economic impact if Rust replaced C++ in all major open-source projects by 2030?\nSteps required:\n1. Search for current C++ vs Rust usage statistics in open-source projects.";
const RESEARCH_PREVIEW = "Research task: What would be the economic impact if Rust replaced C++ in all major open-source projects by 2030? Steps required: 1. Search for current C++ vs Rust usage statistics in open-source projects.";
const RESEARCH_NO_RESULTS = `The result of the previous research step is: no CORS-readable web search results were returned. I do not have verified source data to complete the requested analysis, calculation, table, or sources list yet.\n\nPrior research task: \`${RESEARCH_PREVIEW}\`\n\nNext step: rerun the search with narrower queries or provide source links; then I can calculate the requested impact from those sources.`;
const RESEARCH_OPEN = `There is no verified final research result in the conversation yet. The prior turn was a research request, but I do not see a completed source-backed answer to report.\n\nPrior research task: \`${RESEARCH_PREVIEW}\`\n\nNext step: run the search or provide source links; then I can produce the requested result.`;

test("a research comparison table renders the seeded procedure, never a memorized fact", async () => {
  const search = await solve(RESEARCH_SEARCH);
  await ready;
  const response = await worker.solve(
    "create a comparison table showing:\n- Key differences\n- Use cases for each\n- Advantages and disadvantages",
    [{ role: "user", content: RESEARCH_SEARCH }, { role: "assistant", content: search.content }],
    {}, {}, [], {},
  );
  assert.equal(response.intent, "research_comparison_table");
  assert.equal(response.content, RESEARCH_TABLE);
});

test("a research result follow-up reads its status and wording from the seed", async () => {
  await ready;
  for (const [priorAnswer, expected, status] of [
    ["No CORS-enabled web search results were returned for `x`.\n\nProviders tried: DuckDuckGo.", RESEARCH_NO_RESULTS, "no_results"],
    ["Here is a summary I wrote.", RESEARCH_OPEN, "open_research"],
  ]) {
    const response = await worker.solve(
      "What is the result?",
      [{ role: "user", content: RESEARCH_TASK }, { role: "assistant", content: priorAnswer }],
      {}, {}, [], {},
    );
    assert.equal(response.intent, "research_result_followup", status);
    assert.equal(response.content, expected, status);
    assert.ok(response.evidence.includes(`research_result_followup:status:${status}`), status);
  }
});

test("the network query rules answer the snapshot, the introspection and the user filter", async () => {
  const snapshot = await solve("Export the network");
  assert.equal(snapshot.intent, "network_snapshot");
  await ready;
  const loaded = evaluate(worker, "networkSnapshotLinksNotation()");
  assert.equal(snapshot.content,
    `Here is the current link network as a links-notation snapshot:\n\n\`\`\`links\n${loaded}\n\`\`\``);
  const introspection = await rule("network_query", "What do you know about 'greeting'?");
  assert.equal(introspection.intent, "concept_introspection_greeting");
  assert.equal(introspection.content,
    "Here is what I know about 'greeting':\n\nintent: greeting\nrole: the network records 'greeting' as a concept with rules and example links.");
  const filter = await solve("List the facts I have contributed");
  assert.equal(filter.intent, "filter_user");
  assert.equal(filter.content,
    "No facts have been recorded under your user filter yet. Submit a 'teach this fact' request to start your personal contribution list.");
  assert.ok(filter.evidence.includes("filter:user:self"));
});
