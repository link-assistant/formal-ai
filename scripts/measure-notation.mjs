#!/usr/bin/env node
// Measure the links notation we own (PR #1188, R1188-U6 and R1188-U7) and hold
// it to data/meta/notation-ratchet.lino, whose ceilings only fall.
//
// The scopes, the excluded paths and the seeded abbreviation list live in
// data/meta/notation-rules.lino. Per directory the measure reports:
//
//   files               tracked `.lino` files
//   characters          their size in characters
//   underscore-names    distinct names spelled with `_` (the style prefers `-`)
//   dash-names          distinct names spelled with `-`
//   abbreviated-names   distinct names holding a word of the abbreviation list
//   duplicated-blocks   distinct child blocks of at least
//                       `duplicate-block-minimum-lines` lines that occur more
//                       than once, counted at their outermost occurrence
//   repeated-fields     distinct leaf lines that more than half of at least
//                       four sibling records of one kind repeat verbatim, such
//                       as `manual "not yet confirmed"` on every requirement:
//                       a default that can be stated once on the parent
//
// A name is an unquoted identifier token (scripts/lib/links-notation-names.mjs).
// Names are counted once each, so a new record that reuses existing keys does
// not move a count, while a new `_` name does. The ceilings are kept per scope
// (data/seed, data/meta), so a renamed subdirectory does not break the ratchet.
// Characters are recorded for the concise-form passes and are not a ceiling:
// new seed data rightly adds characters.
//
// Usage:
//   node scripts/measure-notation.mjs [--check]   measure and check the ratchet
//   node scripts/measure-notation.mjs --write     also lower the ceilings to the measure
//   node scripts/measure-notation.mjs --list <underscore|abbreviated|duplicated|repeated> [directory]

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { RULES_FILE, childBlockOf, namesOfLine, parseTree, readRules, wordsOf } from './lib/links-notation-names.mjs';

export const RATCHET_FILE = 'data/meta/notation-ratchet.lino';
const ROOT = join(dirname(fileURLToPath(import.meta.url)), '..');
const CEILINGS = ['underscore-names', 'abbreviated-names', 'duplicated-blocks', 'repeated-fields'];

/**
 * The tracked `.lino` files in the rule scopes, without the excluded paths
 * and without the rules file itself (it lists the abbreviations).
 * @param {ReturnType<typeof readRules>} rules
 * @returns {Array<string>}
 */
export function filesInScope(rules, root = ROOT) {
  const listed = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', ...rules.scopes], {
    cwd: root,
    encoding: 'utf8',
    maxBuffer: 1 << 28,
  });
  const excluded = rules.excluded.map((entry) => entry.path.replace(/\/$/u, ''));
  return listed
    .split('\n')
    .filter((path) => path.endsWith('.lino') && path !== RULES_FILE && path !== RATCHET_FILE && existsSync(join(root, path)))
    .filter((path) => !excluded.some((prefix) => path === prefix || path.startsWith(`${prefix}/`)))
    .sort();
}

/**
 * The scope a path belongs to: the longest rule scope that holds it.
 * @param {Array<string>} scopes
 * @param {string} path
 * @returns {string}
 */
export function scopeOf(scopes, path) {
  return scopes.filter((scope) => path.startsWith(`${scope}/`)).sort((a, b) => b.length - a.length)[0] ?? dirname(path);
}

/**
 * The names and blocks of one document.
 * @param {string} text
 * @param {Map<string, string>} abbreviations
 * @returns {{characters: number, names: Set<string>, abbreviated: Set<string>, tree: Array<object>}}
 */
export function measureDocument(text, abbreviations) {
  const names = new Set();
  const abbreviated = new Set();
  for (const line of text.split('\n')) {
    for (const { name } of namesOfLine(line)) {
      names.add(name);
      if (wordsOf(name).some((word) => abbreviations.has(word.toLowerCase()))) {
        abbreviated.add(name);
      }
    }
  }
  return { characters: text.length, names, abbreviated, tree: parseTree(text) };
}

/**
 * The duplicated child blocks of a set of trees: blocks of at least
 * `minimumLines` lines that occur more than once, counted at the outermost
 * occurrence (a duplicate inside a duplicate is not counted again).
 * @param {Array<Array<object>>} trees
 * @param {number} minimumLines
 * @returns {Map<string, {lines: number, occurrences: number}>}
 */
