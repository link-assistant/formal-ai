#!/usr/bin/env node
// JavaScript twin of `scripts/check-debt-ratchet.rs` (PR #1188, SCRIPTS-B).
//
// Named debt only shrinks. `data/meta/debt-ratchet.lino` names measured
// ceilings; every measured value must equal its ceiling (an `upward` measure
// has a floor instead), and with `--base <rev>` no ceiling may be higher than
// at `<rev>`. `GITHUB_BASE_REF` is honoured when set. The flag is required.
// Same output, exit codes and `::error` annotations as the Rust original; CI
// runs both and diffs them (`data/meta/ci-gates/check-debt-ratchet-js-twin.lino`).
// Read the Rust script's doc comments for why each rule and measure exists.
//
// Usage:
//   node scripts/check-debt-ratchet.mjs --base <rev> [--repo <path>]

import { spawnSync } from 'node:child_process';
import { join } from 'node:path';
import { pathToFileURL } from 'node:url';

import { currentGapCount } from './lib/checks-language-parity.mjs';
import {
  RustError, compareStrings, countMatches, debugStringList, extension, ioErrorMessage,
  isDir, lines, parseUnsigned, readDirEntries, readToString, sortedKeys, splitOnce, trim, trimStart,
} from './lib/checks-rust-compat.mjs';

const LEDGER = 'data/meta/debt-ratchet.lino';
const HANDLER_LEDGER = 'data/meta/handler-migration-ledger.lino';
const ALLOWLIST = 'scripts/hardcoded-language-allowlist.txt';
const AUTHORED_LADDER_RULES = 'experiments/issue_1028_agent_cli_ladder/rules';
const BOUNDARY_LEDGER = 'data/meta/core-boundary-ledger.lino';
const BOOKKEEPING = ['mod.rs', 'modules.rs'];
const UPWARD_MARKER = 'direction upward';
const CORRECTION_MARKER = 'corrected undercount';

/** `unquote`: trim, then strip every leading and trailing `"`. */
function unquote(value) {
  return trim(value).replace(/^"+|"+$/g, '');
}

/** Parse the ledger's `ceiling` blocks; throws `RustError` like `parse_ratchet`. */
export function parseRatchet(text) {
  const ceilings = new Map();
  const upward = new Set();
  const notes = new Map();
  let measure = null;
  let named = null;
  for (const line of lines(text)) {
    const trimmed = trim(line);
    if (trimmed === 'ceiling') {
      measure = null;
      named = null;
      continue;
    }
    if (trimmed.startsWith('measure ')) {
      // A measure name reads in its `-` spelling, so a base revision that still
      // spells it with `_` compares with this one (R1188-U6).
      measure = unquote(trimmed.slice('measure '.length)).replaceAll('_', '-');
      named = measure;
      continue;
    }
    if (trimmed.startsWith('how ') && named !== null && unquote(trimmed.slice(4)).includes(UPWARD_MARKER)) {
      upward.add(named);
    }
    if (trimmed.startsWith('note ') && named !== null) notes.set(named, unquote(trimmed.slice(5)));
    if (trimmed.startsWith('value ')) {
      if (measure === null) throw new RustError(`\`${trimmed}\` has no \`measure\` above it`);
      const name = measure;
      measure = null;
      const parsed = parseUnsigned(unquote(trimmed.slice(6)));
      if (parsed.error) throw new RustError(`${trimmed}: ${parsed.error}`);
      ceilings.set(name, parsed.value);
    }
  }
  if (ceilings.size === 0) throw new RustError('the ratchet names no ceiling');
  return { ceilings, upward, notes };
}

function rustFiles(directory, out) {
  let entries;
  try {
    entries = readDirEntries(directory);
  } catch (error) {
    throw new RustError(`${directory}: ${ioErrorMessage(error)}`);
  }
  for (const entry of entries) {
    if (isDir(entry.path)) rustFiles(entry.path, out);
    else if (extension(entry.path) === 'rs') out.push(entry.path);
  }
}

function relative(root, path) {
  return path.startsWith(`${root}/`) ? path.slice(root.length + 1) : path;
}

function readRelative(root, path) {
  try {
    return readToString(join(root, path));
  } catch (error) {
    throw new RustError(`${path}: ${ioErrorMessage(error)}`);
  }
}

function handlerSourceFiles(root) {
  const text = readRelative(root, BOUNDARY_LEDGER);
  const files = [];
  let path = null;
  for (const line of lines(text)) {
    if (line.startsWith('  source ')) {
      path = trim(line.slice('  source '.length));
      continue;
    }
    const trimmed = trim(line);
    if (trimmed.startsWith('disposition ') && path !== null) {
      const source = path;
      path = null;
      const name = source.slice(source.lastIndexOf('/') + 1);
      if (unquote(trimmed.slice('disposition '.length)) === 'migrate' && !BOOKKEEPING.includes(name)) {
        files.push(source);
      }
    }
  }
  return files.sort(compareStrings);
}

/** The slice of `text` between the first `open` and the next `close` after it. */
function between(text, open, close) {
  const tail = splitOnce(text, open)?.[1];
  if (tail === undefined) return '';
  return splitOnce(tail, close)?.[0] ?? '';
}

