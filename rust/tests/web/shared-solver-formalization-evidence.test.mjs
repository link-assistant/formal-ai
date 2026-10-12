// Shared formalization evidence and lossless native projection (R1013, R1188-U29).
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { nativeSolverLog } from '../../../js/server/solver-log.mjs';
let host;
before(async () => { host = new WorkerHost(); await installNodeHost(host); });

test('browser translation evidence carries real formalization property and item anchors', async () => {
  const answer = await host.solve('translate apple to Russian');
  assert.equal(answer.intent, 'translate_en_to_ru');
  assert.ok(answer.evidence.includes('formalization:predicate_p:wikidata:P5972'));
  assert.ok(answer.evidence.includes('formalization:object_q:wikidata:Q89'));
  assert.ok(answer.rawSolverEvents.length < answer.solverEvents.length);
});

test('repeat server projection preserves raw events and yields the same complete log', async () => {
  const answer = await host.solve('What is 2 + 2?');
  const raw = JSON.stringify(answer.rawSolverEvents);
  const once = nativeSolverLog(answer);
  const twice = nativeSolverLog({ ...answer, solverEvents: once });
  assert.deepEqual(twice, once);
  assert.equal(JSON.stringify(answer.rawSolverEvents), raw);
  assert.ok(once.some((event) => event.kind === 'problem_frame'));
  assert.ok(once.some((event) => event.kind === 'obligation_ledger'));
});

test('a log without a solver prelude remains unchanged', () => {
  assert.deepEqual(nativeSolverLog({ solverEvents: [{ kind: 'observed', payload: 'data' }] }), [{ kind: 'observed', payload: 'data' }]);
});
