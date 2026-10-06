#!/usr/bin/env node
// Natural-language literal ratchet for the JavaScript worker (R1010,
// 2026-10-06 meta-algorithm doctrine).
//
// Counts string literals and template chunks in js/worker/*.js that read as
// natural language (three or more words), using the same tokenizer the ts
// twin is rendered with (scripts/translate-es.mjs), so comments never count.
// Wording belongs in seed data (response templates, cue markers), where every
// language can carry it; the code keeps only the reasoning. The totals are
// held to the ceilings in data/meta/js-literal-ratchet.lino: above fails,
// below fails too, naming the lower ceiling to commit, so the numbers only
// fall. The meta reasoner's modules have their own, separate ceiling, and
// the JavaScript server (js/server/*.mjs, R1013) has a ceiling of zero: its
// wording lives in data/meta/server-messages.lino. The agentic planner port
// (js/agentic/**/*.mjs, R1015) is held at zero too: its wording lives in
// data/meta/agentic-messages.lino and the seed response files.
//
// Usage: node scripts/check-js-literals.mjs [--list]

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import { tokenize } from './translate-es.mjs';

const WORKER_DIR = 'js/worker';
const RATCHET_FILE = 'data/meta/js-literal-ratchet.lino';
const META_MODULE = /^formal_ai_worker_meta_/;
const SERVER_DIR = 'js/server';
const AGENTIC_DIR = 'js/agentic';

/** Every `.mjs` file under `dir`, recursively, as repository-relative paths. */
function moduleFiles(repo, dir) {
  return readdirSync(join(repo, dir)).sort().flatMap((name) => {
    const relative = `${dir}/${name}`;
    if (statSync(join(repo, relative)).isDirectory()) return moduleFiles(repo, relative);
    return name.endsWith('.mjs') ? [relative] : [];
  });
}

/**
 * Whether a literal's text reads as natural language: three or more words
 * of two or more letters, and no code punctuation.
 * @param {string} text
 * @returns {boolean}
 */
export function isNaturalLanguage(text) {
  // Generated code ("let value = input;") and identifiers are not wording.
  if (/[(){};=]|\w_\w/u.test(text)) return false;
  return (text.match(/\p{L}{2,}/gu) || []).length >= 3 && /\p{L}{2,}\s+\p{L}{2,}/u.test(text);
}

/**
 * The natural-language literals of a source, in order.
 * @param {string} source
 * @returns {Array<string>}
 */
export function naturalLiterals(source) {
  const out = [];
  const walk = (trees) => {
    for (const tree of trees) {
      if (tree.$ === 'leaf') {
        if (tree.kind === 'string' && isNaturalLanguage(tree.text.slice(1, -1))) out.push(tree.text);
      } else if (tree.$ === 'group') {
        walk(tree.trees);
      } else if (tree.$ === 'template') {
        for (const part of tree.parts) {
          if (part.$ === 'chunk' && isNaturalLanguage(part.text)) out.push(part.text);
          else if (part.$ === 'interp') walk(part.trees);
        }
      }
    }
  };
  walk(tokenize(source));
  return out;
}

/**
 * @param {string} lino
 * @param {string} name
 * @returns {number}
 */
export function ceilingFrom(lino, name) {
  const match = new RegExp(`^\\s*${name}\\s+(\\d+)\\s*$`, 'm').exec(lino);
  if (!match) {
    throw new Error(`${name} missing`);
  }
  return Number(match[1]);
}

function main(argv) {
  const repo = join(dirname(fileURLToPath(import.meta.url)), '..');
  const lino = readFileSync(join(repo, RATCHET_FILE), 'utf8');
  const totals = { worker: 0, meta: 0, server: 0, agentic: 0 };
  const perFile = [];
  for (const name of readdirSync(join(repo, WORKER_DIR)).filter((file) => file.endsWith('.js')).sort()) {
    const count = naturalLiterals(readFileSync(join(repo, WORKER_DIR, name), 'utf8')).length;
    totals[META_MODULE.test(name) ? 'meta' : 'worker'] += count;
    perFile.push([name, count]);
  }
  for (const name of readdirSync(join(repo, SERVER_DIR)).filter((file) => file.endsWith('.mjs')).sort()) {
    const count = naturalLiterals(readFileSync(join(repo, SERVER_DIR, name), 'utf8')).length;
    totals.server += count;
    perFile.push([`server/${name}`, count]);
  }
  for (const relative of moduleFiles(repo, AGENTIC_DIR)) {
    const count = naturalLiterals(readFileSync(join(repo, relative), 'utf8')).length;
    totals.agentic += count;
    perFile.push([relative.slice('js/'.length), count]);
  }
  let status = 0;
  for (const [key, name] of [['worker', 'worker_ceiling'], ['meta', 'meta_ceiling'], ['server', 'server_ceiling'], ['agentic', 'agentic_ceiling']]) {
    const ceiling = ceilingFrom(lino, name);
    console.log(`natural-language literals (${key}): ${totals[key]} (ceiling ${ceiling})`);
    if (totals[key] > ceiling) {
      console.error(`::error::${totals[key] - ceiling} ${key} literal(s) above the ceiling: move the wording into seed data (R1010).`);
      status = 1;
    } else if (totals[key] < ceiling) {
      console.error(`::error::literals fell: lower ${name} to ${totals[key]} in ${RATCHET_FILE} so it cannot regress.`);
      status = 1;
    }
  }
  if (argv.includes('--list')) {
    for (const [name, count] of perFile) {
      if (count) console.log(`  ${name} ${count}`);
    }
  }
  return status;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
