// Retrieval text is modeled with the selected gh tool's exact result envelope.
// Auxiliary observations come from actual bounded filesystem tools.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, existsSync, readdirSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
import {composeGeneralChangePlan,planLinksNotation} from '../../../js/agentic/general_planner.mjs';
import { finalResult } from '../../../js/agentic/final_result.mjs';
import { executeResult } from '../../../experiments/js_dogfood/drive.mjs';
const task = 'Solve https://github.com/link-assistant/formal-ai/issues/1069 in this checkout as one whole task.';
const prose = 'Produce a legitimate release with no fabricated evidence.\nAdding a bypass flag to check-self-development-release.rs is not acceptable.';
const tools = ['web_fetch', 'write_file', 'bash'];
const auxiliaryTask = 'Create observation.txt containing exactly "auxiliary evidence".';
before(async () => { await installNodeHost(new WorkerHost()); });
function append(messages, call, observation) {
  const id = 'actual-' + messages.length;
  messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
    { role: 'tool', name: call.tool, tool_call_id: id, ...observation });
}
async function fetched(selectedTools = tools, selectedTool = 'bash') {
  const messages = [{ role: 'user', content: task }];
  const plan = await planChatStepResolved(messages, selectedTools);
  assert.equal(plan.kind, 'tool_calls'); assert.equal(plan.calls.length, 1);
  assert.equal(plan.calls[0].tool, selectedTool);
  const command = JSON.parse(plan.calls[0].arguments).command;
  const issue = "'https://github.com/link-assistant/formal-ai/issues/1069'";
  assert.ok(command.startsWith('gh issue view ' + issue + ' --json title --jq .title && echo && gh issue view ' + issue + ' --json body --jq .body; printf '));
  assert.equal(command.slice(command.indexOf('; printf ')), "; printf '" + String.fromCharCode(10) + "__formal_ai_exit=%s" + String.fromCharCode(10) + "' $?");
  // Bind the unchanged issue body to that selected retrieval, including its status marker.
  append(messages, plan.calls[0], { content: 'Output: Issue #1069\n\n' + prose + '\n__formal_ai_exit=0\nExit Code: 0' });
  const result = await planChatStepResolved(messages, selectedTools);
  assert.equal(messages[0].content, task);
  assert.ok(messages.at(-1).content.includes(prose));
  return { messages, result };
}
async function auxiliary() {
  const messages = [{ role: 'user', content: auxiliaryTask }];
  const records = await planChatStepResolved(messages, tools);
  assert.equal(records.kind, 'tool_calls'); assert.equal(records.calls.length, 1);
  assert.equal(records.calls[0].tool, 'bash');
  assert.ok(JSON.parse(records.calls[0].arguments).command.startsWith('mkdir -p -- .formal-ai && (lock='));
  assert.ok(records.calls[0].arguments.includes('.formal-ai/general-change-plan.lino'));
  return { messages, records };
}
for (const observation of [{ content: 'wrote the plan' }, { content: JSON.stringify({ stdout: '', stderr: 'denied', exit_code: 1 }) }, { content: 'denied', is_error: true }]) {
  test('supported literal task: unobserved or failed auxiliary receipt cannot authorize a literal write: ' + JSON.stringify(observation), async () => {
    const { messages, records } = await auxiliary(); append(messages, records.calls[0], observation);
    const plan = await planChatStepResolved(messages, tools);
    assert.equal(plan.kind, 'final'); assert.ok(!plan.answer.includes('Planned, not executed'));
    assert.notEqual(finalResult(plan).disposition, 'finding'); assert.ok(!plan.answer.includes('Executed'));
    assert.equal(plan.calls, undefined);
  });
}
test('original fetched task and two statements retain an honest gap with no journal or coding effects', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'fetched-auxiliary-'));
  try {
    const { result } = await fetched();
    assert.equal(result.kind, 'final');
    const evidence = JSON.parse(result.answer);
    assert.equal(evidence.schema, 'repository-work-item-evidence/v1');
    assert.equal(evidence.reason, 'MissingReadTool');
    assert.equal(evidence.authored, false); assert.equal(evidence.verified, false);
    assert.equal(evidence.decomposition.find(step => step.step === 'implement_requested_change').status, 'not_executed');
    assert.equal(finalResult(result).disposition, 'gap'); assert.equal(result.calls, undefined);
    assert.ok(result.answer.includes('Produce a legitimate release with no fabricated evidence.'));
    assert.ok(result.answer.includes('Adding a bypass flag to check-self-development-release.rs is not acceptable.'));
    assert.equal(existsSync(join(dir, '.formal-ai/general-change-plan.lino')), false);
    assert.equal(existsSync(join(dir, 'check-self-development-release.rs')), false);
    assert.deepEqual(readdirSync(dir), []);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
test('supported literal task advances only after actual auxiliary append and readback observations', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'literal-auxiliary-'));
  try {
    const { messages, records } = await auxiliary(); let pending = records, result;
    for (let turn = 0; turn < 4; turn += 1) {
      for (const call of pending.calls) {
        assert.equal(call.tool, 'bash'); const { command } = JSON.parse(call.arguments);
        assert.ok(command.startsWith('mkdir -p -- .formal-ai && (lock=') || command === 'cat .formal-ai/general-change-plan.lino');
        assert.ok(command.includes('.formal-ai/general-change-plan.lino'));
        append(messages, call, executeResult(dir, call));
      }
      result = await planChatStepResolved(messages, tools);
      if (result.kind === 'tool_calls' && result.calls.some(call => call.tool === 'write_file')) break;
      assert.equal(result.kind, 'tool_calls'); pending = result;
    }
    assert.equal(result.kind, 'tool_calls'); assert.equal(result.calls[0].tool, 'write_file');
    const write = JSON.parse(result.calls[0].arguments); assert.equal(write.content, 'auxiliary evidence');
    assert.ok(readFileSync(join(dir, '.formal-ai/general-change-plan.lino'), 'utf8').endsWith(planLinksNotation(composeGeneralChangePlan(auxiliaryTask))));
    assert.equal(existsSync(join(dir, 'observation.txt')), false);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
test('legacy run_command retrieval does not authorize a shell journal recipe for the original issue', async () => {
  const legacyTools = ['web_fetch', 'write_file', 'run_command'];
  const { result } = await fetched(legacyTools, 'run_command');
  assert.equal(result.kind, 'final'); assert.equal(finalResult(result).disposition, 'gap');
  const evidence = JSON.parse(result.answer);
  assert.equal(evidence.reason, 'MissingReadTool');
  assert.equal(evidence.authored, false); assert.equal(evidence.verified, false);
  assert.ok(!result.answer.includes('Executed')); assert.equal(result.calls, undefined);
});
