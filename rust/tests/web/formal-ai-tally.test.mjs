// R1188-U13: the tally of the tasks delegated to Formal AI is generated from
// the ladder of docs/case-studies/pull-request-1188/formal-ai-dogfood.md, and
// every failure is resolved, with its regression test named by the row or
// citing the row's id (scripts/tally-formal-ai-dogfood.mjs).

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

import {
  agentOf,
  ladderRows,
  outcomeOf,
  renderTally,
  resolutionOf,
  tally,
} from '../../../scripts/tally-formal-ai-dogfood.mjs';
import { REPO_ROOT } from './support/browser-runtime.mjs';

const LADDER = [
  '| # | Task (prompt) | Before | After |',
  '| --- | --- | --- | --- |',
  '| T1 | `Append x` | **Fail, destructive**: overwrote it. | **Pass**: pinned in `a.test.mjs`. |',
  '| T2 | REQ-ROUTE: insert a row | **Pass** | No change needed. |',
  '| T3 | G12: a gap probe | **Fail**: no plan. | Open: G12. |',
  '| T4 | `Run x` | **Fail**: ran `x.`. | **Pass**: runs `x`. |',
  '| T5 | ROUTE2: a nested quote | **Fail**: wrong plan. | Rephrased and passed; recorded as part of G53. |',
].join('\n');

test('ladder rows are read with their four cells', () => {
  const rows = ladderRows(LADDER);
  assert.equal(rows.length, 5);
  assert.deepEqual(rows[1], { id: 'T2', task: 'REQ-ROUTE: insert a row', before: '**Pass**', after: 'No change needed.' });
});

test('outcomes, resolutions and agents are read from the cells', () => {
  assert.equal(outcomeOf('**Fail, unsafe**: ran it'), 'failed');
  assert.equal(outcomeOf('Pass: done'), 'passed');
  assert.equal(outcomeOf('**Partial fail**'), 'partial');
  assert.equal(resolutionOf('**Fixed in T530** (G94).'), 'fixed');
  assert.equal(resolutionOf('**Safe**: cues outside quotes only'), 'fixed');
  assert.equal(resolutionOf('Open (TEACH-D owns it)'), 'open');
  assert.equal(resolutionOf('Not reproduced on a re-run'), 'not-reproduced');
  assert.equal(resolutionOf('Rephrased; recorded as part of G53.'), 'open');
  assert.equal(agentOf('REQ-ROUTE: insert'), 'REQ-ROUTE');
  assert.equal(agentOf('G12: a gap probe'), 'coordinator');
  assert.equal(agentOf('`Append x`'), 'coordinator');
});

test('a fixed failure needs a named test, a cited id or a carrying row', () => {
  const rows = ladderRows(LADDER);
  const uncited = tally(rows);
  assert.deepEqual(uncited.fixedWithoutTest, ['T4']);
  assert.equal(uncited.total.failed, 4);
  assert.equal(uncited.total.fixed, 2);
  assert.equal(uncited.total.open, 2);
  assert.deepEqual(tally(rows, new Set(['T4'])).fixedWithoutTest, []);
  assert.match(renderTally(uncited), /\| \*\*All\*\* \| 5 \| 1 \| 4 \| 0 \| 2 \| 2 \| 0 \| 0 \|/);
});

test('the repository tally is current and every failure is resolved', () => {
  const output = execFileSync('node', ['scripts/tally-formal-ai-dogfood.mjs', '--check'], { cwd: REPO_ROOT, encoding: 'utf8' });
  assert.match(output, /formal-ai tally: \d+ tasks/);
});
