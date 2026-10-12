// Every authoritative artifact needs its own exact target and successful readback.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, readFileSync, writeFileSync, existsSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/server/worker-host.mjs';
import { installNodeHost } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/node-host.mjs';
import { planChatStep, planChatStepResolved } from 'file:///private/tmp/formal-ai-only-1188/payload-action-ownership/planner-prototype.mjs';
import { drive } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/experiments/js_dogfood/drive.mjs';
import { observedPayload } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/tool_result.mjs';
import { finalResult, FinalDisposition, canDeliverFinal } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/final_result.mjs';
import { obligations, successfullyDischarged } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/task_obligations.mjs';
import { clausesWithSpans } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/obligation_ledger.mjs';
import { wordsForRole } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/write_lexicon.mjs';
import { planObligationsStep } from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/planner/obligations.mjs';
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

test('all seeded enumeration surfaces inside literal operands remain payload', () => {
  const quote = String.fromCharCode(96);
  const wrappers = [['«', '»'], [quote, quote], [quote.repeat(4) + 'text' + String.fromCharCode(10), String.fromCharCode(10) + quote.repeat(4)], [quote.repeat(12) + 'text' + String.fromCharCode(10), String.fromCharCode(10) + quote.repeat(12)]];
  const surfaces = wordsForRole('enumeration_cue');
  assert.equal(surfaces.length, 34);
  for (const surface of surfaces) {
    for (const [opening, closing] of wrappers) {
      const literal = opening + 'λ🙂 Alpha. ' + surface + ' beta.' + closing;
      const request = 'Create a.txt with exactly this content ' + literal;
      const clauses = clausesWithSpans(request);
      assert.equal(clauses.length, 1, surface + ' stays in literal');
      assert.equal(clauses[0][0], request);
      assert.deepEqual(clauses[0][1], [0, Buffer.byteLength(request)]);
    }
  }
});
test('every seeded outer cue preserves two complete literals and exact UTF8 spans', () => {
  for (const surface of wordsForRole('enumeration_cue')) {
    const request = 'First, create a.txt with exactly this content «λ🙂 Alpha. Next beta.». ' + surface + ', create b.txt with exactly this content «Gamma. Then delta.».';
    const clauses = clausesWithSpans(request);
    assert.equal(clauses.length, 2, surface + ' starts a genuine outer obligation');
    for (const [clause, [start, end]] of clauses) assert.equal(Buffer.from(request).subarray(start, end).toString('utf8'), clause);
    assert.ok(clauses[0][0].includes('«λ🙂 Alpha. Next beta.»'));
    assert.ok(clauses[1][0].includes('«Gamma. Then delta.»'));
  }
});
test('closed twelve-character fence with internal instructions has one exact readback', async () => {
  const quote = String.fromCharCode(96);
  const body = ['λ🙂 Alpha.', 'Next beta.', 'When ' + quote + 'input' + quote + ' then ' + quote + 'output' + quote + '.', 'Also list behavior rules.', ''].join(String.fromCharCode(10));
  const request = 'Set the contents of a.txt to exactly this content:' + String.fromCharCode(10) + quote.repeat(12) + 'text' + String.fromCharCode(10) + body + quote.repeat(12);
  const result = await fixture(request, { 'a.txt': body });
  exactReceipts(result, { 'a.txt': body });
  assert.equal(result.transcript.filter((entry) => entry.tool === 'write').length, 1);
  assert.doesNotMatch(result.answer, /Behavior rule compiled|Understood\. I'll avoid/);
});
test('independent outer artifacts retain nested enumeration bytes and receipts', async () => {
  const first = 'λ🙂 Alpha. Next beta.';
  const second = 'Gamma. Then delta.';
  const request = 'First, create a.txt with exactly this content «' + first + '». Second, create b.txt with exactly this content «' + second + '».';
  exactReceipts(await fixture(request, { 'a.txt': first, 'b.txt': second }), { 'a.txt': first, 'b.txt': second });
});
