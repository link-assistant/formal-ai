import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, readFileSync, rmSync, mkdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { sourceReadObservation, completeOwnedSourceReadFrame } from '../../../js/agentic/tool_result.mjs';
import { planLiteralAdditionStep } from '../../../js/agentic/literal_addition.mjs';
import { composeGeneralChangePlan, PLAN_PATH } from '../../../js/agentic/general_planner.mjs';
import { planGeneralChangeStep } from '../../../js/agentic/general_execution.mjs';
import { executeResult } from '../../../experiments/js_dogfood/drive.mjs';
import { renderSeededOutcome } from '../../../js/agentic/code_task.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });
function messages(path, result, task) {
  return [
    { role: 'user', content: task },
    { role: 'assistant', tool_calls: [{ id: 'read-owned', type: 'function', function: {
      name: 'read', arguments: JSON.stringify({ file_path: path }),
    } }] },
    { role: 'tool', tool_call_id: 'read-owned', ...result },
  ];
}
function absence(path, extra = {}) {
  return { path, success: false, complete: false, format: 'raw', error_code: 'ENOENT', ...extra };
}
for (const source of [
  '', '\n\n', 'α🙂\r\n', '\uFEFFfirst\n', '\uFFFD', 'Error: authored', '{"is_error":true}',
  '<file>\n1| false authority\n\n(End of file - total 1 lines)\n</file>', '{body}\n{lines}',
]) {
  test('physical complete reader and owned frame preserve every UTF8 byte: ' + JSON.stringify(source), () => {
    const directory = mkdtempSync(join(tmpdir(), 'formal-owned-read-'));
    try {
      writeFileSync(join(directory, 'source.txt'), source);
      const result = executeResult(directory, { tool: 'read', arguments: JSON.stringify({ file_path: 'source.txt' }) });
      assert.equal(result.content, source);
      assert.equal(result.source_read.complete, true);
      assert.deepEqual(Buffer.from(result.content), readFileSync(join(directory, 'source.txt')));
      const frame = completeOwnedSourceReadFrame(source);
      const observed = sourceReadObservation(frame, false, null, 'source.txt');
      assert.equal(observed.complete, true);
      assert.equal(observed.absent, false);
      assert.equal(observed.source, source);
    } finally { rmSync(directory, { recursive: true, force: true }); }
  });
}
test('invalid UTF8 is a real failure without absence or source mutation', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-invalid-read-'));
  try {
    const bytes = Buffer.from([0x41, 0xff, 0x42]);
    writeFileSync(join(directory, 'source.txt'), bytes);
    const result = executeResult(directory, { tool: 'read', arguments: JSON.stringify({ file_path: 'source.txt' }) });
    assert.equal(result.is_error, true);
    assert.notEqual(result.source_read.error_code, 'ENOENT');
    const observed = sourceReadObservation(result.content, true, result.source_read, 'source.txt');
    assert.equal(observed.absent, false);
    assert.equal(observed.complete, false);
    assert.deepEqual(readFileSync(join(directory, 'source.txt')), bytes);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
test('only physical exact-target ENOENT grants absence; directory and escape remain failures', () => {
  const directory = mkdtempSync(join(tmpdir(), 'formal-missing-read-'));
  try {
    mkdirSync(join(directory, 'directory'));
    for (const path of ['missing.txt', 'directory', '../escape.txt']) {
      const result = executeResult(directory, { tool: 'read', arguments: JSON.stringify({ file_path: path }) });
      assert.equal(result.is_error, true);
      const observed = sourceReadObservation(result.content, true, result.source_read, path);
      assert.equal(observed.absent, path === 'missing.txt');
    }
  } finally { rmSync(directory, { recursive: true, force: true }); }
});
const refusals = [
  null,
  absence('other.txt'),
  absence('note.txt', { error_code: 'EACCES' }),
  absence('note.txt', { error_code: undefined }),
  absence('note.txt', { success: true }),
  absence('note.txt', { complete: true }),
  absence('note.txt', { format: 'unknown' }),
  ...['truncated', 'timed_out', 'timedOut', 'interrupted', 'canceled', 'cancelled', 'aborted']
    .map(key => absence('note.txt', { [key]: true })),
  absence('note.txt', { timed_out: 'false' }),
  absence('note.txt', { signal: 'SIGTERM' }),
  absence('note.txt', { signal: false }),
  absence('note.txt', { stream_complete: false }),
];
for (const metadata of refusals) {
  test('failed, foreign or contradictory absence cannot replace unknown content: ' + JSON.stringify(metadata), () => {
    const result = { content: 'File not found: other.txt', is_error: true, source_read: metadata };
    const observed = sourceReadObservation(result.content, true, metadata, 'note.txt');
    assert.equal(observed.absent, false);
    const task = 'append file note.txt containing hello';
    assert.equal(planLiteralAdditionStep(task, messages('note.txt', result, task), ['read', 'write']).kind, 'final');
    const ledgerMetadata = metadata?.path === 'note.txt' ? { ...metadata, path: PLAN_PATH } : metadata;
    const original = 'write 10 to 1.txt file';
    const ledger = planGeneralChangeStep(messages(PLAN_PATH, { ...result, source_read: ledgerMetadata }, original),
      ['read', 'write'], composeGeneralChangePlan(original));
    assert.equal(ledger.kind, 'final');
  });
}
test('a genuine bound Missing receipt can plan both additive content and a first ledger event', () => {
  const metadata = absence('note.txt');
  const result = { content: 'observed OS absence', is_error: true, source_read: metadata };
  const task = 'append file note.txt containing hello';
  const addition = planLiteralAdditionStep(task, messages('note.txt', result, task), ['read', 'write']);
  assert.equal(addition.calls[0].tool, 'write');
  assert.equal(JSON.parse(addition.calls[0].arguments).content, 'hello\n');
  const original = 'write 10 to 1.txt file';
  const ledger = planGeneralChangeStep(messages(PLAN_PATH, { ...result, source_read: absence(PLAN_PATH) }, original),
    ['read', 'write'], composeGeneralChangePlan(original));
  assert.equal(ledger.calls[0].tool, 'write');
  assert.equal(JSON.parse(ledger.calls[0].arguments).file_path, PLAN_PATH);
});

test('missing declared Read reports inability to verify without claiming an observed mismatch', () => {
  const task = 'append file note.txt containing hello';
  const plan = planLiteralAdditionStep(task, [{ role: 'user', content: task }], ['write']);
  assert.equal(plan.result.disposition, 'gap');
  assert.equal(plan.answer, renderSeededOutcome('file-addition-unverified', task, 'note.txt'));
  assert.equal(plan.calls, undefined);
});
test('opaque source prerequisite reports inability to verify without inventing observations', () => {
  const task = 'append file note.txt containing hello';
  const plan = planLiteralAdditionStep(task, messages('note.txt', {
    content: 'unknown source bytes', source_read: { path: 'note.txt', complete: false, format: 'opaque' },
  }, task), ['read', 'write']);
  assert.equal(plan.result.disposition, 'gap');
  assert.equal(plan.answer, renderSeededOutcome('file-addition-unverified', task, 'note.txt'));
  assert.equal(plan.calls, undefined);
});
