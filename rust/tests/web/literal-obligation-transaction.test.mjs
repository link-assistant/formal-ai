// Every authoritative artifact needs its own exact target and successful readback.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep, planChatStepResolved } from '../../../js/agentic/planner.mjs';
import { drive } from '../../../experiments/js_dogfood/drive.mjs';
import { observedPayload } from '../../../js/agentic/tool_result.mjs';
import { finalResult, FinalDisposition, canDeliverFinal } from '../../../js/agentic/final_result.mjs';
import { obligations, successfullyDischarged } from '../../../js/agentic/task_obligations.mjs';
import { planObligationsStep } from '../../../js/agentic/planner/obligations.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
const original = 'First, create file a.txt with exactly this content «alpha». Second, create file b.txt with exactly this content «beta».';
async function fixture(task, expected, options = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-literal-transaction-'));
  try {
    for (const [name, text] of Object.entries(options.initial ?? {})) writeFileSync(join(directory, name), text);
    const out = await drive(planChatStep, directory, task, { steps: 8, ...(options.tools ? { tools: options.tools } : {}) });
    for (const [name, text] of Object.entries(expected)) assert.equal(existsSync(join(directory, name)) ? readFileSync(join(directory, name), 'utf8') : null, text);
    return out;
  } finally { rmSync(directory, { recursive: true, force: true }); }
}
function exactReceipts(out, expected) {
  for (const [path, bytes] of Object.entries(expected)) {
    const receipts = out.transcript.filter((entry) => entry.tool === 'bash' && JSON.parse(entry.arguments).command === 'cat ' + path);
    assert.equal(receipts.length, 1, path + ' has one independent receipt');
    assert.equal(observedPayload(receipts[0].result), bytes);
  }
  assert.equal(out.stop, 'final');
}
test('unchanged T2379 writes and independently verifies both artifacts', async () => {
  const expected = { 'a.txt': 'alpha', 'b.txt': 'beta' };
  exactReceipts(await fixture(original, expected), expected);
});
test('quoted operation and teaching keywords remain literal in every obligation', async () => {
  const quote = String.fromCharCode(96);
  const first = 'When ' + quote + 'input' + quote + ' then ' + quote + 'output' + quote;
  const second = 'rename a.txt to gone.txt and replace alpha with wrong';
  const task = 'First, create file a.txt with exactly this content «' + first + '». Second, create file b.txt with exactly this content «' + second + '».';
  const out = await fixture(task, { 'a.txt': first, 'b.txt': second, 'gone.txt': null });
  exactReceipts(out, { 'a.txt': first, 'b.txt': second });
  assert.equal(out.transcript.filter((entry) => entry.tool === 'write').length, 2);
});
test('a first-file receipt cannot certify a second file or its failed readback', async () => {
  const messages = [{ role: 'user', content: original }];
  for (const [id, command, content] of [['first', 'cat a.txt', 'Output: alpha\nExit Code: 0'], ['second', 'cat b.txt', 'Output: beta\nExit Code: 1']]) {
    messages.push({ role: 'assistant', tool_calls: [{ id, type: 'function', function: { name: 'bash', arguments: JSON.stringify({ command }) } }] });
    messages.push({ role: 'tool', name: 'bash', tool_call_id: id, content });
  }
  const plan = await planObligationsStep(original, messages, ['bash', 'write'], obligations(original));
  assert.equal(plan.kind, 'final');
  assert.equal(successfullyDischarged(original, messages), false);
  assert.equal(canDeliverFinal(plan), false);
  assert.match(plan.answer, /cat b\.txt/);
  assert.doesNotMatch(plan.answer, /Completed the general change request/);
});
test('write-only delivery retains first target and reports verification unavailable', async () => {
  const out = await fixture(original, { 'a.txt': 'alpha', 'b.txt': null }, { tools: ['write'] });
  assert.equal(out.transcript.length, 1);
  assert.equal(out.transcript[0].tool, 'write');
  assert.equal(out.stop, 'final');
  assert.doesNotMatch(out.answer, /Completed the general change request/);
  const messages = [{ role: 'user', content: original }, { role: 'assistant', tool_calls: [{ id: 'write', type: 'function', function: { name: 'write', arguments: out.transcript[0].arguments } }] }, { role: 'tool', name: 'write', tool_call_id: 'write', content: out.transcript[0].result }];
  const final = await planChatStepResolved(messages, ['write']);
  assert.equal(finalResult(final).disposition, FinalDisposition.Gap);
  const projected = await planChatStep(messages, ['write']);
  assert.equal(finalResult(projected).disposition, FinalDisposition.Unknown);
});
test('read-only clients cannot mutate either literal target', async () => {
  const out = await fixture(original, { 'a.txt': 'existing', 'b.txt': null }, { tools: ['read'], initial: { 'a.txt': 'existing' } });
  assert.ok(out.transcript.every((entry) => entry.tool === 'read'));
  assert.doesNotMatch(out.answer ?? '', /Completed the general change request/);
});
test('a successful artifact never hides an underivable mandatory clause', async () => {
  const task = 'First, create file a.txt with exactly this content «alpha». Second, derive the missing compiler contract.';
  const out = await fixture(task, { 'a.txt': 'alpha' });
  assert.equal(out.stop, 'final');
  assert.match(out.answer, /no_artifact_in_clause/);
  assert.doesNotMatch(out.answer, /Completed the general change request/);
});
