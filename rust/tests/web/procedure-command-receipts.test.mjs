// Preserve the original issue674 procedure while observing real bytes and typed operations.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { compileTask, planStep } from '../../../js/agentic/procedure.mjs';
import { artifactLinksNotation } from '../../../js/agentic/crate/skill_procedure_artifact.mjs';
import { conformanceLinksNotation, PROCEDURE_CONFORMANCE_TRIGGER } from '../../../js/agentic/crate/skill_procedure.mjs';
import { procedureWorkspace } from './helpers/procedure-workspace.mjs';
const prompt = 'When I paste a link, fetch its title, translate it to Russian, save both, and reply with the translation.';
let compiled;
before(async () => { await installNodeHost(new WorkerHost()); compiled = compileTask(prompt); assert.ok(compiled); });
function append(messages, call, result, extra = {}) {
  const id = 'procedure-' + messages.length;
  messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
    { role: 'tool', tool_call_id: id, name: call.tool, content: result, ...extra });
}
function replay(transform = raw => raw, outer = {}) {
  const workspace = procedureWorkspace(), messages = [{ role: 'user', content: prompt }], calls = [];
  try {
    for (let turn = 0; turn < 5; turn += 1) {
      const plan = planStep(messages, ['write_file', 'run_command'], compiled);
      if (plan.kind === 'final') return { ...plan, calls, messages };
      assert.equal(plan.calls.length, 1);
      const call = plan.calls[0]; calls.push(call);
      const raw = workspace.execute(call);
      append(messages, call, call.tool === 'run_command' ? transform(raw, JSON.parse(call.arguments).command) : raw,
        call.tool === 'run_command' ? outer : {});
    }
    assert.fail('bounded complete procedure must finish');
  } finally { workspace.close(); }
}
test('original full artifact is physically written, read completely and executed by the library', () => {
  const observed = replay();
  assert.deepEqual(observed.calls.map(call => call.tool), ['write_file', 'run_command', 'run_command']);
  assert.equal(JSON.parse(observed.calls[0].arguments).content, artifactLinksNotation(compiled));
  const read = JSON.parse(observed.messages[4].content);
  assert.equal(read.stdout, artifactLinksNotation(compiled)); assert.equal(read.exit_code, 0);
  const execution = JSON.parse(observed.messages[6].content);
  assert.equal(execution.exit_code, null); assert.equal(execution.operation_success, true);
  assert.equal(execution.stdout, conformanceLinksNotation(compiled, PROCEDURE_CONFORMANCE_TRIGGER));
  assert.ok(observed.answer.includes(execution.stdout));
});
for (const [name, transform] of [
  ['explicit nonzero', raw => JSON.stringify({ ...JSON.parse(raw), exit_code: 7 })],
  ['provider denial', raw => JSON.stringify({ ...JSON.parse(raw), is_error: true })],
  ['incomplete', raw => JSON.stringify({ ...JSON.parse(raw), complete: false })],
  ['truncated', raw => JSON.stringify({ ...JSON.parse(raw), truncated: true })],
  ['timeout', raw => JSON.stringify({ ...JSON.parse(raw), timed_out: true })],
  ['foreign command', raw => JSON.stringify({ ...JSON.parse(raw), command: 'cat foreign.lino' })],
  ['bare actual artifact', raw => JSON.parse(raw).stdout],
  ['operation cannot replace process read', raw => JSON.stringify({ ...JSON.parse(raw), schema: 'procedure-command-receipt/v1', operation_success: true, exit_code: null, error: null })],
]) test('original artifact is not certified with ' + name, () => {
  const observed = replay(transform);
  assert.deepEqual(observed.calls.map(call => call.tool), ['write_file', 'run_command']);
  assert.ok(observed.answer.includes('verification failed')); assert.ok(!observed.answer.includes('executed end to end'));
});
for (const outer of [{ is_error: true }, { name: 'foreign-provider' }]) test('current tool provider veto ' + JSON.stringify(outer), () => {
  assert.ok(replay(raw => raw, outer).answer.includes('verification failed'));
});
for (const patch of [{ operation_success: false }, { exit_code: 0 }, { error: 'denied' }, { complete: false },
  { truncated: true }, { command: 'formal-ai procedure conformance --artifact foreign.lino' }, { stdout: 'wrong' }, { schema: 'foreign-operation/v1' }]) {
  test('exact conformance requires genuine operation boundary ' + JSON.stringify(patch), () => {
    const observed = replay((raw, command) => command.startsWith('cat ') ? raw : JSON.stringify({ ...JSON.parse(raw), ...patch }));
    assert.equal(observed.calls.length, 3); assert.ok(observed.answer.includes('conformance execution failed'));
    assert.ok(!observed.answer.includes('executed end to end'));
  });
}
test('old request and unrelated process calls cannot certify current procedure', () => {
  const complete = replay(), messages = [...complete.messages, { role: 'user', content: prompt }];
  assert.equal(planStep(messages, ['write_file', 'run_command'], compiled).calls[0].tool, 'write_file');
  append(messages, complete.calls[0], '');
  append(messages, { tool: 'run_command', arguments: JSON.stringify({ command: 'cat foreign.lino' }) }, JSON.stringify({ stdout: artifactLinksNotation(compiled), exit_code: 0 }));
  assert.equal(JSON.parse(planStep(messages, ['write_file', 'run_command'], compiled).calls[0].arguments).command, 'cat compiled-procedure.lino');
});
