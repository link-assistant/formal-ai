// The current whole-file mutation owns its digest observation; raw cat remains insufficient.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { readFileSync, mkdtempSync, mkdirSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, dirname } from 'node:path';
import { execFileSync } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
let verify, plan, chat, observedBytesMatch;
before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planWriteDigestVerification: verify } = await import('../../../js/agentic/workspace_change/digest_verification.mjs'));
  ({ planWorkspaceChangeStep: plan } = await import('../../../js/agentic/workspace_change.mjs'));
  ({ planChatStep: chat } = await import('../../../js/agentic/planner.mjs'));
  ({ observedBytesMatch } = await import('../../../js/agentic/tool_result.mjs'));
});
const task = 'In f.txt replace «old» with «new»';
const change = { target: 'f.txt', expected: 'new', intent: 'coding_text_replaced', slots: [['{old}', 'old'], ['{new}', 'new']] };
const tools = ['read', 'write', 'bash'];
function append(messages, tool, argumentsValue, raw, is_error = false) {
  const identity = 'call-' + messages.length;
  messages.push({ role: 'assistant', tool_calls: [{ id: identity, type: 'function', function: { name: tool, arguments: JSON.stringify(argumentsValue) } }] },
    { role: 'tool', tool_call_id: identity, content: raw, is_error });
}
const initial = () => [{ role: 'user', content: task }];
const written = (path = 'f.txt') => { const messages = initial(); append(messages, 'write', { path, content: 'new' }, ''); return messages; };
const receipt = sha256Hex('new') + '  f.txt\n';
const command = 'sha256sum -- f.txt';
const good = (messages) => append(messages, 'bash', { command }, receipt);
function finding(messages, expectedChange = change) {
  const result = verify(task, messages, tools, expectedChange);
  assert.equal(result.kind, 'final');
  assert.equal(result.result.disposition, 'finding');
  assert.equal(result.answer, 'Replaced `old` with `new` in `f.txt` and observed the result.');
}
function failure(messages) {
  const result = verify(task, messages, tools, change);
  assert.equal(result.kind, 'final');
  assert.equal(result.result.disposition, 'failure');
  assert.equal(result.answer, 'Verification failed for `f.txt`: the observed bytes differ from the planned workspace effect.');
}
function pending(messages) {
  const result = verify(task, messages, tools, change);
  assert.equal(result.kind, 'tool_calls');
  assert.equal(result.calls[0].tool, 'bash');
  assert.deepEqual(JSON.parse(result.calls[0].arguments), { command });
}
test('whole Write fallback performs read, actual mutation, fresh digest, and exact final answer', () => {
  let content = 'old'; const messages = initial(), calls = [];
  for (let turn = 0; turn < 5; turn += 1) {
    const next = plan(task, messages, tools);
    if (next.kind === 'final') { assert.equal(next.answer, 'Replaced `old` with `new` in `f.txt` and observed the result.'); break; }
    const call = next.calls[0], argumentsValue = JSON.parse(call.arguments); calls.push(call.tool);
    let raw = '';
    if (call.tool === 'read') raw = content;
    else if (call.tool === 'write') content = argumentsValue.content;
    else { assert.equal(argumentsValue.command, command); raw = sha256Hex(content) + '  f.txt\n'; }
    append(messages, call.tool, argumentsValue, raw);
  }
  assert.equal(content, 'new'); assert.deepEqual(calls, ['read', 'write', 'bash']);
});
test('bare digest and explicit successful receipts preserve the existing digest contract', () => {
  for (const raw of [receipt, JSON.stringify({ stdout: receipt, exit_code: 0 }), 'Output: ' + receipt + '\nExit Code: 0']) {
    const messages = written(); append(messages, 'bash', { command }, raw); finding(messages);
  }
});
test('wrong digest, explicit nonzero, transport error, and outer failure cannot certify current bytes', () => {
  for (const [raw, outer] of [[sha256Hex('wrong') + '  f.txt\n', false], [JSON.stringify({ stdout: receipt, exit_code: 1 }), false], [JSON.stringify({ stdout: receipt, exit_code: 0, error: 'failed' }), false], ['Output: ' + receipt + '\nExit Code: 1', false], [receipt, true]]) {
    const messages = written(); append(messages, 'bash', { command }, raw, outer); failure(messages);
  }
});
test('digest observations and command calls before the mutation remain stale', () => {
  const messages = initial(); good(messages); append(messages, 'write', { path: 'f.txt', content: 'new' }, ''); pending(messages);
  const delayed = initial(), identity = 'prewrite-command';
  delayed.push({ role: 'assistant', tool_calls: [{ id: identity, type: 'function', function: { name: 'bash', arguments: JSON.stringify({ command }) } }] });
  append(delayed, 'write', { path: 'f.txt', content: 'new' }, ''); delayed.push({ role: 'tool', tool_call_id: identity, content: receipt }); pending(delayed);
});
test('latest failed or changed-content writes invalidate an earlier success', () => {
  for (const [content, raw, outer] of [['new', '', true], ['new', JSON.stringify({ error: 'write failed', is_error: true }), false], ['different', '', false]]) {
    const messages = written(); good(messages); append(messages, 'write', { path: 'f.txt', content }, raw, outer); failure(messages);
  }
});
test('relative suffix paths and unrelated commands cannot donate receipt evidence', () => {
  const wrongWrite = written('other/f.txt'); good(wrongWrite); failure(wrongWrite);
  const wrongCommand = written(); append(wrongCommand, 'bash', { command: 'sha256sum -- other/f.txt' }, receipt); pending(wrongCommand);
  const qualified = written('/tmp/observed-workspace/f.txt'); good(qualified); finding(qualified);
});
test('Unicode, BOM, CRLF and large expected sources are hashed as full UTF8 bytes', () => {
  for (const content of ['报告\n', '\ufeffПривет\r\n', 'café '.repeat(9000)]) {
    const messages = initial(); append(messages, 'write', { path: 'f.txt', content }, '');
    append(messages, 'bash', { command }, sha256Hex(content) + '  f.txt\n'); finding(messages, { ...change, expected: content });
    assert.equal(observedBytesMatch(content, content), false);
    assert.equal(observedBytesMatch(content.slice(0, 30000) + '\n(Output was truncated due to length limit)', content), false);
  }
});
const originals = JSON.parse(readFileSync(new URL('../../../experiments/formal_ai_subagent/evidence/formal-ai-only-1188/whole-write-digest/original-agent-cli-fixtures.json', import.meta.url)));
for (const fixture of originals) {
  test('unchanged actual Agent CLI source task advances via a genuine fresh digest: ' + fixture.node, async () => {
    const messages = structuredClone(fixture.original.messages), names = fixture.original.tools.map(tool => tool.function.name);
    const next = await chat(messages, names); assert.equal(next.kind, 'tool_calls');
    const call = next.calls[0], argumentsValue = JSON.parse(call.arguments);
    assert.equal(call.tool, 'bash'); assert.match(argumentsValue.command, /^sha256sum -- /u);
    const target = argumentsValue.command.slice('sha256sum -- '.length);
    const writes = messages.flatMap(message => message.tool_calls ?? []).filter(item => item.function.name === 'write');
    const expected = JSON.parse(writes.at(-1).function.arguments).content;
    const directory = mkdtempSync(join(tmpdir(), 'formal-ai-whole-digest-'));
    try {
      mkdirSync(dirname(join(directory, target)), { recursive: true }); writeFileSync(join(directory, target), expected);
      const raw = execFileSync('sha256sum', ['--', target], { cwd: directory, encoding: 'utf8' });
      assert.equal(raw.split(/\s/u)[0], sha256Hex(expected));
      append(messages, 'bash', argumentsValue, raw);
      const after = await chat(messages, names);
      assert.equal(after.kind, 'tool_calls');
      assert.equal(after.calls[0].tool, 'write');
      const delivery = JSON.parse(after.calls[0].arguments);
      assert.match(delivery.path ?? delivery.filePath ?? delivery.file_path, /\.agent-ladder\/.+-proof\.md$/u);
      assert.equal(readFileSync(join(directory, target), 'utf8'), expected);
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}