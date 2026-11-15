import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { Progress } from '../../../js/agentic/progress.mjs';
import { planChatStepResolved } from '../../../js/agentic/planner.mjs';
before(async () => { await installNodeHost(new WorkerHost(), {sourceUsageRustGrammar: false}); });
const target = 'note.txt', content = 'hello';
const prompt = 'Create note.txt with the following exact content: «hello».';
function frame(messages, tool, args, text, metadata = undefined, isError = false) {
  const id = 'readback-' + messages.length;
  messages.push({role: 'assistant', tool_calls: [{id, type: 'function', function: {name: tool, arguments: JSON.stringify(args)}}]},
    {role: 'tool', tool_call_id: id, name: tool, content: text, ...(metadata === undefined ? {} : {source_read: metadata}), is_error: isError});
}
function history() {
  const messages = [{role: 'user', content: prompt}];
  frame(messages, 'read', {path: target}, content, {path: target, success: true, complete: true, format: 'raw'});
  frame(messages, 'write', {path: target, content}, 'Created file');
  return messages;
}
test('an unrelated read after a write cannot certify matching pre-write target bytes', async () => {
  const messages = history();
  frame(messages, 'read', {path: 'other.txt'}, content, {path: 'other.txt', success: true, complete: true, format: 'raw'});
  const progress = Progress.scan(messages);
  assert.equal(progress.sourceReadFor(target).source, content);
  assert.equal(progress.attemptedReadAfterLatestWriteFor(target), false);
  assert.equal(progress.sourceReadAfterLatestWriteFor(target), null);
  const plan = await planChatStepResolved(messages, ['read', 'write']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls[0].tool, 'read');
  assert.equal(JSON.parse(plan.calls[0].arguments).path, target);
});
for (const [label, receipt, failed] of [
  ['failed', {path: target, success: false, complete: true, format: 'raw'}, true],
  ['truncated', {path: target, success: true, complete: false, format: 'raw'}, false],
  ['missing-metadata', undefined, false],
]) test('latest ' + label + ' target read cannot fall back to the matching preimage', async () => {
  const messages = history();
  frame(messages, 'read', {path: target}, content, receipt, failed);
  const progress = Progress.scan(messages);
  assert.equal(progress.attemptedReadAfterLatestWriteFor(target), true);
  const observed = progress.sourceReadAfterLatestWriteFor(target);
  assert.ok(observed === null || observed.error !== null || !observed.complete);
  const plan = await planChatStepResolved(messages, ['read', 'write']);
  assert.equal(plan.kind, 'final');
  assert.notEqual(plan.result?.disposition, 'finding');
});
test('a complete fresh target read can certify the observed write', async () => {
  const messages = history();
  frame(messages, 'read', {path: target}, content, {path: target, success: true, complete: true, format: 'raw'});
  const progress = Progress.scan(messages);
  assert.equal(progress.sourceReadAfterLatestWriteFor(target).source, content);
  const plan = await planChatStepResolved(messages, ['read', 'write']);
  assert.equal(plan.kind, 'final');
  assert.equal(plan.result?.disposition, 'finding');
});
test('conflicting path aliases cannot count as a fresh target read', () => {
  const messages = history();
  frame(messages, 'read', {path: target, filePath: 'other.txt'}, content, {path: target, success: true, complete: true, format: 'raw'});
  const progress = Progress.scan(messages);
  assert.equal(progress.attemptedReadAfterLatestWriteFor(target), false);
  assert.equal(progress.sourceReadAfterLatestWriteFor(target), null);
});
for (const [tool, args] of [
  ['edit', {path: target, old_string: content, new_string: 'changed'}],
  ['multi_edit', {edits: [{path: target, old_string: content, new_string: 'changed'}]}],
  ['bash', {command: 'opaque workspace operation'}],
  ['write', {path: 'other.txt', content: 'other'}],
]) test('later potentially mutating ' + tool + ' invalidates earlier readback', async () => {
  const messages = history();
  frame(messages, 'read', {path: target}, content, {path: target, success: true, complete: true, format: 'raw'});
  frame(messages, tool, args, 'Completed');
  const progress = Progress.scan(messages);
  assert.equal(progress.attemptedReadAfterLatestWriteFor(target), false);
  assert.equal(progress.sourceReadAfterLatestWriteFor(target), null);
  const plan = await planChatStepResolved(messages, ['read', 'write']);
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls[0].tool, 'read');
  assert.equal(JSON.parse(plan.calls[0].arguments).path, target);
  frame(messages, 'read', {path: target}, 'changed', {path: target, success: true, complete: true, format: 'raw'});
  const changed = await planChatStepResolved(messages, ['read', 'write']);
  assert.equal(changed.kind, 'final');
  assert.notEqual(changed.result?.disposition, 'finding');
});
test('unrelated read-only observations preserve complete fresh target verification', async () => {
  const messages = history();
  frame(messages, 'read', {path: target}, content, {path: target, success: true, complete: true, format: 'raw'});
  frame(messages, 'read', {path: 'other.txt'}, 'other', {path: 'other.txt', success: true, complete: true, format: 'raw'});
  const plan = await planChatStepResolved(messages, ['read', 'write']);
  assert.equal(plan.kind, 'final');
  assert.equal(plan.result?.disposition, 'finding');
});
