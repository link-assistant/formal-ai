#!/usr/bin/env node
// JavaScript twin of `src_has_no_inline_unit_tests`
// (rust/tests/unit/ci-cd/source_test_placement.rs, issue #398 rule 7):
// production code under rust/src carries no inline unit tests. A test-shaped
// line is allowed only inside a complete external registration,
// `#[cfg(test)] #[path = "<file under rust/tests>"] mod <name>;`, at the top
// level of the file, whose target is a real `.rs` file inside rust/tests.
//
// Issue #1191: the Rust test ran only on main, so PR #1483's inline
// `mod tests` in rust/src/agentic_coding/work_item_leads.rs passed every pull
// request check and main then failed. This twin runs in the fast PR lane
// (rust/tests/web/source-test-placement-twin.test.mjs) without compiling.
//
// Usage: node scripts/check-source-test-placement.mjs [crate-root]

import { existsSync, lstatSync, readdirSync, readFileSync, realpathSync } from 'node:fs';
import { dirname, extname, join, relative, sep } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const CRATE = join(dirname(fileURLToPath(import.meta.url)), '..', 'rust');

/** The Rust detector: a real attribute or `mod tests` begins the trimmed line. */
export function isInlineTestMarker(trimmed) {
  if (trimmed.startsWith('#[cfg(test)')) return true;
  if (trimmed.startsWith('mod tests')) return true;
  if (trimmed.startsWith('#[')) {
    const attribute = trimmed.slice(2).trimStart();
    if (attribute.startsWith('test]') || attribute.startsWith('test(')) return true;
    const path = attribute.indexOf('::');
    if (path >= 0) {
      const suffix = attribute.slice(path + 2);
      if (suffix.startsWith('test]') || suffix.startsWith('test(')) return true;
    }
  }
  return false;
}

const WORD = /[_\p{L}\p{N}]/u;
const OPEN = { ')': '(', ']': '[', '}': '{' };

/**
 * Tokens with their nesting depth; comments and literals are atomic, so their
 * braces and test-shaped text never form an item boundary. `null` when the
 * source is unbalanced or unterminated, exactly where the Rust tokenizer gives up.
 */
export function sourceTokens(source) {
  const tokens = [];
  const delimiters = [];
  const charAt = index => String.fromCodePoint(source.codePointAt(index));
  let index = 0;
  while (index < source.length) {
    if (' \t\n\f\r'.includes(source[index])) {
      index += 1;
      continue;
    }
    if (source.startsWith('//', index)) {
      const newline = source.indexOf('\n', index);
      index = newline < 0 ? source.length : newline + 1;
      continue;
    }
    if (source.startsWith('/*', index)) {
      index += 2;
      let comments = 1;
      while (comments > 0) {
        if (index >= source.length) return null;
        if (source.startsWith('/*', index)) {
          comments += 1;
          index += 2;
        } else if (source.startsWith('*/', index)) {
          comments -= 1;
          index += 2;
        } else {
          index += charAt(index).length;
        }
      }
      continue;
    }
    const start = index;
    const depth = delimiters.length;
    const rawStart = source.startsWith('br', index) ? index + 2 : source[index] === 'r' ? index + 1 : index;
    let rawQuote = rawStart;
    while (rawQuote < source.length && source[rawQuote] === '#') rawQuote += 1;
    if (rawStart > index && source[rawQuote] === '"') {
      const closing = `"${'#'.repeat(rawQuote - rawStart)}`;
      const found = source.indexOf(closing, rawQuote + 1);
      if (found < 0) return null;
      index = found + closing.length;
    } else if (source[index] === '"') {
      index += 1;
      for (;;) {
        if (index >= source.length) return null;
        if (source[index] === '\\') {
          index += 1;
          if (index >= source.length) return null;
          index += charAt(index).length;
        } else if (source[index] === '"') {
          index += 1;
          break;
        } else {
          index += charAt(index).length;
        }
      }
    } else if (source[index] === "'") {
      // A lifetime has no closing quote; a character literal is atomic.
      let end = index + 1;
      if (source[end] === '\\') {
        end += 1;
        if (source[end] === 'u' && source[end + 1] === '{') {
          const brace = source.indexOf('}', end);
          if (brace < 0) return null;
          end = brace + 1;
        } else if (source[end] === 'x') {
          end += 3;
        } else {
          if (end >= source.length) return null;
          end += charAt(end).length;
        }
      } else {
        if (end >= source.length) return null;
        end += charAt(end).length;
      }
      index = source[end] === "'" ? end + 1 : index + 1;
    } else {
      const first = charAt(index);
      index += first.length;
      if (WORD.test(first)) {
        while (index < source.length) {
          const next = charAt(index);
          if (!WORD.test(next)) break;
          index += next.length;
        }
      } else if ('([{'.includes(first)) {
        delimiters.push(first);
      } else if (first in OPEN) {
        if (delimiters.pop() !== OPEN[first]) return null;
      }
    }
    tokens.push({ text: source.slice(start, index), start, end: index, depth });
  }
  return delimiters.length === 0 ? tokens : null;
}

