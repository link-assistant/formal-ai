#!/usr/bin/env node
// Issue #1188 R1188-U23 (owner, 2026-10-08, a strict requirement): every
// regular code file in `js/`, `ts/` and `rust/` (and `scripts/`, `packages/`)
// is human-readable multi-line code. Only real distribution bundles and
// generated data that is not code may be exempt, and each exemption is listed
// with its reason in data/meta/readable-code-exemptions.lino.
//
// A tracked code file fails when either rule breaks:
//
//   packed      its characters per line average more than MAXIMUM_AVERAGE,
//               so it has fewer lines than its size needs (a generated
//               single-line file, a minified bundle);
//   long line   one line holds more than MAXIMUM_CODE_CHARACTERS characters
//               of code. The text inside string literals and comments is not
//               counted: a long prompt or expected answer in a test is data
//               on one line, while a long run of code is layout a reader has
//               to scroll through.
//
// An exemption that no longer matches an offender is stale and fails too, so
// the list only shrinks.
//
// Usage:
//   node scripts/check-readable-code.mjs            check the tracked files
//   node scripts/check-readable-code.mjs FILE...    check the named files only

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const REPOSITORY = join(dirname(fileURLToPath(import.meta.url)), '..');
export const EXEMPTIONS_FILE = 'data/meta/readable-code-exemptions.lino';
export const ROOTS = ['js', 'ts', 'rust', 'scripts', 'packages'];
export const MAXIMUM_AVERAGE = 160;
export const MAXIMUM_CODE_CHARACTERS = 300;

/** The code extensions measured, each with the lexical family that reads it. */
const FAMILIES = {
  js: 'script', mjs: 'script', cjs: 'script', jsx: 'script',
  ts: 'script', mts: 'script', cts: 'script', tsx: 'script',
  rs: 'rust', py: 'python', sh: 'shell', bash: 'shell',
};

/**
 * The lexical family of a path, or null when the path is not code.
 * @param {string} path
 * @returns {string|null}
 */
export function familyOf(path) {
  const dot = path.lastIndexOf('.');
  if (dot < 0 || dot < path.lastIndexOf('/')) return null;
  return FAMILIES[path.slice(dot + 1)] ?? null;
}

/** The closing delimiter a Rust raw string opened at `index` needs, or null. */
function rustRawStringEnd(text, index) {
  let position = index + 1;
  if (text[index] === 'b' && text[position] === 'r') position += 1;
  else if (text[index] !== 'r') return null;
  const hashesStart = position;
  while (text[position] === '#') position += 1;
  if (text[position] !== '"') return null;
  return { body: position + 1, close: `"${'#'.repeat(position - hashesStart)}` };
}

/** The length of a Rust character literal starting at `index`, or 0 for a lifetime. */
function rustCharacterLength(text, index) {
  const match = /^'(?:\\(?:u\{[0-9a-fA-F]+\}|x[0-9a-fA-F]{2}|.)|[^\\'\n])'/u.exec(text.slice(index, index + 12));
  return match ? match[0].length : 0;
}

/**
 * Which characters of `text` are code: 1 for a character outside string
 * literals, template text and comments (delimiters count as code), 0 inside.
 * Indexed by UTF-16 unit, like `text` itself.
 * @param {string} text
 * @param {string} family 'script' | 'rust' | 'python' | 'shell'
 * @returns {Uint8Array}
 */
