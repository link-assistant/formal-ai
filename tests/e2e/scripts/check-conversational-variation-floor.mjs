// CI guard for the per-test-case wording-variation floor (issue #933).
//
// Issue #123 asked for "at least 5 variations per 4 languages" for every
// conversational test case, and for CI to enforce it. PR #124 fixed the
// reported prompts but left the floor unenforced, so the corpus drifted back
// to three-or-four-variation groups.
//
// `check:language-parity` and `check:intent-coverage` already answer different
// questions -- "was this change mirrored into the other languages?" and "does
// every routed intent have a localized response?" -- and neither counts
// wordings per test case. `check:language-test-coverage` is diff-aware: it only
// fires when a PR touches language-facing files, and it merely requires each
// registered language to appear somewhere in the added test lines. This check
// is the missing one: an absolute, always-on floor over the recorded corpus.
//
// The corpus is `data/benchmarks/conversational-variations-suite.lino` plus its
// per-language partitions. Every record is a prompt that the deterministic
// engine already routes (`tests/unit/conversational_variations.rs` re-verifies
// that on every run), so a case cannot reach the floor with wordings that only
// look plausible.

import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  decodeLinoValue,
  parseLinoField,
  parseSupportedLanguagesFromAgentInfo,
} from './lino-seed-parser.mjs';

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(scriptDir, '../../..');

export const SUITE_PATH = 'data/benchmarks/conversational-variations-suite.lino';
export const CORPUS_DIRECTORY = 'data/benchmarks';

// Two wordings only count as two if they differ in more than punctuation,
// spacing or letter case. "how are you" and "How are you?" are one wording, so
// a case cannot reach the floor by re-punctuating a single phrase five times.
export function normalizeVariation(prompt) {
  return prompt
    .normalize('NFKC')
    .toLowerCase()
    .replace(/[\p{P}\p{S}\p{Z}\s]+/gu, '');
}

// Records are LiNo: an unindented id line followed by two-space fields.
export function parseVariationRecords(text) {
  const records = [];
  let current = null;

  for (const line of text.split(/\r?\n/)) {
    if (line.trim().length === 0) continue;

    if (!/^\s/.test(line)) {
      current = { record_id: decodeLinoValue(line.trim()) };
      records.push(current);
      continue;
    }

    const field = parseLinoField(line, 2);
    if (field && current) current[field.key] = field.value;
  }

  return records;
}

export function parseSuiteManifest(text) {
  const manifest = { case: [], member_file: [] };

  for (const line of text.split(/\r?\n/)) {
    if (line.trim().length === 0 || !/^\s/.test(line)) continue;
    const field = parseLinoField(line, 2);
    if (!field) continue;
    if (Array.isArray(manifest[field.key])) {
      manifest[field.key].push(field.value);
    } else {
      manifest[field.key] = field.value;
    }
  }

  return manifest;
}

/// Count distinct wordings per (case, language) and report every group that
/// sits below the floor, plus the corpus-integrity problems that would let a
/// group look full when it is not.
export function auditVariationFloor({ records, cases, languages, minimum }) {
  const counts = new Map();
  const problems = [];
  const seenIds = new Set();
  const knownCases = new Set(cases);
  const knownLanguages = new Set(languages);

  for (const caseName of cases) {
    for (const language of languages) {
      counts.set(`${caseName} ${language}`, new Set());
    }
  }

  for (const record of records) {
    const id = record.id || record.record_id;
    for (const field of ['id', 'case', 'language', 'prompt', 'expected_intent']) {
      if (!record[field] || !String(record[field]).trim()) {
        problems.push(`record ${id} is missing ${field}`);
      }
    }
    // Every record also shows the text its prompt produces, so the corpus is
    // documentation and not only a count (R234-2). The capability answer is a
    // multi-paragraph listing, so those records pin its opening line with
    // `expected_answer_contains` instead of inlining the whole page.
    if (!record.expected_answer && !record.expected_answer_contains) {
      problems.push(
        `record ${id} records no answer; add expected_answer (or expected_answer_contains for a multi-line answer)`,
      );
    }
    if (seenIds.has(id)) problems.push(`duplicate case id ${id}`);
    seenIds.add(id);

    if (!knownCases.has(record.case)) {
      problems.push(`record ${id} names case ${JSON.stringify(record.case)}, which the suite manifest does not list`);
      continue;
    }
    if (!knownLanguages.has(record.language)) {
      problems.push(`record ${id} names language ${JSON.stringify(record.language)}, which the suite does not cover`);
      continue;
    }

    const group = counts.get(`${record.case} ${record.language}`);
    const normalized = normalizeVariation(record.prompt || '');
    if (normalized.length === 0) {
      problems.push(`record ${id} has an empty prompt once punctuation is stripped`);
      continue;
    }
    if (group.has(normalized)) {
      problems.push(
        `record ${id} repeats an existing ${record.case}/${record.language} wording ${JSON.stringify(record.prompt)}; only punctuation, spacing or case differs`,
      );
      continue;
    }
    group.add(normalized);
  }

  const shortfalls = [];
  for (const caseName of cases) {
    for (const language of languages) {
      const size = counts.get(`${caseName} ${language}`).size;
      if (size < minimum) {
        shortfalls.push({ case: caseName, language, count: size, minimum });
      }
    }
  }

  return { counts, shortfalls, problems };
}

