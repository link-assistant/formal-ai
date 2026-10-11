#!/usr/bin/env node
// Self-translation between JavaScript, the meta language and Rust for the
// portable-pure-v1 fragment (R1024), in JavaScript first.
//
// The practices are copied from link-foundation/meta-language PR #196
// (docs/self-translation.md: the provenance header, translated and carried
// blocks, lossless round trips, the shared Links Notation corpus and the
// translated-against-hand-written report) and link-foundation/relative-meta-logic
// (test-corpus/ + scripts/check-corpus-parity.mjs, and PR #184's
// portable-natural fragment with its refusal-by-name contract); both are in
// the public domain under the Unlicense. What was adopted and why is
// docs/case-studies/pull-request-1188/conversion-best-practices.md.
//
// Usage:
//   node scripts/self-translate.mjs --to rust [--from js] [--items] FILE
//   node scripts/self-translate.mjs --to meta FILE      the links IR
//   node scripts/self-translate.mjs --from meta --to rust FILE.lino
//   node scripts/self-translate.mjs --check             the corpus matches
//   node scripts/self-translate.mjs --write             rewrite the corpus
//   node scripts/self-translate.mjs --report [DIR...]   translated vs hand-written

import { existsSync, readFileSync, readdirSync, statSync, writeFileSync } from 'node:fs';
import { dirname, extname, join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';

import { fromMeta, languageOf, selfTranslate, toMeta } from './self-translation/envelope.mjs';
import { fromLino } from './self-translation/ir.mjs';
import { lex, topLevelItems } from './self-translation/lexer.mjs';
import { field, parseLinks, textOf } from './self-translation/lino.mjs';

const REPOSITORY = join(dirname(fileURLToPath(import.meta.url)), '..');
export const CORPUS = 'rust/tests/fixtures/self-translation';
const REPORT_ROOTS = ['js/agentic/crate'];

/**
 * The corpus: `(case ...)` and `(call ...)` links of cases.lino.
 * @param {string} [root] the repository
 * @returns {{cases: Array<object>, calls: Array<object>}}
 */
export function readCorpus(root = REPOSITORY) {
  const links = parseLinks(readFileSync(join(root, CORPUS, 'cases.lino'), 'utf8'));
  const value = (link, head) => textOf(field(link, head)[1]);
  const typed = (link) => ({ type: link[0], value: textOf(link[1]) });
  return {
    cases: links.filter((link) => link[0] === 'case').map((link) => ({
      name: link[1],
      source: value(link, 'source'),
      from: value(link, 'from'),
      to: value(link, 'to'),
      expected: value(link, 'expected'),
    })),
    calls: links.filter((link) => link[0] === 'call').map((link) => ({
      case: link[1],
      javascript: value(link, 'javascript'),
      rust: value(link, 'rust'),
      arguments: field(link, 'arguments').slice(1).map(typed),
      result: typed(field(link, 'result')[1]),
    })),
  };
}

/**
 * What a case produces now: the translation and, except for the meta leg,
 * its item list as Links Notation.
 * @param {object} entry a corpus case
 * @param {string} [root]
 * @returns {{code: string, items: string | null}}
 */
export function runCase(entry, root = REPOSITORY) {
  const source = readFileSync(join(root, CORPUS, entry.source), 'utf8');
  if (entry.to === 'Meta') return { code: toMeta(source, entry.from), items: null };
  if (entry.from === 'Meta') return { code: fromMeta(fromLino(source), entry.to), items: null };
  const result = selfTranslate(source, entry.from, entry.to);
  return { code: result.code, items: itemsLino(result.items) };
}

/**
 * One `(item start end term status "reason")` link per item, as
 * meta-language's `.items.lino` files record them.
 * @param {Array<object>} items
 * @returns {string}
 */
export function itemsLino(items) {
  return items.map(({ start, end, term, status, reason }) => `(item ${start} ${end} ${term} ${status}${reason ? ` "${reason.replace(/"/gu, "'")}"` : ''})\n`).join('');
}

/** The committed item list of a case. */
export const itemsPath = (entry) => `expected/${entry.name}.items.lino`;