export function codeMask(text, family) {
  const mask = new Uint8Array(text.length);
  const lineComments = family === 'script' || family === 'rust';
  const hashComments = family === 'python' || family === 'shell';
  // The open contexts, innermost last: { kind: 'code', braces } inside a
  // template interpolation, { kind: 'template' } inside a template literal.
  const stack = [];
  let index = 0;
  const code = (length) => {
    mask.fill(1, index, Math.min(index + length, text.length));
    index += length;
  };
  const skipTo = (end) => {
    index = Math.min(Math.max(end, index), text.length);
  };
  while (index < text.length) {
    const character = text[index];
    const top = stack[stack.length - 1];
    if (top && top.kind === 'template') {
      if (character === '\\') skipTo(index + 2);
      else if (character === '`') { stack.pop(); code(1); }
      else if (character === '$' && text[index + 1] === '{') { stack.push({ kind: 'code', braces: 0 }); code(2); }
      else skipTo(index + 1);
      continue;
    }
    if (top && top.kind === 'code' && character === '{') top.braces += 1;
    if (top && top.kind === 'code' && character === '}') {
      if (top.braces === 0) { stack.pop(); code(1); continue; }
      top.braces -= 1;
    }
    if (lineComments && text.startsWith('//', index)) {
      const end = text.indexOf('\n', index);
      skipTo(end < 0 ? text.length : end);
    } else if (lineComments && text.startsWith('/*', index)) {
      const end = text.indexOf('*/', index + 2);
      skipTo(end < 0 ? text.length : end + 2);
    } else if (hashComments && character === '#' && (index === 0 || /\s/u.test(text[index - 1]))) {
      const end = text.indexOf('\n', index);
      skipTo(end < 0 ? text.length : end);
    } else if (family === 'script' && character === '`') {
      stack.push({ kind: 'template' });
      code(1);
    } else if (family === 'rust' && /[rb]/u.test(character) && !/\w/u.test(text[index - 1] ?? '') && rustRawStringEnd(text, index)) {
      const raw = rustRawStringEnd(text, index);
      const end = text.indexOf(raw.close, raw.body);
      code(1);
      skipTo(end < 0 ? text.length : end + raw.close.length);
    } else if (family === 'rust' && character === '\'') {
      const length = rustCharacterLength(text, index);
      if (length > 0) skipTo(index + length);
      else code(1);
    } else if (character === '"' || character === '\'') {
      const triple = family === 'python' && text.startsWith(character.repeat(3), index);
      const close = triple ? character.repeat(3) : character;
      // Script strings end at a newline; Rust and Python triple-quoted strings
      // span lines. A shell string may too, but nested quoting inside `$(...)`
      // cannot be read without a shell parser, so a line bounds the guess.
      const endsAtNewline = family === 'script' || family === 'shell' || (family === 'python' && !triple);
      // A shell single-quoted string has no escapes.
      const escapes = family !== 'shell' || character === '"';
      let position = index + close.length;
      while (position < text.length && !text.startsWith(close, position)) {
        if (escapes && text[position] === '\\') position += 2;
        else if (endsAtNewline && text[position] === '\n') break;
        else position += 1;
      }
      code(close.length);
      if (text.startsWith(close, position)) {
        skipTo(position);
        code(close.length);
      } else {
        skipTo(position);
      }
    } else {
      code(1);
    }
  }
  return mask;
}

/**
 * The number of code characters on every line of `text` (see `codeMask`),
 * counted in code points.
 * @param {string} text
 * @param {string} family
 * @returns {Array<number>}
 */
export function codeCharactersPerLine(text, family) {
  const mask = codeMask(text, family);
  const counts = [0];
  for (let index = 0; index < text.length; index += 1) {
    const unit = text.charCodeAt(index);
    if (unit === 0x0a) counts.push(0);
    else if (mask[index] === 1 && (unit < 0xdc00 || unit > 0xdfff)) counts[counts.length - 1] += 1;
  }
  return counts;
}

/**
 * The readability problems of one file's text: `packed` and `long line`.
 * @param {string} text
 * @param {string} family
 * @returns {Array<string>}
 */
