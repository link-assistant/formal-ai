// Unit coverage for the counting logic behind the issue #933 CI floor.
//
// The gate itself reads the committed corpus, which (by construction) passes.
// These cases feed it fixture records engineered to trip the floor, so the
// failure path is exercised on every run rather than only when someone
// actually deletes a wording.

import assert from "node:assert/strict";
import { test } from "node:test";

import {
  auditVariationFloor,
  normalizeVariation,
  parseSuiteManifest,
  parseVariationRecords,
} from "../e2e/scripts/check-conversational-variation-floor.mjs";

const LANGUAGES = ["en", "ru", "hi", "zh"];

function fixtureRecords(counts) {
  const records = [];
  for (const [language, total] of Object.entries(counts)) {
    for (let index = 0; index < total; index += 1) {
      records.push({
        id: `greeting_${language}_${index}`,
        case: "greeting",
        language,
        prompt: `wording ${language} ${index}`,
        expected_intent: "greeting",
        expected_evidence: "response:greeting",
      });
    }
  }
  return records;
}

function audit(records) {
  return auditVariationFloor({
    records,
    cases: ["greeting"],
    languages: LANGUAGES,
    minimum: 5,
  });
}

test("five distinct wordings per language clear the floor", () => {
  const { shortfalls, problems } = audit(
    fixtureRecords({ en: 5, ru: 5, hi: 5, zh: 5 }),
  );

  assert.deepEqual(problems, []);
  assert.deepEqual(shortfalls, []);
});

test("a case one wording short in a single language is reported for that language only", () => {
  const { shortfalls } = audit(fixtureRecords({ en: 9, ru: 5, hi: 4, zh: 5 }));

  assert.deepEqual(shortfalls, [
    { case: "greeting", language: "hi", count: 4, minimum: 5 },
  ]);
});

test("a language with no records at all is reported rather than skipped", () => {
  const { shortfalls } = audit(fixtureRecords({ en: 5, ru: 5, hi: 5 }));

  assert.deepEqual(shortfalls, [
    { case: "greeting", language: "zh", count: 0, minimum: 5 },
  ]);
});

test("re-punctuated copies of one wording do not add up to five", () => {
  const records = [
    ...fixtureRecords({ ru: 5, hi: 5, zh: 5 }),
    ...["hello", "Hello!", "hello.", "HELLO", "hello ,"].map((prompt, index) => ({
      id: `greeting_en_${index}`,
      case: "greeting",
      language: "en",
      prompt,
      expected_intent: "greeting",
      expected_evidence: "response:greeting",
    })),
  ];

  const { shortfalls, problems } = audit(records);

  assert.deepEqual(shortfalls, [
    { case: "greeting", language: "en", count: 1, minimum: 5 },
  ]);
  assert.equal(problems.length, 4);
  assert.match(problems[0], /repeats an existing greeting\/en wording/);
});

test("normalization folds case, punctuation and spacing but keeps distinct words apart", () => {
  assert.equal(normalizeVariation("How are you?"), normalizeVariation("how are you"));
  assert.equal(normalizeVariation("你好！"), normalizeVariation("你好"));
  assert.equal(normalizeVariation("до свидания"), normalizeVariation("досвидания"));
  assert.notEqual(normalizeVariation("hello"), normalizeVariation("hey"));
});

test("records naming an unlisted case or language are surfaced, not silently counted", () => {
  const { problems, counts } = audit([
    {
      id: "farewell_en_01",
      case: "farewell",
      language: "en",
      prompt: "bye",
      expected_intent: "farewell",
      expected_evidence: "response:farewell",
    },
    {
      id: "greeting_es_01",
      case: "greeting",
      language: "es",
      prompt: "hola",
      expected_intent: "greeting",
      expected_evidence: "response:greeting",
    },
  ]);

  assert.equal(counts.get("greeting en").size, 0);
  assert.equal(problems.length, 2);
  assert.match(problems[0], /the suite manifest does not list/);
  assert.match(problems[1], /which the suite does not cover/);
});

test("a record missing a required field is reported", () => {
  const { problems } = audit([
    { id: "greeting_en_01", case: "greeting", language: "en", prompt: "hi" },
  ]);

  assert.deepEqual(
    problems.filter((problem) => problem.includes("missing")),
    ["record greeting_en_01 is missing expected_intent"],
  );
});

test("duplicate case ids are reported even when the wordings differ", () => {
  const { problems } = audit([
    {
      id: "greeting_en_01",
      case: "greeting",
      language: "en",
      prompt: "hi",
      expected_intent: "greeting",
      expected_evidence: "response:greeting",
    },
    {
      id: "greeting_en_01",
      case: "greeting",
      language: "en",
      prompt: "hello",
      expected_intent: "greeting",
      expected_evidence: "response:greeting",
    },
  ]);

  assert.deepEqual(problems, ["duplicate case id greeting_en_01"]);
});

test("the LiNo record parser reads an id line plus its two-space fields", () => {
  const records = parseVariationRecords(
    [
      "conversational_variation_case_greeting_en_01",
      '  record_type "conversational_variation_case"',
      '  id "greeting_en_01"',
      '  case "greeting"',
      '  language "en"',
      '  prompt "hi there"',
      "",
      "conversational_variation_case_calculation_en_01",
      '  record_type "conversational_variation_case"',
      '  id "calculation_en_01"',
      '  prompt "two plus two"',
      '  expected_answer_contains "4"',
    ].join("\n"),
  );

  assert.equal(records.length, 2);
  assert.equal(records[0].record_id, "conversational_variation_case_greeting_en_01");
  assert.equal(records[0].prompt, "hi there");
  assert.equal(records[1].expected_answer_contains, "4");
});

test("the manifest parser collects repeated case and member_file fields", () => {
  const manifest = parseSuiteManifest(
    [
      "conversational_variation_suite_issue_933",
      '  record_type "conversational_variation_suite"',
      '  minimum_variations_per_language "5"',
      '  languages "en|ru|hi|zh"',
      '  case "greeting"',
      '  case "farewell"',
      '  member_file "conversational-variations/en.lino"',
      '  member_file "conversational-variations/ru.lino"',
    ].join("\n"),
  );

  assert.equal(manifest.minimum_variations_per_language, "5");
  assert.deepEqual(manifest.languages.split("|"), LANGUAGES);
  assert.deepEqual(manifest.case, ["greeting", "farewell"]);
  assert.deepEqual(manifest.member_file, [
    "conversational-variations/en.lino",
    "conversational-variations/ru.lino",
  ]);
});