/**
 * Compare (or with `write`, rewrite) every expected file of the corpus.
 * A case whose expected file is a source is only compared: it asserts the
 * round trip restores that source byte for byte.
 * @param {boolean} write
 * @param {string} [root]
 * @returns {Array<string>} the paths that differ
 */
export function checkCorpus(write, root = REPOSITORY) {
  const drifted = [];
  for (const entry of readCorpus(root).cases) {
    const produced = runCase(entry, root);
    const outputs = [[entry.expected, produced.code]];
    if (produced.items !== null) outputs.push([itemsPath(entry), produced.items]);
    for (const [path, text] of outputs) {
      const full = join(root, CORPUS, path);
      const committed = existsSync(full) ? readFileSync(full, 'utf8') : null;
      if (committed === text) continue;
      drifted.push(`${CORPUS}/${path}`);
      if (write && !path.startsWith('sources/')) writeFileSync(full, text);
    }
  }
  return drifted;
}

/**
 * The translated-against-hand-written report of meta-language's
 * `generate-self-translation-report.mjs`, for this repository's JavaScript
 * twins: per module, the items by status, the carried reasons, and how many
 * translated functions the Rust module the twin cites defines under the
 * same name.
 * @param {Array<string>} roots repository-relative directories
 * @param {string} [root]
 * @returns {Array<{module: string, items: number, translated: number, carried: number, reasons: Map<string, number>, rust: string | null, named: number, identical: number}>}
 */
export function report(roots, root = REPOSITORY) {
  const rows = [];
  const walk = (directory) => readdirSync(directory).sort().flatMap((name) => {
    const path = join(directory, name);
    return statSync(path).isDirectory() ? walk(path) : ['.mjs', '.js'].includes(extname(name)) ? [path] : [];
  });
  for (const directory of roots) {
    for (const path of walk(join(root, directory))) {
      const source = readFileSync(path, 'utf8');
      let result;
      try {
        result = selfTranslate(source, 'JavaScript', 'Rust');
      } catch (error) {
        rows.push({ module: relative(root, path), items: 0, translated: 0, carried: 0, reasons: new Map([[`unreadable: ${error.message}`, 1]]), rust: null, named: 0, identical: 0 });
        continue;
      }
      const code = result.items.filter((item) => item.term !== 'comment');
      const reasons = new Map();
      for (const item of code) {
        const reason = item.status === 'carried' ? item.reason.replace(/:[\s\S]*$/u, '') : null;
        if (reason) reasons.set(reason, (reasons.get(reason) ?? 0) + 1);
      }
      const cited = /rust\/src\/[A-Za-z0-9_/]+\.rs/u.exec(source);
      const rustPath = cited ? cited[0] : null;
      const definitions = rustPath && existsSync(join(root, rustPath)) ? rustDefinitions(readFileSync(join(root, rustPath), 'utf8')) : new Map();
      const translated = translatedRust(result.code);
      rows.push({
        module: relative(root, path),
        items: code.length,
        translated: code.filter((item) => item.status === 'translated').length,
        carried: code.filter((item) => item.status === 'carried').length,
        reasons,
        rust: rustPath,
        named: [...translated.keys()].filter((name) => definitions.has(name)).length,
        identical: [...translated].filter(([name, tokens]) => definitions.get(name) === tokens).length,
      });
    }
  }
  return rows;
}

// A definition's tokens without attributes and visibility, joined by one
// space: two definitions are identical up to whitespace when these match.
function definitionTokens(tokens) {
  const words = tokens.filter((token) => token.type !== 'comment');
  let at = 0;
  while (words[at]?.text === '#') {
    let depth = 0;
    for (at += 1; at < words.length; at += 1) {
      if (words[at].text === '[') depth += 1;
      if (words[at].text === ']' && --depth === 0) break;
    }
    at += 1;
  }
  if (words[at]?.text === 'pub') at += words[at + 1]?.text === '(' ? 4 : 1;
  return words.slice(at).map((token) => token.text).join(' ');
}

