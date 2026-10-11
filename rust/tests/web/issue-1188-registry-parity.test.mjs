// PR #1188 (JavaScript-first parity): every precedence row that used to be
// "native surface only" for the families ported here now resolves to a worker
// function, so the browser registry binds the same handler vocabulary the
// native table runs.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const PORTED = {
  summarization_text: "trySummarizationText",
  text_rewrite: "tryTextRewrite",
  word_problem: "tryWordProblem",
  statistics: "tryStatistics",
  unit_conversion: "tryUnitConversion",
  number_constraint_reasoning: "tryNumberConstraintReasoning",
  triz_resolution: "tryTrizResolution",
  formalization_request: "tryFormalizationRequest",
  legality_warning: "tryLegalityWarning",
  product_search: "tryProductSearch",
  opinion_question: "tryOpinionQuestion",
  physical_action_question: "tryPhysicalActionQuestion",
  punctuation_only_prompt: "tryPunctuationOnlyPrompt",
  ill_formed: "tryIllFormed",
  clarification: "tryClarification",
  translation: "tryTranslation",
  response_language_followup: "tryResponseLanguageFollowup",
};

test("the ported precedence rows bind worker functions", async () => {
  const worker = createWorkerContext();
  await evaluate(worker, "loadSeed()");
  const registry = plain(evaluate(worker, "WORKER_HANDLER_REGISTRY.workerHandlers"));
  for (const [slug, name] of Object.entries(PORTED)) {
    assert.equal(registry[slug], name, slug);
    assert.equal(evaluate(worker, `typeof ${name}`), "function", name);
  }
});
