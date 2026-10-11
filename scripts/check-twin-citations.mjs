#!/usr/bin/env node
// Twin-citation drift gate (R1024): every JavaScript twin that says it
// "Mirrors `symbol` in rust/src/<path>.rs" names a Rust definition that
// still exists there.
//
// The JavaScript root is ported from and to the Rust crate by hand, module
// by module, and each twin cites its Rust original in a `Mirrors` comment.
// When the Rust side renames, moves or deletes the definition, nothing used
// to notice: the twin kept citing a file or a function that no longer holds
// it, and the next port started from a stale map. meta-language PR #196
// measures every translated module against its hand-written counterpart
// (js/scripts/generate-self-translation-report.mjs), and relative-meta-logic
// gates its two runtimes on one shared corpus
// (scripts/check-corpus-parity.mjs); this is the same discipline for the
// links between this repository's hand-written twins, which the
// self-translation report (`node scripts/self-translate.mjs --report`)
// then measures.
//
// A citation names a symbol the gate can resolve (`name`, `fn name`,
// `Type::member`, `crate::module::name`) or describes behaviour in prose
// (`the canonicalize / is_dir prologue of fn run_agent`, a method chain);
// only the first kind is checked, and the second is counted.
//
// Usage: node scripts/check-twin-citations.mjs [--list]

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const CITATION = /Mirrors\s+`([^`]+)`\s+in\s+(rust\/src\/[A-Za-z0-9_/]+\.rs)/gu;
const KEYWORDS = /^(?:(?:pub(?:\([^)]*\))?|const|async|unsafe|fn|struct|enum|type|trait|static|mod|impl|macro_rules!)\s+)+/u;

/**
 * Every citation in `text` (comment continuations joined, so a citation
 * wrapped over two comment lines still reads as one).
 * @param {string} text
 * @returns {Array<{symbol: string, path: string}>}
 */
export function citations(text) {
  const joined = text.replace(/\n\s*(?:\/\/+|\*)\s*/gu, ' ');
  return [...joined.matchAll(CITATION)].map((match) => ({ symbol: match[1].trim(), path: match[2] }));
}

/**
 * The path segments a citation resolves, or null when it is prose.
 * @param {string} symbol
 * @returns {Array<string> | null}
 */
export function resolvableSegments(symbol) {
  const bare = symbol.replace(KEYWORDS, '').replace(/\(\)$/u, '');
  if (!/^[A-Za-z_][A-Za-z0-9_]*(::[A-Za-z_][A-Za-z0-9_]*)*$/u.test(bare)) return null;
  return bare.split('::').filter((segment) => segment !== 'crate' && segment !== 'self' && segment !== 'super');
}

/**
 * Whether `rust` defines, or imports, one of the cited segments: the last
 * segment (the item) or the first (its type or module).
 * @param {string} rust
 * @param {Array<string>} segments
 * @returns {boolean}
 */
export function resolves(rust, segments) {
  const defined = (name) => new RegExp(`\\b(?:fn|const|static|struct|enum|type|trait|mod|union)\\s+${name}\\b|macro_rules!\\s*${name}\\b`, 'u').test(rust);
  const imported = (name) => new RegExp(`\\buse\\s[^;]*\\b${name}\\b`, 'u').test(rust);
  const last = segments[segments.length - 1];
  if (defined(last)) return true;
  return segments.length > 1 && (defined(segments[0]) || imported(segments[0]));
}

/**
 * Check every tracked JavaScript file under js/.
 * @param {string} repo
 * @returns {{checked: number, prose: number, stale: Array<string>}}
 */
export function checkTwinCitations(repo) {
  const files = execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', '--', 'js'], { cwd: repo, encoding: 'utf8' })
    .split('\n')
    .filter((path) => /\.m?js$/u.test(path) && existsSync(join(repo, path)));
  const rustCache = new Map();
  const rustOf = (path) => {
    if (!rustCache.has(path)) rustCache.set(path, existsSync(join(repo, path)) ? readFileSync(join(repo, path), 'utf8') : null);
    return rustCache.get(path);
  };
  const result = { checked: 0, prose: 0, stale: [] };
  for (const file of files) {
    for (const { symbol, path } of citations(readFileSync(join(repo, file), 'utf8'))) {
      const rust = rustOf(path);
      if (rust === null) {
        result.stale.push(`${file}: \`${symbol}\` cites ${path}, which does not exist`);
        continue;
      }
      const segments = resolvableSegments(symbol);
      if (segments === null) {
        result.prose += 1;
        continue;
      }
      result.checked += 1;
      if (!resolves(rust, segments)) result.stale.push(`${file}: \`${symbol}\` is not defined in ${path}`);
    }
  }
  return result;
}

function main(argv) {
  const repo = join(dirname(fileURLToPath(import.meta.url)), '..');
  const { checked, prose, stale } = checkTwinCitations(repo);
  if (argv.includes('--list') || stale.length > 0) for (const line of stale) console.log(line);
  const missing = stale.filter((line) => line.endsWith('which does not exist')).length;
  console.log(`${checked + missing - stale.length} of ${checked + missing} twin citations resolve to a Rust definition; ${prose} describe behaviour in prose and are not checked`);
  return stale.length === 0 ? 0 : 1;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
