// Verified file receipts must advance all declared artifacts without certifying acknowledgements.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { nextStep } from '../../../js/agentic/task_obligations.mjs';
import { records } from '../../../js/agentic/transcript_evidence.mjs';
import { observedPayload } from '../../../js/agentic/tool_result.mjs';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
import { ownedCreationReadbackCommand } from '../../../js/agentic/general_execution.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const request = 'First, create file a.txt containing alpha. Second, create file b.txt containing beta.';
function receipt(messages, command, raw, name = 'bash', sourceRead = undefined, content = 'alpha.') {
  const id = 'receipt-' + messages.length;
  const args = name === 'write' ? {path: command, content}
    : name === 'read' ? {path: command} : {command};
  messages.push({role: 'assistant', tool_calls: [{id, type: 'function',
    function: {name, arguments: JSON.stringify(args)}}]});
  messages.push({role: 'tool', tool_call_id: id, name, content: raw,
    ...(sourceRead === undefined ? {} : {source_read: sourceRead})});
}
function written(messages, target = 'a.txt', content = 'alpha.') {
  receipt(messages, target, '', 'write', undefined, content);
}
function rawRead(messages, target, content) {
  receipt(messages, target, content, 'read',
    {path: target, success: true, complete: true, format: 'raw'});
}
const initial = () => [{ role: 'user', content: request }];
test('a successful shell envelope advances the exact file obligation and preserves raw records', () => {
  const messages = initial();
  written(messages);
  const command = ownedCreationReadbackCommand('a.txt');
  const raw = 'Output: (alpha.)\nExit Code: 0';
  receipt(messages, command, raw);
  const before = JSON.stringify(records(messages));
  const next = nextStep(request, messages);
  assert.equal(next.kind, 'observe');
  assert.equal(next.node.expectation.path, 'b.txt');
  assert.equal(JSON.stringify(records(messages)), before);
  assert.notEqual(records(messages).at(-1).observed_output_sha256,
    sha256Hex(new TextEncoder().encode(observedPayload(raw))));
});
test('provider-owned raw successful readback remains supported without a process status', () => {
  const messages = initial();
  written(messages);
  rawRead(messages, 'a.txt', 'alpha.');
  assert.equal(nextStep(request, messages).node.expectation.path, 'b.txt');
});
test('bare or unframed shell bytes do not establish write and verification custody', () => {
  for (const includeWrite of [false, true]) {
    for (const raw of ['Output: alpha.\nExit Code: 0', 'alpha.']) {
      const messages = initial();
      if (includeWrite) written(messages);
      receipt(messages, 'cat a.txt', raw);
      assert.equal(nextStep(request, messages).node.expectation.path, 'a.txt');
    }
  }
});
test('write acknowledgements and auxiliary commands do not certify file contents', () => {
  for (const [command, raw, name] of [['a.txt', '', 'write'], ['a.txt', 'alpha.', 'write'], ['printf a.txt', 'Output: alpha.\nExit Code: 0', 'bash'], ['printf a.txt', 'alpha.', 'bash']]) {
    const messages = initial();
    receipt(messages, command, raw, name);
    assert.equal(nextStep(request, messages).node.expectation.path, 'a.txt');
  }
});
test('mismatch, nonzero status and a later failed receipt cannot reuse an older success', () => {
  for (const raw of ['Output: wrong\nExit Code: 0', 'Output: alpha.\nExit Code: 1']) {
    const messages = initial();
    receipt(messages, 'cat a.txt', raw);
    assert.equal(nextStep(request, messages).node.expectation.path, 'a.txt');
  }
  for (const raw of ['Output: (wrong)\nExit Code: 0', 'Output: (alpha.)\nExit Code: 1']) {
    const messages = initial();
    written(messages);
    receipt(messages, ownedCreationReadbackCommand('a.txt'), raw);
    assert.equal(nextStep(request, messages).node.expectation.path, 'a.txt');
  }
  const messages = initial();
  written(messages);
  receipt(messages, ownedCreationReadbackCommand('a.txt'), 'Output: (alpha.)\nExit Code: 0');
  assert.equal(nextStep(request, messages).node.expectation.path, 'b.txt');
  receipt(messages, ownedCreationReadbackCommand('a.txt'), 'Output: (wrong)\nExit Code: 1');
  assert.equal(nextStep(request, messages).node.expectation.path, 'a.txt');
  const unbound = initial();
  receipt(unbound, 'cat a.txt', 'Output: alpha.\nExit Code: 0');
  receipt(unbound, 'cat a.txt', 'Output: wrong\nExit Code: 1');
  assert.equal(nextStep(request, unbound).node.expectation.path, 'a.txt');
});
test('a different path sharing a filename suffix cannot fulfill the declared file', () => {
  const messages = initial();
  for (const raw of ['Output: alpha.\nExit Code: 0', 'alpha.']) {
    const trial = initial();
    receipt(trial, 'cat other-a.txt', raw);
    assert.equal(nextStep(request, trial).node.expectation.path, 'a.txt');
  }
});

