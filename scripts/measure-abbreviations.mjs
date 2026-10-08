#!/usr/bin/env node
// Names are full English words (PR #1188, R1188-U4): measure the abbreviated
// file names and the abbreviated bindings of the code, and hold them to
// data/meta/abbreviation-ratchet.lino, whose ceilings only fall.
//
// The abbreviation list and the proper terms live once in
// data/meta/notation-rules.lino; data/meta/abbreviation-rules.lino says where
// the rule applies. The measure reports:
//
//   abbreviated-file-names  tracked files whose stem holds a listed word
//   abbreviated-bindings    declarations, per code scope, whose name holds
//                           a listed word (`const args`, `let pos`, `fn calc`)
//
// File names are counted per file and bindings per declaration, so a new file
// or a new declaration spelled with an abbreviation raises a count and fails
// the gate, while a rename to full words lowers it.
//
// Usage:
//   node scripts/measure-abbreviations.mjs [--check]   measure and check the ratchet
//   node scripts/measure-abbreviations.mjs --write     also lower the ceilings to the measure
//   node scripts/measure-abbreviations.mjs --list <file-names|bindings> [scope]
//   node scripts/measure-abbreviations.mjs --propose [directory]
//                                     the full-word spelling of each abbreviated file name

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { basename, dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { parseTree, readRules } from './lib/links-notation-names.mjs';

export const RULES_FILE = 'data/meta/abbreviation-rules.lino';
export const RATCHET_FILE = 'data/meta/abbreviation-ratchet.lino';
const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const FILE_NAMES = 'file-names';

/**
 * The rules of data/meta/abbreviation-rules.lino.
 * @param {string} text
 * @returns {{abbreviationsFrom: string, fileNames: {excluded: Array<string>, fixedNames: Set<string>},
 *   bindings: Array<{scope: string, excluded: Array<string>}>}}
 */
export function readAbbreviationRules(text) {
  const top = parseTree(text).find((node) => node.line === 'abbreviation-rules');
  if (!top) {
    throw new Error(`${RULES_FILE} has no abbreviation-rules link`);
  }
  const rules = {
    abbreviationsFrom: '',
    fileNames: { excluded: [], fixedNames: new Set() },
    bindings: [],
  };
  const excludedOf = (node) =>
    node.children.filter((child) => child.line.startsWith('excluded ')).map((child) => child.line.split(/\s+/u)[1]);
  for (const node of top.children) {
    const [key, value] = node.line.split(/\s+/u);
    if (key === 'abbreviations-from') {
      rules.abbreviationsFrom = value;
    } else if (key === FILE_NAMES) {
      rules.fileNames.excluded = excludedOf(node);
      for (const child of node.children.filter((entry) => entry.line.startsWith('fixed-name '))) {
        rules.fileNames.fixedNames.add(child.line.split(/\s+/u)[1]);
      }
    } else if (key === 'bindings') {
      rules.bindings.push({ scope: value, excluded: excludedOf(node) });
    }
  }
  return rules;
}

/**
 * The words of a name: split at `-`, `_`, `$` and at a lower-to-upper case
 * change, lowercased (`parseArgs` is parse, args; `MAX_LEN` is max, len).
 * @param {string} name
 * @returns {Array<string>}
 */
export function wordsOfIdentifier(name) {
  return name
    .replace(/([a-z0-9])([A-Z])/gu, '$1_$2')
    .replace(/([A-Z]+)([A-Z][a-z])/gu, '$1_$2')
    .toLowerCase()
    .split(/[-_$]+/u)
    .filter(Boolean);
}

/**
 * The stem of a file name: the base name before the first `.`.
 * @param {string} path
 * @returns {string}
 */
export function stemOf(path) {
  return basename(path).split('.')[0];
}

/**
 * The abbreviated words of a name.
 * @param {string} name
 * @param {Map<string, string>} abbreviations
 * @returns {Array<string>}
 */
export function abbreviatedWords(name, abbreviations) {
  return wordsOfIdentifier(name).filter((word) => abbreviations.has(word));
}

/**
 * The full-word spelling of a file path: each abbreviated word of the stem
 * becomes its full form, joined with the separator the stem already uses.
 * @param {string} path
 * @param {Map<string, string>} abbreviations
 * @returns {string}
 */
export function fullWordPath(path, abbreviations) {
  const name = basename(path);
  const stem = stemOf(path);
  const separator = stem.includes('_') && !stem.includes('-') ? '_' : '-';
  const spelled = stem
    .split(/([-_])/u)
    .map((part) => (abbreviations.has(part.toLowerCase()) ? abbreviations.get(part.toLowerCase()).replaceAll('-', separator) : part))
    .join('');
  const directory = dirname(path);
  return `${directory === '.' ? '' : `${directory}/`}${spelled}${name.slice(stem.length)}`;
}

/**
 * The declared names of a source text. JavaScript: `const`, `let`, `var`,
 * `function` and `class` names, including `for (const x of ...)`. Rust:
 * `let` bindings, `fn` names and `const`/`static` items; a `let Some(x)` or
 * `let Err(error)` pattern and a `'static` lifetime are not names. Line
 * comments are left out, so prose that reads "let me" is not a binding.
 * @param {string} text
 * @param {'js' | 'rust'} language
 * @returns {Array<string>}
 */
export function declaredNames(text, language) {
  const pattern =
    language === 'rust'
      ? /(?<![\w'])(?:let\s+(?:mut\s+)?(?=[a-z_][a-z0-9_]*(?![\w({]|::))|fn\s+|(?:const|static)\s+(?:mut\s+)?)([A-Za-z_][A-Za-z0-9_]*)/gu
      : /\b(?:const|let|var|function\*?|class)\s+([A-Za-z_$][\w$]*)/gu;
  const names = [];
  for (const line of text.split('\n')) {
    const code = line.replace(/^\s*(?:\/\/|\*|\/\*).*$/u, '').replace(/\s\/\/\s.*$/u, '');
    for (const match of code.matchAll(pattern)) {
      if (match[1] !== 'fn' || language !== 'rust') {
        names.push(match[1]);
      }
    }
  }
  return names;
}

function trackedFiles(root, paths = []) {
  return execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', ...paths], {
    cwd: root,
    encoding: 'utf8',
    maxBuffer: 1 << 28,
  })
    .split('\n')
    .filter((path) => path && existsSync(join(root, path)));
}

const under = (excluded) => (path) => !excluded.some((prefix) => path === prefix || path.startsWith(`${prefix}/`));

/**
 * Measure the abbreviated file names and the abbreviated bindings.
 * @returns {Map<string, {count: number, list: Array<string>}>}
 */
export function measureAbbreviations(root = ROOT) {
  const rules = readAbbreviationRules(readFileSync(join(root, RULES_FILE), 'utf8'));
  const { abbreviations } = readRules(readFileSync(join(root, rules.abbreviationsFrom), 'utf8'));
  const measure = new Map();
  const files = trackedFiles(root)
    .filter(under(rules.fileNames.excluded))
    .filter((path) => !rules.fileNames.fixedNames.has(stemOf(path)))
    .filter((path) => abbreviatedWords(stemOf(path), abbreviations).length > 0)
    .sort();
  measure.set(FILE_NAMES, {
    key: 'abbreviated-file-names',
    count: files.length,
    list: files,
  });
  for (const { scope, excluded } of rules.bindings) {
    const list = [];
    for (const path of trackedFiles(root, [scope]).filter(under(excluded)).sort()) {
      const language = path.endsWith('.rs') ? 'rust' : /\.(?:m?js|jsx|cjs)$/u.test(path) ? 'js' : null;
      if (language === null) {
        continue;
      }
      for (const name of declaredNames(readFileSync(join(root, path), 'utf8'), language)) {
        if (abbreviatedWords(name, abbreviations).length > 0) {
          list.push(`${path}\t${name}`);
        }
      }
    }
    measure.set(`bindings ${scope}`, {
      key: 'abbreviated-bindings',
      count: list.length,
      list,
    });
  }
  return measure;
}

/**
 * The ceilings of the ratchet file: `file-names` and `bindings <scope>`.
 * @param {string} text
 * @returns {Map<string, number>}
 */
export function readRatchet(text) {
  const ceilings = new Map();
  const top = parseTree(text).find((node) => node.line === 'abbreviation-ratchet');
  for (const entry of top?.children ?? []) {
    const value = entry.children.map((child) => child.line.split(/\s+/u)).find(([key]) => key.startsWith('abbreviated-'));
    if (value) {
      ceilings.set(entry.line.trim(), Number(value[1]));
    }
  }
  return ceilings;
}

/**
 * The ratchet file text, keeping the header comment of `previous`.
 * @param {string} previous
 * @param {Map<string, {key: string, count: number}>} ceilings
 * @returns {string}
 */
export function renderRatchet(previous, ceilings) {
  const header = previous
    .split('\n')
    .filter((line, index, lines) => lines.slice(0, index + 1).every((earlier) => earlier.startsWith('#')))
    .join('\n');
  const lines = header ? [header] : [];
  lines.push('abbreviation-ratchet');
  for (const [entry, { key, count }] of ceilings) {
    lines.push(`  ${entry}`, `    ${key} ${count}`);
  }
  return `${lines.join('\n')}\n`;
}

function main(argv) {
  const measure = measureAbbreviations();
  const listAt = argv.indexOf('--list');
  if (listAt !== -1) {
    const kind = argv[listAt + 1];
    const scope = argv[listAt + 2];
    for (const [entry, { list }] of measure) {
      if (
        (kind === FILE_NAMES && entry === FILE_NAMES) ||
        (kind === 'bindings' && entry.startsWith('bindings ') && (!scope || entry === `bindings ${scope}`))
      ) {
        console.log(list.join('\n'));
      }
    }
    return 0;
  }
  if (argv.includes('--propose')) {
    const directory = argv[argv.indexOf('--propose') + 1];
    const { abbreviations } = readRules(
      readFileSync(join(ROOT, readAbbreviationRules(readFileSync(join(ROOT, RULES_FILE), 'utf8')).abbreviationsFrom), 'utf8'),
    );
    for (const path of measure
      .get(FILE_NAMES)
      .list.filter((entry) => !directory || entry.startsWith(`${directory.replace(/\/$/u, '')}/`))) {
      console.log(`${path}\t${fullWordPath(path, abbreviations)}`);
    }
    return 0;
  }
  const path = join(ROOT, RATCHET_FILE);
  const previous = existsSync(path) ? readFileSync(path, 'utf8') : '';
  const ceilings = readRatchet(previous);
  let failed = false;
  for (const [entry, { key, count }] of measure) {
    console.log(`${entry}  ${key} ${count}`);
    const ceiling = ceilings.get(entry);
    if (ceiling === undefined) {
      if (!argv.includes('--write')) {
        console.error(`::error file=${RATCHET_FILE}::${entry} has no ${key} ceiling; run node scripts/measure-abbreviations.mjs --write`);
        failed = true;
      }
    } else if (count > ceiling) {
      const listing = entry === FILE_NAMES ? FILE_NAMES : `bindings ${entry.slice('bindings '.length)}`;
      console.error(
        `::error file=${RATCHET_FILE}::${entry} ${key} rose from ${ceiling} to ${count}: ` +
          'spell new names in full English words (data/meta/notation-rules.lino lists the abbreviations and their full words; ' +
          `list them with --list ${listing}).`,
      );
      failed = true;
    } else if (count < ceiling && !argv.includes('--write')) {
      console.log(`${entry} ${key} fell from ${ceiling} to ${count}; lower the ceiling with --write.`);
    }
  }
  if (argv.includes('--write') && !failed) {
    const lowered = new Map(
      [...measure].map(([entry, { key, count }]) => [entry, { key, count: Math.min(count, ceilings.get(entry) ?? count) }]),
    );
    writeFileSync(path, renderRatchet(previous, lowered));
    console.log(`wrote ${RATCHET_FILE}`);
  }
  return failed ? 1 : 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
