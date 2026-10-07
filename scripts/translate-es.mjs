#!/usr/bin/env node
// The js → ts leg of `formal-ai translate` (plan 16 L2/L3), in JavaScript.
//
// A byte-for-byte twin of rust/src/es_tokenizer.rs (`tokenize`, whose body
// lives in js/agentic/crate/es_tokenizer.mjs so the agentic port can count
// tokens too) and the canonical renderer in rust/src/es_meta.rs
// (`write_source`): every leaf separated by one space, group delimiters spaced, template chunks verbatim.
// It exists so the js-first cycle (2026-10-06 doctrine) can regenerate
// `ts/` without compiling the crate; the rust tier in layered-ci.yml stays
// the authority and diffs this script's output against the native one.
//
// The agentic modules (`js/agentic/**/*.mjs`, the node-side twins of
// rust/src/agentic_coding and its crate helpers) are mirrored the same way
// into `ts/agentic/**/*.mts`: the ES-module extension maps to its TypeScript
// counterpart, so their relative `./x.mjs` imports resolve to `./x.mts` under
// NodeNext resolution. The native leg (`formal-ai translate --from js --to
// ts --write`) renders the same `.mts` set through
// `meta_translate::MODULE_SUBTREES`, so both translators own one file set.
//
// Usage:
//   node scripts/translate-es.mjs --write    regenerate ts/ from js/
//   node scripts/translate-es.mjs --check    exit 1 when ts/ drifted
//   node scripts/translate-es.mjs FILE.js    print one rendered file
//
// Portable-subset style (link-foundation/meta-language): plain functions,
// const/let, tagged objects ({ $: 'leaf' | 'group' | 'template' }), no
// classes, so this file can itself be translated onward to Rust.

import { readFileSync, writeFileSync, mkdirSync, readdirSync, statSync, rmSync } from 'node:fs';
import { dirname, join, relative } from 'node:path';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

import { tokenize as tokenizeTree } from '../js/agentic/crate/es_tokenizer.mjs';

const OPENS = { paren: '(', bracket: '[', brace: '{' };
const CLOSES = { paren: ')', bracket: ']', brace: '}' };

/** The sentence `TokenizeError`'s `Display` renders for each variant slug. */
const TOKENIZE_ERROR_TEXT = {
  unterminated_string: 'unterminated string literal',
  unterminated_template: 'unterminated template literal',
  unterminated_regexp: 'unterminated regular expression',
  unterminated_comment: 'unterminated block comment',
  unclosed_group: 'unclosed group',
  unexpected_closing: 'unexpected closing delimiter',
  unexpected_char: 'unexpected character',
};

/**
 * Tokenize ES source into the token tree es_tokenizer.rs builds (the
 * tokenizer itself is js/agentic/crate/es_tokenizer.mjs); a failure reads
 * as the native `Display` does.
 * Leaves: { $: 'leaf', text, kind }; groups: { $: 'group', delim, trees };
 * templates: { $: 'template', parts: [{ $: 'chunk', text } | { $: 'interp', trees }] }.
 * @param {string} source
 * @returns {Array<object>}
 */
export function tokenize(source) {
  try {
    return tokenizeTree(source);
  } catch (error) {
    const text = error && TOKENIZE_ERROR_TEXT[error.kind];
    if (text) throw new Error(`${text} at byte ${error.start}`);
    throw error;
  }
}

/**
 * The canonical rendering es_meta.rs `write_source` emits.
 * @param {Array<object>} trees
 * @returns {string}
 */
export function renderSource(trees) {
  const pieces = [];
  for (const tree of trees) {
    switch (tree.$) {
      case 'leaf':
        pieces.push(tree.text);
        break;
      case 'group':
        pieces.push(
          tree.trees.length === 0
            ? OPENS[tree.delim] + CLOSES[tree.delim]
            : `${OPENS[tree.delim]} ${renderSource(tree.trees)} ${CLOSES[tree.delim]}`,
        );
        break;
      case 'template': {
        let text = '`';
        for (const part of tree.parts) {
          text += part.$ === 'chunk' ? part.text : `\${${renderSource(part.trees)}}`;
        }
        pieces.push(`${text}\``);
        break;
      }
      default:
        throw new Error('unknown tree tag');
    }
  }
  return pieces.join(' ');
}

