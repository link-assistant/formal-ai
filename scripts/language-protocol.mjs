#!/usr/bin/env node
// Issue #706 N→N+1 language protocol and deterministic coverage generator.

import fs from 'node:fs';
import path from 'node:path';
import process from 'node:process';
import { fileURLToPath } from 'node:url';
import { parseRegisteredLanguages } from '../tests/e2e/scripts/lino-seed-parser.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const args = process.argv.slice(2);

function option(name) {
  const index = args.indexOf(name);
  return index >= 0 ? args[index + 1] : '';
}

function read(relativePath) {
  return fs.readFileSync(path.join(root, relativePath), 'utf8');
}

function suiteStates(text) {
  const states = new Map();
  for (const match of text.matchAll(/^  suite ([a-z_]+) (pass|gap)$/gmu)) {
    states.set(match[1], match[2]);
  }
  return states;
}

function registeredSuiteStates(ledger, language) {
  const lines = ledger.split(/\r?\n/u);
  const start = lines.indexOf(`  language ${language}`);
  if (start < 0) return new Map();
  const end = lines.findIndex(
    (line, index) => index > start && /^  language /u.test(line),
  );
  const block = lines.slice(start + 1, end < 0 ? lines.length : end).join('\n');
  const list = block.match(/suites_passing \(([^)]*)\)/u)?.[1] ?? '';
  return new Map(
    [...list.matchAll(/"([a-z_]+)"/gu)].map((match) => [match[1], 'pass']),
  );
}

function isPartial(ledger, language) {
  const lines = ledger.split(/\r?\n/u);
  const start = lines.indexOf(`  language ${language}`);
  if (start < 0) return false;
  const end = lines.findIndex(
    (line, index) => index > start && /^  language /u.test(line),
  );
  return lines
    .slice(start + 1, end < 0 ? lines.length : end)
    .includes('    status partial');
}

function checkFile(relativePath, expected) {
  const actual = read(relativePath);
  if (actual !== expected) {
    console.error(
      `${relativePath} is stale; run language-protocol.mjs --language ${language} --write`,
    );
    process.exitCode = 1;
  }
}

function matrixDocument(languages) {
  const lines = [
    'language_round_trip_matrix',
    '  generated_by scripts/language-protocol.mjs',
  ];
  for (const language of languages) {
    lines.push(`  same_language ${language}`);
    lines.push(`    route ${language}_to_meta_to_${language}`);
  }
  for (const source of languages) {
    for (const target of languages) {
      lines.push(`  pair ${source}_${target}`);
      lines.push(`    route ${source}_to_meta_to_${target}`);
    }
  }
  return `${lines.join('\n')}\n`;
}

function coverageDocument(language, states, partial) {
  const total = states.size;
  const passed = [...states.values()].filter((state) => state === 'pass').length;
  const permille = total === 0 ? 0 : Math.floor((passed * 1000) / total);
  const lines = [
    'coverage_report',
    `  language ${language}`,
    '  generated_by scripts/language-protocol.mjs',
    '  code_changes 0',
    `  suites_total ${total}`,
    `  suites_passing ${passed}`,
    `  suite_coverage_permille ${permille}`,
    `  meaning_coverage ${partial ? 'partial' : 'reported_by_ledger'}`,
    '  fallback_policy explicit_gap',
  ];
  for (const [suite, state] of states) {
    lines.push(`  suite ${suite}`);
    lines.push(`    status ${state}`);
    if (state === 'gap') lines.push('    event language_gap');
  }
  if (partial && ![...states.values()].includes('gap')) {
    lines.push('  uncovered_meanings');
    lines.push('    event language_gap');
  }
  return `${lines.join('\n')}\n`;
}

const language = option('--language');
if (!language) {
  console.error('usage: language-protocol.mjs --language CODE [--candidate FILE] [--dry-run|--write]');
  process.exit(2);
}

const ledger = read('data/seed/languages.lino');
const registered = parseRegisteredLanguages(ledger);
const candidatePath = option('--candidate');
const candidate = candidatePath ? read(candidatePath) : '';
if (!candidate && !registered.includes(language)) {
  console.error(`language ${language} is neither registered nor supplied as a candidate`);
  process.exit(2);
}

const states = candidate
  ? suiteStates(candidate)
  : registeredSuiteStates(ledger, language);
const partial = candidate.length > 0 || isPartial(ledger, language);
const coverage = coverageDocument(language, states, partial);
const matrix = matrixDocument(registered);

if (args.includes('--write')) {
  const docs = path.join(root, 'docs/case-studies/issue-706');
  fs.mkdirSync(docs, { recursive: true });
  fs.writeFileSync(path.join(docs, 'round-trip-matrix.lino'), matrix);
  fs.writeFileSync(path.join(docs, `coverage-${language}.lino`), coverage);
}

if (args.includes('--check')) {
  checkFile('docs/case-studies/issue-706/round-trip-matrix.lino', matrix);
  checkFile(`docs/case-studies/issue-706/coverage-${language}.lino`, coverage);
}

process.stdout.write(coverage);