export function duplicatedBlocks(trees, minimumLines) {
  const counts = new Map();
  const blockOf = new Map();
  const collect = (nodes) => {
    for (const node of nodes) {
      if (node.children.length > 0) {
        const block = childBlockOf(node);
        blockOf.set(node, block);
        if (block.lines >= minimumLines) {
          counts.set(block.text, (counts.get(block.text) ?? 0) + 1);
        }
        collect(node.children);
      }
    }
  };
  trees.forEach(collect);
  const duplicated = new Map();
  const outermost = (nodes) => {
    for (const node of nodes) {
      const block = blockOf.get(node);
      if (block && block.lines >= minimumLines && counts.get(block.text) > 1) {
        const entry = duplicated.get(block.text) ?? { lines: block.lines, occurrences: 0 };
        entry.occurrences += 1;
        duplicated.set(block.text, entry);
        continue;
      }
      outermost(node.children);
    }
  };
  trees.forEach(outermost);
  for (const [text, entry] of duplicated) {
    if (entry.occurrences < 2) {
      duplicated.delete(text);
    }
  }
  return duplicated;
}

/**
 * The repeated fields of a set of trees: a leaf line (a line without
 * children) that more than half of the sibling records of one kind repeat
 * verbatim, where a parent holds at least `minimumRecords` records of that
 * kind. Such a line is shared structure that can be stated once, as a default
 * on the parent. Keyed by `<record kind> <line>`, with the occurrences.
 * @param {Array<Array<object>>} trees
 * @param {number} minimumRecords
 * @returns {Map<string, number>}
 */
export function repeatedFields(trees, minimumRecords = 4) {
  const repeated = new Map();
  const visit = (nodes) => {
    const kinds = new Map();
    for (const node of nodes) {
      const kind = node.line.split(/\s+/u)[0];
      if (node.children.length > 0) {
        kinds.set(kind, [...(kinds.get(kind) ?? []), node]);
      }
      visit(node.children);
    }
    for (const [kind, records] of kinds) {
      if (records.length < minimumRecords) {
        continue;
      }
      const counts = new Map();
      for (const record of records) {
        const leaves = new Set(record.children.filter((child) => child.children.length === 0).map((child) => child.line));
        leaves.forEach((line) => counts.set(line, (counts.get(line) ?? 0) + 1));
      }
      for (const [line, count] of counts) {
        if (count * 2 > records.length && /\s/u.test(line)) {
          const key = `${kind} ${line}`;
          repeated.set(key, (repeated.get(key) ?? 0) + count);
        }
      }
    }
  };
  trees.forEach(visit);
  return repeated;
}

/**
 * Measure every directory and every scope.
 * @returns {{directories: Map<string, object>, scopes: Map<string, object>}}
 */
export function measureNotation(root = ROOT) {
  const rules = readRules(readFileSync(join(root, RULES_FILE), 'utf8'));
  const directories = new Map();
  const scopes = new Map();
  const empty = () => ({ files: 0, characters: 0, names: new Set(), abbreviated: new Set(), trees: [] });
  for (const path of filesInScope(rules, root)) {
    const document = measureDocument(readFileSync(join(root, path), 'utf8'), rules.abbreviations);
    for (const [map, key] of [
      [directories, dirname(path)],
      [scopes, scopeOf(rules.scopes, path)],
    ]) {
      const entry = map.get(key) ?? empty();
      entry.files += 1;
      entry.characters += document.characters;
      document.names.forEach((name) => entry.names.add(name));
      document.abbreviated.forEach((name) => entry.abbreviated.add(name));
      entry.trees.push(document.tree);
      map.set(key, entry);
    }
  }
  const summarize = (entry) => {
    const names = [...entry.names];
    const duplicated = duplicatedBlocks(entry.trees, rules.minimumBlockLines);
    const repeated = repeatedFields(entry.trees);
    return {
      files: entry.files,
      characters: entry.characters,
      'underscore-names': names.filter((name) => name.includes('_')).length,
      'dash-names': names.filter((name) => name.includes('-')).length,
      'abbreviated-names': entry.abbreviated.size,
      'duplicated-blocks': duplicated.size,
      'repeated-fields': repeated.size,
      underscoreList: names.filter((name) => name.includes('_')).sort(),
      abbreviatedList: [...entry.abbreviated].sort(),
      repeatedList: [...repeated].sort((a, b) => b[1] - a[1]),
      duplicatedList: [...duplicated].sort((a, b) => b[1].lines * b[1].occurrences - a[1].lines * a[1].occurrences),
    };
  };
  const summarized = (map) => new Map([...map].sort(([a], [b]) => a.localeCompare(b)).map(([key, entry]) => [key, summarize(entry)]));
  return { directories: summarized(directories), scopes: summarized(scopes) };
}

/**
 * The ceilings of the ratchet file, per scope.
 * @param {string} text
 * @returns {Map<string, Record<string, number>>}
 */
