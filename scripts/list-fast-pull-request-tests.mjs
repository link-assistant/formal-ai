#!/usr/bin/env node
// Explicit offline PR inventory; the full suite stays selected on main.
import assert from 'node:assert/strict';
import {readFileSync,existsSync} from 'node:fs';
export function selectFastTests(manifest, available = existsSync) {
  assert.equal(manifest.schema, 'fast-pr-javascript-tests/v1');
  // Distinct existing paths bound the inventory to the checked-out test tree.
  // A fixed count rejects otherwise valid newly added regression suites; actual
  // execution remains bounded by the measured shard plan and job deadlines.
  assert.ok(Array.isArray(manifest.files) && manifest.files.length > 0);
  assert.equal(new Set(manifest.files).size, manifest.files.length, 'duplicate fast test');
  for (const file of manifest.files) {
    assert.match(file, /^rust\/tests\/web\/[a-zA-Z0-9_-]+[.]test[.]mjs$/u);
    assert.ok(available(file), 'missing fast test: ' + file);
  }
  return [...manifest.files];
}
if (process.argv[1] && import.meta.url === new URL(process.argv[1], 'file:').href) {
  const manifest = JSON.parse(readFileSync('data/benchmarks/fast-pull-request-javascript-tests.json', 'utf8'));
  process.stdout.write(selectFastTests(manifest).join('\n') + '\n');
}