/**
 * Translate one js source to its ts rendering.
 * @param {string} source
 * @returns {string}
 */
export function translateJsToTs(source) {
  return renderSource(tokenize(source));
}

/**
 * @param {string} repo
 * @param {string} directory
 * @returns {Array<string>}
 */
function walk(repo, directory) {
  const out = [];
  for (const name of readdirSync(directory).sort()) {
    const path = join(directory, name);
    if (statSync(path).isDirectory()) {
      out.push(...walk(repo, path));
    } else if (isTranslatedSource(relative(repo, path).split('\\').join('/'))) {
      out.push(path);
    }
  }
  return out;
}

/** The module roots whose `.mjs` sources gain a `.mts` twin. */
const MODULE_ROOTS = ['js/agentic/'];

/**
 * Whether a repository-relative path is a translated source.
 * @param {string} line
 * @returns {boolean}
 */
function isTranslatedSource(line) {
  return line.endsWith('.js') || (line.endsWith('.mjs') && MODULE_ROOTS.some((root) => line.startsWith(root)));
}

/**
 * The ts path a js source renders to: `.js` → `.ts`, `.mjs` → `.mts`.
 * @param {string} rel path relative to js/
 * @returns {string}
 */
export function tsTwinPath(rel) {
  return rel.endsWith('.mjs') ? rel.replace(/\.mjs$/, '.mts') : rel.replace(/\.js$/, '.ts');
}

/**
 * The js sources a checkout carries: tracked plus new unignored files, so a
 * locally built bundle (gitignored, absent from CI) never gains a ts twin.
 * Falls back to the directory walk outside a git work tree.
 * @param {string} repo
 * @returns {Array<string>}
 */
function sourceFiles(repo) {
  try {
    const listed = execFileSync(
      'git',
      ['ls-files', '--cached', '--others', '--exclude-standard', '--', 'js'],
      { cwd: repo, encoding: 'utf8' },
    );
    return listed
      .split('\n')
      .filter(isTranslatedSource)
      .map((line) => join(repo, line))
      .filter((path) => {
        try {
          return statSync(path).isFile();
        } catch {
          return false;
        }
      })
      .sort();
  } catch {
    return walk(repo, join(repo, 'js'));
  }
}

function main(argv) {
  const repo = join(dirname(fileURLToPath(import.meta.url)), '..');
  const mode = argv[0] || '--check';
  if (mode !== '--write' && mode !== '--check') {
    process.stdout.write(translateJsToTs(readFileSync(mode, 'utf8')));
    return 0;
  }
  const jsRoot = join(repo, 'js');
  const drifted = [];
  const expected = new Set();
  for (const file of sourceFiles(repo)) {
    const rel = tsTwinPath(relative(jsRoot, file));
    const target = join(repo, 'ts', rel);
    expected.add(target);
    const rendered = translateJsToTs(readFileSync(file, 'utf8'));
    let committed = null;
    try {
      committed = readFileSync(target, 'utf8');
    } catch {
      committed = null;
    }
    if (committed !== rendered) {
      drifted.push(`ts/${rel}`);
      if (mode === '--write') {
        mkdirSync(dirname(target), { recursive: true });
        writeFileSync(target, rendered);
      }
    }
  }
  const orphans = listTs(join(repo, 'ts')).filter((file) => !expected.has(file));
  for (const orphan of orphans) {
    drifted.push(relative(repo, orphan));
    if (mode === '--write') {
      rmSync(orphan);
    }
  }
  if (drifted.length === 0) {
    console.log(`ts/ matches js/ (${expected.size} files)`);
    return 0;
  }
  console.log(`${mode === '--write' ? 'regenerated' : 'drifted'}: ${drifted.join(', ')}`);
  return mode === '--write' ? 0 : 1;
}

/**
 * @param {string} directory
 * @returns {Array<string>}
 */
function listTs(directory) {
  const out = [];
  for (const name of readdirSync(directory).sort()) {
    const path = join(directory, name);
    if (statSync(path).isDirectory()) {
      out.push(...listTs(path));
    } else if (name.endsWith('.ts') || name.endsWith('.mts')) {
      out.push(path);
    }
  }
  return out;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
