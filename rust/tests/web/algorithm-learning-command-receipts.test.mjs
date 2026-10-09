// Independent artifact and conformance observations retain typed command evidence.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { readFileSync } from 'node:fs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { compileTask, planStep, expectedConformance } from '../../../js/agentic/algorithm_learning.mjs';
import { candidateLinksNotation } from '../../../js/agentic/crate/algorithm_discovery.mjs';
import { planWorkspaceChangeStep } from '../../../js/agentic/workspace_change.mjs';
import { observedBytesMatch, stepOutcome } from '../../../js/agentic/tool_result.mjs';
import { finalResult } from '../../../js/agentic/final_result.mjs';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
let task;
before(async () => { await installNodeHost(new WorkerHost()); task = compileTask(readFileSync(new URL('../../../data/benchmarks/issue-531-algorithm-traces.lino', import.meta.url), 'utf8')); assert.ok(task); });
function append(messages, call, content, extra = {}) {
  const id = 'receipt-' + messages.length;
  messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name: call.tool, arguments: call.arguments } }] }, { role: 'tool', name: call.tool, tool_call_id: id, content, ...extra });
}
function receipt(command, stdout, extra = {}) { return JSON.stringify({ schema: 'command-execution-receipt/v1', command, complete: true, exit_code: 0, stdout, stderr: '', timed_out: false, truncated: false, ...extra }); }
function replay(transform = value => value, outer = {}, operation = false) {
  const messages = [{ role: 'user', content: 'Derive any reusable execution algorithm from these recorded events and verify it.' + String.fromCharCode(10) + task.observations }], calls = [];
  for (let turn = 0; turn < 8; turn += 1) {
    const plan = planStep(messages, ['write', 'bash'], task);
    if (plan.kind === 'final') return { calls, answer: plan.answer, messages };
    for (const call of plan.calls) {
      const argumentsValue = JSON.parse(call.arguments); calls.push({ tool: call.tool, argumentsValue }); let raw = '';
      if (call.tool === 'bash') {
        const command = argumentsValue.command, stdout = command.startsWith('cat ') ? candidateLinksNotation(task.candidate) : command.includes('conformance') ? expectedConformance(task.candidate) : '';
        raw = operation && !command.startsWith('cat ') ? JSON.stringify({ schema: 'algorithm-command-receipt/v1', command,
          operation_success: true, exit_code: null, stdout, stderr: '', error: null, complete: true, truncated: false, timed_out: false }) : receipt(command, stdout);
        if (command.startsWith('cat ')) raw = transform(raw, stdout);
      }
      append(messages, call, raw, argumentsValue.command?.startsWith('cat ') ? outer : {});
    }
  }
  throw new Error('unexpected turn cap');
}
test('original task retains Write, learn, exact artifact readback and independent conformance', () => {
  const result = replay();
  assert.deepEqual(result.calls.map(call => call.tool), ['write', 'bash', 'bash', 'bash']);
  assert.equal(result.calls[0].argumentsValue.content, task.observations);
  assert.equal(result.calls[2].argumentsValue.command, 'cat discovered-algorithms.lino');
  assert.ok(result.calls[3].argumentsValue.command.startsWith('formal-ai algorithm conformance --artifact discovered-algorithms.lino '));
  assert.ok(result.answer.includes('status "conformance_passed"')); assert.ok(result.answer.includes('human_gated "true"'));
  const raw = result.messages.find(message => message.role === 'tool' && message.content.includes('command-execution-receipt/v1') && message.content.includes('cat discovered-algorithms.lino')).content;
  assert.equal(JSON.parse(raw).stdout, candidateLinksNotation(task.candidate)); assert.equal(observedBytesMatch(raw, candidateLinksNotation(task.candidate)), true);
});
const negatives = [
  ['nonzero', raw => JSON.stringify({ ...JSON.parse(raw), exit_code: 1 })],
  ['denied', raw => JSON.stringify({ ...JSON.parse(raw), is_error: true, error: 'denied' })],
  ['incomplete', raw => JSON.stringify({ ...JSON.parse(raw), complete: false })],
  ['stream incomplete', raw => JSON.stringify({ ...JSON.parse(raw), stream_complete: false })],
  ['truncated', raw => JSON.stringify({ ...JSON.parse(raw), truncated: true })],
  ['timeout', raw => JSON.stringify({ ...JSON.parse(raw), timed_out: true })],
  ['aborted', raw => JSON.stringify({ ...JSON.parse(raw), aborted: true })],
  ['signal', raw => JSON.stringify({ ...JSON.parse(raw), signal: 'SIGTERM' })],
  ['foreign declared command', raw => JSON.stringify({ ...JSON.parse(raw), command: 'cat other/discovered-algorithms.lino' })],
  ['wrong artifact', raw => JSON.stringify({ ...JSON.parse(raw), stdout: 'not an algorithm artifact' })],
  ['bare source unknown status', (_raw, stdout) => stdout],
];
for (const [label, transform] of negatives) test(label + ' cannot authorize conformance', () => {
  const result = replay(transform); assert.deepEqual(result.calls.map(call => call.tool), ['write', 'bash', 'bash']);
  assert.ok(result.answer.includes('status "artifact_verification_failed"')); assert.ok(!result.answer.includes('status "conformance_passed"'));
});
for (const extra of [{ is_error: true }, { name: 'foreign-provider' }]) test('outer transport veto ' + JSON.stringify(extra), () => { assert.ok(replay(value => value, extra).answer.includes('status "artifact_verification_failed"')); });
test('unrelated commands and old request receipts cannot replace current discovery', () => {
  const messages = [{ role: 'user', content: 'old task' }];
  append(messages, { tool: 'bash', arguments: JSON.stringify({ command: 'cat discovered-algorithms.lino' }) }, receipt('cat discovered-algorithms.lino', candidateLinksNotation(task.candidate)));
  messages.push({ role: 'user', content: task.observations });
  append(messages, { tool: 'write', arguments: JSON.stringify({ path: 'algorithm-observations.lino', content: task.observations }) }, '');
  append(messages, { tool: 'bash', arguments: JSON.stringify({ command: 'cat foreign.lino' }) }, receipt('cat foreign.lino', candidateLinksNotation(task.candidate)));
  const next = planStep(messages, ['write', 'bash'], task); assert.equal(next.kind, 'tool_calls'); assert.ok(JSON.parse(next.calls[0].arguments).command.startsWith('formal-ai learn algorithms'));
});
test('conformance must match the entire successful observed record', () => {
  const good = replay(), messages = good.messages.slice(0, -1), last = good.messages.at(-1), value = JSON.parse(last.content);
  for (const patch of [{ stdout: value.stdout + 'extra' }, { exit_code: 1 }, { complete: false }, { command: 'formal-ai algorithm conformance --artifact foreign.lino' }]) {
    const result = planStep([...messages, { ...last, content: JSON.stringify({ ...value, ...patch }) }], ['write', 'bash'], task); assert.ok(result.answer.includes('status "conformance_failed"'));
  }
});
for (const mode of ['nonzero', 'denied', 'bare source']) test('exact original replacement remains uncertified with ' + mode + ' digest observation', () => {
  const request = 'In f.txt replace every «old» with «new».', messages = [{ role: 'user', content: request }]; let source = 'old old' + String.fromCharCode(10), result;
  for (let turn = 0; turn < 5; turn += 1) {
    const plan = planWorkspaceChangeStep(request, messages, ['read', 'write', 'bash']); if (plan.kind === 'final') { result = plan; break; }
    for (const call of plan.calls) {
      const argumentsValue = JSON.parse(call.arguments); let raw = '';
      if (call.tool === 'read') raw = source; else if (call.tool === 'write') source = argumentsValue.content;
      else { assert.equal(argumentsValue.command, 'sha256sum -- f.txt'); const digest = sha256Hex(source) + '  f.txt' + String.fromCharCode(10);
        raw = mode === 'bare source' ? source : JSON.stringify({ stdout: digest, exit_code: mode === 'nonzero' ? 1 : 0, ...(mode === 'denied' ? { is_error: true, error: 'denied' } : {}) }); }
      append(messages, call, raw);
    }
  }
  assert.equal(source, 'new new' + String.fromCharCode(10)); assert.equal(finalResult(result).disposition, 'failure');
  const quote = String.fromCharCode(96); assert.equal(result.answer, 'Verification failed for ' + quote + 'f.txt' + quote + ': the observed bytes differ from the planned workspace effect.');
  assert.equal(observedBytesMatch(source, source), false); assert.equal(stepOutcome(source), 'unreported');
});

