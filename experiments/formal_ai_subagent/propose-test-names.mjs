#!/usr/bin/env node
// Proposes a behaviour name for every test file named only by an issue number
// (PR #1188, R1188-U5, rename tree 4), and writes the reviewed names into the
// rename map.
//
// A test file named `rust/tests/unit/issue_<n>.rs` or
// `rust/tests/e2e/tests/issue-<n>.spec.js` says which issue asked for it, not
// what it pins. The proposal is read from the file itself: its first doc
// comment and the names of its tests. The words the test names repeat most,
// in first-seen order, become the proposed name; the doc comment's first
// sentence becomes the rename's reason. A person (or an agent) reviews every
// proposal and may replace the name; the review is recorded on the rename.
//
// The issue number stays in the file for traceability: `--keep-issue` adds a
// first doc comment line `Issue #<n>: <reason>` to every renamed file whose
// header comment does not already cite `#<n>`.
//
// Usage:
//   node experiments/formal_ai_subagent/propose-test-names.mjs [--skip <path>]...
//       prints `path <TAB> proposal <TAB> reason <TAB> tests` for every file
//   node experiments/formal_ai_subagent/propose-test-names.mjs --write <tree> --reviewed <tsv>
//       appends `tree <tree>` to data/meta/rename-map.lino from the reviewed
//       table (`path <TAB> new name <TAB> reason`, one line per file)
//   node experiments/formal_ai_subagent/propose-test-names.mjs --keep-issue <tree>
//       adds the issue line to every renamed file of the tree that lacks one

import { execFileSync } from 'node:child_process';
import { appendFileSync, existsSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseRenameMap } from './rename-by-rule.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const MAP = 'data/meta/rename-map.lino';
const TEST_DIRECTORIES = ['rust/tests/', 'desktop/scripts/', 'tests/'];
const SUFFIXES = ['.test.mjs', '.spec.js', '.rs'];

/** Words that say nothing about the behaviour a test pins. */
const STOPWORDS = new Set([
  'a', 'an', 'the', 'and', 'or', 'of', 'to', 'in', 'on', 'for', 'is', 'are', 'be', 'by', 'with', 'from', 'its', 'it',
  'not', 'no', 'never', 'every', 'each', 'all', 'one', 'same', 'still', 'only', 'instead', 'than', 'that', 'this',
  'as', 'at', 'into', 'but', 'when', 'before', 'after', 'issue', 'issues', 'reported', 'regression', 'test', 'tests',
  'returns', 'uses', 'use', 'does', 'do', 'keeps', 'stays', 'must', 'should', 'can', 'cannot', 'has', 'have',
]);

/** The suffix of a test file, or null. */
export function testSuffix(path) {
  return SUFFIXES.find((suffix) => path.endsWith(suffix)) ?? null;
}

/** The issue number a test file is named by alone (`issue_<n>.rs`, `issue-<n>.spec.js`), or null. */
export function issueOnlyNumber(path) {
  const suffix = testSuffix(path);
  if (!suffix) return null;
  const match = /^issue[-_](\d+)$/u.exec(basename(path).slice(0, -suffix.length));
  return match ? match[1] : null;
}

