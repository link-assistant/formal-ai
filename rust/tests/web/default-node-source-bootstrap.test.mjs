import test from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { host } from '../../../js/agentic/host.mjs';
import { installDefaultNodeSourceHost } from '../../../js/server/default-node-source-bootstrap.mjs';
test('bootstrap preserves every existing host field and installs one constructor-owned operation', async () => {
  const worker = new WorkerHost();
  await installNodeHost(worker);
  const names = Object.keys(host());
  const types = Object.fromEntries(names.map(k => [k, typeof host()[k]]));
  const realm = await installDefaultNodeSourceHost(worker);
  for (const name of names) assert.equal(typeof host()[name], types[name]);
  assert.equal(host().realm, realm);
  assert.equal(host().sourceOperation, host().sourceSession.sourceOperation);
  assert.equal(host().sourceSession.active(), false);
});
test('external receipt data and copied operation shape provide no source authority', () => {
  const op = host().sourceOperation;
  for (const receipt of [{}, {
    owner: {},
    verified: true
  }, {
    accepted_operation_sources: [],
    status: 0
  }]) assert.throws(() => op.acceptedSources(receipt, 'request', 'command', 'workspace'), /MissingSourceOperationContext/);
  assert.throws(() => ({
    ...op
  }).acceptedSources({}, 'request', 'command', 'workspace'), /MissingSourceOperationContext/);
});
test('unrelated request executes passthrough without acquiring a source proof', async () => {
  let invoked = 0;
  const session = host().sourceSession;
  await session.run({
    request: 'What is two plus two?',
    workspace: '.',
    tools: ['read', 'write', 'bash']
  }, () => {
    invoked++;
    assert.throws(() => host().sourceOperation.acceptedSources({}, 'request', 'command', 'workspace'), /MissingSourceOperationContext/);
  });
  assert.equal(invoked, 1);
  assert.equal(session.active(), false);
});
