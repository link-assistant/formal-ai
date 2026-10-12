import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {workspaceDiscoveryContract, workspaceDiscoveryContractBound}
  from '../../../js/agentic/workspace_discovery.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });
const task = 'Inspect the task model. Unconsumed α😀 tail';
function valid() { return workspaceDiscoveryContract(task); }

test('source-bound discovery preserves Unicode residual and Unknown disposition', () => {
  const contract = valid();
  assert.ok(workspaceDiscoveryContractBound(contract, task));
  assert.equal(task.slice(...contract.subjectSpan), 'task model');
  assert.equal(task.slice(...contract.remainingSpan), '. Unconsumed α😀 tail');
  assert.equal(contract.model, 'Unknown');
  assert.equal(contract.schema, 'Unknown');
  assert.equal(contract.fulfilled, false);
});
const mutations = [
  ['foreign source', value => { value.source += 'different'; }],
  ['foreign subject', value => { value.subject = 'queue model'; }],
  ['shifted subject', value => { value.subjectSpan[0]++; }],
  ['reversed subject', value => { value.subjectSpan.reverse(); }],
  ['outside source', value => { value.subjectSpan[1] = task.length + 1; }],
  ['fractional span', value => { value.subjectSpan[0] += 0.5; }],
  ['residual gap', value => { value.remainingSpan[0]++; }],
  ['residual overlap', value => { value.remainingSpan[0]--; }],
  ['cropped residual', value => { value.remainingSpan[1]--; }],
  ['foreign words', value => { value.words[0] = 'queue'; }],
  ['missing words', value => { value.words.pop(); }],
];
for (const [name, mutate] of mutations) {
  test(name + ' cannot bind a discovery contract', () => {
    const contract = valid();
    mutate(contract);
    assert.equal(workspaceDiscoveryContractBound(contract, task), false);
  });
}
test('contracts never bind another live request or malformed source', () => {
  assert.equal(workspaceDiscoveryContractBound(valid(), task + ' changed'), false);
  assert.equal(workspaceDiscoveryContract('Inspect the task model. \ud800'), null);
  assert.equal(workspaceDiscoveryContractBound(null, task), false);
});
