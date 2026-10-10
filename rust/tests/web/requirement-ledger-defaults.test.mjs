import test from 'node:test';
import assert from 'node:assert/strict';
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
  let count = 0;
  for (const [path, text] of generated(root)) {
    if (path.endsWith('/requirement-status-ledger.lino')) continue;
    const rows = parseRequirementLedger(text).records;
    count += rows.length;
    assert.equal(new Set(rows.map(row => row.id)).size, rows.length);
  }
  assert.equal(count, 1374);
});
