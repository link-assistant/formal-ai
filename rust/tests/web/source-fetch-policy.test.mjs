import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import * as policy from '../../../js/agentic/portable/coding_research_kernel.mjs';
import { sourceFetchObservation } from '../../../js/agentic/source_fetch_observation.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost({
  WebAssembly: {
    instantiate() {
      throw Error('LocalProductWasmExecutionForbidden');
    },
    compile() {
      throw Error('LocalProductWasmExecutionForbidden');
    }
  }
}));
const fixture = JSON.parse(fs.readFileSync(new URL('../fixtures/source-fetch-policy.json', import.meta.url)));
for (const row of fixture.policyCases) test('canonical captured-source policy ' + row.id, () => assert.equal(policy[row.function](...row.arguments), row.expected));
for (const row of fixture.observerCases) test('held-out production observer ' + row.id, () => assert.deepEqual(sourceFetchObservation(row.messages, row.start, row.url, row.text), row.expected));
test('numeric host observations cannot promote NaN or infinity', () => {
  for (const score of [NaN, Infinity, -Infinity]) assert.equal(policy.sourceCaptureTrustAccepted(true, 2, true, score, true, 1), false);
});
