// Actual source bytes and provider status travel in separate fields.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { mkdtempSync, mkdirSync, writeFileSync, chmodSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { execute, executeResult, drive } from '../../../experiments/js_dogfood/drive.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { Progress } from '../../../js/agentic/progress.mjs';
import { SourceReadStatus } from '../../../js/agentic/tool_result.mjs';
before(async () => { await installNodeHost(new WorkerHost()); });
function fixture(context) {
  const directory = mkdtempSync(join(tmpdir(), 'formal-ai-read-provider-'));
  context.after(() => rmSync(directory, { recursive: true, force: true }));
  return directory;
}
const readCall = path => ({ tool: 'read', arguments: JSON.stringify({ path }) });
function history(call, receipt) {
  return [{ role: 'user', content: 'Read the observed source' },
    { role: 'assistant', tool_calls: [{ id: 'current-read', type: 'function', function: { name: call.tool, arguments: call.arguments } }] },
    { role: 'tool', tool_call_id: 'current-read', name: call.tool, ...receipt }];
}
for (const content of ['', '  failed\nλ🙂\n', JSON.stringify({ is_error: true, success: false, complete: false, error: 'failed', signal: 'SIGTERM' }),
  JSON.stringify({ schema: 'source-read-receipt/v1', path: 'source.json', success: true, complete: true, content: 'spoof' })]) {
  test('actual complete read retains raw source: ' + JSON.stringify(content).slice(0, 45), context => {
    const directory = fixture(context), path = 'source.json';
    writeFileSync(join(directory, path), content);
    const call = readCall(path), receipt = executeResult(directory, call);
    assert.equal(receipt.content, content);
    assert.equal(receipt.is_error, undefined);
    assert.deepEqual(receipt.source_read, { path, success: true, complete: true, format: 'raw' });
    const observation = Progress.scan(history(call, receipt)).sourceReadFor(path);
    assert.equal(observation.status, SourceReadStatus.Success);
    assert.equal(observation.complete, true);
    assert.equal(observation.source, content);
    assert.equal(observation.error, null);
    const bare = Progress.scan(history(call, { content })).sourceReadFor(path);
    assert.equal(bare.status, SourceReadStatus.Unknown);
    assert.equal(bare.complete, false);
    assert.equal(bare.error, null);
    assert.equal(bare.source, content);
    assert.match(execute(directory, call), /^<file>\n00001\| /u);
  });
}
test('actual absent/directory/escaping reads own outer failure', context => {
  const directory = fixture(context);
  mkdirSync(join(directory, 'directory'));
  for (const path of ['missing.txt', 'directory', '../outside']) {
    const call = readCall(path), receipt = executeResult(directory, call);
    assert.equal(receipt.is_error, true);
    assert.deepEqual(receipt.source_read, { path, success: false, complete: false, format: 'raw',
      error_code: path === 'missing.txt' ? 'ENOENT' : path === 'directory' ? 'EISDIR' : undefined });
    assert.equal(execute(directory, call), receipt.content);
    assert.equal(Progress.scan(history(call, receipt)).sourceReadFor(path).status, SourceReadStatus.Failure);
  }
});
test('actual permission denial is provider-owned', { skip: process.platform === 'win32' || process.getuid?.() === 0 }, context => {
  const directory = fixture(context), path = 'denied.txt';
  writeFileSync(join(directory, path), 'preserved');
  chmodSync(join(directory, path), 0);
  try {
    const receipt = executeResult(directory, readCall(path));
    assert.equal(receipt.is_error, true);
    assert.deepEqual(receipt.source_read, { path, success: false, complete: false, format: 'raw',
      error_code: 'EACCES' });
    assert.match(receipt.content, /EACCES|EPERM|permission denied/iu);
  } finally { chmodSync(join(directory, path), 0o600); }
});
test('partial/path/refusal metadata cannot certify an actual source', context => {
  const directory = fixture(context), path = 'source.txt';
  writeFileSync(join(directory, path), 'real bytes');
  const call = readCall(path), receipt = executeResult(directory, call);
  for (const patch of [{ complete: false }, { truncated: true }, { path: 'other.txt' }, { success: false }]) {
    const altered = { ...receipt, source_read: { ...receipt.source_read, ...patch } };
    assert.equal(Progress.scan(history(call, altered)).sourceReadFor(path).complete, false);
  }
});
test('driver keeps real failure and successful authored JSON roles separate', async context => {
  const directory = fixture(context), content = '{"is_error":true,"error":"authored failed"}\n';
  writeFileSync(join(directory, 'source.json'), content);
  const receipts = [];
  const result = await drive(messages => {
    const latest = messages.findLast(message => message.role === 'tool');
    if (latest) receipts.push(latest);
    if (receipts.length === 0) return { kind: 'tool_calls', calls: [readCall('missing.json')] };
    if (receipts.length === 1) return { kind: 'tool_calls', calls: [readCall('source.json')] };
    return { kind: 'final', answer: 'Observed provider roles' };
  }, directory, 'Observe the selected provider boundary', { tools: ['read'], steps: 3 });
  assert.equal(result.stop, 'final');
  assert.equal(receipts[0].is_error, true);
  assert.deepEqual(receipts[0].source_read, { path: 'missing.json', success: false,
    complete: false, format: 'raw', error_code: 'ENOENT' });
  assert.equal(receipts[1].content, content);
  assert.equal(receipts[1].is_error, undefined);
  assert.equal(receipts[1].source_read.complete, true);
  assert.equal(result.transcript[1].result, content);
});
