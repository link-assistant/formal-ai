#!/usr/bin/env node
// JavaScript twin of scripts/audit-seed-metadata.rs (issue #918): audits the
// problem-solving metadata on seed meaning records and keeps the per-source
// gap files under data/meta/seed-metadata-gaps/ current. It prints exactly
// what the Rust original prints, with the same exit code, so an agent can
// regenerate the gap files without compiling Rust
// (`node scripts/lib/checks-twin-parity.mjs audit-seed-metadata` compares
// the two in CI).
//
// Usage:
//   node scripts/audit-seed-metadata.mjs
//   node scripts/audit-seed-metadata.mjs --write

import { existsSync, mkdirSync, readdirSync, readFileSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { dirname, join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

import { ioErrorDisplay } from './lib/requirements-rust-compat.mjs';

const SCHEMA_PATH = 'data/meta/seed-metadata-schema.lino';
const SEED_ROOT = 'data/seed';
const GAP_DIRECTORY = 'data/meta/seed-metadata-gaps';
const AUDIT_SCOPE = 'problem-solving concept records under data/seed meanings roots';

/** Rust `str::cmp`: byte order of the UTF-8 encodings. */
function compareBytes(left, right) {
  return Buffer.compare(Buffer.from(left, 'utf8'), Buffer.from(right, 'utf8'));
}

/** Rust `str::lines`: split on `\n`, drop one trailing `\r`, no final empty line. */
function lines(text) {
  const parts = text.split('\n');
  if (parts.length && parts[parts.length - 1] === '') parts.pop();
  return parts.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
}

function unquote(value) {
  return value.length >= 2 && value.startsWith('"') && value.endsWith('"') ? value.slice(1, -1) : value;
}

/** Mirrors `parse_schema`. */
export function parseSchema(text) {
  const requiredFields = lines(text).filter((line) => line.startsWith('  required_field '))
    .map((line) => line.slice('  required_field '.length));
  const completeSources = [...new Set(lines(text).filter((line) => line.startsWith('  complete_source '))
    .map((line) => unquote(line.slice('  complete_source '.length))))].sort(compareBytes);
  if (requiredFields.length === 0 || new Set(requiredFields).size !== requiredFields.length) {
    throw new Error('schema required_field rows must be nonempty and unique');
  }
  if (completeSources.length === 0) throw new Error('schema must name at least one complete_source');
  return { requiredFields, completeSources };
}

function indentation(line) {
  let count = 0;
  while (line[count] === ' ') count += 1;
  return count;
}

/** Mirrors `is_comment`: a `#` line, whatever its indentation. */
function isComment(line) {
  return line.trimStart().startsWith('#');
}

/** Mirrors `parse_meanings`. */
export function parseMeanings(source, text) {
  const all = lines(text);
  const root = all.findIndex((line) => line.trim() !== '' && !isComment(line));
  if (root < 0 || all[root] !== 'meanings') return [];
  const records = [];
  let current = null;
  for (let index = root + 1; index < all.length; index += 1) {
    const line = all[index];
    if (line.trim() === '' || isComment(line)) continue;
    const depth = indentation(line);
    if (depth === 2) {
      if (current) records.push(current);
      const name = line.trim().split(/\s+/u)[0];
      if (!name) throw new Error(`${source}:${index + 1}: empty meaning record`);
      current = { source, name, fields: new Set() };
    } else if (depth === 4) {
      if (!current) throw new Error(`${source}:${index + 1}: direct field before a meaning record`);
      const trimmed = line.trim();
      const offset = trimmed.search(/\s/u);
      if (offset >= 0) {
        const field = trimmed.slice(0, offset);
        if (unquote(trimmed.slice(offset).trim()).trim() !== '') current.fields.add(field);
      }
    }
  }
  if (current) records.push(current);
  const names = new Set();
  for (const record of records) {
    if (names.has(record.name)) throw new Error(`${source}: duplicate top-level meaning record ${record.name}`);
    names.add(record.name);
  }
  return records;
}

/** Every `.lino` file below `directory`. */
function linoFiles(directory, files = []) {
  for (const name of readdirSync(directory)) {
    const path = join(directory, name);
    if (statSync(path).isDirectory()) linoFiles(path, files);
    else if (name.endsWith('.lino')) files.push(path);
  }
  return files;
}

function relativePath(root, path) {
  return relative(root, path).split(sep).join('/');
}

/** Rust's derived `Ord` on `MeaningRecord`: source, name, then the sorted fields. */
function compareRecords(left, right) {
  const bySource = compareBytes(left.source, right.source);
  if (bySource) return bySource;
  const byName = compareBytes(left.name, right.name);
  if (byName) return byName;
  const leftFields = [...left.fields].sort(compareBytes);
  const rightFields = [...right.fields].sort(compareBytes);
  for (let index = 0; index < Math.min(leftFields.length, rightFields.length); index += 1) {
    const byField = compareBytes(leftFields[index], rightFields[index]);
    if (byField) return byField;
  }
  return leftFields.length - rightFields.length;
}

function collectRecords(root) {
  const records = [];
  for (const path of linoFiles(join(root, SEED_ROOT))) {
    records.push(...parseMeanings(relativePath(root, path), readFileSync(path, 'utf8')));
  }
  return records.sort(compareRecords);
}

/** Mirrors `find_gaps`. */
export function findGaps(records, schema) {
  const seen = new Set();
  const gaps = [];
  for (const record of records) {
    seen.add(record.source);
    const missing = schema.requiredFields.filter((field) => !record.fields.has(field));
    if (missing.length) gaps.push({ source: record.source, record: record.name, missing });
  }
  for (const source of schema.completeSources) {
    if (!seen.has(source)) throw new Error(`complete_source ${source} has no meaning records`);
    const sourceGaps = gaps.filter((gap) => gap.source === source);
    if (sourceGaps.length) {
      const details = sourceGaps.map((gap) => `${gap.record} [${gap.missing.join(',')}]`).join('; ');
      throw new Error(`coding-path complete_source ${source} has metadata gaps: ${details}`);
    }
  }
  return gaps;
}

/** Mirrors `stable_hash`: FNV-1a over `source#record`. */
function stableHash(gap) {
  let hash = 0xcbf29ce484222325n;
  for (const byte of Buffer.from(`${gap.source}#${gap.record}`, 'utf8')) {
    hash ^= BigInt(byte);
    hash = (hash * 0x100000001b3n) & 0xffffffffffffffffn;
  }
  return hash;
}

function quoted(value) {
  return `"${value.replace(/\\/gu, '\\\\').replace(/"/gu, '\\"')}"`;
}

/** Mirrors `shared_value`: the value more than half of `values` share, else ''. */
function sharedValue(values) {
  const counts = new Map();
  for (const value of values) counts.set(value, (counts.get(value) ?? 0) + 1);
  const shared = [...counts.keys()].sort(compareBytes).find((value) => counts.get(value) * 2 > values.length);
  return shared ?? '';
}

function gapFile(source) {
  const relativeSource = source.startsWith(SEED_ROOT) ? source.slice(SEED_ROOT.length).replace(/^\/+/u, '') : source;
  return `${GAP_DIRECTORY}/${relativeSource}`;
}

/** Mirrors `render_gap_files`: gap file path → content, in path order. */
export function renderGapFiles(gaps) {
  const bySource = new Map();
  for (const gap of gaps) {
    if (!bySource.has(gap.source)) bySource.set(gap.source, []);
    bySource.get(gap.source).push(gap);
  }
  const ids = new Set();
  const files = new Map();
  for (const source of [...bySource.keys()].sort(compareBytes)) {
    const sourceGaps = bySource.get(source);
    const missing = sourceGaps.map((gap) => gap.missing.join(','));
    const shared = sharedValue(missing);
    let output = `seed-metadata-gaps\n  issue 918\n  source ${quoted(source)}\n  audit-scope ${quoted(AUDIT_SCOPE)}\n`;
    if (shared) output += `  missing ${quoted(shared)}\n`;
    sourceGaps.forEach((gap, index) => {
      const id = `seed-metadata-gap-${stableHash(gap).toString(16).padStart(16, '0')}`;
      if (ids.has(id)) throw new Error(`stable gap id collision: ${id}`);
      ids.add(id);
      output += `  gap ${id}\n    record ${quoted(gap.record)}\n`;
      if (missing[index] !== shared) output += `    missing ${quoted(missing[index])}\n`;
    });
    files.set(gapFile(source), output);
  }
  return files;
}

function checkOrWrite(root, expected, write) {
  const present = new Set(existsSync(join(root, GAP_DIRECTORY))
    ? linoFiles(join(root, GAP_DIRECTORY)).map((path) => relativePath(root, path))
    : []);
  if (write) {
    for (const path of [...present].sort(compareBytes)) if (!expected.has(path)) rmSync(join(root, path));
    for (const [path, content] of expected) {
      mkdirSync(dirname(join(root, path)), { recursive: true });
      writeFileSync(join(root, path), content);
    }
    return;
  }
  const errors = [];
  for (const path of [...expected.keys()].sort(compareBytes)) {
    let actual = null;
    try {
      actual = readFileSync(join(root, path), 'utf8');
    } catch (error) {
      errors.push(`missing ${path}: ${ioErrorDisplay(error)}; run with --write`);
      continue;
    }
    if (actual !== expected.get(path)) errors.push(`stale ${path}; run this script with --write`);
  }
  for (const path of [...present].sort(compareBytes)) {
    if (!expected.has(path)) errors.push(`unexpected stale gap file ${path}; run with --write`);
  }
  if (errors.length) throw new Error(errors.join('\n'));
}

/** Mirrors `fn main`: returns the exit code. */
export function main(argv = process.argv.slice(2), root = process.cwd()) {
  const write = argv.includes('--write');
  try {
    if (!existsSync(join(root, SCHEMA_PATH))) {
      throw new Error(`run from the repository root; ${SCHEMA_PATH} was not found`);
    }
    const schema = parseSchema(readFileSync(join(root, SCHEMA_PATH), 'utf8'));
    const records = collectRecords(root);
    const gaps = findGaps(records, schema);
    checkOrWrite(root, renderGapFiles(gaps), write);
    console.log(`seed metadata: audited ${records.length} concepts; ${gaps.length} per-record gaps captured in data`);
    return 0;
  } catch (error) {
    console.error(`seed metadata audit failed:\n${error.message}`);
    return 1;
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main();
}
