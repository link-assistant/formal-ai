#!/usr/bin/env node
// A local source-bound next-work plan; this never dispatches work or changes verdicts.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {lstatSync, readdirSync, readFileSync} from 'node:fs';
import {join} from 'node:path';
import {ledgerRecords, levelOf, rootOf} from './render-progressive-plan.mjs';

const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const keyOf = record => JSON.stringify([record.shard, record.id]);
const verdicts = new Set(['implemented', 'not-delivered', 'partial', 'superseded', 'withdrawn']);
const rootOrder = new Map([['javascript', 0], ['rust', 1], ['none', 2]]);

/** Capture the complete current maintained ledger, without status promotion. */
export function captureProgressiveInputs(root) {
  const directory = join(root, 'data/meta/requirement-status-ledger');
  assert(lstatSync(directory).isDirectory() && !lstatSync(directory).isSymbolicLink());
  const names = readdirSync(directory).filter(name => name.endsWith('.lino')).sort();
  assert(names.length > 0, 'complete ledger is required');
  const sources = [];
  const records = [];
  for (const name of names) {
    const path = join(directory, name);
    const stat = lstatSync(path);
    assert(stat.isFile() && !stat.isSymbolicLink(), 'regular ledger source required');
    const bytes = readFileSync(path);
    sources.push({path: 'data/meta/requirement-status-ledger/' + name, sha256: digest(bytes)});
    for (const record of ledgerRecords(bytes.toString('utf8'))) {
      assert(record.id && record.shard && verdicts.has(record.verdict), 'unknown requirement contract');
      records.push(record);
    }
  }
  assert(records.length > 0, 'empty requirement inventory');
  assert.equal(new Set(records.map(keyOf)).size, records.length, 'duplicate requirement identity');
  return {sources, records};
}

/** Missing low-level work remains blocked; a higher level is never substituted. */
export function planProgressiveWork(root, captured, availableKeys) {
  assert(Array.isArray(availableKeys), 'explicit work availability is required');
  assert(availableKeys.every(key => typeof key === 'string'));
  assert.equal(new Set(availableKeys).size, availableKeys.length, 'duplicate available work');
  const fresh = captureProgressiveInputs(root);
  assert.deepEqual(captured, fresh, 'stale progressive source snapshot');
  const known = new Set(fresh.records.map(keyOf));
  assert(availableKeys.every(key => known.has(key)), 'unknown available work');
  const open = fresh.records.filter(record => levelOf(record) > 0 && levelOf(record) < 4);
  const level = open.length ? Math.min(...open.map(levelOf)) : null;
  const cohort = open.filter(record => levelOf(record) === level).sort((left, right) =>
    rootOrder.get(rootOf(left)) - rootOrder.get(rootOf(right)) || keyOf(left).localeCompare(keyOf(right), 'en'));
  const available = new Set(availableKeys);
  const selected = cohort.filter(record => available.has(keyOf(record)));
  const blocked = cohort.filter(record => !available.has(keyOf(record)));
  assert.deepEqual(captureProgressiveInputs(root), fresh, 'ledger changed during selection');
  return {level, selected, blocked, allAtLowestLevel: cohort.length,
    sources: fresh.sources, dispatch: false, verdictsChanged: false};
}
