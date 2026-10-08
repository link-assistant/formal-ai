#!/usr/bin/env node
// JavaScript-first parity ratchet (R998, 2026-10-06 doctrine).
//
// Counts the handler rows the browser registry still leaves to the native
// engine ("native surface only") and holds the count to the ceiling in
// data/meta/js-parity-ratchet.lino. Above the ceiling fails; below it fails
// too, naming the lower ceiling to commit, so the number only falls.
//
// Usage: node scripts/check-js-parity.mjs [--list]

import { readFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const REGISTRY_FILE = 'js/worker/formal_ai_worker_solve.js';
const RATCHET_FILE = 'data/meta/js-parity-ratchet.lino';

/**
 * The registry body: from the definition's opening line to the first
 * closing brace at column zero.
 * @param {string} source
 * @returns {string}
 */
export function registryBody(source) {
  const start = source.indexOf('function workerHandlerRegistryDefinition()');
  if (start < 0) {
    throw new Error('workerHandlerRegistryDefinition not found');
  }
  const end = source.indexOf('\n}', start);
  return source.slice(start, end < 0 ? source.length : end);
}

/**
 * Handler keys the registry leaves to the native engine.
 * @param {string} body
 * @returns {Array<string>}
 */
export function nativeOnlyKeys(body) {
  const keys = [];
  for (const line of body.split('\n')) {
    const match = /^\s*([a-z0-9_]+):\s*null,\s*\/\/\s*native( rule)? surface only/.exec(line);
    if (match) {
      keys.push(match[1]);
    }
  }
  return keys;
}

/**
 * @param {string} lino
 * @returns {number}
 */
export function ceilingFrom(lino) {
  const match = /^\s*native[-_]only[-_]ceiling\s+(\d+)\s*$/m.exec(lino);
  if (!match) {
    throw new Error('native-only-ceiling missing');
  }
  return Number(match[1]);
}

function main(argv) {
  const repo = join(dirname(fileURLToPath(import.meta.url)), '..');
  const keys = nativeOnlyKeys(registryBody(readFileSync(join(repo, REGISTRY_FILE), 'utf8')));
  const ceiling = ceilingFrom(readFileSync(join(repo, RATCHET_FILE), 'utf8'));
  console.log(`native-only handler rows: ${keys.length} (ceiling ${ceiling})`);
  if (argv.includes('--list')) {
    for (const key of keys) {
      console.log(`  ${key}`);
    }
  }
  if (keys.length > ceiling) {
    console.error(
      `::error::${keys.length - ceiling} handler row(s) above the ceiling: implement the JavaScript twin first (R997).`,
    );
    return 1;
  }
  if (keys.length < ceiling) {
    console.error(
      `::error::parity improved: lower native-only-ceiling to ${keys.length} in ${RATCHET_FILE} so it cannot regress.`,
    );
    return 1;
  }
  return 0;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  process.exitCode = main(process.argv.slice(2));
}
