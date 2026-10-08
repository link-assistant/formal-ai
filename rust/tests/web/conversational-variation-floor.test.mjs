// Unit coverage for the counting logic behind the issue #933 CI floor.
//
// The gate itself reads the committed corpus, which (by construction) passes.
// These cases feed it fixture records engineered to trip the floor, so the
// failure path is exercised on every run rather than only when someone
// actually deletes a wording.

import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

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
        expected_answer: "Hi, how may I help you?",
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
      expected_answer: "Hi, how may I help you?",
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
  assert.equal(normalizeVariation("Ａ１"), normalizeVariation("a1"));
  // U+03D2 only becomes lowercaseable after NFKC maps it to Greek upsilon.
  assert.equal(normalizeVariation("ϒ"), normalizeVariation("υ"));
  assert.notEqual(normalizeVariation("क"), normalizeVariation("का"));
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
      expected_answer: "Goodbye!",
    },
    {
      id: "greeting_es_01",
      case: "greeting",
      language: "es",
      prompt: "hola",
      expected_intent: "greeting",
      expected_evidence: "response:greeting",
      expected_answer: "Hi, how may I help you?",
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
      expected_answer: "Hi, how may I help you?",
    },
    {
      id: "greeting_en_01",
      case: "greeting",
      language: "en",
      prompt: "hello",
      expected_intent: "greeting",
      expected_evidence: "response:greeting",
      expected_answer: "Hi, how may I help you?",
    },
  ]);

  assert.deepEqual(problems, ["duplicate case id greeting_en_01"]);
});

test("a record that does not show its answer is reported (R234-2)", () => {
  const shown = {
    id: "capabilities_en_01",
    case: "greeting",
    language: "en",
    prompt: "what can you do",
    expected_intent: "greeting",
    expected_evidence: "response:greeting",
    // A multi-line answer is pinned by its opening line, which still counts.
    expected_answer_contains: "I can answer questions about",
  };
  const hidden = { ...shown, id: "capabilities_en_02", prompt: "what can you do now" };
  delete hidden.expected_answer_contains;

  const { problems } = audit([shown, hidden]);

  assert.deepEqual(
    problems.filter((problem) => problem.includes("records no answer")),
    [
      "record capabilities_en_02 records no answer; add expected_answer (or expected_answer_contains for a multi-line answer)",
    ],
  );
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

// ---- the gate run end to end (R933-3, R933-4, R933-6, R933-7, R933-8) -------
//
// The cases above feed the counting logic fixtures. These run the real gate
// script as CI does: over the committed corpus, and over a copy of it with one
// Hindi wording removed (the manual check of R933-6, automated).

const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));
const GATE = "rust/tests/e2e/scripts/check-conversational-variation-floor.mjs";

function runGate(root) {
  return spawnSync(process.execPath, [path.join(root, GATE)], { encoding: "utf8" });
}

test("the committed corpus clears the floor and prints its count table on success", () => {
  const run = runGate(REPO_ROOT);
  assert.equal(run.status, 0, run.stderr);
  const lines = run.stdout.trim().split("\n");
  assert.equal(lines[0], "Conversational wording variations per case (floor: 5 per language)");
  assert.equal(lines[1], "  case                  en  ru  hi  zh");
  assert.equal(lines.length, 14);
  assert.equal(
    lines.at(-1),
    "Conversational variation floor OK: 10 cases x 4 languages, 228 verified prompts, every group at or above 5.",
  );
});

test("removing one Hindi wording fails the gate and names exactly that case", () => {
  // The gate runs `main` only when argv[1] is its own resolved path, so the
  // mirror is named by its real path (macOS links /var to /private/var).
  const mirror = realpathSync(mkdtempSync(path.join(tmpdir(), "variation-floor-")));
  try {
    for (const relative of [
      GATE,
      "rust/tests/e2e/scripts/lino-seed-parser.mjs",
      "data/seed/agent-info.lino",
      "data/benchmarks/conversational-variations-suite.lino",
      ...LANGUAGES.map((language) => `data/benchmarks/conversational-variations/${language}.lino`),
    ]) {
      mkdirSync(path.dirname(path.join(mirror, relative)), { recursive: true });
      copyFileSync(path.join(REPO_ROOT, relative), path.join(mirror, relative));
    }
    const hindi = path.join(mirror, "data/benchmarks/conversational-variations/hi.lino");
    // A record is its unindented id line plus the indented fields under it.
    const lines = readFileSync(hindi, "utf8").split("\n");
    const start = lines.indexOf("conversational_variation_case_assistant_name_hi_05");
    let end = start + 1;
    while (end < lines.length && lines[end].startsWith("  ")) end += 1;
    assert.ok(start >= 0 && end - start > 5);
    lines.splice(start, end - start);
    writeFileSync(hindi, lines.join("\n"));

    const run = runGate(mirror);
    assert.equal(run.status, 1);
    assert.match(run.stdout, /\n {2}assistant_name {9}5 {3}5 {3}4 {3}5\n/);
    assert.deepEqual(
      run.stderr.split("\n").filter((line) => line.startsWith("- ")),
      [
        "- case assistant_name has 4 hi variation(s); the floor is 5",
        "- data/benchmarks/conversational-variations-suite.lino declares minimum_pass_count 228 but the partitions hold 227 cases",
      ],
    );
  } finally {
    rmSync(mirror, { recursive: true, force: true });
  }
});

test("the gate is registered as a web-stage CI gate that runs the npm script", () => {
  const gate = readFileSync(path.join(REPO_ROOT, "data/meta/ci-gates/check-conversational-variation-floor.lino"), "utf8");
  assert.match(gate, /^ci_gate check[-_]conversational[-_]variation[-_]floor\n {2}stage web\n/m);
  assert.match(gate, /^ {2}run "npm run --prefix rust\/tests\/e2e check:variation-floor"$/m);
  const scripts = JSON.parse(readFileSync(path.join(REPO_ROOT, "rust/tests/e2e/package.json"), "utf8")).scripts;
  assert.equal(scripts["check:variation-floor"], "node scripts/check-conversational-variation-floor.mjs");
});

test("the case study records why the floor does not duplicate the existing language gates (R933-10)", () => {
  const readme = readFileSync(path.join(REPO_ROOT, "docs/case-studies/issue-933/README.md"), "utf8");
  for (const gate of ["check:language-test-coverage", "check:language-change-parity", "check:intent-coverage"]) {
    assert.ok(readme.includes(`| \`${gate}\` |`), `the comparison table is missing ${gate}`);
  }
});