export function readRatchet(text) {
  const ceilings = new Map();
  const top = parseTree(text).find((node) => node.line === 'notation-ratchet');
  for (const scope of top?.children ?? []) {
    const [key, path] = scope.line.split(/\s+/u);
    if (key !== 'scope') {
      continue;
    }
    const values = {};
    for (const child of scope.children) {
      const [name, value] = child.line.split(/\s+/u);
      values[name] = Number(value);
    }
    ceilings.set(path, values);
  }
  return ceilings;
}

/**
 * The ratchet file text for `scopes`, keeping the header comment of `previous`.
 * @param {string} previous
 * @param {Map<string, object>} scopes
 * @returns {string}
 */
export function renderRatchet(previous, scopes) {
  const header = previous
    .split('\n')
    .filter((line, index, lines) => lines.slice(0, index + 1).every((earlier) => earlier.startsWith('#')))
    .join('\n');
  const lines = header ? [header] : [];
  lines.push('notation-ratchet');
  for (const [scope, measure] of scopes) {
    lines.push(`  scope ${scope}`);
    for (const key of CEILINGS) {
      lines.push(`    ${key} ${measure[key]}`);
    }
    lines.push(`    characters ${measure.characters}`);
  }
  return `${lines.join('\n')}\n`;
}

function report(measure) {
  const columns = ['files', 'characters', 'underscore-names', 'dash-names', 'abbreviated-names', 'duplicated-blocks', 'repeated-fields'];
  const rows = [['directory', ...columns]];
  for (const [directory, values] of measure.directories) {
    rows.push([directory, ...columns.map((column) => String(values[column]))]);
  }
  for (const [scope, values] of measure.scopes) {
    rows.push([`${scope} (scope)`, ...columns.map((column) => String(values[column]))]);
  }
  const widths = rows[0].map((_, column) => Math.max(...rows.map((row) => row[column].length)));
  return rows.map((row) => row.map((cell, column) => (column === 0 ? cell.padEnd(widths[column]) : cell.padStart(widths[column]))).join('  '));
}

function listing(measure, kind, directory) {
  const source = directory ? measure.directories.get(directory) ?? measure.scopes.get(directory) : null;
  const entries = source ? [source] : [...measure.scopes.values()];
  const lines = [];
  for (const entry of entries) {
    if (kind === 'underscore') {
      lines.push(...entry.underscoreList);
    } else if (kind === 'abbreviated') {
      lines.push(...entry.abbreviatedList);
    } else if (kind === 'repeated') {
      lines.push(...entry.repeatedList.map(([field, count]) => `${count} x ${field}`));
    } else {
      for (const [text, { lines: size, occurrences }] of entry.duplicatedList) {
        lines.push(`${occurrences} x ${size} lines:`, ...text.split('\n').map((line) => `    ${line}`));
      }
    }
  }
  return lines;
}

function main(argv) {
  const measure = measureNotation();
  const listAt = argv.indexOf('--list');
  if (listAt !== -1) {
    console.log(listing(measure, argv[listAt + 1], argv[listAt + 2]).join('\n'));
    return 0;
  }
  console.log(report(measure).join('\n'));
  const path = join(ROOT, RATCHET_FILE);
  let previous = '';
  try {
    previous = readFileSync(path, 'utf8');
  } catch {
    previous = '';
  }
  const ceilings = readRatchet(previous);
  let failed = false;
  for (const [scope, values] of measure.scopes) {
    const ceiling = ceilings.get(scope);
    for (const key of CEILINGS) {
      if (ceiling === undefined || ceiling[key] === undefined) {
        if (!argv.includes('--write')) {
          console.error(`::error file=${RATCHET_FILE}::scope ${scope} has no ${key} ceiling; run node scripts/measure-notation.mjs --write`);
          failed = true;
        }
      } else if (values[key] > ceiling[key]) {
        console.error(
          `::error file=${RATCHET_FILE}::${scope} ${key} rose from ${ceiling[key]} to ${values[key]}: ` +
            'write new names with `-` and full English words, and state shared structure once ' +
            '(docs/links-notation-style.md; list them with --list).',
        );
        failed = true;
      } else if (values[key] < ceiling[key] && !argv.includes('--write')) {
        console.log(`${scope} ${key} fell from ${ceiling[key]} to ${values[key]}; lower the ceiling with --write.`);
      }
    }
  }
  if (argv.includes('--write') && !failed) {
    const lowered = new Map(
      [...measure.scopes].map(([scope, values]) => {
        const ceiling = ceilings.get(scope) ?? {};
        const kept = { characters: values.characters };
        for (const key of CEILINGS) {
          kept[key] = Math.min(values[key], ceiling[key] ?? values[key]);
        }
        return [scope, kept];
      }),
    );
    writeFileSync(path, renderRatchet(previous, lowered));
    console.log(`wrote ${RATCHET_FILE}`);
  }
  return failed ? 1 : 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
