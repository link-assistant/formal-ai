#!/usr/bin/env node
// The js → ts leg of `formal-ai translate` (plan 16 L2/L3), in JavaScript.
//
// A byte-for-byte twin of rust/src/es_tokenizer.rs (`tokenize`, whose body
// lives in js/agentic/crate/es_tokenizer.mjs so the agentic port can count
// tokens too) and the renderer in rust/src/es_meta.rs (`layout_of` and
// `write_source`). The pivot carries the source's layout: the text before
// every token (whitespace, comments, a leading shebang line) and after the
// last one. The renderer writes each token after its own layout text, so a
// ts twin keeps its js source's lines, indentation and comments; only a
// tree with no layout (a hand-built one) renders in canonical spacing
// (every leaf separated by one space, group delimiters spaced, template
// chunks verbatim). Issue #1188 R1188-U23: every committed ts file is
// readable multi-line code.
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

/** The bytes `es_tokenizer` skips between tokens: Rust's `u8::is_ascii_whitespace`. */
const WHITESPACE = ' \t\n\f\r';

/**
 * The offset just past the line that holds `start` (past its newline).
 * @param {string} source
 * @param {number} start
 * @returns {number}
 */
function lineEnd(source, start) {
  const newline = source.indexOf('\n', start);
  return newline < 0 ? source.length : newline + 1;
}

/**
 * The offset of the next token: past whitespace, comments and, at the very
 * start, a shebang line, exactly what the tokenizer skips.
 * @param {string} source
 * @param {number} start
 * @returns {number}
 */
function skipLayout(source, start) {
  let position = start;
  if (position === 0 && source.startsWith('#!')) {
    position = lineEnd(source, 2);
  }
  for (;;) {
    while (position < source.length && WHITESPACE.includes(source[position])) {
      position += 1;
    }
    if (source.startsWith('//', position)) {
      position = lineEnd(source, position + 2);
    } else if (source.startsWith('/*', position)) {
      position = source.indexOf('*/', position + 2) + 2;
    } else {
      return position;
    }
  }
}

/**
 * Record the layout text before the next token, then step over that token's
 * own text, which must stand there.
 * @param {{ source: string, cursor: number, layout: Array<string> }} state
 * @param {string} text the token text expected next
 */
function stepOver(state, text) {
  const start = state.cursor;
  state.cursor = skipLayout(state.source, start);
  state.layout.push(state.source.slice(start, state.cursor));
  takeText(state, text);
}

/**
 * Step over text that must stand at the cursor, with no layout before it.
 * @param {{ source: string, cursor: number }} state
 * @param {string} text
 */
function takeText(state, text) {
  if (!state.source.startsWith(text, state.cursor)) {
    throw new Error(`the layout lost its place at offset ${state.cursor}`);
  }
  state.cursor += text.length;
}

/**
 * Walk one tree level in source order, recording each slot's layout text.
 * @param {{ source: string, cursor: number, layout: Array<string> }} state
 * @param {Array<object>} trees
 */
function collectLayout(state, trees) {
  for (const tree of trees) {
    switch (tree.$) {
      case 'leaf':
        stepOver(state, tree.text);
        break;
      case 'group':
        stepOver(state, OPENS[tree.delim]);
        collectLayout(state, tree.trees);
        stepOver(state, CLOSES[tree.delim]);
        break;
      case 'template':
        stepOver(state, '`');
        for (const part of tree.parts) {
          if (part.$ === 'chunk') {
            takeText(state, part.text);
          } else {
            takeText(state, '${');
            collectLayout(state, part.trees);
            stepOver(state, '}');
          }
        }
        takeText(state, '`');
        break;
      default:
        throw new Error('unknown tree tag');
    }
  }
}

/**
 * The layout of one source (mirrors es_meta.rs `layout_of`): the text before
 * every slot of its token tree, in the order the renderer visits them, then
 * the text after the last token. A slot is a leaf, a group's opening and
 * closing delimiter, a template's opening backtick, and an interpolation's
 * closing brace; everything else in a template is its verbatim chunks.
 * @param {string} source
 * @param {Array<object>} trees the tree `tokenize(source)` returned
 * @returns {Array<string>}
 */
export function layoutOf(source, trees) {
  const state = { source, cursor: 0, layout: [] };
  collectLayout(state, trees);
  state.layout.push(source.slice(state.cursor));
  return state.layout;
}

/**
 * The text of the next slot: the layout's own when it has one, else the
 * canonical spacing given.
 * @param {{ layout: Array<string>, slot: number }} state
 * @param {string} canonical
 * @returns {string}
 */
function slotText(state, canonical) {
  const text = state.slot < state.layout.length ? state.layout[state.slot] : canonical;
  state.slot += 1;
  return text;
}

/**
 * Render one tree level; `lead` is the canonical text before its first item.
 * @param {{ layout: Array<string>, slot: number }} state
 * @param {Array<object>} trees
 * @param {string} lead
 * @returns {string}
 */
function writeTrees(state, trees, lead) {
  let out = '';
  for (let index = 0; index < trees.length; index += 1) {
    const tree = trees[index];
    out += slotText(state, index === 0 ? lead : ' ');
    switch (tree.$) {
      case 'leaf':
        out += tree.text;
        break;
      case 'group':
        out += OPENS[tree.delim];
        out += writeTrees(state, tree.trees, ' ');
        out += slotText(state, tree.trees.length === 0 ? '' : ' ');
        out += CLOSES[tree.delim];
        break;
      case 'template':
        out += '`';
        for (const part of tree.parts) {
          if (part.$ === 'chunk') {
            out += part.text;
          } else {
            out += '${';
            out += writeTrees(state, part.trees, '');
            out += slotText(state, '');
            out += '}';
          }
        }
        out += '`';
        break;
      default:
        throw new Error('unknown tree tag');
    }
  }
  return out;
}

/**
 * The rendering es_meta.rs `write_source` emits: every token after its
 * layout text, or after canonical spacing when `layout` is empty.
 * @param {Array<object>} trees
 * @param {Array<string>} [layout] the source's `layoutOf`, when it has one
 * @returns {string}
 */
export function renderSource(trees, layout = []) {
  const state = { layout, slot: 0 };
  const body = writeTrees(state, trees, '');
  return body + slotText(state, '');
}

/**
 * Translate one js source to its ts rendering, layout carried.
 * @param {string} source
 * @returns {string}
 */
export function translateJsToTs(source) {
  const trees = tokenize(source);
  return renderSource(trees, layoutOf(source, trees));
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
