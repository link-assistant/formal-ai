#!/usr/bin/env node
// JavaScript twin of `scripts/generate-requirement-status.rs` (PR #1188, SCRIPTS-A).
//
// Same flags, exit codes, stdout/stderr lines and written bytes as the Rust
// original, so the requirement pipeline runs without compiling Rust locally. CI
// runs both and fails when they disagree
// (`data/meta/ci-gates/check-generate-requirement-status-js-twin.lino`). Read
// the Rust source for the reasoning behind each rule.
//
// Usage:
//   node scripts/generate-requirement-status.mjs           # check the ledger is current
//   node scripts/generate-requirement-status.mjs --write   # regenerate it
import { mkdirSync, rmSync, writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

import {
  asciiLowercase, comparePaths, debugList, extension, fileName, ioErrorDisplay, isAlphanumeric, isFile,
  isWhitespace, joinPath, leadingDigits, lines, readDirPaths, readOrEmpty, readOrNull, readToString,
  sortedStrings, stripRoot, trim, trimMatches, trimStart,
} from './lib/requirements-rust-compat.mjs';
import { readRegister, requirementIds } from './lib/requirements-register.mjs';

const REQUIREMENT_SHARDS = 'docs/requirements';
const TRACEABILITY = 'docs/requirements-traceability.md';
const MANIFEST = 'data/meta/requirement-status-ledger.lino';
const LEDGER_DIRECTORY = 'data/meta/requirement-status-ledger';

/** Mirrors `fn is_definition_line`. */
export function isDefinitionLine(line) {
  const trimmed = trimStart(line);
  return trimmed.startsWith('| R') || trimmed.startsWith('### R') || trimmed.startsWith('- R');
}

/** Mirrors `fn defined_requirement_ids`, sorted as the `BTreeSet` iterates. */
export function definedRequirementIds(text) {
  const ids = new Set();
  for (const line of lines(text)) {
    if (!isDefinitionLine(line)) continue;
    const id = requirementIds(line)[0];
    if (id !== undefined) ids.add(id);
  }
  return sortedStrings(ids);
}

/** Mirrors `fn quoted`. */
export function quoted(value) {
  if (value.includes('"')) return `'${value.replaceAll('\\', '\\\\').replaceAll("'", '\\x27')}'`;
  return `"${value.replaceAll('\\', '\\\\')}"`;
}

const SEPARATORS = '`|,;()[]';

/** Mirrors `fn test_path`. */
export function testPath(text, root) {
  let rustTest = '';
  const tokens = [''];
  for (const character of text) {
    if (isWhitespace(character) || SEPARATORS.includes(character)) tokens.push('');
    else tokens[tokens.length - 1] += character;
  }
  for (const token of tokens) {
    const candidate = trimMatches(token, '.:"');
    let found = null;
    for (const suffix of ['.test.mjs', '.spec.js', '.rs']) {
      const end = candidate.indexOf(suffix);
      if (end >= 0) {
        found = candidate.slice(0, end + suffix.length);
        break;
      }
    }
    if (found === null) continue;
    if ((found.startsWith('tests/') || found.startsWith('rust/tests/') || found.startsWith('desktop/scripts/'))
      && isFile(joinPath(root, found))) {
      // JavaScript first (R1188-U29): a row that names tests in both roots
      // cites the JavaScript one.
      if (!found.endsWith('.rs')) return found;
      if (rustTest === '') rustTest = found;
    }
  }
  return rustTest;
}

/** Mirrors `fn trace_rows`. */
export function traceRows(text, root) {
  const rows = new Map();
  for (const line of lines(text)) {
    if (!line.startsWith('| R')) continue;
    const cells = trimMatches(line, '|').split('|').map(trim);
    if (cells.length < 5) continue;
    const id = requirementIds(cells[0])[0];
    if (id === undefined) continue;
    rows.set(id, { delivered: cells[2], automatedTest: testPath(cells[3], root), manual: cells[4] });
  }
  return rows;
}

/** Mirrors `fn issue_from_shard`. */
export function issueFromShard(path) {
  const name = fileName(path);
  return name.startsWith('issue-') ? leadingDigits(name.slice('issue-'.length)) : '';
}

const isWord = (character) => character !== undefined && (isAlphanumeric(character) || character === '_' || character === '-');

/** Mirrors `fn contains_word`. */
export function containsWord(text, needle) {
  for (let start = text.indexOf(needle); start >= 0; start = text.indexOf(needle, start + needle.length)) {
    const before = [...text.slice(Math.max(0, start - 2), start)].at(-1);
    const after = [...text.slice(start + needle.length, start + needle.length + 2)][0];
    if (!isWord(before) && !isWord(after)) return true;
  }
  return false;
}

/** Mirrors `fn verdict`. */
export function verdict(line, automatedTest) {
  let status = line;
  if (trimStart(line).startsWith('|')) {
    const cells = trimMatches(trim(line), '|').split('|');
    if (cells.length > 2) status = cells.slice(2).join('|');
  }
  const lower = asciiLowercase(status);
  // A status that opens with its verdict states it (R1188-U27): "Partial: …
  // delivered …" is partial and "Implemented: … planned …" implemented,
  // whatever words follow.
  const opening = trimStart(lower).replace(/^\*+/u, '');
  if (opening.startsWith('partial')) return 'partial';
  if (opening.startsWith('not delivered')) return 'not-delivered';
  if (/^(?:implemented|delivered)\s*[:.(]/u.test(opening) && automatedTest !== '') return 'implemented';
  if (containsWord(lower, 'withdrawn')) return 'withdrawn';
  if (containsWord(lower, 'superseded')) return 'superseded';
  if (['not delivered', 'not implemented', 'pending', 'planned'].some((needle) => containsWord(lower, needle))) {
    return 'not-delivered';
  }
  if (automatedTest !== '' && ['implemented', 'delivered', 'complete', 'covered by'].some((needle) => lower.includes(needle))) {
    return 'implemented';
  }
  return 'partial';
}

/** Mirrors `fn requirement_rows`; throws the Rust error text. */
export function requirementRows(root) {
  const expected = definedRequirementIds(readRegister(root));
  const expectedSet = new Set(expected);
  const trace = traceRows(readOrEmpty(joinPath(root, TRACEABILITY)), root);
  const ownership = new Map();
  let paths;
  try {
    paths = readDirPaths(joinPath(root, REQUIREMENT_SHARDS));
  } catch (error) {
    throw new Error(`cannot read ${REQUIREMENT_SHARDS}: ${ioErrorDisplay(error)}`);
  }
  paths = paths.filter((path) => extension(path) === 'md').sort(comparePaths);
  for (const path of paths) {
    const relative = stripRoot(path, root);
    let source;
    try {
      source = readToString(path);
    } catch (error) {
      throw new Error(`cannot read ${path}: ${ioErrorDisplay(error)}`);
    }
    for (const line of lines(source)) {
      const ids = requirementIds(line);
      const defined = ids[0];
      for (const id of ids) {
        if (!expectedSet.has(id)) continue;
        if (!ownership.has(id) || (isDefinitionLine(line) && defined === id)) ownership.set(id, [relative, line]);
      }
    }
  }
  const missing = expected.filter((id) => !ownership.has(id));
  if (missing.length) throw new Error(`${missing.length} assembled requirement ids have no owning shard: ${debugList(missing)}`);
  return expected.map((id) => {
    const [shard, line] = ownership.get(id);
    const traced = trace.get(id) ?? { delivered: '', automatedTest: '', manual: '' };
    const sourceTest = testPath(line, root);
    const automatedTest = sourceTest === '' ? traced.automatedTest : sourceTest;
    return {
      id,
      shard,
      verdict: verdict(line, automatedTest),
      delivered: traced.delivered,
      issue: issueFromShard(shard),
      automatedTest,
      manual: traced.manual === '' ? 'not yet confirmed' : traced.manual,
    };
  });
}

/** Mirrors `fn render_manifest`. */
export function renderManifest(rows, ledgerFiles) {
  const implemented = rows.filter((row) => row.verdict === 'implemented').length;
  let output = '# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n'
    + 'requirement_status_ledger\n'
    + '  source "REQUIREMENTS.md and docs/requirements/*.md"\n'
    + '  verdict_vocabulary "implemented | partial | not-delivered | superseded | withdrawn"\n'
    + '  manual_column "aspirational since 2026-09-30 (issue #1090): not yet confirmed rows carry no debt"\n'
    + `  requirement_count ${rows.length}\n`
    + `  implemented_count ${implemented}\n`;
  for (const name of ledgerFiles) output += `  shard "${LEDGER_DIRECTORY}/${name}.lino"\n`;
  return output;
}

/** Mirrors `fn shared_value`: the value more than half of `values` share, or ''. */
export function sharedValue(values) {
  const counts = new Map();
  for (const value of values) counts.set(value, (counts.get(value) ?? 0) + 1);
  for (const [value, count] of counts) if (count * 2 > values.length) return value;
  return '';
}

/** Mirrors `const SHARED_FIELDS`: `[ledger field, row property]`, in record order. */
export const SHARED_FIELDS = [['delivered', 'delivered'], ['automated_test', 'automatedTest'], ['manual', 'manual']];


/** Select a smaller minority default without changing the majority rule. */
export function sharedDefault(values, field) {
  if (!SHARED_FIELDS.some(([name]) => name === field)) throw new Error('unowned shared field');
  const majority = sharedValue(values);
  if (majority !== '') return majority;
  const counts = new Map();
  for (const value of values) counts.set(value, (counts.get(value) ?? 0) + 1);
  const emptyCount = counts.get('') ?? 0;
  let selected = '';
  let saved = 0;
  const bytes = value => Buffer.byteLength('    ' + field + ' ' + quoted(value) + '\n');
  for (const [value, count] of counts) {
    if (value === '' || count < 2 || count - 1 <= emptyCount) continue;
    const gain = (count - 1) * bytes(value) + 2 - emptyCount * bytes('');
    if (gain > saved) { selected = value; saved = gain; }
  }
  return selected;
}

/** Mirrors `fn render_shard`: shared structure is stated once on the file. */
export function renderShard(rows) {
  const shared = SHARED_FIELDS.map(([name, property]) => sharedDefault(rows.map((row) => row[property]), name));
  let output = '# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n'
    + 'requirement_status_ledger_shard\n';
  if (rows.length) output += `  shard ${quoted(rows[0].shard)}\n  issue ${quoted(rows[0].issue)}\n`;
  SHARED_FIELDS.forEach(([name], index) => {
    if (shared[index] !== '') output += `  ${name} ${quoted(shared[index])}\n`;
  });
  for (const row of rows) {
    output += '  requirement\n'
      + `    id ${quoted(row.id)}\n`
      + `    verdict ${quoted(row.verdict)}\n`;
    SHARED_FIELDS.forEach(([name, property], index) => {
      if (row[property] !== shared[index]) output += `    ${name} ${quoted(row[property])}\n`;
    });
  }
  return output;
}

/** Mirrors `fn ledger_name`: the shard's file stem. */
export function ledgerName(shard) {
  const name = fileName(shard);
  const dot = name.lastIndexOf('.');
  return dot > 0 ? name.slice(0, dot) : name;
}

/** Mirrors `fn generated`: `[path, content]` pairs in `BTreeMap<PathBuf>` order. */
export function generated(root) {
  const rows = requirementRows(root);
  const byShard = new Map();
  for (const row of rows) {
    const name = ledgerName(row.shard);
    if (!byShard.has(name)) byShard.set(name, []);
    byShard.get(name).push(row);
  }
  const names = sortedStrings(byShard.keys());
  const files = [[joinPath(root, MANIFEST), renderManifest(rows, names)]];
  for (const name of names) files.push([joinPath(root, `${LEDGER_DIRECTORY}/${name}.lino`), renderShard(byShard.get(name))]);
  return files.sort((left, right) => comparePaths(left[0], right[0]));
}

/** Mirrors `fn main`: returns the exit code, printing what Rust prints. */
export function main(argv = process.argv.slice(2), root = process.cwd(), out = console.log, err = console.error) {
  const write = argv[0] === '--write';
  let files;
  try {
    files = generated(root);
  } catch (error) {
    err(`generate-requirement-status: ${error.message}`);
    return 1;
  }
  if (write) {
    const directory = joinPath(root, LEDGER_DIRECTORY);
    mkdirSync(directory, { recursive: true });
    const expected = new Set(files.map(([path]) => path));
    for (const path of readDirPaths(directory)) {
      if (extension(path) === 'lino' && !expected.has(path)) rmSync(path);
    }
    for (const [path, content] of files) writeFileSync(path, content);
    out(`generated ${files.length} requirement-status files`);
    return 0;
  }
  const stale = files.filter(([path, content]) => readOrNull(path) !== content).map(([path]) => stripRoot(path, root));
  if (!stale.length) {
    out(`requirement-status ledger is current (${files.length} files)`);
    return 0;
  }
  err(`stale generated requirement-status files: ${debugList(stale)}`);
  err('run rust-script scripts/generate-requirement-status.rs --write');
  return 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
