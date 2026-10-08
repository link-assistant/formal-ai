#!/usr/bin/env node
// Planner twin ratchet (R1188-U16).
//
// The agentic planner is JavaScript first: every function a module under
// js/agentic/ exports has a Rust twin in rust/src/, under the snake_case form
// of its name or under the name its doc comment cites as `Mirrors \`symbol\``
// (a `fn`, or the `const`, `static`, `struct` or `enum` it builds or returns),
// or stands for a Rust built-in or a dependency crate's item its doc comment
// names (`Rust built-in \`x\``, `Rust dependency \`crate::x\``).
// This script counts the exported functions with no such twin and holds the
// count to the ceiling in data/meta/planner-twin-ratchet.lino.
// Above the ceiling fails, so a planner function cannot land in JavaScript
// alone; below it fails too, naming the lower ceiling to commit, so the
// number only falls. Modules that exist only on the JavaScript side by design
// (the host port, the Rust `str` semantics shims) are listed in the ratchet
// file with their reason.
//
// Usage: node scripts/check-planner-twins.mjs [--list]

import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const PLANNER_ROOT = 'js/agentic';
const RUST_ROOT = 'rust/src';
const RATCHET_FILE = 'data/meta/planner-twin-ratchet.lino';

/**
 * The snake_case form of a camelCase JavaScript name, as the Rust twin spells it.
 * @param {string} name
 * @returns {string}
 */
export function snakeCase(name) {
  return name
    .replace(/([a-z0-9])([A-Z])/g, '$1_$2')
    .replace(/([A-Z])([A-Z][a-z])/g, '$1_$2')
    .toLowerCase();
}

/**
 * The comment directly above `offset`: a block comment or a run of line comments.
 * @param {string} source
 * @param {number} offset
 * @returns {string}
 */
export function leadingComment(source, offset) {
  const before = source.slice(0, offset).trimEnd();
  if (before.endsWith('*/')) {
    const start = before.lastIndexOf('/*');
    return start < 0 ? '' : before.slice(start);
  }
  const lines = before.split('\n');
  const comment = [];
  while (lines.length > 0 && lines[lines.length - 1].trim().startsWith('//')) {
    comment.unshift(lines.pop());
  }
  return comment.join('\n');
}

const ITEM_KEYWORDS = /^(?:(?:pub(?:\([^)]*\))?|const|async|unsafe|fn|struct|enum|static|type)\s+)+/u;

/**
 * The Rust item names a comment says it mirrors (`Mirrors \`Type::method\``
 * gives `method`, `Mirrors \`fn answer\`` gives `answer`, `Mirrors \`struct
 * AppliedRule\`` gives `AppliedRule`), read as scripts/check-twin-citations.mjs
 * reads them: comment continuations joined, so a citation wrapped over two
 * comment lines still reads as one.
 * @param {string} comment
 * @returns {Array<string>}
 */
export function mirroredNames(comment) {
  const joined = comment.replace(/\n\s*(?:\/\/+|\*)\s*/gu, ' ');
  return [...joined.matchAll(/Mirrors\s+`([^`]+)`/g)].map((match) =>
    match[1].trim().replace(ITEM_KEYWORDS, '').replace(/\(.*$/u, '').split(/::|\./u).pop());
}

/**
 * Whether a comment says the function stands for a Rust language or standard
 * library feature (`Rust built-in \`str::char_indices\``, `Rust built-in
 * \`#[derive(Default)]\``) or for an item of a crate the Rust root depends
 * on (`Rust dependency \`serde_json::from_str\``), which has no `fn` of its
 * own in rust/src/ to point at.
 * @param {string} comment
 * @returns {boolean}
 */
