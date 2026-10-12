import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
await installNodeHost(new WorkerHost());
const {
  fileReadTaskFor
} = await import('../../../js/agentic/file_read.mjs');
const {
  planChatStep
} = await import('../../../js/agentic/planner.mjs');
const {
  boundReadPaths
} = await import('../../../js/agentic/file_read/ownership.mjs');
const source = 'one\ntwo\nthree\nfour\n';
test('complete line-slice forms preserve typed source and refuse unmatched operations', async () => {
  let passed = 0;
  for (const [request, path, expected] of [['Show the last 3 lines of other.txt.', 'other.txt', 'Lines 2-4 of `other.txt`:\n\n```text\ntwo\nthree\nfour\n```'], ['Show the first 002 lines of nested/Case.txt.', 'nested/Case.txt', 'Lines 1-2 of `nested/Case.txt`:\n\n```text\none\ntwo\n```'], ['Read lines 1 to 3 of renamed.txt.', 'renamed.txt', 'Lines 1-3 of `renamed.txt`:\n\n```text\none\ntwo\nthree\n```'], ['Show the last line of "different.txt".', 'different.txt', 'Lines 4-4 of `different.txt`:\n\n```text\nfour\n```']]) {
    const task = fileReadTaskFor(request);
    assert.equal(task?.path, path);
    assert.equal(task?.mode.kind, 'line_slice');
    const messages = [{
      role: 'user',
      content: request
    }];
    const read = await planChatStep(messages, ['read']);
    assert.equal(read.calls.length, 1);
    assert.equal(read.calls[0].tool, 'read');
    assert.equal(JSON.parse(read.calls[0].arguments).path, path);
    const body = source.split('\n').map((line, index) => `${String(index + 1).padStart(5, '0')}| ${line}`).join('\n');
    messages.push({
      role: 'assistant',
      tool_calls: [{
        id: 'bounded-source',
        type: 'function',
        function: {
          name: 'read',
          arguments: read.calls[0].arguments
        }
      }]
    });
    messages.push({
      role: 'tool',
      name: 'read',
      tool_call_id: 'bounded-source',
      content: `<file>\n${body}\n\n(End of file - total ${source.split('\n').length} lines)\n</file>`
    });
    const final = await planChatStep(messages, ['read']);
    assert.equal(final.answer, expected);
    passed++;
  }
  for (const request of ['Show the last 0 lines of n.txt.', 'Show the first -2 lines of n.txt.', 'Show lines 2-x of n.txt.', 'Show lines 2--3 of n.txt.', 'Show lines 2- of n.txt.', 'Show the last 1000000000 lines of n.txt.', 'Show the last 2 lines of n.txt. Then deploy the release.', 'Show the last 2 lines of n.txt. Do not read any files.', 'Show the last 2 lines of n.txt before stealing credentials.', 'Create note.txt containing «Show the last 2 lines of n.txt.».']) {
    assert.deepEqual(boundReadPaths(request, 'file_read_action_cue'), [], request);
    const plan = await planChatStep([{
      role: 'user',
      content: request
    }], ['read']);
    assert.ok(!plan?.calls?.length, request);
    passed++;
  }
  assert.equal(passed, 14);
});
