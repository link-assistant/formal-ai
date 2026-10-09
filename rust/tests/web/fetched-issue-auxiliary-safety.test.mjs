// Fetch text is modeled; auxiliary observations come from actual bounded filesystem tools.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { finalResult } from '../../../js/agentic/final_result.mjs';
import { executeResult } from '../../../experiments/js_dogfood/drive.mjs';
const task = 'Solve https://github.com/link-assistant/formal-ai/issues/1069 in this checkout as one whole task.';
const prose = 'Produce a legitimate release with no fabricated evidence.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.';
const tools = ['web_fetch', 'write_file', 'run_command'];
before(async () => { await installNodeHost(new WorkerHost()); });
function append(messages, call, observation) {
  const id = 'actual-' + messages.length;
  messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
    { role: 'tool', name: call.tool, tool_call_id: id, ...observation });
}
async function fetched() {
  const messages = [{ role: 'user', content: task }];
  const plan = await planChatStepResolved(messages, tools);
  assert.equal(plan.kind, 'tool_calls'); assert.equal(plan.calls[0].tool, 'web_fetch');
  append(messages, plan.calls[0], { content: prose });
  const records = await planChatStepResolved(messages, tools);
  assert.equal(records.kind, 'tool_calls');
  assert.ok(records.calls[0].arguments.includes('.formal-ai/general-change-plan.lino'));
  return { messages, records };
}
for (const observation of [{ content: 'wrote the plan' }, { content: JSON.stringify({ stdout: '', stderr: 'denied', exit_code: 1 }) }, { content: 'denied', is_error: true }]) {
  test('unobserved or failed auxiliary receipt cannot authorize a literal write: ' + JSON.stringify(observation), async () => {
    const { messages, records } = await fetched(); append(messages, records.calls[0], observation);
    const plan = await planChatStepResolved(messages, tools);
    assert.equal(plan.kind, 'final'); assert.ok(!plan.answer.includes('Planned, not executed'));
    assert.notEqual(finalResult(plan).disposition, 'finding'); assert.ok(!plan.answer.includes('Executed'));
  });
}
test('original fetched two statements retain Planned not executed after actual auxiliary observations', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'fetched-auxiliary-'));
  try {
    const { messages, records } = await fetched(); let pending = records, result;
    for (let turn = 0; turn < 4; turn += 1) {
      for (const call of pending.calls) {
        assert.equal(call.tool, 'run_command'); const { command } = JSON.parse(call.arguments);
        assert.ok(command.startsWith('mkdir -p -- .formal-ai && (lock=') || command === 'cat .formal-ai/general-change-plan.lino');
        assert.ok(command.includes('.formal-ai/general-change-plan.lino'));
        append(messages, call, executeResult(dir, { ...call, tool: 'bash' }));
      }
      result = await planChatStepResolved(messages, tools); if (result.kind === 'final') break; pending = result;
    }
    assert.equal(result.kind, 'final'); assert.ok(result.answer.includes('Planned, not executed'));
    assert.ok(readFileSync(join(dir, '.formal-ai/general-change-plan.lino'), 'utf8').includes(task));
    assert.equal(existsSync(join(dir, 'check-self-development-release.rs')), false);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
