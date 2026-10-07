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