const isSymlink = path => lstatSync(path, { throwIfNoEntry: false })?.isSymbolicLink() ?? null;

/** The real `.rs` file under `<crate>/tests` a `#[path]` literal names, or null. */
export function externalTestFile(crateRoot, sourcePath, literal) {
  if (!literal.startsWith('"') || !literal.endsWith('"') || literal.length < 2) return null;
  const path = literal.slice(1, -1);
  if (path === '' || path.includes('\\') || path.split('/').some(part => part === '' || part === '.')) return null;
  let root;
  let target;
  try {
    root = realpathSync(crateRoot);
    target = realpathSync(dirname(sourcePath));
  } catch {
    return null;
  }
  const inside = (child, parent) => child === parent || child.startsWith(parent + sep);
  if (!inside(target, root)) return null;
  const tests = join(root, 'tests');
  let entered = false;
  for (const part of path.split('/')) {
    if (part === '..' && !entered && target !== root) {
      target = dirname(target);
    } else if (part !== '..') {
      entered = true;
      target = join(target, part);
      if (isSymlink(target) !== false) return null;
    } else {
      return null;
    }
  }
  const testsStat = lstatSync(tests, { throwIfNoEntry: false });
  const targetStat = lstatSync(target, { throwIfNoEntry: false });
  if (!inside(target, tests) || extname(target) !== '.rs' || !testsStat?.isDirectory()
    || testsStat.isSymbolicLink() || !targetStat?.isFile()) return null;
  try {
    if (realpathSync(target) !== target) return null;
  } catch {
    return null;
  }
  return target;
}

const FIXED = [[0, '#'], [1, '['], [2, 'cfg'], [3, '('], [4, 'test'], [5, ')'], [6, ']'], [7, '#'], [8, '['],
  [9, 'path'], [10, '='], [12, ']'], [13, 'mod'], [15, ';']];

/** Spans of complete top-level `#[cfg(test)] #[path = "..."] mod name;` registrations. */
export function externalTestRegistrations(crateRoot, sourcePath, source) {
  const tokens = sourceTokens(source);
  if (tokens === null) return [];
  const registrations = [];
  tokens.forEach((token, index) => {
    if (token.depth !== 0 || (index > 0 && ![';', '}'].includes(tokens[index - 1].text))) return;
    const item = tokens.slice(index, index + 16);
    if (item.length < 16 || FIXED.some(([offset, expected]) => item[offset].text !== expected)) return;
    if (!/^[_\p{L}][_\p{L}\p{N}]*$/u.test(item[14].text)) return;
    const path = externalTestFile(crateRoot, sourcePath, item[11].text);
    if (path !== null) registrations.push({ start: token.start, end: item[15].end, path });
  });
  return registrations;
}

/** `path:line: text` for every inline test marker outside an external registration. */
export function inlineTestViolations(crateRoot, path, content) {
  const registrations = externalTestRegistrations(crateRoot, path, content);
  const violations = [];
  let offset = 0;
  content.split(/(?<=\n)/u).forEach((chunk, index) => {
    const line = chunk.replace(/[\r\n]+$/u, '');
    const external = registrations.some(({ start, end }) => offset + line.length > start && offset < end);
    if (isInlineTestMarker(line.trimStart()) && !external) violations.push(`${path}:${index + 1}: ${line.trim()}`);
    offset += chunk.length;
  });
  return violations;
}

function rustFiles(directory) {
  const files = [];
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) files.push(...rustFiles(path));
    else if (entry.isFile() && entry.name.endsWith('.rs')) files.push(path);
  }
  return files.sort();
}

/** Every violation under `<crate>/src`, with paths relative to the crate root. */
export function checkCrate(crateRoot = CRATE) {
  const files = rustFiles(join(crateRoot, 'src'));
  const violations = files.flatMap(path => inlineTestViolations(crateRoot, path, readFileSync(path, 'utf8'))
    .map(violation => relative(crateRoot, violation.slice(0, path.length)) + violation.slice(path.length)));
  return { files: files.length, violations };
}

export function main(argv = process.argv.slice(2)) {
  const crateRoot = argv[0] ?? CRATE;
  if (!existsSync(join(crateRoot, 'src'))) {
    process.stderr.write(`check-source-test-placement: ${crateRoot} has no src directory\n`);
    return 2;
  }
  const { files, violations } = checkCrate(crateRoot);
  if (violations.length > 0) {
    process.stderr.write(`production code under src/ must not contain inline tests; move them to tests/ and register them with #[cfg(test)] #[path = "..."] mod name;:\n${violations.join('\n')}\n`);
    return 1;
  }
  process.stdout.write(`source test placement: ${files} files under src/ carry no inline unit tests\n`);
  return 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) process.exitCode = main();
