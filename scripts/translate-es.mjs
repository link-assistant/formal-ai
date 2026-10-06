#!/usr/bin/env node
// The js → ts leg of `formal-ai translate` (plan 16 L2/L3), in JavaScript.
//
// A byte-for-byte twin of rust/src/es_tokenizer.rs (`tokenize`) and the
// canonical renderer in rust/src/es_meta.rs (`write_source`): every leaf
// separated by one space, group delimiters spaced, template chunks verbatim.
// It exists so the js-first cycle (2026-10-06 doctrine) can regenerate
// `ts/` without compiling the crate; the rust tier in layered-ci.yml stays
// the authority and diffs this script's output against the native one.
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
import { fileURLToPath } from 'node:url';

const REGEX_CONTEXT_KEYWORDS = [
  'await', 'case', 'delete', 'do', 'else', 'in', 'instanceof', 'new', 'of', 'return', 'throw',
  'typeof', 'void', 'yield',
];

const PUNCTUATORS = [
  '>>>=', '...', '===', '!==', '**=', '<<=', '>>=', '>>>', '&&=', '||=', '??=', '=>', '==', '!=',
  '<=', '>=', '&&', '||', '??', '?.', '++', '--', '+=', '-=', '*=', '/=', '%=', '&=', '|=', '^=',
  '<<', '>>', '**', '{', '}', '(', ')', '[', ']', ';', ',', '<', '>', '+', '-', '*', '/', '%',
  '&', '|', '^', '!', '~', '?', ':', '=', '.', '@', '#',
];

const OPENS = { paren: '(', bracket: '[', brace: '{' };
const CLOSES = { paren: ')', bracket: ']', brace: '}' };

/** @param {number} b @returns {boolean} */
function isWhitespace(b) {
  // Rust's u8::is_ascii_whitespace: space, \t, \n, \x0C, \r (not \x0B).
  return b === 0x20 || b === 0x09 || b === 0x0a || b === 0x0c || b === 0x0d;
}

/** @param {number} b @returns {boolean} */
function isDigit(b) {
  return b >= 0x30 && b <= 0x39;
}

/** @param {number} b @returns {boolean} */
function isAlpha(b) {
  return (b >= 0x41 && b <= 0x5a) || (b >= 0x61 && b <= 0x7a);
}

/** @param {number} b @returns {boolean} */
function isIdentByte(b) {
  return isAlpha(b) || isDigit(b) || b === 0x5f || b === 0x24 || b >= 0x80;
}

/**
 * @param {Uint8Array} bytes
 * @param {number} start
 * @returns {number}
 */
function skipLine(bytes, start) {
  let pos = start;
  while (pos < bytes.length && bytes[pos] !== 0x0a) {
    pos += 1;
  }
  return pos < bytes.length ? pos + 1 : pos;
}

/**
 * @param {Uint8Array} bytes
 * @param {number} start
 * @returns {number} index of the closing `*\/`, or -1
 */
function findCommentEnd(bytes, start) {
  for (let pos = start; pos + 1 < bytes.length; pos += 1) {
    if (bytes[pos] === 0x2a && bytes[pos + 1] === 0x2f) {
      return pos;
    }
  }
  return -1;
}

/**
 * @param {Uint8Array} bytes
 * @param {number} start
 * @returns {number} index of the closing slash, or -1
 */
function scanRegex(bytes, start) {
  let pos = start + 1;
  let inClass = false;
  while (pos < bytes.length) {
    const b = bytes[pos];
    if (b === 0x5c) {
      pos += 2;
    } else if (b === 0x5b) {
      inClass = true;
      pos += 1;
    } else if (b === 0x5d) {
      inClass = false;
      pos += 1;
    } else if (b === 0x2f && !inClass) {
      return pos;
    } else if (b === 0x0a || b === 0x0d) {
      return -1;
    } else {
      pos += 1;
    }
  }
  return -1;
}

/**
 * @param {Uint8Array} bytes
 * @param {number} start
 * @param {number} quote
 * @returns {{end: number, closed: boolean}}
 */
function scanString(bytes, start, quote) {
  let pos = start + 1;
  while (pos < bytes.length) {
    const b = bytes[pos];
    if (b === 0x5c) {
      pos += 2;
    } else if (b === quote) {
      return { end: pos + 1, closed: true };
    } else if (b === 0x0a || b === 0x0d) {
      return { end: pos, closed: false };
    } else {
      pos += 1;
    }
  }
  return { end: bytes.length, closed: false };
}

