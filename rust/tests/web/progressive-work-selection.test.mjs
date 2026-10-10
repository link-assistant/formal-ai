import assert from 'node:assert/strict';
import test from 'node:test';
import fs from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {captureProgressiveInputs, planProgressiveWork} from '../../../scripts/select-progressive-work.mjs';
const key = record => JSON.stringify([record.shard, record.id]);
const text = records => 'ledger\n  shard "docs/requirements/fixture.md"\n' + records.map(record =>
  '  requirement\n    id "' + record.id + '"\n    verdict "' + record.verdict + '"\n    automated_test "' + record.test + '"\n').join('');
function fixture(records, run) {
  const root = fs.mkdtempSync(join(tmpdir(), 'progressive-selection-'));
  const directory = join(root, 'data/meta/requirement-status-ledger');
  fs.mkdirSync(directory, {recursive: true});
  const file = join(directory, 'fixture.lino');
  fs.writeFileSync(file, text(records));
  try {return run(root, file, captureProgressiveInputs(root));}
  finally {fs.rmSync(root, {recursive: true, force: true});}
}
const records = [
  {id: 'R1', verdict: 'not-delivered', test: 'rust/tests/unit/a.rs'},
  {id: 'R2', verdict: 'not-delivered', test: 'rust/tests/web/a.test.mjs'},
  {id: 'R3', verdict: 'partial', test: 'rust/tests/web/b.test.mjs'},
  {id: 'R4', verdict: 'implemented', test: 'rust/tests/web/c.test.mjs'},
  {id: 'R5', verdict: 'superseded', test: ''},
];
test('lowest open cohort is JavaScript first and preserves every original record', () => fixture(records, (root, file, captured) => {
  const before = fs.readFileSync(file);
  const plan = planProgressiveWork(root, captured, captured.records.map(key));
  assert.equal(plan.level, 2);
  assert.deepEqual(plan.selected.map(record => record.id), ['R2', 'R1']);
  assert.equal(plan.allAtLowestLevel, 2);
  assert.deepEqual(fs.readFileSync(file), before);
  assert.equal(plan.dispatch, false); assert.equal(plan.verdictsChanged, false);
}));
test('blocked lowest requirements never substitute higher JavaScript work', () => fixture(records, (root, file, captured) => {
  const plan = planProgressiveWork(root, captured, captured.records.filter(record => record.id === 'R3').map(key));
  assert.equal(plan.level, 2); assert.deepEqual(plan.selected, []);
  assert.deepEqual(plan.blocked.map(record => record.id), ['R2', 'R1']);
}));
test('unmeasured requirement precedes measured and partial work', () => fixture([...records, {id: 'R6', verdict: 'not-delivered', test: ''}], (root, file, captured) => {
  assert.deepEqual(planProgressiveWork(root, captured, captured.records.map(key)).selected.map(record => record.id), ['R6']);
}));
test('no open requirement yields no invented task', () => fixture(records.slice(3), (root, file, captured) => {
  const plan = planProgressiveWork(root, captured, captured.records.map(key));
  assert.equal(plan.level, null); assert.deepEqual(plan.selected, []); assert.deepEqual(plan.blocked, []);
}));
test('changed source bytes, added shard and forged snapshot all refuse', () => {
  for (const mutate of [(root, file) => fs.appendFileSync(file, '\n'),
    root => fs.writeFileSync(join(root, 'data/meta/requirement-status-ledger/new.lino'), text([{id: 'R9', verdict: 'partial', test: ''}]))]) {
    fixture(records, (root, file, captured) => {mutate(root, file); assert.throws(() => planProgressiveWork(root, captured, []), /stale/);});
  }
  fixture(records, (root, file, captured) => {captured.records[0].verdict = 'implemented'; assert.throws(() => planProgressiveWork(root, captured, []), /stale/);});
});
test('unknown and duplicate availability refuse', () => fixture(records, (root, file, captured) => {
  assert.throws(() => planProgressiveWork(root, captured, ['unknown']), /unknown/);
  const identity = key(captured.records[0]);
  assert.throws(() => planProgressiveWork(root, captured, [identity, identity]), /duplicate/);
}));
test('unknown verdict and duplicate identity refuse rather than hide low work', () => {
  for (const invalid of [[...records, {...records[0]}], [{id: 'R9', verdict: 'unreviewed', test: ''}]]) {
    const root = fs.mkdtempSync(join(tmpdir(), 'progressive-refusal-'));
    const directory = join(root, 'data/meta/requirement-status-ledger'); fs.mkdirSync(directory, {recursive: true});
    fs.writeFileSync(join(directory, 'fixture.lino'), text(invalid));
    try {assert.throws(() => captureProgressiveInputs(root), /unknown|duplicate/);}
    finally {fs.rmSync(root, {recursive: true, force: true});}
  }
});
