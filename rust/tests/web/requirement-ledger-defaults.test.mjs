import test from 'node:test';
import assert from 'node:assert/strict';
import {readdirSync, readFileSync} from 'node:fs';
import {generated, sharedValue, sharedDefault, renderShard}
  from '../../../scripts/generate-requirement-status.mjs';
import {parseRequirementLedger} from '../../../scripts/lib/requirement-ledger.mjs';

test('minority defaults preserve majority and deterministic source order', () => {
  assert.equal(sharedValue(['a', 'a', 'b']), 'a');
  assert.equal(sharedDefault(['a', 'a', 'b'], 'manual'), 'a');
  assert.equal(sharedDefault(['aa', 'aa', 'bb', 'bb', 'c'], 'manual'), 'aa');
  assert.equal(sharedDefault(['bb', 'bb', 'aa', 'aa', 'c'], 'manual'), 'bb');
  assert.equal(sharedDefault(['', '', 'a'], 'manual'), '');
  assert.throws(() => sharedDefault(['a', 'a'], 'foreign'));
});

test('a minority default retains explicit empty overrides and owned rows', () => {
  const values = ['shared', 'shared', 'shared', '', 'b', 'c', 'd', 'e'];
  const rows = values.map((manual, index) => ({
    id: 'R' + index, shard: 'scope', issue: '1', verdict: 'partial',
    manual, delivered: '', automatedTest: '',
  }));
  const text = renderShard(rows);
  assert.match(text, /    manual ""/u);
  assert.deepEqual(parseRequirementLedger(text).records, rows);
});

test('all maintained generated records retain the complete producer objects', () => {
  const root = new URL('../../../', import.meta.url).pathname;
  const expectedIds = new Set();
  for (const name of readdirSync(root + 'docs/requirements').filter(name => name !== "README.md" && /\.md$/u.test(name))) {
    const text = readFileSync(root + 'docs/requirements/' + name, 'utf8');
    for (const line of text.split('\n')) {
      if (!/^\s*(?:\| R|### R|- R)/u.test(line)) continue;
      const id = line.match(/R[0-9][A-Za-z0-9_-]*/u)?.[0];
      if (id) expectedIds.add(id);
    }
  }
  const actualIds = [];
  let count = 0;
  for (const [path, text] of generated(root)) {
    if (path.endsWith('/requirement-status-ledger.lino')) continue;
    const rows = parseRequirementLedger(text).records;
    count += rows.length;
    actualIds.push(...rows.map(row => row.id));
    assert.equal(new Set(rows.map(row => row.id)).size, rows.length);
  }
  assert.ok(expectedIds.size > 0);
  assert.equal(count, expectedIds.size);
  assert.deepEqual(actualIds.toSorted(), [...expectedIds].toSorted());
});

test('strict-majority verdicts use the notation four-record threshold without losing identities', () => {
  for (const verdict of ['implemented', 'partial', 'not-delivered', 'superseded', 'withdrawn']) {
    const rows = Array.from({length: 4}, (_, index) => ({
      id: 'R' + index, shard: 'scope', issue: '1', verdict,
      manual: 'not yet confirmed', delivered: '', automatedTest: 'rust/tests/owned.rs',
    }));
    const text = renderShard(rows);
    assert.equal((text.match(/^  verdict /gmu) ?? []).length, 1);
    assert.equal((text.match(/^    verdict /gmu) ?? []).length, 0);
    assert.equal((text.match(/^    id /gmu) ?? []).length, 4);
    assert.deepEqual(parseRequirementLedger(text).records, rows);
    for (const size of [0, 1, 2, 3]) {
      const small = renderShard(rows.slice(0, size));
      assert.equal((small.match(/^  verdict /gmu) ?? []).length, 0);
      assert.equal((small.match(/^    verdict /gmu) ?? []).length, size);
    }
    const mixed = rows.map((row, index) => ({...row, verdict: index === 3 ? 'other' : verdict}));
    assert.equal((renderShard(mixed).match(/^  verdict /gmu) ?? []).length, 1);
    assert.equal((renderShard(mixed).match(/^    verdict /gmu) ?? []).length, 1);
    assert.ok(renderShard(mixed).includes('    verdict \"other\"'));
    const tied = rows.map((row, index) => ({...row, verdict: index >= 2 ? 'other' : verdict}));
    assert.equal((renderShard(tied).match(/^  verdict /gmu) ?? []).length, 0);
    assert.equal((renderShard(tied).match(/^    verdict /gmu) ?? []).length, 4);
    assert.deepEqual(parseRequirementLedger(renderShard(tied)).records, tied);
    assert.deepEqual(parseRequirementLedger(renderShard(mixed)).records, mixed);
    const override = text.replace('    id "R1"\n', '    id "R1"\n    verdict "override"\n');
    assert.deepEqual(parseRequirementLedger(override).records,
      rows.map((row, index) => ({...row, verdict: index === 1 ? 'override' : verdict})));
    assert.throws(() => parseRequirementLedger(text.replace('    id "R1"\n', '')));
    assert.throws(() => parseRequirementLedger(text.replace('    id "R1"\n',
      '    id "R1"\n    id "R2"\n')));
  }
});
