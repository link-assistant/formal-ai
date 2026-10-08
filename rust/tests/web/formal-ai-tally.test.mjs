// R1188-U13: the tally of the tasks delegated to Formal AI is generated from
// the ladder of docs/case-studies/pull-request-1188/formal-ai-dogfood.md, and
// every failure is resolved, with its regression test named by the row or
// citing the row's id (scripts/tally-formal-ai-dogfood.mjs).

import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import test from 'node:test';

import {
  agentOf,
  handRows,
  ladderRows,
  outcomeOf,
  renderHandEdits,
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

test('R1188-U13: edits by hand are read per agent and set beside Formal AI\'s', () => {
  const rows = handRows([
    '| # | Who | Round | By Formal AI | By hand | Why by hand |',
    '| H1 | RENAME | 19 | 9 | 1 | New tool. |',
    '| H2 | LEAD | 19 | 1 | 3 | Workflow splits. |',
    '| H3 | LEAD | 20 | 2 | 0 | — |',
  ].join('\n'));
  assert.deepEqual(rows.map((row) => [row.id, row.agent, row.delegated, row.byHand]), [
    ['H1', 'RENAME', 9, 1],
    ['H2', 'LEAD', 1, 3],
    ['H3', 'LEAD', 2, 0],
  ]);
  const page = renderHandEdits(rows).join('\n');
  assert.match(page, /\| \*\*All\*\* \| 12 \| 4 \| 75% \|/u);
  assert.match(page, /\| LEAD \| 3 \| 3 \| 50% \|/u);
  assert.match(page, /\| RENAME \| 9 \| 1 \| 90% \|/u);
  assert.deepEqual(renderHandEdits([]), []);
});