function tryDispatchEntries(root) {
  const dispatch = readRelative(root, 'rust/src/solver_dispatch.rs');
  return lines(between(dispatch, 'const HANDLER_FUNCTIONS', '];'))
    .filter((line) => {
      const parts = splitOnce(line, ',');
      return parts !== null && trimStart(parts[1]).startsWith('try_');
    }).length;
}

function promotionPredicates(root) {
  return countMatches(readRelative(root, 'rust/src/intent_formalization/prompt_relevants.rs'), '"handler:');
}

function dispatchNameSpecialCases(root) {
  const text = readRelative(root, 'rust/src/meta_method_dispatch.rs');
  return countMatches(text, 'name == "') + countMatches(between(text, 'match name', '_ => return None'), '" =>');
}

function workerSyncHandlerLiterals(root) {
  const workerDir = join(root, 'js/worker');
  let entries;
  try {
    entries = readDirEntries(workerDir);
  } catch (error) {
    throw new RustError(`${workerDir}: ${ioErrorMessage(error)}`);
  }
  const owners = [];
  for (const entry of entries) {
    if (extension(entry.path) !== 'js') continue;
    let text;
    try {
      text = readToString(entry.path);
    } catch (error) {
      throw new RustError(`${entry.path}: ${ioErrorMessage(error)}`);
    }
    if (text.includes('function synchronousHandlerCandidates')) owners.push(text);
  }
  if (owners.length !== 1) {
    throw new RustError(`expected exactly one synchronousHandlerCandidates owner under js/worker, found ${owners.length}`);
  }
  return countMatches(owners[0], 'name: "');
}

function readRustFile(root, file) {
  try {
    return readToString(file);
  } catch (error) {
    throw new RustError(`${relative(root, file)}: ${ioErrorMessage(error)}`);
  }
}

function storeReadShare(root) {
  const files = [];
  rustFiles(join(root, 'rust/src'), files);
  return files.reduce((total, file) => total + countMatches(readRustFile(root, file), 'from_store('), 0);
}

function countEntries(directory, label, keep) {
  let entries;
  try {
    entries = readDirEntries(directory);
  } catch (error) {
    throw new RustError(`${label}: ${ioErrorMessage(error)}`);
  }
  return entries.filter(keep).length;
}

/** Measure every value in a checkout, in the Rust original's order. */
export function measure(root) {
  const files = [];
  rustFiles(join(root, 'rust/src'), files);
  const handlerFiles = handlerSourceFiles(root).length;
  let literals = 0;
  for (const file of files) {
    const text = readRustFile(root, file);
    literals += countMatches(text, 'contains("') + countMatches(text, 'starts_with("');
  }
  const pending = countMatches(readRelative(root, HANDLER_LEDGER), 'status pending');
  const rows = lines(readRelative(root, ALLOWLIST))
    .filter((line) => trim(line) !== '' && !line.startsWith('#')).length;
  const measured = new Map();
  measured.set('handler-files', handlerFiles);
  measured.set('handler-migration-pending', pending);
  measured.set('literal-predicates', literals);
  measured.set('hardcoded-language-rows', rows);
  measured.set('try-dispatch-entries', tryDispatchEntries(root));
  measured.set('promotion-predicates', promotionPredicates(root));
  measured.set('dispatch-name-special-cases', dispatchNameSpecialCases(root));
  measured.set('worker-sync-handler-literals', workerSyncHandlerLiterals(root));
  measured.set('store-read-share', storeReadShare(root));
  measured.set('docs-requirements-suites', countEntries(join(root, 'rust/tests/unit'), 'tests/unit', (entry) => entry.name.startsWith('docs_')));
  const ladder = join(root, AUTHORED_LADDER_RULES);
  measured.set('authored-ladder-rules', countEntries(ladder, ladder, (entry) => extension(entry.path) === 'lino'));
  measured.set('language-parity-gaps', currentGapCount(root));
  return measured;
}

/** Every measured value exactly at its ceiling (an upward measure inverts). */
export function checkMeasured(ratchet, measured) {
  const failures = [];
  for (const name of sortedKeys(ratchet.ceilings)) {
    const ceiling = ratchet.ceilings.get(name);
    if (!measured.has(name)) {
      failures.push(`ceiling \`${name}\` has no measurement`);
      continue;
    }
    const value = measured.get(name);
    const upward = ratchet.upward.has(name);
    if ((!upward && value > ceiling) || (upward && value < ceiling)) {
      failures.push(`${name}: measured ${value}, ceiling ${ceiling}; move the behaviour into data/seed or data/meta rules instead of raising the ceiling (issue #1085 D1)`);
    } else if ((!upward && value < ceiling) || (upward && value > ceiling)) {
      failures.push(`${name}: improved from ${ceiling} to ${value}; lower the reviewed ceiling in data/meta/debt-ratchet.lino in this commit`);
    }
  }
  return failures;
}

