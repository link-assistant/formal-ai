#!/usr/bin/env node
// Apply the links notation rules of data/meta/notation-rules.lino by rule,
// one family at a time (PR #1188, R1188-U6; docs/links-notation-style.md).
//
// The rule: a name of the notation we own is spelled with `-`, not `_`.
// A family names its `.lino` files. Its names are the unquoted `_` names of
// those files that occur in no other `.lino` file (a shared name waits for the
// family that owns all its files), less the names a shell, Python or workflow
// file spells (the pass rewrites JavaScript and Rust only) and the names the
// family keeps with a reason. For each name the pass rewrites:
//
//   - the family's `.lino` files, and their byte mirrors (rust/embedded/,
//     and the untracked deploy copies js/seed/ and the engine package assets);
//   - the code that reads them: the name as a token inside a string literal
//     of js/, scripts/, rust/src/, rust/tests/ and the other reader roots, and
//     inside a JavaScript regular expression, where it accepts both spellings
//     during the transition (`specialization[-_]ceiling`); a comment mention
//     too, unless the same spelling is a code identifier somewhere, since code
//     identifiers follow their language's convention and are never touched;
//   - the backticked mentions in the documentation we own (not the case
//     studies and logs, which are history).
//
// Every renamed name is recorded in data/meta/notation-renames.lino, so the
// check can prove no reader went back to an old spelling. ts/ is regenerated
// with `node scripts/translate-es.mjs --write` after a pass.
//
// Usage:
//   node experiments/formal_ai_subagent/apply-notation-rules.mjs --family <name> --write
//   node experiments/formal_ai_subagent/apply-notation-rules.mjs --family <name> --list
//   node experiments/formal_ai_subagent/apply-notation-rules.mjs --family <name> --files [--lines]   (what would change)
//   node experiments/formal_ai_subagent/apply-notation-rules.mjs --check

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { RULES_FILE, namesOfLine, parseTree, readRules } from '../../scripts/lib/links-notation-names.mjs';
import {
  identifiersIn,
  replaceTokens,
  rewriteDocument,
  rewriteNotation,
  rewriteSource,
  sourceSpans,
} from '../../scripts/lib/notation-substitution.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
export const RENAMES_FILE = 'data/meta/notation-renames.lino';
const GENERATOR = 'node experiments/formal_ai_subagent/apply-notation-rules.mjs --family <name> --write';

/** The roots whose code reads the notation. */
const READER_ROOTS = ['js', 'scripts', 'rust/src', 'rust/tests', 'rust/examples', 'packages', 'desktop', 'experiments/js_dogfood', 'experiments/formal_ai_subagent'];
/** Copies and generated output, never edited as readers. */
const NOT_READER = /^(?:js\/seed|js\/vendor|rust\/embedded|ts)\/|\.bundle\.js$|(?:^|\/)node_modules\/|^experiments\/formal_ai_subagent\/sandboxes\//u;
/** The `.lino` files that are history or copies, never notation to rename. */
const NOT_NOTATION = /^(?:rust\/embedded|docs\/case-studies|dev|experiments)\/|\/sandboxes\//u;
/**
 * Shell, Python and workflow files: the pass does not rewrite them, so a name
 * they spell waits until a family moves it together with them.
 */
const UNREWRITTEN = /\.(?:sh|py|yml|yaml|toml)$/u;
const UNREWRITTEN_ROOTS = ['.github', 'scripts', 'experiments', 'rust', 'js', 'packages', 'desktop'];

/** The documentation we own: history (case studies, logs, changelogs) stays. */
const DOCUMENTS = /^(?:docs\/(?!case-studies\/).*|[^/]+|data\/README|rust\/README|js\/README)\.md$/u;

function tracked(paths = []) {
  return execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', ...paths], {
    cwd: ROOT,
    encoding: 'utf8',
    maxBuffer: 1 << 28,
  })
    .split('\n')
    .filter((path) => path && existsSync(join(ROOT, path)));
}

const read = (path) => readFileSync(join(ROOT, path), 'utf8');

/** The files a family names, by glob. */
export function familyFiles(family) {
  return tracked(family.files.map((pattern) => `:(glob)${pattern}`)).filter((path) => path.endsWith('.lino')).sort();
}

/** Every `.lino` file whose names are notation (not history, not a copy). */
function notationFiles() {
  return tracked(['*.lino']).filter((path) => !NOT_NOTATION.test(path) && path !== RULES_FILE && path !== RENAMES_FILE);
}

function namesOfText(text) {
  const names = new Set();
  for (const line of text.split('\n')) {
    namesOfLine(line).forEach(({ name }) => names.add(name));
  }
  return names;
}

