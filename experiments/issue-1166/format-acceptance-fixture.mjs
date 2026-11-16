#!/usr/bin/env node
// Re-formats rust/tests/fixtures/issue-1166/acceptance.json so that every value
// whose compact JSON fits in WIDTH columns stays on one line. The data is
// unchanged (asserted below); only the layout shrinks under the repository's
// 1500-line limit for maintained files.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';

const path = process.argv[2] ?? 'rust/tests/fixtures/issue-1166/acceptance.json';
const WIDTH = 110;
const original = JSON.parse(readFileSync(path, 'utf8'));

function format(value, indent) {
  const compact = JSON.stringify(value);
  if (value === null || typeof value !== 'object' || indent.length + compact.length <= WIDTH) return compact;
  const inner = indent + '  ';
  const items = Array.isArray(value)
    ? value.map((item) => inner + format(item, inner))
    : Object.entries(value).map(([key, item]) => `${inner}${JSON.stringify(key)}: ${format(item, inner)}`);
  const [open, close] = Array.isArray(value) ? ['[', ']'] : ['{', '}'];
  return `${open}\n${items.join(',\n')}\n${indent}${close}`;
}

const text = `${format(original, '')}\n`;
assert.deepEqual(JSON.parse(text), original);
writeFileSync(path, text);
console.log(`${path}: ${text.split('\n').length - 1} lines`);
