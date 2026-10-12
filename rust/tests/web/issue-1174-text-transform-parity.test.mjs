// Issue #1188 JS parity for the issue #1174 text-transform family: the browser
// worker answers free-text summarization (summarization_text, the dependency
// summary of R1188-U21) and register /
// grammar / genre rewriting (text_rewrite) with the same intents and bodies as
// rust/tests/unit/web-engine-core/issue_1174_text_transform.rs pins for the native handlers.
// Every probe below is one of the native test's prompts.

import assert from "node:assert/strict";
import test from "node:test";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

async function solve(prompt) {
  await seeded;
  return worker.solve(prompt, [], {}, {}, [], {});
}

/**
 * Free-text prompts and the dependency summaries both runtimes answer
 * (R1188-U21): the worker runs js/agentic/crate/dependency_summarization.mjs
 * itself (formal_ai_worker_crate_modules.js), and
 * rust/tests/unit/web-engine-core/issue_1174_text_transform.rs pins the same answers natively.
 */
const SUMMARIES = [
  [
    "Summarize this paragraph: The Halley research station, opened in 1956, is used to study " +
      "the Antarctic ice shelf. It provides year-round measurements of ozone and sea " +
      "temperature. The crew rotates every summer. Supplies arrive by ship in February. Radar " +
      "masts surround the living quarters.",
    "The Halley research station opened in 1956 is used to study the Antarctic ice shelf.",
  ],
  ["Summarize: The parser reads the file. It builds a tree. The tree is checked.", "The parser reads the file."],
  ["Резюмируй: Парсер читает файл. Он строит дерево. Дерево проверяется.", "Парсер читает файл."],
  [
    "संक्षेप में लिखें: पार्सर फ़ाइल पढ़ता है। वह एक पेड़ बनाता है। पेड़ जाँचा जाता है।",
    "पार्सर फ़ाइल पढ़ता है।",
  ],
  ["总结一下：解析器读取文件。它构建一棵树。树被检查。", "解析器读取文件。"],
  [
    "Resume esto: El analizador lee el archivo. Construye un árbol. El árbol se comprueba.",
    "El analizador lee el archivo.",
  ],
];

test("summarization keeps the statements the text depends on", async () => {
  for (const [prompt, summary] of SUMMARIES) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "summarization_free_text", prompt);
    assert.equal(answer.content, summary, prompt);
  }
});

test("summarization traces every statement and duplicate", async () => {
  const answer = await solve("Summarize: The parser reads the file. The parser reads the file. It builds a tree.");
  assert.equal(answer.content, "The parser reads the file.");
  const traced = plain(answer.evidence).filter((link) => link.startsWith("summarization_"));
  assert.deepEqual(traced, [
    "summarization_statement:kept The parser reads the file.",
    "summarization_statement:dropped It builds a tree.",
    "summarization_duplicate:The parser reads the file.",
    "summarization_bound:1/2",
    "summarization_selected:The parser reads the file.",
  ]);
});

test("the worker summarizer is the JavaScript root's own module", async () => {
  await seeded;
  const text = "The parser reads the file. It builds a tree. The tree is checked.";
  const inWorker = evaluate(worker, `crateModule("crate/dependency_summarization.mjs").summarizeByDependency(${JSON.stringify(text)}, "en").text`);
  assert.equal(inWorker, "The parser reads the file.");
});

test("summarization declines a prompt without text", async () => {
  const declined = evaluate(worker, 'trySummarizationText("Summarize this.", normalizePrompt("Summarize this."))');
  assert.equal(declined, null);
});

test("summarization declines a single statement", async () => {
  await seeded;
  const prompt = "Summarize: Cats sleep a lot.";
  const declined = evaluate(worker, `trySummarizationText(${JSON.stringify(prompt)}, normalizePrompt(${JSON.stringify(prompt)}))`);
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
