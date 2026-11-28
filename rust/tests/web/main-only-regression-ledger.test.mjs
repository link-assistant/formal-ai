// Issue #1191 acceptance: "a recorded production-only regression fails in its
// narrow pre-merge check". data/meta/main-only-regressions.lino lists every
// failure main recorded after a green pull request. This test keeps that list
// honest: each check exists, runs before merge, and still carries its replay.

import test from 'node:test';
import assert from 'node:assert/strict';
import YAML from 'yaml';
import { existsSync, readFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('../../../', import.meta.url));
const read = relative => readFileSync(join(ROOT, relative), 'utf8');
const KINDS = new Set(['javascript-twin', 'workflow-policy', 'focused-native']);
const FIELDS = new Set(['introduced-by', 'failed', 'workflow', 'check', 'marker']);

/**
 * The ledger as flat entries: `main-run <id>` holds kind groups, a kind holds
 * `regression <id>` blocks, and each field of a block appears at most once.
 */
function parseLedger(text) {
  const entries = [];
  let run = null;
  let kind = null;
  text.split('\n').forEach((line, index) => {
    if (line.trim() === '' || line.startsWith('#')) return;
    const fail = why => { throw new Error(`line ${index + 1}: ${why} ${JSON.stringify(line)}`); };
    let match;
    if ((match = /^main-run (\d+)$/u.exec(line))) {
      [run, kind] = [match[1], null];
    } else if ((match = /^ {2}([a-z-]+)$/u.exec(line))) {
      if (run === null || !KINDS.has(match[1])) fail('unknown kind');
      kind = match[1];
    } else if ((match = /^ {4}regression ([a-z0-9-]+)$/u.exec(line))) {
      if (kind === null) fail('regression outside a kind');
      entries.push({ id: match[1], 'main-run': run, kind });
    } else if ((match = /^ {6}([a-z-]+) (.+)$/u.exec(line)) && FIELDS.has(match[1]) && entries.length > 0) {
      const entry = entries.at(-1);
      if (match[1] in entry) fail(`${entry.id} repeats ${match[1]}`);
      entry[match[1]] = match[2].startsWith('"') ? JSON.parse(match[2]) : match[2];
    } else {
      fail('unreadable');
    }
  });
  return entries;
}

const ledger = parseLedger(read('data/meta/main-only-regressions.lino'));
const inventory = new Set(JSON.parse(read('data/benchmarks/fast-pull-request-javascript-tests.json')).files);

test('every recorded regression is complete and unique', () => {
  assert.ok(ledger.length >= 10, `the ledger keeps the recorded failures (found ${ledger.length})`);
  assert.equal(new Set(ledger.map(entry => entry.id)).size, ledger.length);
  for (const entry of ledger) {
    assert.match(entry['main-run'] ?? '', /^\d{11}$/u, `${entry.id}: a GitHub Actions run id`);
    for (const field of ['introduced-by', 'failed', 'check', 'marker']) assert.ok(entry[field], `${entry.id}: ${field}`);
    assert.ok(KINDS.has(entry.kind), `${entry.id}: kind ${entry.kind}`);
    // Only the focused native check names a workflow, and it must.
    assert.equal(entry.workflow !== undefined, entry.kind === 'focused-native', `${entry.id}: workflow`);
  }
});

test('each check exists, runs before merge and still carries the replay', () => {
  const nativeJob = YAML.parse(read('.github/workflows/release.yml')).jobs['native-type-check'];
  for (const entry of ledger) {
    assert.ok(existsSync(join(ROOT, entry.check)), `${entry.id}: ${entry.check} exists`);
    assert.ok(inventory.has(entry.check), `${entry.id}: ${entry.check} runs in the fast pull-request inventory`);
    assert.ok(read(entry.check).includes(entry.marker), `${entry.id}: ${entry.check} still carries ${JSON.stringify(entry.marker)}`);
    if (entry.workflow !== undefined) {
      assert.ok(existsSync(join(ROOT, entry.workflow)), `${entry.id}: ${entry.workflow} exists`);
      // release.yml calls the reusable native check on pull requests (and only off main).
      assert.equal(nativeJob.uses, `./${entry.workflow}`, `${entry.id}: release.yml calls ${entry.workflow}`);
      assert.ok(nativeJob.needs.includes('fast-pr-lint'));
      for (const condition of ["!cancelled()", "needs.fast-pr-lint.result == 'success'",
        "needs.fast-pr-lint.outputs.tested-sha == github.sha", "github.event_name == 'pull_request'",
        "github.ref != 'refs/heads/main'"]) assert.ok(nativeJob.if.includes(condition), condition);
    }
  }
});

test('both main runs of the issue are covered, each by every failure it reported', () => {
  const byRun = run => ledger.filter(entry => entry['main-run'] === run).map(entry => entry.id).sort();
  assert.deepEqual(byRun('43710854562'), ['rust-source-e0308', 'tests-as-documentation-loose-only']);
  assert.deepEqual(byRun('43730270231'), [
    'clippy-manual-contains', 'inline-tests-work-item-leads', 'issue-698-planner-expectation',
    'module-map-standing-scope', 'top-level-examples', 'work-item-read-normalized-payload',
  ]);
});

test('the parser refuses an unknown field or kind and a repeated field', () => {
  const block = body => `main-run 1\n  javascript-twin\n    regression a\n${body}`;
  assert.throws(() => parseLedger(block('      colour "red"\n')), /unreadable/u);
  assert.throws(() => parseLedger(block('      check x\n      check y\n')), /repeats check/u);
  assert.throws(() => parseLedger('main-run 1\n  native\n'), /unknown kind/u);
  assert.deepEqual(parseLedger(`# c\n${block('      check x\n      marker "m n"\n')}`),
    [{ id: 'a', 'main-run': '1', kind: 'javascript-twin', check: 'x', marker: 'm n' }]);
});