/**
 * @param {Uint8Array} bytes
 * @param {number} start
 * @returns {number}
 */
function scanNumber(bytes, start) {
  let pos = start;
  if (bytes[pos] === 0x2e) {
    pos += 1;
  }
  const next = bytes[start + 1];
  const radixPrefixed =
    bytes[start] === 0x30 &&
    (next === 0x78 || next === 0x58 || next === 0x6f || next === 0x4f || next === 0x62 || next === 0x42);
  if (radixPrefixed) {
    pos += 2;
  }
  let last = 0;
  while (pos < bytes.length) {
    const b = bytes[pos];
    if (isAlpha(b) || isDigit(b) || b === 0x5f) {
      last = b;
      pos += 1;
      continue;
    }
    if (!radixPrefixed && (last === 0x65 || last === 0x45) && (b === 0x2b || b === 0x2d)) {
      last = b;
      pos += 1;
      continue;
    }
    break;
  }
  return pos;
}

/**
 * @param {Uint8Array} bytes
 * @param {number} start
 * @param {number} templateStart
 * @returns {{end: number, next: string}}
 */
function scanTemplateChunk(bytes, start, templateStart) {
  let pos = start;
  while (pos < bytes.length) {
    const b = bytes[pos];
    if (b === 0x5c) {
      pos += 2;
    } else if (b === 0x60) {
      return { end: pos, next: 'backtick' };
    } else if (b === 0x24 && bytes[pos + 1] === 0x7b) {
      return { end: pos, next: 'dollar' };
    } else {
      pos += 1;
    }
  }
  throw new Error(`unterminated template literal at byte ${templateStart}`);
}

/**
 * @param {Uint8Array} bytes
 * @param {number} pos
 * @returns {string}
 */
function matchPunctuator(bytes, pos) {
  for (const punct of PUNCTUATORS) {
    let matches = pos + punct.length <= bytes.length;
    for (let i = 0; matches && i < punct.length; i += 1) {
      matches = bytes[pos + i] === punct.charCodeAt(i);
    }
    if (matches) {
      if (punct === '?.' && isDigit(bytes[pos + 2])) {
        return '?';
      }
      return punct;
    }
  }
  return '';
}

/**
 * Tokenize ES source into the token tree es_tokenizer.rs builds.
 * Leaves: { $: 'leaf', text, kind }; groups: { $: 'group', delim, trees };
 * templates: { $: 'template', parts: [{ $: 'chunk', text } | { $: 'interp', trees }] }.
 * @param {string} source
 * @returns {Array<object>}
 */
