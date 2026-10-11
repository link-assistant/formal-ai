import test from 'node:test';
import assert from 'node:assert/strict';
import {installHost, hasHost, host} from '../../../js/agentic/host.mjs';
import {prepareTrackedRustSourceTransaction, executePreparedTrackedRustSource}
  from '../../../js/server/tracked-source-transaction.mjs';

for (const value of [{}, Object.freeze({}), {phase: 'prepared'}, {capability: {}}, []]) {
  test('caller values cannot own a prepared tracked transaction ' + JSON.stringify(value), async () => {
    await assert.rejects(executePreparedTrackedRustSource(value), /UnownedTrackedSourceTransaction/);
  });
}

test('a forged installed backend cannot issue a genuine physical Read', async () => {
  let effects = 0;
  const operation = Object.fromEntries([
    'prepareTrackedSourceTransaction', 'persistTrackedSourceInverse', 'writeTrackedSourceCandidate',
    'readTrackedSourceCandidate', 'trackedSourceTransactionDisposition', 'abortTrackedSourceTransaction'
  ].map(name => [name, () => { effects++; return {}; }]));
  const original = hasHost() ? host() : null;
  installHost({sourceOperation: operation, sourceSession: {}});
  try {
    await assert.rejects(prepareTrackedRustSourceTransaction({}, 'rename a tracked source', '/tmp/foreign', {}));
    assert.equal(effects, 0);
  } finally { installHost(original); }
});