/** The top-level `fn` and `const` definitions of a Rust module by name. */
function rustDefinitions(source) {
  const table = new Map();
  let items;
  try {
    items = topLevelItems(lex(source, 'Rust'));
  } catch {
    // A module the fragment lexer cannot read (raw strings, lifetimes in
    // odd places) is matched by name only.
    for (const match of source.matchAll(/\b(?:fn|const|static)\s+([A-Za-z_][A-Za-z0-9_]*)/gu)) table.set(match[1], null);
    return table;
  }
  for (const item of items) {
    const words = item.tokens.filter((token) => token.type !== 'comment').map((token) => token.text);
    const at = words.findIndex((word, index) => (word === 'fn' || word === 'const' || word === 'static') && words[index + 1] !== 'fn');
    if (at >= 0 && words[at + 1]) table.set(words[at + 1], definitionTokens(item.tokens));
  }
  return table;
}

/** The translated definitions of a self-translation's Rust by name. */
function translatedRust(code) {
  const table = new Map();
  for (const item of topLevelItems(lex(code, 'Rust'))) {
    if (item.comment) continue;
    const words = item.tokens.map((token) => token.text);
    const at = words.findIndex((word, index) => (word === 'fn' || word === 'const') && words[index + 1] !== 'fn');
    if (at >= 0) table.set(words[at + 1], definitionTokens(item.tokens));
  }
  return table;
}

function printReport(rows) {
  const total = { items: 0, translated: 0, carried: 0, named: 0, identical: 0 };
  const reasons = new Map();
  for (const row of rows) {
    for (const key of Object.keys(total)) total[key] += row[key];
    for (const [reason, count] of row.reasons) reasons.set(reason, (reasons.get(reason) ?? 0) + count);
    if (row.translated > 0) console.log(`${row.module}: ${row.translated}/${row.items} translated, ${row.named} named and ${row.identical} identical in ${row.rust ?? 'no cited Rust module'}`);
  }
  console.log(`${rows.length} modules, ${total.items} items: ${total.translated} translated, ${total.carried} carried; of the translated, ${total.named} are defined under the same name in the cited Rust module and ${total.identical} identically up to whitespace`);
  for (const [reason, count] of [...reasons].sort((a, b) => b[1] - a[1])) console.log(`  carried ${count}: ${reason}`);
}

function main(argv) {
  if (argv[0] === '--check' || argv[0] === '--write') {
    const drifted = checkCorpus(argv[0] === '--write');
    if (drifted.length === 0) {
      console.log(`${CORPUS} matches the translator`);
      return 0;
    }
    console.log(`${argv[0] === '--write' ? 'rewrote' : 'drifted'}: ${drifted.join(', ')}`);
    if (argv[0] === '--write') return drifted.some((path) => path.includes('/sources/')) ? 1 : 0;
    console.log('regenerate with: node scripts/self-translate.mjs --write');
    return 1;
  }
  if (argv[0] === '--report') {
    printReport(report(argv.length > 1 ? argv.slice(1) : REPORT_ROOTS));
    return 0;
  }
  const option = (name) => {
    const at = argv.indexOf(name);
    return at < 0 ? null : argv[at + 1];
  };
  const file = argv.filter((arg, index) => !arg.startsWith('--') && !['--to', '--from'].includes(argv[index - 1])).pop();
  const target = option('--to');
  if (!file || !target) {
    console.error('usage: node scripts/self-translate.mjs --to rust|js|meta [--from js|rust|meta] [--items] FILE');
    return 2;
  }
  const source = readFileSync(file, 'utf8');
  const fromName = option('--from') ?? extname(file).slice(1);
  if (['meta', 'lino'].includes(fromName.toLowerCase())) {
    process.stdout.write(fromMeta(fromLino(source), target));
    return 0;
  }
  const from = languageOf(fromName);
  if (target.toLowerCase() === 'meta') {
    process.stdout.write(toMeta(source, from));
    return 0;
  }
  const result = selfTranslate(source, from, target);
  process.stdout.write(argv.includes('--items') ? itemsLino(result.items) : result.code);
  return 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
