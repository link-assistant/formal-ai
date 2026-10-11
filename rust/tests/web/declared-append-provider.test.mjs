import assert from 'node:assert/strict';
import {mkdtempSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {before, test} from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {appendDefinition, projectAppendContracts} from '../../../js/agentic/append_contract.mjs';
import {composeGeneralChangePlan} from '../../../js/agentic/general_planner.mjs';
import {planGeneralChangeStep} from '../../../js/agentic/general_execution.mjs';
import {atomicRecordAppend} from '../../../experiments/js_dogfood/atomic-record-append.mjs';
before(async () => {await installNodeHost(new WorkerHost());});
const request = 'Create a file note.txt containing exactly: hello';
const tools = ['Bash', 'Write'];
function session() {
  const messages = [{role: 'user', content: request}];
  const plan = composeGeneralChangePlan(request);
  const next = () => planGeneralChangeStep(projectAppendContracts(messages, [appendDefinition('Write')]), tools, plan);
  return {messages, next};
}
function record(messages, call, content, metadata = {}) {
  const id = 'call-' + messages.length;
  messages.push({role: 'assistant', content: '', tool_calls: [{id, type: 'function',
    function: {name: call.tool, arguments: call.arguments}}]});
  messages.push({role: 'tool', name: call.tool, tool_call_id: id, content, ...metadata});
}
test('a declared physical append receipt is preferred over shell and unlocks the useful write', () => {
  const directory = mkdtempSync(join(tmpdir(), 'plan-append-priority-'));
  try {
    const {messages, next} = session();
    const call = next().calls[0];
    assert.equal(call.tool, 'Write');
    const args = JSON.parse(call.arguments);
    assert.equal(args.append_mode, 'atomic_record_append');
    const receipt = atomicRecordAppend(directory, args);
    record(messages, call, receipt.content, receipt);
    const useful = next().calls[0];
    assert.equal(useful.tool, 'Write');
    assert.equal(JSON.parse(useful.arguments).path, 'note.txt');
    record(messages, useful, '');
    const verification = next().calls[0];
    assert.equal(JSON.parse(verification.arguments).command, 'cat note.txt');
  } finally {rmSync(directory, {recursive: true, force: true});}
});
test('empty, forged or failed append receipts never substitute for the declared provider proof', () => {
  for (const metadata of [{}, {append_receipt: {success: true}}, {is_error: true},
    {append_receipt: {contract: 'atomic-record-append/v1', path: 'other.txt'}}]) {
    const {messages, next} = session();
    const call = next().calls[0];
    record(messages, call, '', metadata);
    const result = next();
    assert.equal(result.kind, 'final');
    assert(!result.calls?.some(value => JSON.parse(value.arguments).path === 'note.txt'));
  }
});
test('without a declared append provider the original shell fallback remains available', () => {
  const messages = [{role: 'user', content: request}];
  const plan = composeGeneralChangePlan(request);
  const result = planGeneralChangeStep(messages, tools, plan);
  assert.equal(result.calls[0].tool, 'Bash');
  assert.match(JSON.parse(result.calls[0].arguments).command, /general-change-plan/u);
});
