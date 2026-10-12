#!/usr/bin/env node
// Measures requirement extraction (R1188-U20) against the requirement rows
// the repository already keeps for each issue.
//
// Every cached issue body in data/benchmarks/issue-requirements/issue-N.md is
// paired with the rows of docs/requirements/issue-NNNN-*.md (column two). A
// row is found when some extracted requirement shares at least
// MATCH_OVERLAP of the smaller content-word set with it; an extracted
// requirement is relevant when it finds some row. Recall and precision are
// printed, and `--check` compares them with the floors in
// data/meta/text-capability-ratchet.lino, which may only rise.
//
// Usage:
//   node scripts/measure-requirement-extraction.mjs           print the scores
//   node scripts/measure-requirement-extraction.mjs --check   fail below the floors
//   node scripts/measure-requirement-extraction.mjs --write   raise the floors

import { readFileSync, readdirSync, writeFileSync, existsSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { WorkerHost } from '../js/server/worker-host.mjs';
import { installNodeHost } from '../js/agentic/node-host.mjs';
import { contentWords, extractRequirements, overlap } from '../js/agentic/crate/requirement_extraction.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const ISSUES = join(ROOT, 'data/benchmarks/issue-requirements');
const SHARDS = join(ROOT, 'docs/requirements');
const RATCHET = join(ROOT, 'data/meta/text-capability-ratchet.lino');

/** A gold row and an extracted requirement match at this content-word overlap. */
const MATCH_OVERLAP = 0.35;

/** Requirement rows (column two) of every shard written for issue `number`. */
export function goldRows(number) {
  const prefix = `issue-${String(number).padStart(4, '0')}-`;
  const rows = [];
  for (const name of readdirSync(SHARDS).filter((file) => file.startsWith(prefix) && file.endsWith('.md'))) {
    for (const line of readFileSync(join(SHARDS, name), 'utf8').split('\n')) {
      const cells = line.split('|').map((cell) => cell.trim());
      if (cells.length < 4 || cells[1] === 'ID' || /^-+$/u.test(cells[1]) || cells[2] === '') continue;
      rows.push(cells[2]);
    }
  }
  return rows;
}

/** Recall and precision of the extractor over every cached issue. */
export function measure() {
  let found = 0;
  let rows = 0;
  let relevant = 0;
  let extracted = 0;
  const perIssue = [];
  for (const file of readdirSync(ISSUES).filter((name) => /^issue-\d+\.md$/u.test(name))) {
    const number = Number(/\d+/u.exec(file)[0]);
    const gold = goldRows(number).map(contentWords);
    if (gold.length === 0) continue;
    const requirements = extractRequirements(readFileSync(join(ISSUES, file), 'utf8')).map(contentWords);
    const matches = (left, right) => overlap(left, right) >= MATCH_OVERLAP;
    const issueFound = gold.filter((row) => requirements.some((requirement) => matches(row, requirement))).length;
    found += issueFound;
    rows += gold.length;
    relevant += requirements.filter((requirement) => gold.some((row) => matches(row, requirement))).length;
    extracted += requirements.length;
    perIssue.push({ number, rows: gold.length, found: issueFound, extracted: requirements.length });
  }
  const ratio = (part, whole) => (whole === 0 ? 0 : Math.floor((part / whole) * 1000) / 1000);
  return { issues: perIssue.length, rows, found, extracted, relevant, recall: ratio(found, rows), precision: ratio(relevant, extracted), perIssue };
}

/** The floors kept in the ratchet file. */
export function readFloors() {
  if (!existsSync(RATCHET)) return {};
  const floors = {};
  for (const match of readFileSync(RATCHET, 'utf8').matchAll(/^\s+requirement-extraction-(recall|precision) ([\d.]+)$/gmu)) {
    floors[match[1]] = Number(match[2]);
  }
  return floors;
}

function renderRatchet(score) {
  return [
    'text-capability-ratchet',
    '  source scripts/measure-requirement-extraction.mjs',
    `  requirement-extraction-recall ${score.recall}`,
    `  requirement-extraction-precision ${score.precision}`,
    '',
  ].join('\n');
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  await installNodeHost(new WorkerHost());
  const score = measure();
  const mode = process.argv[2];
  console.log(`issues ${score.issues} rows ${score.rows} found ${score.found} extracted ${score.extracted} relevant ${score.relevant}`);
  console.log(`recall ${score.recall} precision ${score.precision}`);
  if (process.argv.includes('--verbose')) {
    for (const issue of score.perIssue) console.log(`  #${issue.number} rows ${issue.rows} found ${issue.found} extracted ${issue.extracted}`);
  }
  if (mode === '--write') {
    writeFileSync(RATCHET, renderRatchet(score));
    console.log(`wrote ${RATCHET}`);
  } else if (mode === '--check') {
    const floors = readFloors();
    const failures = ['recall', 'precision'].filter((name) => score[name] < (floors[name] ?? 0));
    for (const name of failures) console.error(`requirement extraction ${name} ${score[name]} fell below ${floors[name]}`);
    if (failures.length > 0) process.exit(1);
  }
}
