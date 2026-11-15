#!/usr/bin/env node
// JavaScript twin of `scripts/check-generated-version-literals.rs` (issue
// #1569, after the PR #1188 SCRIPTS-B twins).
//
// No memorized third-party versions under `data/` (issue #1168 R7): a literal
// 40-hex commit SHA after `@`, or a bare toolchain version behind `version:` /
// `java-version:` / `kotlin-version:` / `python-version:`, is a pin that
// stopped being resolved. Captured evidence is not scanned. Same output and
// exit codes as the Rust original; CI runs both and diffs them
// (`data/meta/ci-gates/check-generated-version-literals-js-twin.lino`). Read
// the Rust script's doc comments for why each rule exists.
//
// The Rust gate runs only on main, so PR #1498 merged three captured action
// SHAs that turned main red; the fast pull-request inventory runs this twin
// (rust/tests/web/issue-1569-generated-version-literals.test.mjs).
//
// Usage:
//   node scripts/check-generated-version-literals.mjs [--repo <path>]

import { basename, join, relative, sep } from 'node:path';
import { pathToFileURL } from 'node:url';

import { RustError, isDir, isFile, lines, readDirEntries, readToString, trimStart } from './lib/checks-rust-compat.mjs';

/** Directories under `data/` that hold captured evidence, not generated code. */
export const CAPTURED_EVIDENCE = [
  'cache',
  'source-cache',
  'benchmarks/issue-requirements',
  'benchmarks/ci-phase-timing/baseline-runs',
];

const isHex = (byte) => (byte >= 0x30 && byte <= 0x39) || (byte >= 0x41 && byte <= 0x46) || (byte >= 0x61 && byte <= 0x66);

/** The byte offset of an `@` followed by exactly forty hex characters, or null. */
export function literalActionSha(line) {
  const bytes = Buffer.from(line, 'utf8');
  for (let offset = 0; offset < Math.max(0, bytes.length - 40); offset += 1) {
    if (bytes[offset] !== 0x40) continue;
    let hex = true;
    for (let index = offset + 1; index < offset + 41; index += 1) hex &&= isHex(bytes[index]);
    if (hex && (offset + 41 >= bytes.length || !isHex(bytes[offset + 41]))) return offset;
  }
  return null;
}

/** The `version:`-shaped key this line pins a bare version behind, or null. */
export function literalToolchainVersion(line) {
  for (const key of ['kotlin-version:', 'python-version:', 'java-version:', 'version:']) {
    const offset = line.indexOf(key);
    if (offset < 0) continue;
    const value = Array.from(trimStart(line.slice(offset + key.length)));
    const quoted = value[0] === "'" || value[0] === '"';
    const digit = quoted ? value[1] : value[0];
    if (digit !== undefined && digit >= '0' && digit <= '9') return key.slice(0, -1);
  }
  return null;
}

function isCapturedEvidence(root, path) {
  const under = relative(root, path).split(sep).join('/');
  return CAPTURED_EVIDENCE.some((captured) => (captured.includes('/') ? under === captured : basename(path) === captured));
}

function scanFile(root, path, findings) {
  let text;
  try {
    text = readToString(path);
  } catch (error) {
    if (error instanceof RustError) return;
    throw error;
  }
  const shown = join(basename(root), relative(root, path));
  lines(text).forEach((line, index) => {
    const offset = literalActionSha(line);
    if (offset !== null) {
      const sha = Buffer.from(line, 'utf8').subarray(offset, offset + 41).toString('utf8');
      findings.push({ path: shown, line: index + 1,
        detail: `literal action SHA at column ${offset + 1}: pin the resolved SHA or a {placeholder} instead (${sha})` });
    }
    const key = literalToolchainVersion(line);
    if (key !== null) {
      findings.push({ path: shown, line: index + 1, detail: `literal toolchain version behind \`${key}\`: resolve it at generation time` });
    }
  });
}

function walk(root, directory, findings) {
  let entries;
  try {
    entries = readDirEntries(directory);
  } catch {
    return;
  }
  for (const entry of entries) {
    if (isDir(entry.path)) {
      if (!isCapturedEvidence(root, entry.path)) walk(root, entry.path, findings);
    } else if (isFile(entry.path)) scanFile(root, entry.path, findings);
  }
}

/** Every memorized pin under `data/`, captured evidence excluded, in the Rust walk order. */
export function scanData(data) {
  const findings = [];
  walk(data, data, findings);
  return findings;
}

function usage(message) {
  process.stderr.write(`check-generated-version-literals: ${message}\n`);
  process.exit(2);
}

function main(argv) {
  let repo = '.';
  for (let index = 0; index < argv.length; index += 1) {
    if (argv[index] !== '--repo') usage(`unknown flag ${argv[index]}`);
    index += 1;
    if (index >= argv.length) usage('--repo needs a path');
    repo = argv[index];
  }
  const data = join(repo, 'data');
  if (!isDir(data)) usage('the repository root must contain data/');
  const findings = scanData(data);
  const out = findings.map((finding) => `${finding.path}:${finding.line} ${finding.detail}\n`);
  out.push(`generated-version literals: ${findings.length} finding(s) under data/ (captured evidence excluded)\n`);
  if (findings.length === 0) {
    out.push('generated-version literals: clean\n');
    process.stdout.write(out.join(''));
    return;
  }
  out.push('generated-version literals: FAIL - resolve these pins or replace them with {placeholders}\n');
  process.stdout.write(out.join(''), () => process.exit(1));
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) main(process.argv.slice(2));