export function tokenize(source) {
  const bytes = new TextEncoder().encode(source);
  const decoder = new TextDecoder();
  const slice = (start, end) => decoder.decode(bytes.subarray(start, end));
  const root = [];
  const stack = [];
  const parentTrees = () => (stack.length === 0 ? root : stack[stack.length - 1].trees);
  const emit = (text, kind) => parentTrees().push({ $: 'leaf', text, kind });
  let pos = 0;
  let regexAllowed = true;
  if (bytes[0] === 0x23 && bytes[1] === 0x21) {
    pos = skipLine(bytes, 2);
  }
  for (;;) {
    const top = stack[stack.length - 1];
    if (top && top.$ === 'template') {
      const scanned = scanTemplateChunk(bytes, pos, top.start);
      if (scanned.end > top.chunkStart) {
        top.parts.push({ $: 'chunk', text: slice(top.chunkStart, scanned.end) });
      }
      pos = scanned.end;
      if (scanned.next === 'backtick') {
        stack.pop();
        parentTrees().push({ $: 'template', parts: top.parts });
        regexAllowed = false;
        pos += 1;
      } else {
        stack.push({ $: 'interp', start: pos, trees: [] });
        pos += 2;
      }
      continue;
    }
    while (pos < bytes.length && isWhitespace(bytes[pos])) {
      pos += 1;
    }
    if (pos >= bytes.length) {
      break;
    }
    const b = bytes[pos];
    if (b === 0x2f && bytes[pos + 1] === 0x2f) {
      pos = skipLine(bytes, pos + 2);
    } else if (b === 0x2f && bytes[pos + 1] === 0x2a) {
      const end = findCommentEnd(bytes, pos + 2);
      if (end < 0) {
        throw new Error(`unterminated block comment at byte ${pos}`);
      }
      pos = end + 2;
    } else if (b === 0x2f && regexAllowed) {
      const bodyEnd = scanRegex(bytes, pos);
      if (bodyEnd < 0) {
        throw new Error(`unterminated regular expression at byte ${pos}`);
      }
      let flagEnd = bodyEnd + 1;
      while (flagEnd < bytes.length && isAlpha(bytes[flagEnd])) {
        flagEnd += 1;
      }
      emit(slice(pos, flagEnd), 'regexp');
      regexAllowed = false;
      pos = flagEnd;
    } else if (b === 0x27 || b === 0x22) {
      const scanned = scanString(bytes, pos, b);
      if (!scanned.closed) {
        throw new Error(`unterminated string literal at byte ${pos}`);
      }
      emit(slice(pos, scanned.end), 'string');
      regexAllowed = false;
      pos = scanned.end;
    } else if (b === 0x60) {
      stack.push({ $: 'template', start: pos, chunkStart: pos + 1, parts: [] });
      pos += 1;
    } else if (b === 0x28 || b === 0x5b || b === 0x7b) {
      const delim = b === 0x28 ? 'paren' : b === 0x5b ? 'bracket' : 'brace';
      stack.push({ $: 'group', delim, start: pos, trees: [] });
      regexAllowed = true;
      pos += 1;
    } else if (b === 0x29 || b === 0x5d || b === 0x7d) {
      const found = b === 0x29 ? 'paren' : b === 0x5d ? 'bracket' : 'brace';
      const frame = stack[stack.length - 1];
      if (frame && frame.$ === 'group' && frame.delim === found) {
        stack.pop();
        parentTrees().push({ $: 'group', delim: found, trees: frame.trees });
        regexAllowed = found === 'brace';
      } else if (frame && frame.$ === 'interp' && found === 'brace') {
        stack.pop();
        const template = stack[stack.length - 1];
        template.parts.push({ $: 'interp', trees: frame.trees });
        template.chunkStart = pos + 1;
      } else {
        throw new Error(`unexpected closing delimiter at byte ${pos}`);
      }
      pos += 1;
    } else if (isDigit(b) || (b === 0x2e && isDigit(bytes[pos + 1]))) {
      const end = scanNumber(bytes, pos);
      emit(slice(pos, end), 'numeric');
      regexAllowed = false;
      pos = end;
    } else if (isIdentByte(b) || (b === 0x23 && isIdentByte(bytes[pos + 1]))) {
      let end = pos + (b === 0x23 ? 2 : 1);
      while (end < bytes.length && isIdentByte(bytes[end])) {
        end += 1;
      }
      const text = slice(pos, end);
      emit(text, 'identifier');
      regexAllowed = REGEX_CONTEXT_KEYWORDS.includes(text);
      pos = end;
    } else {
      const punct = matchPunctuator(bytes, pos);
      if (punct === '') {
        throw new Error(`unexpected character at byte ${pos}`);
      }
      emit(punct, 'punctuator');
      regexAllowed = punct !== ')' && punct !== ']';
      pos += punct.length;
    }
  }
  if (stack.length > 0) {
    const templateFrame = stack.slice().reverse().find((frame) => frame.$ === 'template');
    if (templateFrame) {
      throw new Error(`unterminated template literal at byte ${templateFrame.start}`);
    }
    throw new Error(`unclosed group at byte ${stack[stack.length - 1].start}`);
  }
  return root;
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
 * @param {string} directory
 * @returns {Array<string>}
 */
function walk(directory) {
  const out = [];
  for (const name of readdirSync(directory).sort()) {
    const path = join(directory, name);
    if (statSync(path).isDirectory()) {
      out.push(...walk(path));
    } else if (name.endsWith('.js')) {
      out.push(path);
    }
  }
  return out;
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
  for (const file of walk(jsRoot)) {
    const rel = relative(jsRoot, file).replace(/\.js$/, '.ts');
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
    } else if (name.endsWith('.ts')) {
      out.push(path);
    }
  }
  return out;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