export function standsForBuiltIn(comment) {
  return /Rust (?:built-in|dependency)\s+`[^`]+`/u.test(comment.replace(/\n\s*(?:\/\/+|\*)\s*/gu, ' '));
}

/**
 * The functions a JavaScript module exports by declaration: each name and the
 * Rust names its doc comment says it mirrors, and whether it stands for a Rust
 * built-in.
 * @param {string} source
 * @returns {Array<{name: string, mirrors: Array<string>, builtIn: boolean}>}
 */
export function exportedFunctions(source) {
  const pattern = /^export\s+(?:async\s+)?function\s*\*?\s*([A-Za-z0-9_$]+)/gm;
  return [...source.matchAll(pattern)].map((match) => {
    const comment = leadingComment(source, match.index);
    return { name: match[1], mirrors: mirroredNames(comment), builtIn: standsForBuiltIn(comment) };
  });
}

/**
 * The names of every item Rust source defines that a planner function can
 * twin: a `fn` (met by the snake_case name or a citation), and a `const`,
 * `static`, `struct` or `enum` (met only by a citation, since their names are
 * upper case or CamelCase: a JavaScript function that returns a constant
 * table or builds a struct's record twins that item).
 * @param {string} source
 * @returns {Set<string>}
 */
export function rustFunctions(source) {
  const functions = /\bfn\s+([a-z0-9_]+)/g;
  const items = /\b(?:const|static|struct|enum)\s+(?!(?:fn|unsafe|async)\b)([A-Za-z0-9_]+)/g;
  return new Set([...source.matchAll(functions), ...source.matchAll(items)].map((match) => match[1]));
}

/**
 * The ceiling and the exempt modules the ratchet file records.
 * @param {string} lino
 * @returns {{ceiling: number, exempt: Set<string>}}
 */
export function ratchetFrom(lino) {
  const ceiling = /^\s*no_twin_ceiling\s+(\d+)\s*$/m.exec(lino);
  if (!ceiling) {
    throw new Error('no_twin_ceiling missing');
  }
  const exempt = new Set([...lino.matchAll(/^\s*exempt\s+(\S+)/gm)].map((match) => match[1]));
  return { ceiling: Number(ceiling[1]), exempt };
}

/**
 * The exported planner functions that have no Rust twin, as `module:name`.
 * @param {Array<{path: string, source: string}>} modules
 * @param {Set<string>} rust
 * @param {Set<string>} exempt
 * @returns {Array<string>}
 */
export function functionsWithoutTwin(modules, rust, exempt) {
  const missing = [];
  for (const { path, source } of modules) {
    if (exempt.has(path)) {
      continue;
    }
    for (const { name, mirrors, builtIn } of exportedFunctions(source)) {
      const twins = [snakeCase(name), ...mirrors];
      if (!builtIn && !twins.some((twin) => rust.has(twin))) {
        missing.push(`${path}:${name}`);
      }
    }
  }
  return missing;
}

/**
 * Tracked files under `root` with the given extension.
 * @param {string} repo
 * @param {string} root
 * @param {string} extension
 * @returns {Array<string>}
 */
function trackedFiles(repo, root, extension) {
  return execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', root], {
    cwd: repo,
    encoding: 'utf8',
  })
    .split('\n')
    .filter((path) => path.endsWith(extension));
}

function main(argv) {
  const repo = join(dirname(fileURLToPath(import.meta.url)), '..');
  const read = (path) => readFileSync(join(repo, path), 'utf8');
  const { ceiling, exempt } = ratchetFrom(read(RATCHET_FILE));
  for (const path of exempt) {
    try {
      read(path);
    } catch {
      console.error(`::error file=${RATCHET_FILE}::exempt module ${path} does not exist; remove the row.`);
      return 1;
    }
  }
  const modules = trackedFiles(repo, PLANNER_ROOT, '.mjs').map((path) => ({ path, source: read(path) }));
  const rust = rustFunctions(trackedFiles(repo, RUST_ROOT, '.rs').map(read).join('\n'));
  const missing = functionsWithoutTwin(modules, rust, exempt);
  console.log(`planner functions without a Rust twin: ${missing.length} (ceiling ${ceiling})`);
  if (argv.includes('--list')) {
    for (const entry of missing) {
      console.log(`  ${entry}`);
    }
  }
  if (missing.length > ceiling) {
    console.error(
      `::error file=${RATCHET_FILE}::${missing.length - ceiling} planner function(s) above the ceiling: ` +
        'write the Rust twin under the snake_case name in rust/src/ (R1188-U16). ' +
        'Run with --list to see them.',
    );
    return 1;
  }
  if (missing.length < ceiling) {
    console.error(
      `::error file=${RATCHET_FILE}::parity improved: lower no_twin_ceiling to ${missing.length} ` +
        'in this commit so it cannot regress.',
    );
    return 1;
  }
  return 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
