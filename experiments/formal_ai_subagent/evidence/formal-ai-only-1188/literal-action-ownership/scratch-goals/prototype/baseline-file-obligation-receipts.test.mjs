// Verified file receipts must advance all declared artifacts without certifying acknowledgements.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/server/worker-host.mjs';
import { installNodeHost } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/node-host.mjs';
import { nextStep } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/task_obligations.mjs';
import { records } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/transcript_evidence.mjs';
import { observedPayload } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/tool_result.mjs';
import { sha256Hex } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/source_fetch.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const request = 'First, create file a.txt containing alpha. Second, create file b.txt containing beta.';
function receipt(messages, command, raw, name = 'bash') {
  const id = 'receipt-' + messages.length;
  const args = name === 'write' ? { path: command, content: 'alpha.' } : { command };
  messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name, arguments: JSON.stringify(args) } }] });
  messages.push({ role: 'tool', tool_call_id: id, name, content: raw });
}
const initial = () => [{ role: 'user', content: request }];
test('a successful shell envelope advances the exact file obligation and preserves raw records', () => {
  const messages = initial();
  const raw = 'Output: alpha.\nExit Code: 0';
  receipt(messages, 'cat a.txt', raw);
  const before = JSON.stringify(records(messages));
  const next = nextStep(request, messages);
  assert.equal(next.kind, 'observe');
  assert.equal(next.node.expectation.path, 'b.txt');
  assert.equal(JSON.stringify(records(messages)), before);
  assert.notEqual(records(messages)[0].observed_output_sha256, sha256Hex(new TextEncoder().encode(observedPayload(raw))));
});
test('bare successful readback remains supported', () => {
  const messages = initial();
  receipt(messages, 'cat a.txt', 'alpha.');
  assert.equal(nextStep(request, messages).node.expectation.path, 'b.txt');
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
  const messages = initial();
  receipt(messages, 'cat a.txt', 'Output: alpha.\nExit Code: 0');
  receipt(messages, 'cat a.txt', 'Output: wrong\nExit Code: 1');
  assert.equal(nextStep(request, messages).node.expectation.path, 'a.txt');
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
  const { drive } = await import('file:///Users/konard/Code/Archive/link-assistant/formal-ai/experiments/js_dogfood/drive.mjs');
  const { planChatStep } = await import('file:///private/tmp/formal-ai-only-1188/payload-action-ownership/planner-prototype.mjs');
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-file-obligation-'));
  try {
    const original = 'Two files. First, create file notes/attribution.md containing Gemfile.lock. Second, create file changelog/fragment.md containing bump patch.';
    const out = await drive(planChatStep, directory, original, { steps: 8 });
    assert.equal(out.stop, 'final');
    assert.equal(readFileSync(join(directory, 'notes/attribution.md'), 'utf8'), 'Gemfile.lock.');
    assert.equal(readFileSync(join(directory, 'changelog/fragment.md'), 'utf8'), 'bump patch.');
    const checks = out.transcript.filter((entry) => entry.tool === 'bash').map((entry) => JSON.parse(entry.arguments).command);
    assert.ok(checks.includes('cat notes/attribution.md'));
    assert.ok(checks.includes('cat changelog/fragment.md'));
    assert.match(out.answer, /no_artifact_in_clause/);
    assert.doesNotMatch(out.answer, /Completed the general change request/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
test('exact compact JSON and UTF8 payload bytes, including terminal newline, discharge only their file', () => {
  const body = '{"z":"café αβ","a":1}' + String.fromCharCode(10);
  const task = 'First, create file a.txt containing «' + body + '». Second, create file b.txt containing beta.';
  const messages = [{ role: 'user', content: task }];
  receipt(messages, 'cat a.txt', 'Output: ' + body + String.fromCharCode(10) + 'Exit Code: 0');
  const next = nextStep(task, messages);
  assert.equal(next.kind, 'observe');
  assert.equal(next.node.expectation.path, 'b.txt');
});