/** The first doc comment of a source, joined into one line. */
export function firstDocComment(text) {
  const lines = [];
  for (const line of text.split('\n')) {
    const trimmed = line.trim();
    const comment = /^(?:\/\/!|\/\/\/|\/\/|\/\*\*|\/\*|\*\/|\*)\s?(.*)$/u.exec(trimmed);
    if (comment) {
      const body = comment[1].replace(/\*\/$/u, '').replace(/^@ts-check\s*/u, '').trim();
      if (body !== '') lines.push(body);
      continue;
    }
    if (trimmed === '' && lines.length === 0) continue;
    if (/^['"]use strict['"];?$/u.test(trimmed)) continue;
    break;
  }
  return lines.join(' ');
}

/** The names of the tests a source declares: Rust `#[test] fn` names and JS `test(`/`it(` titles. */
export function testNames(text) {
  const names = [];
  for (const match of text.matchAll(/#\[test\][\s\S]*?fn\s+(\w+)|\b(?:test|it|describe)(?:\.\w+)?\(\s*(['"`])(.*?)\2/gu)) {
    names.push(match[1] ?? match[3]);
  }
  return names;
}

/** The first sentence of a doc comment, without its `Issue #n:` lead. */
export function firstSentence(doc) {
  const text = doc.replace(/^(?:Regression (?:coverage|tests|gates) for |Issue #?\d+\s*(?:\([^)]*\))?\s*[:—-]\s*)/iu, '');
  const end = text.search(/[.!?](?:\s|$)/u);
  return (end >= 0 ? text.slice(0, end + 1) : text).trim();
}

/** The words of a name or a title, lower case, without numbers and stopwords. */
function words(text) {
  return text.toLowerCase().split(/[^a-z]+/u).filter((word) => word.length > 1 && !STOPWORDS.has(word));
}

/**
 * The proposed name: the three words the test names (or, without tests, the
 * doc comment) repeat most, in first-seen order, joined in the file's style.
 * @param {string} path
 * @param {string} text
 */
export function proposeName(path, text) {
  const source = testNames(text).join(' ') || firstDocComment(text);
  const counts = new Map();
  const order = [];
  for (const word of words(source)) {
    if (!counts.has(word)) order.push(word);
    counts.set(word, (counts.get(word) ?? 0) + 1);
  }
  const chosen = new Set([...order].sort((left, right) => counts.get(right) - counts.get(left) || order.indexOf(left) - order.indexOf(right)).slice(0, 3));
  const picked = order.filter((word) => chosen.has(word));
  const suffix = testSuffix(path);
  return suffix === '.rs' ? `${picked.join('_')}${suffix}` : `${picked.join('-')}${suffix}`;
}

function trackedTestFiles() {
  const output = execFileSync('git', ['ls-files', '-z', ...TEST_DIRECTORIES], { cwd: ROOT, maxBuffer: 64 * 1024 * 1024 });
  return output.toString('utf8').split('\0').filter((path) => path !== '' && existsSync(join(ROOT, path)) && issueOnlyNumber(path));
}

function quoted(value) {
  return `"${value.replaceAll('"', "'")}"`;
}

/** The map lines of one tree from a reviewed table. */
export function treeLines(tree, rows, proposals) {
  const lines = [`  tree ${tree}`];
  for (const { path, name, reason } of rows) {
    const proposal = proposals.get(path) ?? '';
    const to = `${dirname(path)}/${name}`;
    const review = proposal === name
      ? `Reviewed on 2026-10-08: kept the proposal read from the test names.`
      : `Reviewed on 2026-10-08: the proposal read from the test names was ${proposal}; renamed for the behaviour the doc comment and the tests pin. The issue number stays in the doc comment.`;
    lines.push('    rename', `      from ${quoted(path)}`, `      to ${quoted(to)}`, `      reason ${quoted(reason)}`, `      review ${quoted(review)}`);
  }
  return lines;
}

/** Add `Issue #<n>: <reason>` as the first header comment line of a renamed test file that does not cite `#<n>`. */
export function withIssueLine(text, path, issue, reason) {
  const header = text.split('\n').slice(0, 40).join('\n');
  if (new RegExp(`#${Number(issue)}\\b`, 'u').test(firstDocComment(header)) || new RegExp(`#0*${Number(issue)}\\b`, 'u').test(firstDocComment(header))) return text;
  const line = `Issue #${Number(issue)}: ${reason}`;
  if (path.endsWith('.rs')) return `//! ${line}\n${text.startsWith('//!') ? '//!\n' : '\n'}${text}`;
  const lines = text.split('\n');
  const at = lines[0].trim() === '// @ts-check' ? 1 : 0;
  lines.splice(at, 0, `// ${line}`);
  return lines.join('\n');
}

function main() {
  const args = process.argv.slice(2);
  const option = (name) => (args.indexOf(name) >= 0 ? args[args.indexOf(name) + 1] : null);
  const skipped = new Set(args.flatMap((arg, index) => (args[index - 1] === '--skip' ? [arg] : [])));
  const map = parseRenameMap(readFileSync(join(ROOT, MAP), 'utf8'));
  const renamed = new Set(map.renames.map((rename) => rename.from));
  if (option('--keep-issue')) {
    const tree = option('--keep-issue');
    let added = 0;
    for (const rename of map.renames.filter((entry) => entry.tree === tree)) {
      const issue = issueOnlyNumber(rename.from);
      const full = join(ROOT, rename.to);
      if (!issue || !existsSync(full)) continue;
      const text = readFileSync(full, 'utf8');
      const next = withIssueLine(text, rename.to, issue, rename.reason);
      if (next !== text) {
        writeFileSync(full, next);
        added += 1;
      }
    }
    console.log(`issue line added to ${added} file(s) of tree ${tree}`);
    return;
  }
  const files = trackedTestFiles().filter((path) => !skipped.has(path) && !renamed.has(path));
  const proposals = new Map(files.map((path) => [path, proposeName(path, readFileSync(join(ROOT, path), 'utf8'))]));
  if (option('--write')) {
    const rows = readFileSync(option('--reviewed'), 'utf8').split('\n').filter((line) => line.trim() !== '').map((line) => {
      const [path, name, reason] = line.split('\t');
      return { path, name, reason };
    });
    for (const row of rows) {
      if (!proposals.has(row.path)) throw new Error(`${row.path} is not a test file named only by an issue number, or it is skipped`);
    }
    appendFileSync(join(ROOT, MAP), `${treeLines(option('--write'), rows, proposals).join('\n')}\n`);
    console.log(`wrote ${rows.length} rename(s) to ${MAP}`);
    return;
  }
  for (const path of files) {
    const text = readFileSync(join(ROOT, path), 'utf8');
    console.log([path, proposals.get(path), firstSentence(firstDocComment(text)), testNames(text).slice(0, 4).join(' ; ')].join('\t'));
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) main();
