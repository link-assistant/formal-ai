// Coding requirements can mention plural nouns without requesting a shell listing.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

test('plural nouns ending in ls preserve the explicitly named source read', async () => {
  for (const noun of ['calls', 'details', 'signals', 'goals']) {
    const prompt = 'Read alpha.mjs and inspect ' + noun + ' with their supplied context.';
    const plan = await planChatStep([{ role: 'user', content: prompt }], ['read', 'bash']);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls.length, 1);
    assert.equal(plan.calls[0].tool, 'read', noun);
    assert.equal(JSON.parse(plan.calls[0].arguments).filePath, 'alpha.mjs');
  }
});

test('the standalone ls command still lists before reading its selected file', async () => {
  const plan = await planChatStep([{ role: 'user', content: 'ls then read the first one alphabetically' }], ['read', 'bash']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls[0].tool, 'bash');
  assert.match(JSON.parse(plan.calls[0].arguments).command, /^find \. -maxdepth 1 -type f/);
});