test('the unchanged issue1099 request persists both artifacts through real shell receipts', async () => {
  const { mkdtempSync, readFileSync, rmSync } = await import('node:fs');
  const { tmpdir } = await import('node:os');
  const { join } = await import('node:path');
  const { drive } = await import('../../../experiments/js_dogfood/drive.mjs');
  const { planChatStep } = await import('../../../js/agentic/planner.mjs');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-file-obligation-'));
  try {
    const original = 'Two files. First, create file notes/attribution.md containing Gemfile.lock. Second, create file changelog/fragment.md containing bump patch.';
    const out = await drive(planChatStep, directory, original, { steps: 8 });
    assert.equal(out.stop, 'final');
    assert.equal(readFileSync(join(directory, 'notes/attribution.md'), 'utf8'), 'Gemfile.lock.');
    assert.equal(readFileSync(join(directory, 'changelog/fragment.md'), 'utf8'), 'bump patch.');
    for (const [target, bytes] of [
      ['notes/attribution.md', 'Gemfile.lock.'],
      ['changelog/fragment.md', 'bump patch.']
    ]) {
      const writeIndex = out.transcript.findLastIndex(entry =>
        entry.tool === 'write' && JSON.parse(entry.arguments).path === target);
      const readIndex = out.transcript.findLastIndex(entry =>
        entry.tool === 'read' && entry.source_read?.path === target);
      assert.ok(writeIndex >= 0 && readIndex > writeIndex, target + ' has its own post-write Read');
      const read = out.transcript[readIndex];
      assert.notEqual(read.is_error, true);
      assert.equal(read.source_read.success, true);
      assert.equal(read.source_read.complete, true);
      assert.equal(read.source_read.format, 'raw');
      assert.equal(read.result, bytes);
    }
    assert.match(out.answer, /no_artifact_in_clause/);
    assert.doesNotMatch(out.answer, /Completed the general change request/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
test('exact compact JSON and UTF8 payload bytes, including terminal newline, discharge only their file', () => {
  const body = '{"z":"café αβ","a":1}' + String.fromCharCode(10);
  const task = 'First, create file a.txt containing «' + body + '». Second, create file b.txt containing beta.';
  const messages = [{ role: 'user', content: task }];
  const unbound = [{role: 'user', content: task}];
  receipt(unbound, 'cat a.txt', 'Output: ' + body + String.fromCharCode(10) + 'Exit Code: 0');
  assert.equal(nextStep(task, unbound).node.expectation.path, 'a.txt');
  written(messages, 'a.txt', body);
  rawRead(messages, 'a.txt', body);
  const next = nextStep(task, messages);
  assert.equal(next.kind, 'observe');
  assert.equal(next.node.expectation.path, 'b.txt');
});
