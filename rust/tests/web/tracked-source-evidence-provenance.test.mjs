import test from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { host, installHost } from '../../../js/agentic/host.mjs';
import { proposeTrackedSourceEvidence } from '../../../js/server/tracked-source-evidence-proposal.mjs';
import { issuePreparedTrackedSourceEffectReceipt } from '../../../js/server/tracked-source-transaction.mjs';
import { executeTrackedSourceEvidence } from '../../../js/server/tracked-source-evidence-consumer.mjs';

test('public observation callbacks cannot mint evidence proposal provenance', async () => {
  await installNodeHost(new WorkerHost());
  const original = host();
  let calls = 0;
  installHost({ ...original, sourceOperation: {
    trackedSourceEffectObservation() {
      calls += 1;
      return Object.freeze({kind: 'OnlyTrackedSourceEffect'});
    }
  }});
  try {
    for (const token of [{}, Object.freeze({}), JSON.parse('{}')]) {
      await assert.rejects(proposeTrackedSourceEvidence(token, 'unowned request', '/absent/workspace'));
      await assert.rejects(executeTrackedSourceEvidence(token, 'unowned request', '/absent/workspace'));
    }
    assert.equal(calls, 0);
    await assert.rejects(issuePreparedTrackedSourceEffectReceipt(Object.freeze({})));
  } finally { installHost(original); }
});
