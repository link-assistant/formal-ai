// Issue #1191: the Self-development status workflow failed on main twice after
// green pull requests, because its debt ratchet ran only on main.
//   run 41317019358 (2026-11-01): literal-predicates: measured 445, ceiling 437
//   run 41804771645 (2026-11-04): language-parity-gaps: measured 52, ceiling 42
// Since f216155d8, the fast pull-request lane compares the current source with
// the committed ceilings (scripts/check-fast-governance.mjs
// --current-debt-ceilings). This test keeps that check in the lane and replays
// both failures through the JavaScript checker it calls.

import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { checkMeasured, parseRatchet } from '../../../scripts/check-debt-ratchet.mjs';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const read = relative => readFileSync(join(ROOT, relative), 'utf8');
const ratchet = parseRatchet(read('data/meta/debt-ratchet.lino'));

/** Every measure exactly at its ceiling, except the ones given. */
function measuredWith(overrides) {
  const measured = new Map(ratchet.ceilings);
  for (const [name, delta] of Object.entries(overrides)) measured.set(name, ratchet.ceilings.get(name) + delta);
  return measured;
}

test('the fast pull-request lane runs the current debt-ceiling check', () => {
  const release = read('.github/workflows/release.yml');
  const start = release.indexOf('\n  fast-pr-lint:\n');
  const job = release.slice(start, release.indexOf('\n  native-type-check:\n', start));
  assert.match(job, /if: \(github\.event_name == 'pull_request'/u, 'fast-pr-lint runs on pull requests');
  assert.match(job, /node scripts\/check-fast-governance\.mjs\n/u);
  const governance = read('scripts/check-fast-governance.mjs');
  assert.match(governance, /name: 'current-debt-ceilings'.*'--current-debt-ceilings'/u);
  assert.match(governance, /import\('\.\/check-debt-ratchet\.mjs'\)/u);
});

test('the committed ledger holds the measures both main runs exceeded', () => {
  for (const name of ['literal-predicates', 'language-parity-gaps']) {
    assert.ok(ratchet.ceilings.has(name), name);
    assert.ok(!ratchet.upward.has(name), `${name} is a ceiling, not an upward floor`);
  }
  assert.deepEqual(checkMeasured(ratchet, measuredWith({})), []);
});

test('replays main run 41317019358: literal predicates above the ceiling fail before merge', () => {
  const failures = checkMeasured(ratchet, measuredWith({ 'literal-predicates': 8 }));
  assert.equal(failures.length, 1);
  assert.match(failures[0], /^literal-predicates: measured \d+, ceiling \d+; move the behaviour into data\/seed/u);
});

test('replays main run 41804771645: language-parity gaps above the ceiling fail before merge', () => {
  const failures = checkMeasured(ratchet, measuredWith({ 'language-parity-gaps': 10 }));
  assert.equal(failures.length, 1);
  assert.match(failures[0], /^language-parity-gaps: measured \d+, ceiling \d+;/u);
});

test('a lower measurement also fails until the ceiling is lowered in the same commit', () => {
  const failures = checkMeasured(ratchet, measuredWith({ 'literal-predicates': -1 }));
  assert.match(failures.join('\n'), /literal-predicates: improved from \d+ to \d+; lower the reviewed ceiling/u);
});
