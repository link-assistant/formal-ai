#!/usr/bin/env node
// Generalize, don't specialize (R1188-U1).
//
// A fix is the smallest universal rule that covers a class of requests, never
// a branch for one prompt. This gate notices the plainest form of a
// specialization: a prompt that a test sends to Formal AI (a literal of at
// least four words passed to `solve(`, `drive(`, `prompt:` and the like)
// that appears verbatim in code under rust/src/ or js/ outside comments. Such
// a string in code is a branch, a cue or a canned answer keyed to that one
// prompt; the general form lives in seed data.
//
// The count of (source file, prompt) pairs is held to the ceiling in
// data/meta/prompt-specialization-ratchet.lino, which only falls.
//
// Usage: node scripts/check-prompt-specialization.mjs [--list]

import { execFileSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { renderBootstrap, SOURCE as RESPONSE_SEED } from './generate-worker-bootstrap.mjs';

const RATCHET = 'data/meta/prompt-specialization-ratchet.lino';
const TEST_ROOTS = ['rust/tests/web', 'rust/tests/unit'];
const SOURCE_ROOTS = ['rust/src', 'js'];
const MINIMUM_WORDS = 4;

/** A string literal passed to a call or field that carries a prompt. */
const PROMPT_LITERAL = new RegExp(
  String.raw`(?:\b(?:solve|answer|ask|drive|chat|respond|reply|planChatStep|plan_chat_step|run_prompt|solve_prompt|prompt|user)\s*\(\s*(?:&|\[\s*)?` +
    String.raw`|\b(?:prompt|content|question)\s*[:=]\s*)` +
    String.raw`(?:"((?:[^"\\\n]|\\.){12,200})"|'((?:[^'\\\n]|\\.){12,200})')`,
  'gu',
);

/** Built output, fixtures and seed mirrors are not hand-written code. */
const NOT_CODE = /(?:^|\/)(?:tests?|fixtures|vendor|dist|seed)\/|\.bundle\.js$|docs\/docs\.js$/u;

/**
 * The prompts a test file sends: literals of at least four words that read as
 * natural language, not code.
 * @param {string} source
 * @returns {Array<string>}
 */
export function promptsOf(source) {
  const prompts = [];
  for (const match of source.matchAll(PROMPT_LITERAL)) {
    const text = (match[1] ?? match[2]).trim();
    const words = text.split(/\s+/u).length;
    const readsAsCode = /[{}$\\]|::|=>|\.(?:rs|mjs|js|lino|md)\b/u.test(text);
    if (words >= MINIMUM_WORDS && /\p{L}/u.test(text) && !readsAsCode) {
      prompts.push(text);
    }
  }
  return prompts;
}

/**
 * Source text without its comment lines.
 * @param {string} source
 * @returns {string}
 */
export function codeLines(source) {
  return source
    .split('\n')
    .filter((line) => !/^\s*(?:\/\/|\*|\/\*|#)/u.test(line))
    .join('\n');
}

/**
 * The (source file, prompt) pairs where code holds a test's prompt verbatim.
 * @param {Array<string>} prompts
 * @param {Array<{path: string, code: string, source?: string}>} sources
 * @param {Array<string>} seedProjections Complete canonical data rendered by checked generators.
 * @returns {Array<{path: string, prompt: string}>}
 */
export function specializations(prompts, sources, seedProjections = []) {
  // Only complete byte-equal canonical data is exempt; headers and paths are insufficient.
  const projectedData = new Set(seedProjections);
  const found = [];
  for (const prompt of [...new Set(prompts)].sort()) {
    for (const { path, code, source } of sources) {
      if (typeof source === "string" && projectedData.has(source)) continue;
      if (code.includes(prompt)) {
        found.push({ path, prompt });
      }
    }
  }
  return found;
}

function ceilingOf(text) {
  const match = /^\s*specialization[-_]ceiling\s+(\d+)\s*$/mu.exec(text);
  if (!match) {
    throw new Error(`${RATCHET} names no specialization-ceiling`);
  }
  return Number(match[1]);
}

function main(argv) {
  const root = join(dirname(fileURLToPath(import.meta.url)), '..');
  const listed = (roots, extensions = /\.(?:rs|mjs|js)$/u) =>
    execFileSync('git', ['ls-files', '--cached', '--others', '--exclude-standard', ...roots], {
      cwd: root,
      encoding: 'utf8',
      maxBuffer: 1 << 28,
    })
      .split('\n')
      .filter((path) => extensions.test(path) && existsSync(join(root, path)));
  const read = (path) => readFileSync(join(root, path), 'utf8');
  const prompts = listed(TEST_ROOTS).flatMap((path) => promptsOf(read(path)));
  // Source code includes the browser app's JSX: its local fallbacks and
  // suggestion lists answer prompts too.
  const sources = listed(SOURCE_ROOTS, /\.(?:rs|mjs|js|jsx)$/u)
    .filter((path) => !NOT_CODE.test(path))
    .map((path) => {
      const source = read(path);
      return { path, source, code: codeLines(source) };
    });
  const seedData = renderBootstrap(read('js/seed_loader.js'), read(RESPONSE_SEED));
  const found = specializations(prompts, sources, [seedData]);
  const ceiling = ceilingOf(read(RATCHET));
  console.log(`test prompts held verbatim in code: ${found.length} (ceiling ${ceiling})`);
  if (argv.includes('--list')) {
    for (const { path, prompt } of found) {
      console.log(`  ${path}: ${prompt}`);
    }
  }
  if (found.length > ceiling) {
    console.error(
      `::error file=${RATCHET}::${found.length - ceiling} new test prompt(s) held verbatim in code: ` +
        'answer the class through seed data instead of the one prompt (R1188-U1). Run with --list.',
    );
    return 1;
  }
  if (found.length < ceiling) {
    console.error(`::error file=${RATCHET}::specializations fell to ${found.length}; lower specialization-ceiling in this commit.`);
    return 1;
  }
  return 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