export function readabilityProblems(text, family) {
  const problems = [];
  const characters = [...text].length;
  const lines = text.split('\n').length - (text.endsWith('\n') ? 1 : 0);
  const needed = Math.ceil(characters / MAXIMUM_AVERAGE);
  if (lines < needed) {
    problems.push(`packed: ${lines} line(s) for ${characters} characters (at least ${needed} needed)`);
  }
  const counts = codeCharactersPerLine(text, family);
  const long = counts.flatMap((codeCharacters, line) => (codeCharacters > MAXIMUM_CODE_CHARACTERS ? [line + 1] : []));
  if (long.length > 0) {
    const shown = long.slice(0, 5).map((line) => `${line} (${counts[line - 1]})`).join(', ');
    problems.push(`long line: ${long.length} line(s) over ${MAXIMUM_CODE_CHARACTERS} code characters, at ${shown}${long.length > 5 ? ', …' : ''}`);
  }
  return problems;
}

/**
 * The exemptions: `readable-code-exemption <path>` records, each with a
 * `reason`. A path ending in `/` exempts every file under it.
 * @param {string} text
 * @returns {Array<{ path: string, reason: string }>}
 */
export function parseExemptions(text) {
  const exemptions = [];
  for (const line of text.split('\n')) {
    const record = /^readable-code-exemption\s+(\S+)\s*$/u.exec(line);
    const reason = /^\s+reason\s+"(.*)"\s*$/u.exec(line);
    if (record) exemptions.push({ path: record[1], reason: '' });
    else if (reason && exemptions.length > 0) exemptions[exemptions.length - 1].reason = reason[1];
  }
  return exemptions;
}

/** Whether an exemption covers a path. */
export function exempts(exemption, path) {
  return exemption.path.endsWith('/') ? path.startsWith(exemption.path) : path === exemption.path;
}

/** The tracked code files under the measured roots. */
function trackedCodeFiles(repository) {
  return execFileSync('git', ['ls-files', '--', ...ROOTS], { cwd: repository, encoding: 'utf8', maxBuffer: 1 << 28 })
    .split('\n')
    .filter((path) => path && familyOf(path));
}

/**
 * Check files; returns the report lines and whether the check failed.
 * @param {string} repository
 * @param {Array<string>} [only] repository-relative paths to check instead of every tracked one
 */
export function checkReadableCode(repository, only) {
  const exemptions = parseExemptions(readFileSync(join(repository, EXEMPTIONS_FILE), 'utf8'));
  const report = [];
  const used = new Set();
  let failed = false;
  for (const exemption of exemptions) {
    if (!exemption.reason) {
      report.push(`${EXEMPTIONS_FILE}: ${exemption.path} has no reason`);
      failed = true;
    }
  }
  const files = only ?? trackedCodeFiles(repository);
  let offenders = 0;
  for (const path of files) {
    let text;
    try {
      text = readFileSync(join(repository, path), 'utf8');
    } catch {
      continue;
    }
    const problems = readabilityProblems(text, familyOf(path) ?? 'script');
    if (problems.length === 0) continue;
    const exemption = exemptions.find((candidate) => exempts(candidate, path));
    if (exemption) {
      used.add(exemption.path);
      continue;
    }
    offenders += 1;
    failed = true;
    report.push(`${path}: ${problems.join('; ')}`);
  }
  if (!only) {
    for (const exemption of exemptions) {
      if (!used.has(exemption.path)) {
        failed = true;
        report.push(`${EXEMPTIONS_FILE}: ${exemption.path} is exempt but readable (or gone); remove the exemption`);
      }
    }
  }
  report.push(failed
    ? `readable code: ${offenders} unreadable file(s) of ${files.length}; break the lines (fix the generator for a generated file), or list a true bundle with its reason in ${EXEMPTIONS_FILE}`
    : `readable code: ${files.length} code files are multi-line and readable (${used.size} exemption(s) in use)`);
  return { report, failed };
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const named = process.argv.slice(2);
  const { report, failed } = checkReadableCode(REPOSITORY, named.length > 0 ? named : undefined);
  for (const line of report) console.log(line);
  process.exitCode = failed ? 1 : 0;
}
