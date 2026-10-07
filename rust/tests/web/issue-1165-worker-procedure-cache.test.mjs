// Issue #1165 R1165-10: the browser worker's catalog `write_program` arm logs
// the `procedure_cache` event the native `WriteProgram` branch of
// rust/src/solver.rs appends. The worker ships no cache rows (the committed
// data/cache/coding-procedure-cache.lino is empty), so an unmodified catalog
// request is the miss, with `research_missing` read from the miss_route rows of
// data/seed/program-cache-policy.lino; a request that customised the template
// is not answered from the cache and logs no `procedure_cache` event.

import assert from 'node:assert/strict';
import { before, test } from 'node:test';

import { missResearchMissing } from '../../../js/agentic/crate/discovery_production.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

let host;
before(async () => {
  host = new WorkerHost();
  await host.boot();
  await installNodeHost(new WorkerHost());
});

const cacheEvents = (result) => (result.solverEvents || []).filter((event) => event.kind === 'procedure_cache');

test('the worker and the agentic root read the same miss_route gap', () => {
  assert.deepEqual(missResearchMissing(), ['reviewer_approval']);
});

test('an unmodified catalog request logs the exact native procedure_cache miss', async () => {
  const result = await host.solve('Write a hello world program in Python', []);
  assert.equal(result.intent, 'write_program');
  assert.deepEqual(cacheEvents(result), [
    { kind: 'procedure_cache', payload: 'outcome=miss language=python task=hello_world research_missing=reviewer_approval' },
  ]);
  const kinds = result.solverEvents.map((event) => event.kind);
  assert.equal(kinds.indexOf('procedure_cache'), kinds.indexOf('legacy_intent') + 1);
});

test('a request that customised the template logs no procedure_cache event', async () => {
  const result = await host.solve('Write a hello world program in Python and replace "Hello, world!" with "Hi there"', []);
  assert.equal(result.intent, 'write_program');
  assert.deepEqual(cacheEvents(result), []);
});

test('a non-program answer logs no procedure_cache event', async () => {
  assert.deepEqual(cacheEvents(await host.solve('What is 2 + 2?', [])), []);
});