/**
 * The renames of one family: `{mapping, shared, kept, collisions}`.
 * @param {object} family
 */
export function planFamily(family) {
  const files = familyFiles(family);
  const inFamily = new Set(files);
  const own = new Set();
  files.forEach((path) => namesOfText(read(path)).forEach((name) => own.add(name)));
  const elsewhere = new Set();
  for (const path of notationFiles()) {
    if (!inFamily.has(path)) {
      namesOfText(read(path)).forEach((name) => elsewhere.add(name));
    }
  }
  const unrewritten = new Set();
  for (const path of tracked(UNREWRITTEN_ROOTS).filter((candidate) => UNREWRITTEN.test(candidate) && !NOT_READER.test(candidate))) {
    for (const match of read(path).matchAll(/[a-z][a-z0-9]*(?:_[a-z0-9]+)+/gu)) {
      if (own.has(match[0])) {
        unrewritten.add(match[0]);
      }
    }
  }
  const mapping = new Map();
  const shared = [];
  const outsideCode = [];
  const kept = [];
  const collisions = [];
  for (const name of [...own].sort()) {
    if (!name.includes('_')) {
      continue;
    }
    const renamed = name.replace(/_/gu, '-');
    if (family.keepNames.has(name)) {
      kept.push(name);
    } else if (elsewhere.has(name)) {
      shared.push(name);
    } else if (unrewritten.has(name)) {
      outsideCode.push(name);
    } else if (own.has(renamed) || elsewhere.has(renamed)) {
      collisions.push(name);
    } else {
      mapping.set(name, renamed);
    }
  }
  return { files, mapping, shared, kept, collisions, outsideCode };
}

function languageOf(path) {
  if (path.endsWith('.rs')) {
    return 'rust';
  }
  return /\.(?:mjs|cjs|js|jsx)$/u.test(path) ? 'javascript' : null;
}

function readerFiles() {
  return tracked(READER_ROOTS).filter((path) => languageOf(path) !== null && !NOT_READER.test(path));
}

function mirrorsOf(path) {
  const mirrors = [`rust/embedded/${path}`];
  if (path.startsWith('data/seed/')) {
    const name = path.slice('data/seed/'.length);
    mirrors.push(`js/seed/${name}`, `packages/formal-ai-engine/assets/seed/${name}`);
  }
  return mirrors.filter((mirror) => existsSync(join(ROOT, mirror)));
}

let dryRun = false;
let showLines = false;

function writeIfChanged(path, before, after, changed) {
  if (after !== before) {
    if (!dryRun) {
      writeFileSync(join(ROOT, path), after);
    }
    if (showLines) {
      const old = before.split('\n');
      after.split('\n').forEach((line, index) => {
        if (line !== old[index]) {
          console.log(`${path}:${index + 1}\n  - ${old[index].trim()}\n  + ${line.trim()}`);
        }
      });
    }
    changed.push(path);
  }
}

/**
 * Apply one family's renames: the notation, its mirrors, the readers and the
 * documents. Returns the changed paths.
 * @param {Map<string, string>} mapping
 * @param {Array<string>} files
 * @returns {Array<string>}
 */
export function applyMapping(mapping, files) {
  const changed = [];
  for (const path of files) {
    const before = read(path);
    const after = rewriteNotation(before, mapping);
    writeIfChanged(path, before, after, changed);
    for (const mirror of mirrorsOf(path)) {
      writeIfChanged(mirror, read(mirror), after, changed);
    }
  }
  const readers = readerFiles().map((path) => ({ path, language: languageOf(path), text: read(path) }));
  const identifiers = new Set();
  readers.forEach(({ text, language }) => identifiersIn(text, language, mapping).forEach((name) => identifiers.add(name)));
  for (const { path, language, text } of readers) {
    writeIfChanged(path, text, rewriteSource(text, language, mapping, identifiers).text, changed);
  }
  for (const path of tracked(['*.md']).filter((candidate) => DOCUMENTS.test(candidate))) {
    const text = read(path);
    writeIfChanged(path, text, rewriteDocument(text, mapping).text, changed);
  }
  return changed;
}

/** The paths `applyMapping` would change, without writing. */
export function applyMappingDry(mapping, files) {
  dryRun = true;
  try {
    return applyMapping(mapping, files);
  } finally {
    dryRun = false;
  }
}

/** The recorded renames, per family. */
export function readRenames(text) {
  const renames = new Map();
  const top = parseTree(text).find((node) => node.line === 'notation-renames');
  for (const family of top?.children ?? []) {
    const name = family.line.split(/\s+/u)[1];
    renames.set(name, new Map(family.children.map((entry) => entry.line.split(/\s+/u))));
  }
  return renames;
}

