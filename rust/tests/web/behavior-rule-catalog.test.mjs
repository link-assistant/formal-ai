// R1188-U1 (generalize, don't specialize): the behavior-rule catalog, the
// capability listings and the offline translation phrases are seed data that
// both runtimes read, not prose and example prompts held in code. The browser
// twin's answers are pinned against the native pins themselves: the expected
// rule lists and the Russian detail are read from
// rust/tests/unit/specification/behavior_rules.rs, so the two runtimes hold
// one text. The Rust twin of the new cases is the same file.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";

import { REPO_ROOT, createWorkerContext, evaluate } from "./support/browser-runtime.mjs";

const worker = createWorkerContext();
const ready = evaluate(worker, "loadSeed()");

async function solve(prompt, history = []) {
  await ready;
  return worker.solve(prompt, history, {}, {}, [], {});
}

const NATIVE_PINS = readFileSync(
  path.join(REPO_ROOT, "rust/tests/unit/specification/behavior_rules.rs"),
  "utf8",
);

/** A `const NAME: &str = "…";` of the native pins, unescaped. */
function nativeConstant(name) {
  const match = new RegExp(`const ${name}: &str = ("(?:[^"\\\\]|\\\\.)*");`, "u").exec(NATIVE_PINS);
  assert.ok(match, `the native pins define ${name}`);
  return JSON.parse(match[1]);
}

const LISTS = [
  ["ENGLISH_RULE_LIST", "Show rules"],
  ["RUSSIAN_RULE_LIST", "Покажи список своих правил"],
  ["HINDI_RULE_LIST", "अपने नियमों की सूची दिखाओ"],
  ["CHINESE_RULE_LIST", "显示你的规则列表"],
];

for (const [name, prompt] of LISTS) {
  test(`the browser lists the rules byte-identically to the native ${name}`, async () => {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "behavior_rules_list");
    assert.equal(answer.content, nativeConstant(name));
  });
}

test("the browser renders the Russian rule detail byte-identically to the native pin", async () => {
  const expected = JSON.parse(/"(Резервное правило для неизвестного запроса(?:[^"\\]|\\.)*)"/u.exec(NATIVE_PINS)[0]);
  const answer = await solve("Покажи правило unknown");
  assert.equal(answer.intent, "behavior_rule_detail");
  assert.equal(answer.content, expected);
});

test("a rule detail is asked for through the seeded rule_detail_request openings in every language", async () => {
  for (const [prompt, label] of [
    ["Show Behaviour Rule farewell", "Farewell rule"],
    ["读取规则 rule_identity", "身份规则"],
    ["नियम पढ़ो rule_identity", "पहचान नियम"],
    ["muestra la regla capabilities", null],
  ]) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "behavior_rule_detail", prompt);
    if (label) assert.equal(answer.content.split("\n")[0], label, prompt);
  }
});

test("the count and the brief follow-up read the seeded sentences", async () => {
  const count = await solve("How many behavior rules are there?");
  assert.equal(count.intent, "behavior_rules_count");
  assert.ok(count.content.startsWith("Total behavior rules: 8 (built-in: 8; dialog-local: 0)."), count.content);
  const brief = await solve("briefly", [
    { role: "user", content: "Show rules" },
    { role: "assistant", content: nativeConstant("ENGLISH_RULE_LIST") },
  ]);
  assert.equal(brief.intent, "behavior_rules_brief");
  assert.equal(
    brief.content,
    "Briefly: 8 behavior rules (8 built-in, 0 dialog-local): greetings, farewells, small talk, identity, assistant name, capabilities, program templates, and the unknown fallback.",
  );
});

test("a taught rule is confirmed with the seeded title and hint", async () => {
  const answer = await solve("When `ping` then `pong`");
  assert.equal(answer.intent, "behavior_rule_update");
  assert.ok(answer.content.startsWith("Behavior rule compiled for this dialog.\n\nWhen the user says `ping` then respond with `pong`."));
  assert.ok(answer.content.includes("behavior_rule_runtime"));
  assert.ok(answer.content.endsWith("Send `ping` now and I will answer with the configured response. Export memory to keep this rule message with the dialog."));
});

test("the capability listings are the seeded responses, in Spanish too", async () => {
  for (const [prompt, language] of [["What can you do?", "en"], ["Что ты умеешь?", "ru"], ["¿Qué puedes hacer?", "es"]]) {
    const answer = await solve(prompt);
    assert.equal(answer.intent, "capabilities", prompt);
    assert.equal(answer.content, evaluate(worker, `answerFor("capabilities", ${JSON.stringify(language)})`), prompt);
  }
});

test("the offline translation phrases are translation_phrase meanings of the seed", async () => {
  await ready;
  assert.ok(evaluate(worker, "translationPhraseRegistry().length") >= 14);
  for (const [prompt, expected] of [
    ["translate 'What is your name?' to Russian", '"Как тебя зовут?"'],
    ["Translate \"Привет\" to English", '"Hello"'],
    ["translate apple to russian", '"яблоко"'],
    ["Translate 'Hello' to Spanish", '"Hola"'],
  ]) {
    const answer = await solve(prompt);
    assert.equal(answer.content, expected, prompt);
  }
});