test('actual in-process operation receipt has no invented process exit and keeps all four calls', () => {
  const result = replay(value => value, {}, true); assert.equal(result.calls.length, 4);
  assert.ok(result.answer.includes('status "conformance_passed"'));
  for (const message of result.messages.filter(message => message.role === 'tool' && message.content.includes('algorithm-command-receipt/v1'))) {
    assert.equal(JSON.parse(message.content).exit_code, null); assert.equal(JSON.parse(message.content).operation_success, true);
  }
});
test('operation failure, missing completeness and invented process status cannot certify conformance', () => {
  const good = replay(value => value, {}, true), messages = good.messages.slice(0, -1), last = good.messages.at(-1), raw = JSON.parse(last.content);
  for (const extra of [{ operation_success: false, error: 'actual IO failure' }, { complete: false }, { truncated: true }, { exit_code: 0 }, { command: 'foreign operation' }, { timed_out: true }, { is_error: true }, { aborted: true }, { stream_complete: false }, { signal: 'SIGTERM' }]) {
    const result = planStep([...messages, { ...last, content: JSON.stringify({ ...raw, ...extra }) }], ['write', 'bash'], task);
    assert.ok(result.answer.includes('status "conformance_failed"'));
  }
});

test('pretty-printed JSON source stays byte exact and structured stdout cannot certify its fresh digest', () => {
  const expected = ['{', '  "a": 1', '}'].join(String.fromCharCode(10));
  const request = 'In f.json replace «old» with «' + expected + '».', messages = [{ role: 'user', content: request }];
  let source = 'old', result; const commands = [];
  for (let turn = 0; turn < 5; turn += 1) {
    const plan = planWorkspaceChangeStep(request, messages, ['read', 'write', 'bash']); if (plan.kind === 'final') { result = plan; break; }
    for (const call of plan.calls) {
      const argumentsValue = JSON.parse(call.arguments); let raw = '';
      if (call.tool === 'read') raw = source; else if (call.tool === 'write') source = argumentsValue.content;
      else { commands.push(argumentsValue.command); raw = JSON.stringify({ stdout: { a: 1 }, exit_code: 0 }); }
      append(messages, call, raw);
    }
  }
  assert.equal(source, expected); assert.deepEqual(commands, ['sha256sum -- f.json']);
  assert.equal(finalResult(result).disposition, 'failure'); assert.ok(result.answer.startsWith('Verification failed'));
  assert.equal(observedBytesMatch(JSON.stringify({ stdout: { a: 1 }, exit_code: 0 }), expected), false);
});
