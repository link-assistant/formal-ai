#!/usr/bin/env node
// JavaScript twin of `scripts/check-requirement-status.rs` (PR #1188, SCRIPTS-A).
//
// Same exit codes and stdout/stderr lines as the Rust original, so the
// requirement pipeline runs without compiling Rust locally. CI runs both and
// fails when they disagree (`data/meta/ci-gates/check-requirement-status-js-twin.lino`).
//
// Usage:
//   node scripts/check-requirement-status.mjs
import { pathToFileURL } from 'node:url';

import {
  comparePaths, extension, isFile, joinPath, lines, readDirPaths, readOrEmpty, readToString, sortedStrings,
  trim, trimStart, ioErrorDisplay,
} from './lib/requirements-rust-compat.mjs';
import { readRegister } from './lib/requirements-register.mjs';

const LEDGER_DIRECTORY = 'data/meta/requirement-status-ledger';
const ALLOWED = ['implemented', 'partial', 'not-delivered', 'superseded', 'withdrawn'];
const ID_RUN = /[A-Za-z0-9_-]*/y;

/** Mirrors `fn defined_requirement_ids`, sorted as the `BTreeSet` iterates. */
export function definedRequirementIds(text) {
  const ids = new Set();
  for (const raw of lines(text)) {
    const line = trimStart(raw);
    if (!(line.startsWith('| R') || line.startsWith('### R') || line.startsWith('- R'))) continue;
    ID_RUN.lastIndex = line.indexOf('R');
    const id = ID_RUN.exec(line)[0];
    if (/^R[0-9]/.test(id)) ids.add(id);
  }
  return sortedStrings(ids);
}

/** Mirrors `fn unquote`: doubled quotes and the backslash dialect. */
export function unquote(value) {
  const trimmed = trim(value);
  const inner = trimmed.length >= 2 && trimmed.startsWith('"') && trimmed.endsWith('"') ? trimmed.slice(1, -1) : trimmed;
  let out = '';
  const characters = [...inner];
  for (let index = 0; index < characters.length; index += 1) {
    const character = characters[index];
    if (character === '"' && characters[index + 1] === '"') {
      out += '"';
      index += 1;
    } else if (character === '\\') {
      const next = characters[index + 1];
      index += 1;
      if (next === '"' || next === '\\') out += next;
      else if (next !== undefined) out += `\\${next}`;
      else out += '\\';
    } else {
      out += character;
    }
  }
  return out;
}

class Panic extends Error {}

/** Mirrors `fn ledger_rows`: a `Map` in id order, plus the duplicate failures. */
export function ledgerRows(root, failures) {
  const directory = joinPath(root, LEDGER_DIRECTORY);
  let paths;
  try {
    paths = readDirPaths(directory);
  } catch (error) {
    throw new Panic(`${directory} readable: ${ioErrorDisplay(error)}`);
  }
  paths = paths.filter((path) => extension(path) === 'lino').sort(comparePaths);
  const rows = new Map();
  for (const path of paths) {
    let source;
    try {
      source = readToString(path);
    } catch (error) {
      throw new Panic(`${path} readable: ${ioErrorDisplay(error)}`);
    }
    let id = '';
    let row = { shard: '', verdict: '', automatedTest: '' };
    const close = () => {
      if (id === '') return;
      if (rows.has(id)) failures.push('the status ledger contains a duplicate requirement id');
      rows.set(id, row);
      id = '';
      row = { shard: '', verdict: '', automatedTest: '' };
    };
    for (const line of lines(source)) {
      const trimmed = trim(line);
      if (trimmed === 'requirement') close();
      else if (trimmed.startsWith('id ')) id = unquote(trimmed.slice(3));
      else if (trimmed.startsWith('shard ')) row.shard = unquote(trimmed.slice(6));
      else if (trimmed.startsWith('verdict ')) row.verdict = unquote(trimmed.slice(8));
      else if (trimmed.startsWith('automated_test ')) row.automatedTest = unquote(trimmed.slice(15));
    }
    close();
  }
  return new Map(sortedStrings(rows.keys()).map((key) => [key, rows.get(key)]));
}

/** Mirrors `fn main`: returns the exit code, printing what Rust prints. */
export function main(root = process.cwd(), out = console.log, err = console.error) {
  try {
    let requirements;
    try {
      requirements = readRegister(root);
    } catch (error) {
      throw new Panic(error.message);
    }
    const expected = definedRequirementIds(requirements);
    const failures = [];
    const rows = ledgerRows(root, failures);
    const expectedSet = new Set(expected);
    for (const missing of expected) if (!rows.has(missing)) failures.push(`${missing}: absent from the status ledger`);
    for (const stale of rows.keys()) {
      if (!expectedSet.has(stale)) failures.push(`${stale}: absent from the assembled REQUIREMENTS.md parts`);
    }
    for (const [id, row] of rows) {
      if (!ALLOWED.includes(row.verdict)) failures.push(`${id}: unknown verdict \`${row.verdict}\``);
      const shard = joinPath(root, row.shard);
      if (!isFile(shard)) failures.push(`${id}: owning shard \`${row.shard}\` does not exist`);
      else if (!readOrEmpty(shard).includes(id)) failures.push(`${id}: owning shard \`${row.shard}\` does not name it`);
      if (row.verdict === 'implemented') {
        if (row.automatedTest === '') failures.push(`${id}: implemented without an automated test`);
        else if (!isFile(joinPath(root, row.automatedTest))) failures.push(`${id}: automated test \`${row.automatedTest}\` does not exist`);
      }
    }
    if (!failures.length) {
      out(`requirement-status parity holds for ${rows.size} assembled requirements`);
      return 0;
    }
    for (const failure of failures) err(`requirement-status: ${failure}`);
    return 1;
  } catch (error) {
    if (!(error instanceof Panic)) throw error;
    // A Rust panic: the message and exit code match; the source location
    // line names this script rather than the compiled Rust one.
    err(`thread 'main' panicked at scripts/check-requirement-status.mjs:\n${error.message}`);
    return 101;
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  process.exitCode = main();
}
