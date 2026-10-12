// Issue #1186 R10 (with R5): the per-language formalization probe set,
// data/benchmarks/formalization/{en,ru,hi,zh}.lino, answered by the browser
// worker — the twin of rust/tests/unit/web-engine-core/issue_1186_formalization_probe_set.rs.
// Every probe must formalize to its expected first-order clause, and every
// rendered target (Lean 4, Rocq, Links Notation) must deformalize back to the
// same clause structure in the probe's own language.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { createWorkerContext, evaluate, plain } from "./support/browser-runtime.mjs";

const ROOT = join(dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const LANGUAGES = ["en", "ru", "hi", "zh"];
const SHAPES = ["universal", "existential", "negation", "relation"];
const DIRECTION_CUES = {
  en: "Deformalize in plain English",
  ru: "Деформализуй",
  hi: "सहज भाषा में",
  zh: "自然语言",
};

const worker = createWorkerContext();
const seeded = evaluate(worker, "loadSeed()");

function probes(language) {
  const text = readFileSync(join(ROOT, "data", "benchmarks", "formalization", `${language}.lino`), "utf8");
  const records = [];
  let current = null;
  for (const line of text.split("\n")) {
    if (line.startsWith("#") || line.trim().length === 0) continue;
    if (!line.startsWith(" ")) {
      current = {};
      records.push(current);
      continue;
    }
    const trimmed = line.trim();
    const space = trimmed.indexOf(" ");
    current[trimmed.slice(0, space)] = trimmed.slice(space + 1).replace(/^"|"$/gu, "");
  }
  return records;
}

async function handlerAnswer(prompt) {
  await seeded;
  const literal = JSON.stringify(prompt);
  const answer = plain(evaluate(worker, `tryFormalizationRequest(${literal}, normalizePrompt(${literal}))`));
  assert.ok(answer, `the formalization handler should answer: ${prompt}`);
  return answer.content;
}

function fenced(answer, tag) {
  const open = "```" + tag + "\n";
  const start = answer.indexOf(open);
  assert.ok(start !== -1, `answer should carry a \`${tag}\` fence: ${answer}`);
  const from = start + open.length;
  const end = answer.indexOf("```", from);
  return answer.slice(from, end).replace(/\n$/u, "");
}

test("every language carries ten or more probes covering all four shapes", () => {
  for (const language of LANGUAGES) {
    const records = probes(language);
    assert.ok(records.length >= 10, `${language}: ${records.length} probes`);
    for (const shape of SHAPES) {
      assert.ok(records.some((record) => record.shape === shape), `${language} lacks a ${shape} probe`);
    }
    for (const record of records) {
      assert.equal(record.language, language);
      assert.ok(record.expected_fol.startsWith(
        { forall: "∀", exists: "∃", no: "¬∃" }[record.expected_quantifier],
      ), record.id);
    }
  }
});

for (const language of LANGUAGES) {
  test(`the ${language} probes formalize to their expected clauses`, async () => {
    for (const record of probes(language)) {
      const answer = await handlerAnswer(record.prompt);
      assert.equal(fenced(answer, "fol"), record.expected_fol, `${record.id}: ${answer}`);
      for (const predicate of record.expected_predicates.split("|")) {
        assert.ok(fenced(answer, "fol").includes(`${predicate}(x`), `${record.id} names ${predicate}`);
      }
    }
  });

  test(`the ${language} probes deformalize from every rendered target`, async () => {
    for (const record of probes(language)) {
      const answer = await handlerAnswer(record.prompt);
      for (const target of ["fol", "lean", "rocq", "lino"]) {
        const rendered = fenced(answer, target);
        const back = await handlerAnswer(`${DIRECTION_CUES[language]}:\n${rendered}`);
        assert.ok(back.includes("structure preserved"), `${record.id} ${target}: ${back}`);
      }
    }
  });
}
