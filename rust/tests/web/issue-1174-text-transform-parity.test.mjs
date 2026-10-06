// Issue #1188 JS parity for the issue #1174 text-transform family: the browser
// worker answers free-text summarization (summarization_text) and register /
// grammar / genre rewriting (text_rewrite) with the same intents and bodies as
// rust/tests/unit/issue_1174_text_transform.rs pins for the native handlers.
// Every probe below is one of the native test's prompts.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await seeded;
  return worker.solve(prompt, [], {}, {}, [], {});
}

test("summarization selects the weightiest statements", async () => {
  const answer = await solve(
    "Summarize this paragraph: The Halley research station, opened in 1956, is used to study " +
      "the Antarctic ice shelf. It provides year-round measurements of ozone and sea " +
      "temperature. The crew rotates every summer. Supplies arrive by ship in February. Radar " +
      "masts surround the living quarters.",
  );
  assert.equal(answer.intent, "summarization_free_text");
  assert.equal(
    answer.content,
    "The Halley research station, opened in 1956, is used to study the Antarctic ice shelf. " +
      "It provides year-round measurements of ozone and sea temperature.",
  );
  assert.ok(!answer.content.includes("Radar masts"));
});

test("summarization declines a prompt without text", async () => {
  const declined = evaluate(worker, 'trySummarizationText("Summarize this.", normalizePrompt("Summarize this."))');
  assert.equal(declined, null);
});

test("register rewrite substitutes every informal token", async () => {
  const answer = await solve("Rewrite this formally: can u send me the report asap thx");
  assert.equal(answer.intent, "text_transform_register");
  assert.equal(answer.content, "could you please send me the report as soon as possible thank you");
});

test("register rewrite reports an already formal text", async () => {
  const answer = await solve("Rewrite this formally: please send the report tomorrow");
  assert.equal(answer.intent, "text_transform_register");
  assert.equal(
    answer.content,
    "please send the report tomorrow\n\nThe text already uses a formal register.",
  );
});

test("grammar correction re-agrees subjects and pluralizes", async () => {
  const answer = await solve("Correct the grammar: She don't like apples and he have two cat.");
  assert.equal(answer.intent, "text_transform_grammar");
  assert.equal(
    answer.content,
    "Corrected text: She doesn't like apples and he has two cats.\n\nCorrections:\n" +
      "don't → doesn't (rule: third_person_do_not)\n" +
      "have → has (rule: third_person_have)\n" +
      "cat → cats (rule: plural_noun_after_numeral)",
  );
});

test("grammar correction reports an already clean text", async () => {
  const answer = await solve("Correct the grammar: She likes apples and he has two cats.");
  assert.equal(answer.intent, "text_transform_grammar");
  assert.ok(answer.content.includes("already grammatical"), answer.content);
});

test("commit message composes the conventional line", async () => {
  const answer = await solve(
    "Write a commit message: correct an off-by-one error in the pagination helper",
  );
  assert.equal(answer.intent, "text_transform_genre_commit_message");
  assert.equal(answer.content, "fix(pagination): correct off-by-one error");
});

test("commit message names its missing slots", async () => {
  const answer = await solve("Write a commit message: in the pagination helper");
  assert.equal(answer.intent, "text_transform_genre_commit_message");
  assert.ok(answer.content.includes("missing") && answer.content.includes("verb"), answer.content);
});

test("email composes every declared part", async () => {
  const answer = await solve(
    "Write an email to my team: I am taking a day off on Friday because I am tired.",
  );
  assert.equal(answer.intent, "text_transform_genre_email");
  for (const part of [
    "Hello team,",
    "I am writing to let you know that I am taking a day off on Friday.",
    "The reason is that I am tired.",
    "When: Friday",
    "Best regards,",
  ]) {
    assert.ok(answer.content.includes(part), `${part} in ${answer.content}`);
  }
});
