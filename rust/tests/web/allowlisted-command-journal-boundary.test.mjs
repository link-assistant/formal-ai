import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {mkdtempSync, readFileSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {execFileSync} from 'node:child_process';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {composeGeneralChangePlan, PLAN_PATH} from '../../../js/agentic/general_planner.mjs';
import {planGeneralChangeStep} from '../../../js/agentic/general_execution.mjs';
before(async () => {await installNodeHost(new WorkerHost());});
const cases = [
  ['Create file ci-diagnostic-audit-invariant.md containing CI diagnostics are classified, actionable causes are fixed, and uncertain cache backends expose opt-in tracing',
    'ci-diagnostic-audit-invariant.md', 'CI diagnostics are classified, actionable causes are fixed, and uncertain cache backends expose opt-in tracing'],
  ['Create file ci-template-comparison-invariant.md containing Template audits compare immutable workflow and script trees and report shared defects upstream',
    'ci-template-comparison-invariant.md', 'Template audits compare immutable workflow and script trees and report shared defects upstream'],
];
function record(messages, call, content) {
  const id = 'actual-' + messages.length;
  messages.push({role: 'assistant', content: '', tool_calls: [{id, type: 'function',
    function: {name: call.tool, arguments: call.arguments}}]});
  messages.push({role: 'tool', name: call.tool, tool_call_id: id, content});
}
for (const [request, target, expected] of cases) test('allowlisted command preserves requested write: ' + target, () => {
  const directory = mkdtempSync(join(tmpdir(), 'allowlisted-journal-boundary-'));
  try {
    const messages = [{role: 'user', content: request}], plan = composeGeneralChangePlan(request);
    const tools = ['web_search', 'web_fetch', 'write_file', 'run_command', 'translate'];
    let final = null; const calls = [];
    for (let turn = 0; turn < 12; turn++) {
      const next = planGeneralChangeStep(messages, tools, plan);
      if (next.kind === 'final') {final = next; break;}
      const call = next.calls[0], argumentsValue = JSON.parse(call.arguments); calls.push(call);
      if (call.tool === 'write_file') {
        assert.equal(argumentsValue.path, target); assert.equal(argumentsValue.content, expected);
        writeFileSync(join(directory, target), argumentsValue.content, {flag: 'wx'});
        record(messages, call, 'wrote ' + Buffer.byteLength(expected) + ' byte(s) to ' + target);
      } else {
        assert.equal(call.tool, 'run_command'); assert.equal(argumentsValue.command, 'cat ' + target);
        const output = execFileSync('cat', [target], {cwd: directory, encoding: 'utf8', timeout: 2000});
        record(messages, call, JSON.stringify({schema: 'command-execution-receipt/v1', command: argumentsValue.command,
          exit_code: 0, stdout: output, stderr: '', complete: true, truncated: false, timed_out: false}));
      }
    }
    assert.ok(final); assert.equal(readFileSync(join(directory, target), 'utf8'), expected);
    assert.deepEqual(calls.map(call => call.tool), ['write_file', 'run_command']);
    assert.equal(final.result.disposition, 'gap');
    assert.equal(final.result.origin, 'auxiliary_event_unavailable');
    assert.throws(() => readFileSync(join(directory, PLAN_PATH)), /ENOENT/);
  } finally {rmSync(directory, {recursive: true, force: true});}
});
test('a source-owned shell alias still retains exact atomic journal operation', () => {
  const request = cases[0][0], plan = composeGeneralChangePlan(request);
  const next = planGeneralChangeStep([{role: 'user', content: request}], ['write_file', 'Bash'], plan);
  assert.equal(next.calls[0].tool, 'Bash');
  const command = JSON.parse(next.calls[0].arguments).command;
  assert.ok(command.startsWith('mkdir -p -- .formal-ai && '));
  assert.ok(command.includes('awk -v identity='));
});
