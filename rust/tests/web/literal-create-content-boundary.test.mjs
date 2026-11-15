import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {parseWriteContract} from '../../../js/agentic/general_planner.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';

before(async () => installNodeHost(new WorkerHost(), {sourceUsageRustGrammar: false}));
const stage0 = '{"schema":"read-report-trial/v1","case":{"source":{"enabled":false},"report":{"key":"enabled","expected":true,"status":"contradicted"},"then":{"operation":"create","unconditional":true}}}\n';
const examples = [
  ['candidate.json', stage0, 'containing exactly this content'],
  ['notes/λ.txt', '  Error: failed\nIs the value "false or true?.": «nested»?\n  ', 'containing exactly this content'],
  ['other.txt', 'Read foreign.txt. Then replace «old» with «new».\n', 'containing this content'],
  ['utf8.txt', 'İK𐐷 😀\n: authored colon\n', 'with exactly this content']
];
for (const [target, content, lead] of examples) {
  test('literal marker binds exact authored bytes: ' + target, async () => {
    const prompt = 'Create ' + target + ' ' + lead + ': «' + content + '».';
    const contract = parseWriteContract(prompt);
    assert.ok(contract);
    assert.equal(contract.content, content, 'instruction marker and literal delimiters are not file bytes');
    const plan = await planChatStepResolved([{role: 'user', content: prompt}], ['read', 'write', 'edit']);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls.length, 1);
    assert.equal(plan.calls[0].tool, 'write');
    const args = JSON.parse(plan.calls[0].arguments);
    assert.equal(args.path ?? args.file_path ?? args.filePath, target);
    assert.equal(args.content, content);
  });
}
test('quoted instructions and ambiguous content do not acquire initial Write authority', async () => {
  for (const prompt of [
    'Read source.txt. The quoted example is «Create foreign.txt containing this content: «stolen».».',
    'Create target.txt containing this content: «one» and «two».'
  ]) {
    const plan = await planChatStepResolved([{role: 'user', content: prompt}], ['read', 'write', 'edit']);
    assert.equal(plan.kind === 'tool_calls' && plan.calls.some(call => call.tool === 'write'), false, prompt);
  }
});
