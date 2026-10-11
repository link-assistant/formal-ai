#!/usr/bin/env node
// JavaScript twin of `scripts/render-status.rs` (PR #1188, SCRIPTS-A).
//
// Same flags, exit codes, stdout/stderr lines and written bytes as the Rust
// original, so the requirement pipeline runs without compiling Rust locally. CI
// runs both and fails when they disagree
// (`data/meta/ci-gates/check-render-status-js-twin.lino`).
//
// Usage:
//   node scripts/render-status.mjs --write
//   node scripts/render-status.mjs --check
import { writeFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

import {
  comparePaths, compareStrings, debugList, extension, ioErrorDisplay, isWhitespace, joinPath, lines,
  parseUnsigned, readDirPaths, readOrNull, readToString, stripRoot, trim, trimEnd, trimMatches,
} from './lib/requirements-rust-compat.mjs';

import {parseRequirementLedger} from './lib/requirement-ledger.mjs';

const STATUS_DOCUMENT = 'docs/status.md';
const BENCHMARKS_DOCUMENT = 'docs/benchmarks.md';
const README_DOCUMENT = 'README.md';
const LANGUAGE_REGISTRY = 'data/seed/languages.lino';
const LEDGERS = [
  'data/benchmarks/external-results.lino',
  'data/meta/self-hosting-ledger.lino',
  'data/meta/debt-ratchet.lino',
  'data/meta/core-boundary-ledger.lino',
  'data/meta/handler-migration-ledger.lino',
  'data/meta/ladder-ratchet.lino',
  'data/meta/requirement-status-ledger.lino',
];

/** Mirrors `fn unquote`. */
const unquote = (value) => trimMatches(trim(value), '"');

/** `str::split_once(' ')`. */
function splitOnce(text) {
  const at = text.indexOf(' ');
  return at < 0 ? null : [text.slice(0, at), text.slice(at + 1)];
}

/** `fs::read_to_string`, with the error text `describe` builds. */
function read(path, describe) {
  try {
    return readToString(path);
  } catch (error) {
    throw new Error(describe(ioErrorDisplay(error)));
  }
}

/** Mirrors `fn root_records`. */
export function rootRecords(source) {
  const records = [];
  let current = new Map();
  for (const line of lines(source)) {
    const trimmed = trim(line);
    const first = [...line.slice(0, 2)][0];
    if (first !== undefined && !isWhitespace(first) && trimmed !== '' && !trimmed.startsWith('#')) {
      if (current.size) records.push(current);
      current = new Map([['record', trimmed]]);
    } else if (line.startsWith('  ') && !line.startsWith('    ')) {
      const pair = splitOnce(trimmed);
      if (pair) current.set(pair[0], unquote(pair[1]));
    }
  }
  if (current.size) records.push(current);
  return records;
}

const sliceOf = (row) => parseUnsigned(row.get('slice') ?? '', 64) ?? 0n;

/** Mirrors `fn latest_benchmarks`. */
export function latestBenchmarks(root) {
  const path = joinPath(root, LEDGERS[0]);
  const source = read(path, (error) => `${path}: ${error}`);
  const latest = new Map();
  for (const row of rootRecords(source)) {
    if (row.get('record_type') !== 'external_benchmark_result') continue;
    const suite = row.get('suite');
    if (suite === undefined) continue;
    const previous = latest.get(suite);
    let replace = previous === undefined;
    if (!replace) {
      const order = compareStrings(row.get('date') ?? '', previous.get('date') ?? '');
      replace = order > 0 || (order === 0 && sliceOf(row) > sliceOf(previous));
    }
    if (replace) latest.set(suite, row);
  }
  return [...latest.keys()].sort(compareStrings).map((suite) => latest.get(suite));
}

/** Mirrors `fn benchmark_table`. */
export function benchmarkTable(rows) {
  let output = '| Suite | Date | Slice | Passed | Total | Solver |\n| --- | --- | ---: | ---: | ---: | --- |\n';
  for (const row of rows) {
    const value = (key) => row.get(key) ?? '';
    output += `| \`${value('suite')}\` | ${value('date')} | ${value('slice')} | ${value('passed')} | ${value('total')} | ${value('solver_version')} |\n`;
  }
  return output;
}

/** Mirrors `fn requirement_counts`: `[verdict, count]` in verdict order. */
export function requirementCounts(root) {
  const directory = joinPath(root, 'data/meta/requirement-status-ledger');
  let paths;
  try {
    paths = readDirPaths(directory);
  } catch (error) {
    throw new Error(`${directory}: ${ioErrorDisplay(error)}`);
  }
  const counts = new Map();
  for (const path of paths) {
    if (extension(path) !== 'lino') continue;
    const parsed = parseRequirementLedger(read(path, (error) => error), {unknownFields: 'preserve'});
    for (const {verdict} of parsed.records) counts.set(verdict, (counts.get(verdict) ?? 0) + 1);
  }
  return [...counts.keys()].sort(compareStrings).map((verdict) => [verdict, counts.get(verdict)]);
}

/** Mirrors `fn language_rows`. */
export function languageRows(root) {
  const source = read(joinPath(root, LANGUAGE_REGISTRY), (error) => `${LANGUAGE_REGISTRY}: ${error}`);
  const rows = [];
  let current = new Map();
  for (const line of lines(source)) {
    const trimmed = trim(line);
    if (line.startsWith('  language ')) {
      if (current.size) rows.push(current);
      current = new Map([['code', unquote(line.slice('  language '.length))]]);
    } else if (current.size && line.startsWith('    ') && !line.startsWith('      ')) {
      const pair = splitOnce(trimmed);
      if (pair) current.set(pair[0], unquote(pair[1]));
    }
  }
  if (current.size) rows.push(current);
  if (!rows.length) throw new Error(`${LANGUAGE_REGISTRY}: no language rows`);
  return rows;
}

/** Mirrors `fn language_table`. */
export function languageTable(rows) {
  let output = '| Code | Language | Status | Uncovered behavior |\n| --- | --- | --- | --- |\n';
  for (const row of rows) {
    const value = (key) => row.get(key) ?? '';
    output += `| \`${value('code')}\` | ${value('name')} | \`${value('status')}\` | \`${value('uncovered_behavior')}\` |\n`;
  }
  return output;
}

/** Mirrors `fn latest_release`. */
export function latestRelease(root) {
  const source = read(joinPath(root, LEDGERS[1]), (error) => error);
  let latest = new Map();
  let inRelease = false;
  for (const line of lines(source)) {
    const trimmed = trim(line);
    if (line.startsWith('  release')) {
      latest = new Map();
      inRelease = true;
    } else if (inRelease && line.startsWith('    ')) {
      const pair = splitOnce(trimmed);
      if (pair) latest.set(pair[0], unquote(pair[1]));
    }
  }
  if (!latest.size) throw new Error('self-hosting ledger has no release row');
  return latest;
}

/** Mirrors `fn status_document`. */
export function statusDocument(root) {
  let output = '<!-- Generated by `rust-script scripts/render-status.rs --write`. -->\n\n'
    + '# Repository Status\n\n'
    + 'This document is a deterministic projection of committed ledgers.\n\n'
    + '## Language coverage\n\n';
  output += languageTable(languageRows(root));
  output += '\n## Requirement verdicts\n\n| Verdict | Count |\n| --- | ---: |\n';
  for (const [verdict, count] of requirementCounts(root)) output += `| \`${verdict}\` | ${count} |\n`;
  output += '\n## Latest external benchmark rows\n\n';
  output += benchmarkTable(latestBenchmarks(root));
  output += '\n## Latest self-hosting release\n\n';
  const release = latestRelease(root);
  for (const key of ['tag', 'percentage_basis_points', 'trailing_percentage_basis_points', 'target_percentage_basis_points']) {
    output += `- \`${key}\`: \`${release.get(key) ?? ''}\`\n`;
  }
  output += '\n## Ledger inventory\n\n| Input | Lines |\n| --- | ---: |\n';
  for (const relative of LEDGERS) {
    const source = read(joinPath(root, relative), (error) => `${relative}: ${error}`);
    output += `| \`${relative}\` | ${lines(source).length} |\n`;
  }
  const languageSource = read(joinPath(root, LANGUAGE_REGISTRY), (error) => `${LANGUAGE_REGISTRY}: ${error}`);
  output += `| \`${LANGUAGE_REGISTRY}\` | ${lines(languageSource).length} |\n`;
  const workerDirectory = joinPath(root, 'data/meta/worker-line-budget');
  let workerFiles;
  try {
    workerFiles = readDirPaths(workerDirectory).filter((path) => extension(path) === 'lino').length;
  } catch (error) {
    throw new Error(`${workerDirectory}: ${ioErrorDisplay(error)}`);
  }
  output += `| \`data/meta/worker-line-budget/*.lino\` | ${workerFiles} files |\n`;
  return output;
}

/** Mirrors `fn replace_region`. */
export function replaceRegion(source, name, body) {
  const begin = `<!-- status:begin ${name} -->`;
  const end = `<!-- status:end ${name} -->`;
  const start = source.indexOf(begin);
  const finish = source.indexOf(end);
  if (start >= 0 && finish >= 0 && finish >= start) {
    return `${source.slice(0, start)}${begin}\n${body}\n${end}${source.slice(finish + end.length)}`;
  }
  if (start < 0 && finish < 0) return `${trimEnd(source)}\n\n${begin}\n${body}\n${end}\n`;
  throw new Error(`${name}: unmatched status region marker`);
}

/** Mirrors `fn generated_files`: `[path, content]` in `BTreeMap<PathBuf>` order. */
export function generatedFiles(root) {
  const benchmarks = latestBenchmarks(root);
  const release = latestRelease(root);
  const benchmarkBody = `Generated from \`data/benchmarks/external-results.lino\`.\n\n${trimEnd(benchmarkTable(benchmarks))}`;
  const field = (key) => release.get(key) ?? '';
  const selfHostingBody = `Latest ledger row: \`${field('tag')}\`; release share \`${field('percentage_basis_points')}\` basis points, trailing share \`${field('trailing_percentage_basis_points')}\` basis points, target \`${field('target_percentage_basis_points')}\` basis points.`;
  const benchmarkSource = read(joinPath(root, BENCHMARKS_DOCUMENT), (error) => `${BENCHMARKS_DOCUMENT}: ${error}`);
  const readmeSource = read(joinPath(root, README_DOCUMENT), (error) => `${README_DOCUMENT}: ${error}`);
  const files = [
    [joinPath(root, STATUS_DOCUMENT), statusDocument(root)],
    [joinPath(root, BENCHMARKS_DOCUMENT), replaceRegion(benchmarkSource, 'benchmarks', benchmarkBody)],
    [joinPath(root, README_DOCUMENT), replaceRegion(readmeSource, 'self-hosting', selfHostingBody)],
  ];
  return files.sort((left, right) => comparePaths(left[0], right[0]));
}

/** Mirrors `fn main`: returns the exit code, printing what Rust prints. */
export function main(argv = process.argv.slice(2), root = process.cwd(), out = console.log, err = console.error) {
  const mode = argv[0] ?? '--check';
  if (mode !== '--write' && mode !== '--check') {
    err(`render-status: unknown mode ${mode}; expected --write or --check`);
    return 2;
  }
  let files;
  try {
    files = generatedFiles(root);
  } catch (error) {
    err(`render-status: ${error.message}`);
    return 1;
  }
  if (mode === '--write') {
    for (const [path, content] of files) writeFileSync(path, content);
    out(`rendered ${files.length} status surfaces`);
    return 0;
  }
  const stale = files.filter(([path, content]) => readOrNull(path) !== content).map(([path]) => stripRoot(path, root));
  if (!stale.length) {
    out('status surfaces are current');
    return 0;
  }
  err(`stale status surfaces: ${debugList(stale)}`);
  err('run rust-script scripts/render-status.rs --write');
  return 1;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