function renderRenames(renames) {
  const lines = [
    `# Generated by \`${GENERATOR}\`; checked by its --check.`,
    '#',
    '# Every name a notation rule pass renamed, per family of',
    `# ${RULES_FILE}: the old spelling, then the new one. The check fails`,
    '# when a reader holds an old spelling again. This file keeps the old',
    '# spellings on purpose, so scripts/measure-notation.mjs does not count it.',
    'notation-renames',
  ];
  for (const [family, mapping] of renames) {
    lines.push(`  family ${family}`);
    for (const [old, renamed] of mapping) {
      lines.push(`    ${old} ${renamed}`);
    }
  }
  return `${lines.join('\n')}\n`;
}

function readRenamesFile() {
  return existsSync(join(ROOT, RENAMES_FILE)) ? readRenames(read(RENAMES_FILE)) : new Map();
}

/**
 * The problems of the applied families: an old spelling left in the notation
 * or in a reader's literal, or a family file holding one of its own `_` names.
 * @returns {Array<string>}
 */
export function checkApplied(rules) {
  const problems = [];
  const renames = readRenamesFile();
  const old = new Map();
  for (const [family, mapping] of renames) {
    if (!rules.families.some((entry) => entry.name === family)) {
      problems.push(`${RENAMES_FILE}: family ${family} is not in ${RULES_FILE}`);
    }
    mapping.forEach((renamed, name) => old.set(name, renamed));
  }
  for (const path of notationFiles()) {
    for (const name of namesOfText(read(path))) {
      if (old.has(name)) {
        problems.push(`${path}: the renamed name ${name} is spelled ${old.get(name)} now`);
      }
    }
  }
  for (const path of readerFiles()) {
    const text = read(path);
    for (const span of sourceSpans(text, languageOf(path))) {
      if (span.kind !== 'string') {
        continue;
      }
      replaceTokens(text.slice(span.start, span.end), old, (name) => {
        problems.push(`${path}: a literal holds the old spelling ${name} (now ${old.get(name)})`);
        return name;
      });
    }
  }
  for (const family of rules.families.filter((entry) => renames.has(entry.name))) {
    const plan = planFamily(family);
    for (const name of plan.mapping.keys()) {
      problems.push(`family ${family.name}: ${name} is still spelled with _; run --family ${family.name} --write`);
    }
  }
  return problems;
}

function main(argv) {
  const rules = readRules(read(RULES_FILE));
  if (argv.includes('--check')) {
    const problems = checkApplied(rules);
    problems.forEach((problem) => console.error(`::error::${problem}`));
    console.log(`notation rule families applied: ${readRenamesFile().size}; problems: ${problems.length}`);
    return problems.length === 0 ? 0 : 1;
  }
  const familyName = argv[argv.indexOf('--family') + 1];
  const family = rules.families.find((entry) => entry.name === familyName);
  if (!argv.includes('--family') || !family) {
    console.error(`name a family of ${RULES_FILE} with --family: ${rules.families.map((entry) => entry.name).join(', ')}`);
    return 2;
  }
  const plan = planFamily(family);
  console.log(`family ${family.name}: ${plan.files.length} files, ${plan.mapping.size} names to rename`);
  console.log(`  shared with other files (wait for their family): ${plan.shared.join(' ') || 'none'}`);
  console.log(`  spelled in shell, Python or workflow files (wait for their family): ${plan.outsideCode.join(' ') || 'none'}`);
  console.log(`  kept: ${plan.kept.join(' ') || 'none'}`);
  console.log(`  collisions with an existing - spelling: ${plan.collisions.join(' ') || 'none'}`);
  if (argv.includes('--list')) {
    plan.mapping.forEach((renamed, name) => console.log(`  ${name} -> ${renamed}`));
  }
  if (argv.includes('--files')) {
    dryRun = true;
    showLines = argv.includes('--lines');
    applyMapping(plan.mapping, plan.files).forEach((path) => console.log(`  would change ${path}`));
    return 0;
  }
  if (!argv.includes('--write')) {
    return 0;
  }
  const changed = applyMapping(plan.mapping, plan.files);
  const renames = readRenamesFile();
  const recorded = renames.get(family.name) ?? new Map();
  plan.mapping.forEach((renamed, name) => recorded.set(name, renamed));
  renames.set(family.name, new Map([...recorded].sort(([a], [b]) => a.localeCompare(b))));
  writeFileSync(join(ROOT, RENAMES_FILE), renderRenames(renames));
  console.log(`changed ${changed.length} files:`);
  changed.forEach((path) => console.log(`  ${path}`));
  console.log(`recorded the renames in ${RENAMES_FILE}; regenerate ts/ with node scripts/translate-es.mjs --write`);
  return 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
