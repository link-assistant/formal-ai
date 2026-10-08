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
const RECORDS_PER_SHARD = 80;

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
      return found;
    }
  }
  return '';
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

const pad2 = (number) => String(number).padStart(2, '0');

/** Mirrors `fn render_manifest`. */
export function renderManifest(rows, shardCount) {
  const implemented = rows.filter((row) => row.verdict === 'implemented').length;
  let output = '# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n'
    + 'requirement_status_ledger\n'
    + '  source "REQUIREMENTS.md and docs/requirements/*.md"\n'
    + '  verdict_vocabulary "implemented | partial | not-delivered | superseded | withdrawn"\n'
    + '  manual_column "aspirational since 2026-09-30 (issue #1090): not yet confirmed rows carry no debt"\n'
    + `  requirement_count ${rows.length}\n`
    + `  implemented_count ${implemented}\n`;
  for (let index = 0; index < shardCount; index += 1) {
    output += `  shard "${LEDGER_DIRECTORY}/requirements-${pad2(index + 1)}.lino"\n`;
  }
  return output;
}

/** Mirrors `fn render_shard`. */
export function renderShard(rows, index) {
  let output = '# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n'
    + `requirement_status_ledger_shard requirements_${pad2(index + 1)}\n`;
  for (const row of rows) {
    output += '  requirement\n'
      + `    id ${quoted(row.id)}\n`
      + `    shard ${quoted(row.shard)}\n`
      + `    verdict ${quoted(row.verdict)}\n`
      + `    delivered ${quoted(row.delivered)}\n`
      + `    issue ${quoted(row.issue)}\n`
      + `    automated_test ${quoted(row.automatedTest)}\n`
      + `    manual ${quoted(row.manual)}\n`;
  }
  return output;
}

/** Mirrors `fn generated`: `[path, content]` pairs in `BTreeMap<PathBuf>` order. */
export function generated(root) {
  const rows = requirementRows(root);
  const chunks = [];
  for (let start = 0; start < rows.length; start += RECORDS_PER_SHARD) chunks.push(rows.slice(start, start + RECORDS_PER_SHARD));
  const files = [[joinPath(root, MANIFEST), renderManifest(rows, chunks.length)]];
  chunks.forEach((chunk, index) => {
    files.push([joinPath(root, `${LEDGER_DIRECTORY}/requirements-${pad2(index + 1)}.lino`), renderShard(chunk, index)]);
  });
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
