#!/usr/bin/env node
// JavaScript twin of `scripts/check-language-parity.rs` and the debt half of
// `scripts/language-parity-lib.rs` (PR #1188, LEXEMES).
//
// Check structural en/ru/hi/zh/es lexeme parity and its explicit dated debt:
// every meaning that owns lexemes in some of the five target languages but not
// all of them needs one exact `uncovered_behavior` row in
// `data/meta/language-parity-debt.lino`. Same output, same exit code and the
// same `--write` regeneration as the Rust original, so a coding agent can run
// the gate without compiling Rust; CI runs both and diffs them
// (`data/meta/ci-gates/check-language-parity-js-twin.lino`).
//
// Usage:
//   node scripts/check-language-parity.mjs
//   node scripts/check-language-parity.mjs --count
//   node scripts/check-language-parity.mjs --write --date YYYY-MM-DD
//
// The census itself (which lines are lexemes, who owns them) lives in
// `scripts/lib/checks-language-parity.mjs`, shared with the debt ratchet twin.

import { writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

import {
  SEED_ROOT, TARGET_LANGUAGES, joinPath, repositoryGaps,
} from './lib/checks-language-parity.mjs';
import {
  RustError, compareStrings, compareTuples, debugStringList, ioErrorMessage, isWhitespace, lines,
  readToString, splitOnce, trim,
} from './lib/checks-rust-compat.mjs';

export const DEBT_FILE = 'data/meta/language-parity-debt.lino';

const USAGE = 'usage: check-language-parity.rs [--repo <path>] [--check|--count|--write --date YYYY-MM-DD]';

const ESCAPES = new Map([
  ['\\', '\\'],
  ['"', '"'],
  ['n', '\n'],
  ['r', '\r'],
  ['t', '\t'],
]);

/** `LanguageGap::description`. */
function describeGap(gap) {
  return `meaning \`${gap.meaning}\` lacks lexeme languages: ${gap.missing.join(',')}`;
}

/** The `(source, owner_path)` key a gap and a debt row share. */
function rowKey(row) {
  return [row.source, row.ownerPath];
}

/** `indentation`: the count of leading spaces; a tab in the indent is an error. */
function indentation(line, source, lineNumber) {
  const prefix = line.length - line.replace(/^[ \t]*/, '').length;
  if (line.slice(0, prefix).includes('\t')) {
    throw new RustError(`${source}:${lineNumber}: tab indentation is not a structural Links Notation indent`);
  }
  return prefix;
}

/** `quote`: a Links Notation string literal. */
export function quote(value) {
  let output = '"';
  for (const character of value) {
    switch (character) {
      case '\\': output += '\\\\'; break;
      case '"': output += '\\"'; break;
      case '\n': output += '\\n'; break;
      case '\r': output += '\\r'; break;
      case '\t': output += '\\t'; break;
      default: output += character;
    }
  }
  return `${output}"`;
}

/** `unquote`: the value of a quoted Links Notation string. */
function unquote(rawValue) {
  const value = trim(rawValue);
  if (!value.startsWith('"') || !value.endsWith('"') || Buffer.byteLength(value, 'utf8') < 2) {
    throw new RustError(`expected a quoted value, found \`${value}\``);
  }
  let output = '';
  let escaped = false;
  for (const character of value.slice(1, -1)) {
    if (escaped) {
      if (!ESCAPES.has(character)) {
        throw new RustError(`unsupported escape \`\\${character}\` in ${value}`);
      }
      output += ESCAPES.get(character);
      escaped = false;
    } else if (character === '\\') {
      escaped = true;
    } else {
      output += character;
    }
  }
  if (escaped) throw new RustError(`unfinished escape in ${value}`);
  return output;
}

/** `parse_csv`: a quoted comma-separated list; the empty string is no items. */
function parseCsv(value) {
  const text = unquote(value);
  return text === '' ? [] : text.split(',');
}

/** `split_quoted_words`: whitespace-separated words, quotes grouping a word. */
function splitQuotedWords(value) {
  const words = [];
  let current = '';
  let quoted = false;
  let escaped = false;
  let active = false;
  for (const character of value) {
    if (escaped) {
      if (!ESCAPES.has(character)) {
        throw new RustError(`unsupported escape \`\\${character}\` in \`${value}\``);
      }
      current += ESCAPES.get(character);
      escaped = false;
    } else if (quoted && character === '\\') {
      escaped = true;
    } else if (character === '"') {
      quoted = !quoted;
      active = true;
    } else if (isWhitespace(character) && !quoted) {
      if (active) {
        words.push(current);
        current = '';
        active = false;
      }
    } else {
      current += character;
      active = true;
    }
  }
  if (quoted || escaped) throw new RustError(`unfinished quoted value in \`${value}\``);
  if (active) words.push(current);
  return words;
}

function isLeapYear(year) {
  return year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
}

/** `valid_date`: a real `YYYY-MM-DD` calendar date. */
export function validDate(value) {
  const parts = value.split('-');
  const shaped = parts.length === 3
    && parts[0].length === 4
    && parts[1].length === 2
    && parts[2].length === 2
    && /^[0-9-]*$/.test(value);
  if (!shaped) return false;
  const [year, month, day] = parts.map(Number);
  const daysInMonth = {
    1: 31, 2: isLeapYear(year) ? 29 : 28, 3: 31, 4: 30, 5: 31, 6: 30,
    7: 31, 8: 31, 9: 30, 10: 31, 11: 30, 12: 31,
  }[month];
  if (daysInMonth === undefined) return false;
  return year > 0 && day > 0 && day <= daysInMonth;
}

/** `finish_row`: turn one row's quoted fields into a debt row. */
function finishRow(fields, rows) {
  if (fields.size === 0) return;
  const take = (name) => {
    if (!fields.has(name)) throw new RustError(`uncovered_behavior row lacks \`${name}\``);
    const value = fields.get(name);
    fields.delete(name);
    return value;
  };
  const source = unquote(take('source'));
  const ownerPath = unquote(take('owner_path'));
  const meaning = unquote(take('meaning'));
  const present = parseCsv(take('present_languages'));
  const missing = parseCsv(take('missing_languages'));
  const observedOn = unquote(take('observed_on'));
  const gap = unquote(take('gap'));
  if (fields.size !== 0) {
    const unknown = [...fields.keys()].sort(compareStrings);
    throw new RustError(`uncovered_behavior row has unknown fields: ${debugStringList(unknown)}`);
  }
  if (!validDate(observedOn)) throw new RustError(`invalid observed_on date \`${observedOn}\``);
  rows.push({ source, ownerPath, meaning, present, missing, observedOn, gap });
}

const EXPECTED_HEADERS = ['measured_on', 'record_type', 'source_root', 'target_languages'];

/** `parse_debt`: the dated debt ledger; throws `RustError` with Rust's text. */
export function parseDebt(text) {
  const header = new Map();
  const rows = [];
  let sawRoot = false;

  lines(text).forEach((line, index) => {
    const lineNumber = index + 1;
    const trimmed = trim(line);
    if (trimmed === '' || trimmed.startsWith('#')) return;
    const indent = indentation(line, DEBT_FILE, lineNumber);
    if (!sawRoot) {
      if (indent !== 0 || trimmed !== 'language_parity_debt') {
        throw new RustError(`${DEBT_FILE}:${lineNumber}: expected \`language_parity_debt\` root`);
      }
      sawRoot = true;
      return;
    }
    if (indent === 2 && trimmed.startsWith('uncovered_behavior ')) {
      const words = splitQuotedWords(trimmed);
      if (words[0] !== 'uncovered_behavior' || words.length !== 15) {
        throw new RustError(`${DEBT_FILE}:${lineNumber}: uncovered_behavior must name exactly seven fields`);
      }
      const fields = new Map();
      for (let at = 1; at + 1 < words.length; at += 2) {
        const [key, value] = [words[at], words[at + 1]];
        if (fields.has(key)) {
          throw new RustError(`${DEBT_FILE}:${lineNumber}: duplicate debt row field \`${key}\``);
        }
        fields.set(key, quote(value));
      }
      finishRow(fields, rows);
      return;
    }
    const pair = splitOnce(trimmed, ' ');
    if (pair === null) {
      throw new RustError(`${DEBT_FILE}:${lineNumber}: expected a field and value`);
    }
    const [key, value] = pair;
    if (indent !== 2 || rows.length !== 0) {
      throw new RustError(`${DEBT_FILE}:${lineNumber}: unexpected indentation or field \`${trimmed}\``);
    }
    if (header.has(key)) throw new RustError(`duplicate debt header field \`${key}\``);
    header.set(key, value);
  });

  if (!sawRoot) throw new RustError(`${DEBT_FILE}: missing root`);
  const found = [...header.keys()].sort(compareStrings);
  if (found.join('\u0000') !== EXPECTED_HEADERS.join('\u0000')) {
    throw new RustError(
      `debt header fields must be exactly ${debugStringList(EXPECTED_HEADERS)}; found ${debugStringList(found)}`,
    );
  }
  const recordType = unquote(header.get('record_type'));
  const measuredOn = unquote(header.get('measured_on'));
  const sourceRoot = unquote(header.get('source_root'));
  const targetLanguages = unquote(header.get('target_languages'));
  if (recordType !== 'language_parity_debt') throw new RustError(`unexpected record_type \`${recordType}\``);
  if (sourceRoot !== SEED_ROOT) throw new RustError(`unexpected source_root \`${sourceRoot}\``);
  if (targetLanguages !== TARGET_LANGUAGES.join(',')) {
    throw new RustError(`unexpected target_languages \`${targetLanguages}\``);
  }
  if (!validDate(measuredOn)) throw new RustError(`invalid measured_on date \`${measuredOn}\``);
  return { measuredOn, rows };
}

/** `expected_row`: the debt row a gap needs on `date`. */
function expectedRow(gap, date) {
  return {
    source: gap.source,
    ownerPath: gap.ownerPath,
    meaning: gap.meaning,
    present: [...gap.present],
    missing: [...gap.missing],
    observedOn: date,
    gap: describeGap(gap),
  };
}

/** Derived `PartialEq for DebtRow`: rows built in one field order compare as JSON. */
function sameRow(left, right) {
  return JSON.stringify(left) === JSON.stringify(right);
}

/** `render_debt`: the canonical ledger text for `gaps` measured on `date`. */
export function renderDebt(gaps, date) {
  let output = 'language_parity_debt\n'
    + '  record_type "language_parity_debt"\n'
    + `  measured_on ${quote(date)}\n`
    + '  target_languages "en,ru,hi,zh,es"\n'
    + '  source_root "data/seed"\n';
  for (const gap of gaps) {
    const row = expectedRow(gap, date);
    output += `  uncovered_behavior source ${quote(row.source)}`
      + ` owner_path ${quote(row.ownerPath)}`
      + ` meaning ${quote(row.meaning)}`
      + ` present_languages ${quote(row.present.join(','))}`
      + ` missing_languages ${quote(row.missing.join(','))}`
      + ` observed_on ${quote(row.observedOn)}`
      + ` gap ${quote(row.gap)}\n`;
  }
  return output;
}

/** A map keyed by `(source, owner_path)` with entries in `BTreeMap` order. */
function sortedEntries(map) {
  return [...map.values()].sort((left, right) => compareTuples(left.key, right.key));
}

/** `check_debt`: every way the ledger disagrees with the live gaps. */
export function checkDebt(gaps, text) {
  let debt;
  try {
    debt = parseDebt(text);
  } catch (error) {
    if (error instanceof RustError) return [error.message];
    throw error;
  }
  const failures = [];
  const actual = new Map();
  for (const gap of gaps) {
    const key = rowKey(gap);
    actual.set(key.join('\u0000'), { key, value: gap });
  }
  const declared = new Map();
  for (const row of debt.rows) {
    const key = rowKey(row);
    const id = key.join('\u0000');
    if (declared.has(id)) {
      failures.push(`duplicate debt row for ${key[0]} at structural owner ${key[1]}`);
    }
    declared.set(id, { key, value: row });
  }
  for (const { key, value: gap } of sortedEntries(actual)) {
    const declaredRow = declared.get(key.join('\u0000'));
    if (declaredRow === undefined) {
      failures.push(`missing debt row for ${gap.source} meaning \`${gap.meaning}\` (missing ${gap.missing.join(',')})`);
    } else if (!sameRow(declaredRow.value, expectedRow(gap, debt.measuredOn))) {
      failures.push(
        `stale debt row for ${gap.source} meaning \`${gap.meaning}\`; `
          + 'present/missing languages, date, or gap text no longer matches',
      );
    }
  }
  for (const { key, value: row } of sortedEntries(declared)) {
    if (!actual.has(key.join('\u0000'))) {
      failures.push(
        `stale debt row for ${row.source} meaning \`${row.meaning}\`; the structural owner is complete or absent`,
      );
    }
  }
  if (failures.length === 0 && text !== renderDebt(gaps, debt.measuredOn)) {
    failures.push(
      `${DEBT_FILE} is not in canonical source/owner order; regenerate it with --write --date ${debt.measuredOn}`,
    );
  }
  return failures;
}

/** The value after a flag, or a `RustError` naming what the flag needs. */
function flagValue(args, message) {
  const next = args.next();
  if (next.done) throw new RustError(message);
  return next.value;
}

/** `run`: the message to print on success; throws `RustError` on failure. */
export function run(argv) {
  let root = '.';
  let writeDate = null;
  let countOnly = false;
  const args = argv[Symbol.iterator]();
  for (let next = args.next(); !next.done; next = args.next()) {
    const argument = next.value;
    switch (argument) {
      case '--repo':
        root = flagValue(args, '--repo requires a path');
        break;
      case '--write':
        if (writeDate === null) writeDate = '';
        break;
      case '--date':
        writeDate = flagValue(args, '--date requires YYYY-MM-DD');
        break;
      case '--count':
        countOnly = true;
        break;
      case '--check':
        break;
      case '--help':
      case '-h':
        return USAGE;
      default:
        throw new RustError(`unknown argument: ${argument}`);
    }
  }
  const gaps = repositoryGaps(root);
  if (countOnly) {
    if (writeDate !== null) throw new RustError('--count and --write are mutually exclusive');
    return String(gaps.length);
  }
  if (writeDate !== null) {
    if (!validDate(writeDate)) throw new RustError('--write requires --date YYYY-MM-DD');
    try {
      writeFileSync(joinPath(root, DEBT_FILE), renderDebt(gaps, writeDate));
    } catch (error) {
      throw new RustError(`cannot write ${DEBT_FILE}: ${ioErrorMessage(error)}`);
    }
    return `recorded ${gaps.length} language-parity gaps in ${DEBT_FILE}`;
  }
  let debt;
  try {
    debt = readToString(joinPath(root, DEBT_FILE));
  } catch (error) {
    throw new RustError(`cannot read ${DEBT_FILE}: ${ioErrorMessage(error)}`);
  }
  const failures = checkDebt(gaps, debt);
  if (failures.length !== 0) throw new RustError(failures.join('\n'));
  return `language parity: ${gaps.length} explicit dated gaps; debt rows are exact`;
}

function main() {
  try {
    process.stdout.write(`${run(process.argv.slice(2))}\n`);
  } catch (error) {
    if (!(error instanceof RustError)) throw error;
    process.stderr.write(`language parity check failed:\n${error.message}\n`);
    process.exitCode = 1;
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) main();
