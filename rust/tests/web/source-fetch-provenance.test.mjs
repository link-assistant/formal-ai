import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import { Progress } from '../../../js/agentic/progress.mjs';
import { sourceFetchObservation } from '../../../js/agentic/source_fetch_observation.mjs';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';

await installNodeHost(new WorkerHost({ WebAssembly: {
  instantiate() { throw Error('LocalProductWasmExecutionForbidden'); },
  compile() { throw Error('LocalProductWasmExecutionForbidden'); },
} }));
const vectors = JSON.parse(fs.readFileSync(new URL('../fixtures/source-fetch-provenance.json', import.meta.url)));
for (const row of vectors) {
  test('source observation: ' + row.id, () => {
    const messages = row.messagesJson ? JSON.parse(row.messagesJson) : row.messages;
    const progress = Progress.scan(messages);
    assert.deepEqual(sourceFetchObservation(messages, row.start, row.url, row.text), row.expected);
    const pair = progress.fetched_pages.find(([url, text]) => url === row.url && text === row.text);
    if (pair) assert.deepEqual(progress.sourceObservationFor(...pair), row.expected);
  });
}
test('actual research terminal carries reported metadata without changing prior acceptance witnesses', async () => {
  const index = JSON.parse(fs.readFileSync(new URL('../fixtures/coding-research/terminal-evidence-index.json', import.meta.url)));
  const rows = index.shards.flatMap(name => JSON.parse(fs.readFileSync(new URL('../fixtures/coding-research/' + name, import.meta.url))).rows);
  const row = rows.find(item => item.phase !== 'tool' && item.expected.research?.verified);
  assert(row);
  const messages = structuredClone(row.messages);
  const fetched = messages.find(message => message.role === 'tool' && message.name === 'web_fetch');
  assert(fetched, 'actual captured fetch receipt');
  const declaration = messages.flatMap(message => message.tool_calls ?? []).find(call => call.id === fetched.tool_call_id);
  const url = JSON.parse(declaration.function.arguments).url;
  fetched.source_capture = { schema: 'formal-ai-source-capture-v1', callId: fetched.tool_call_id,
    url, sha256: sha256Hex(fetched.content), fetchedAt: '1730000000', cached: false,
    trust: { score: 0, policy: 'observed-fixture-policy' }, licence: { name: 'declared-fixture', url: 'https://licence.invalid/terms' } };
  const actual = JSON.parse(JSON.stringify(await planChatStepResolved(messages, row.tools)));
  const observation = actual.research.sourceObservation;
  assert.deepEqual(observation, sourceFetchObservation(messages, 0, url, fetched.content));
  assert.equal(observation.authority, 'Unknown');
  assert.equal(observation.cached.value, false);
  assert.equal(observation.trust.value.score, 0);
  delete actual.research.sourceObservation;
  assert.deepEqual(actual, row.expected);
});