function readRepoFile(relativePath) {
  return fs.readFileSync(path.join(repoRoot, relativePath), 'utf8');
}

function main() {
  const manifest = parseSuiteManifest(readRepoFile(SUITE_PATH));
  const supportedLanguages = parseSupportedLanguagesFromAgentInfo(
    readRepoFile('data/seed/agent-info.lino'),
  );
  const suiteLanguages = (manifest.languages || '').split('|').filter(Boolean);
  const minimum = Number.parseInt(manifest.minimum_variations_per_language, 10);
  const errors = [];

  if (!Number.isInteger(minimum) || minimum < 5) {
    errors.push(
      `${SUITE_PATH} must declare minimum_variations_per_language of at least 5, got ${JSON.stringify(manifest.minimum_variations_per_language)}`,
    );
  }
  if (manifest.case.length === 0) {
    errors.push(`${SUITE_PATH} must list at least one case`);
  }
  if (suiteLanguages.slice().sort().join('|') !== supportedLanguages.slice().sort().join('|')) {
    errors.push(
      `${SUITE_PATH} must cover exactly the languages agent-info.lino advertises: expected ${supportedLanguages.join(', ')}, got ${suiteLanguages.join(', ') || '(none)'}`,
    );
  }

  const records = [];
  for (const language of suiteLanguages) {
    const member = `conversational-variations/${language}.lino`;
    if (!manifest.member_file.includes(member)) {
      errors.push(`${SUITE_PATH} must list member_file ${JSON.stringify(member)}`);
      continue;
    }
    const partition = path.join(CORPUS_DIRECTORY, member);
    const parsed = parseVariationRecords(readRepoFile(partition));
    for (const record of parsed) {
      if (record.record_type !== 'conversational_variation_case') {
        errors.push(`${partition} record ${record.record_id} must set record_type "conversational_variation_case"`);
      }
      if (record.language !== language) {
        errors.push(
          `${partition} record ${record.record_id} declares language ${JSON.stringify(record.language)} but lives in the ${language} partition`,
        );
      }
    }
    records.push(...parsed);
  }

  const { counts, shortfalls, problems } = auditVariationFloor({
    records,
    cases: manifest.case,
    languages: suiteLanguages,
    minimum: Number.isInteger(minimum) ? minimum : 5,
  });
  errors.push(...problems);

  const declaredTotal = Number.parseInt(manifest.minimum_pass_count, 10);
  if (declaredTotal !== records.length) {
    errors.push(
      `${SUITE_PATH} declares minimum_pass_count ${manifest.minimum_pass_count} but the partitions hold ${records.length} cases`,
    );
  }

  // The counts print on success as well as failure: the point of the gate is
  // that the numbers are visible in every run, not only when one of them drops.
  console.log(
    `Conversational wording variations per case (floor: ${minimum} per language)`,
  );
  const width = Math.max(...manifest.case.map((name) => name.length), 4);
  console.log(`  ${'case'.padEnd(width)}  ${suiteLanguages.map((language) => language.padStart(3)).join(' ')}`);
  for (const caseName of manifest.case) {
    const cells = suiteLanguages
      .map((language) => String(counts.get(`${caseName} ${language}`)?.size ?? 0).padStart(3))
      .join(' ');
    console.log(`  ${caseName.padEnd(width)}  ${cells}`);
  }

  if (shortfalls.length > 0) {
    console.error('');
    console.error('Conversational variation floor check failed.');
    for (const shortfall of shortfalls) {
      console.error(
        `- case ${shortfall.case} has ${shortfall.count} ${shortfall.language} variation(s); the floor is ${shortfall.minimum}`,
      );
    }
    console.error(
      `Add wordings to data/benchmarks/conversational-variations/<language>.lino, then re-run \`cargo test --test unit conversational_variation\` so the new prompts are verified against the engine.`,
    );
  }

  if (errors.length > 0) {
    console.error('');
    console.error('Conversational variation corpus is inconsistent:');
    for (const error of errors) console.error(`- ${error}`);
  }

  if (shortfalls.length > 0 || errors.length > 0) process.exit(1);

  console.log('');
  console.log(
    `Conversational variation floor OK: ${manifest.case.length} cases x ${suiteLanguages.length} languages, ${records.length} verified prompts, every group at or above ${minimum}.`,
  );
}

if (process.argv[1] && fileURLToPath(import.meta.url) === path.resolve(process.argv[1])) {
  main();
}