/** No ceiling higher than before, and no upward floor lower. */
export function checkAgainstPrevious(previous, current) {
  const failures = [];
  for (const name of sortedKeys(previous.ceilings)) {
    const before = previous.ceilings.get(name);
    if (!current.ceilings.has(name)) {
      failures.push(`ceiling \`${name}\` was removed; a ratchet is not lowered by deleting it`);
      continue;
    }
    const now = current.ceilings.get(name);
    if (current.upward.has(name)) {
      if (now < before) {
        failures.push(`${name}: floor lowered from ${before} to ${now}; an upward measure's floor can only move up`);
      }
    } else if (now > before && !announcesCorrection(current, name, before)) {
      failures.push(`${name}: ceiling raised from ${before} to ${now}; a ceiling can only move down. If the old value counted the wrong set, say so in this measure's \`note\` as "${CORRECTION_MARKER}" and name the ${before} it corrects (plan 00 §6.2)`);
    }
  }
  return failures;
}

export function announcesCorrection(ratchet, name, before) {
  const note = ratchet.notes.get(name);
  if (note === undefined || !note.includes(CORRECTION_MARKER)) return false;
  return note.split(/[^0-9]/).some((token) => token === String(before));
}

function git(repo, args) {
  const output = spawnSync('git', ['-C', repo, ...args], { maxBuffer: 1 << 30 });
  if (output.error) throw new RustError(`could not run git ${debugStringList(args)}: ${ioErrorMessage(output.error)}`);
  if (output.status !== 0) {
    throw new RustError(`git ${debugStringList(args)} failed: ${trim(output.stderr.toString('utf8'))}`);
  }
  return trim(output.stdout.toString('utf8'));
}

function ratchetAt(repo, revision) {
  let text;
  try {
    text = git(repo, ['show', `${revision}:${LEDGER}`]);
  } catch (error) {
    if (error instanceof RustError
      && (error.message.includes('does not exist') || error.message.includes('exists on disk, but not in'))) {
      return null;
    }
    throw error;
  }
  return parseRatchet(text);
}

function parseOptions(args) {
  let repo = '.';
  const baseRef = process.env.GITHUB_BASE_REF;
  let base = baseRef ? `origin/${baseRef}` : null;
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === '--repo' || arg === '--base') {
      if (index + 1 >= args.length) throw new RustError(`${arg} requires a value`);
      index += 1;
      if (arg === '--repo') repo = args[index];
      else base = args[index];
    } else if (arg === '--help' || arg === '-h') {
      throw new RustError('usage: check-debt-ratchet.rs --base <rev> [--repo <path>]');
    } else {
      throw new RustError(`unknown argument: ${arg}`);
    }
  }
  if (base === null) {
    throw new RustError('--base <rev> is required: without it the ceilings are compared against nothing and a raised ceiling passes. Use --base origin/main locally; CI supplies GITHUB_BASE_REF.');
  }
  return { repo, base };
}

/** `Path::new(p) == Path::new(".")`: a leading `.` and nothing but `.` or `/` after it. */
function isCurrentDirectory(path) {
  const parts = path.split('/');
  return parts[0] === '.' && parts.slice(1).every((part) => part === '' || part === '.');
}

function run(out) {
  const options = parseOptions(process.argv.slice(2));
  const repo = isCurrentDirectory(options.repo) ? git(options.repo, ['rev-parse', '--show-toplevel']) : options.repo;
  let text;
  try {
    text = readToString(join(repo, LEDGER));
  } catch (error) {
    throw new RustError(`${LEDGER}: ${ioErrorMessage(error)}`);
  }
  const ratchet = parseRatchet(text);
  const measured = measure(repo);
  out(`debt ratchet (${LEDGER}):`);
  for (const name of sortedKeys(ratchet.ceilings)) {
    out(`  ${name}: measured ${measured.get(name) ?? 0} / ceiling ${ratchet.ceilings.get(name)}`);
  }
  const failures = checkMeasured(ratchet, measured);
  let previous;
  try {
    previous = ratchetAt(repo, options.base);
  } catch (error) {
    if (!(error instanceof RustError)) throw error;
    out(`  (skipping the base comparison: ${error.message})`);
    previous = undefined;
  }
  if (previous === null) {
    out(`  (${options.base} has no ${LEDGER}; nothing to compare against)`);
  } else if (previous !== undefined) {
    for (const name of sortedKeys(previous.ceilings)) {
      const before = previous.ceilings.get(name);
      const now = ratchet.ceilings.get(name) ?? before;
      if (now > before && announcesCorrection(ratchet, name, before)) {
        out(`  (${name}: ${before} -> ${now}, announced as a ${CORRECTION_MARKER} in the ledger's note)`);
      }
    }
    failures.push(...checkAgainstPrevious(previous, ratchet));
  }
  if (failures.length === 0) {
    out('debt ratchet holds');
    return;
  }
  for (const failure of failures) out(`::error file=${LEDGER}::${failure}`);
  throw new RustError(`${failures.length} debt ratchet failure(s)`);
}

function main() {
  try {
    run((line) => process.stdout.write(`${line}\n`));
  } catch (error) {
    if (!(error instanceof RustError)) throw error;
    process.stderr.write(`check-debt-ratchet: ${error.message}\n`);
    process.exitCode = 1;
  }
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) main();
