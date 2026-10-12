import assert from 'node:assert/strict';
import test from 'node:test';
import { sourceNeedIsRead } from '../../../js/agentic/module_function/source-need-preflight.mjs';
import { observedCallableGoalLedger } from '../../../js/agentic/module_function/discovery.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());

test('only maintained Read Need kinds classify as declared prerequisites', () => {
  for (const kind of ['read-destination', 'read-source']) {
    assert.equal(sourceNeedIsRead({kind}), true);
  }
  for (const kind of ['read-deploy', 'read-write', 'read-', 'Read-source',
    'goal-binding', 'source-effects', 'import-initialization', 'declared-verification', '', null]) {
    assert.equal(sourceNeedIsRead({kind}), false);
  }
  assert.equal(sourceNeedIsRead(null), false);
  assert.equal(sourceNeedIsRead({}), false);
});

test('actual producer classification preserves every unresolved non-Read Need', () => {
  const request = {name: 'caption', parameters: ['item'], destination: 'result.mjs',
    inputs: ['input.mjs'], acceptance: ['gate.mjs'], command: 'node --test gate.mjs'};
  const ledger = observedCallableGoalLedger(request, [{role: 'user', content: 'Add exported caption(item) to result.mjs. Read input.mjs before authoring. Run node --test gate.mjs.'}]);
  assert.deepEqual(ledger.needs.filter(sourceNeedIsRead).map(need => need.kind),
    ['read-destination', 'read-source']);
  assert.deepEqual(ledger.needs.filter(need => !sourceNeedIsRead(need)).map(need => need.kind),
    ['goal-binding', 'source-effects', 'import-initialization', 'declared-verification']);
  assert.equal(ledger.authored, false);
  assert.equal(ledger.verified, false);
  assert.equal(ledger.semantics, 'unbound');
});
