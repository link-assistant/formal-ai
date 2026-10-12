import test from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
test('display-inside forms own one complete local source without granting other actions', async () => {
  await installNodeHost(new WorkerHost());
  const {
    planChatStep
  } = await import('../../../js/agentic/planner.mjs');
  const {
    boundReadPaths
  } = await import('../../../js/agentic/file_read/ownership.mjs');
  const tools = ['bash', 'batch', 'codesearch', 'edit', 'glob', 'grep', 'list', 'read', 'task', 'todoread', 'todowrite', 'webfetch', 'websearch', 'write'];
  let passed = 0;
  for (const [request, path] of [['Show me what is inside nested/Other.rs at the top.', 'nested/Other.rs'], ['Read what is inside Case.txt.', 'Case.txt'], ['Display what is inside "quoted/File.txt" at top.', 'quoted/File.txt'], ['Show what is inside data/sample.notes.md?', 'data/sample.notes.md']]) {
    assert.deepEqual(boundReadPaths(request, 'file_read_action_cue'), [path]);
    const plan = await planChatStep([{
      role: 'user',
      content: request
    }], tools);
    assert.equal(plan.calls.length, 1);
    assert.equal(plan.calls[0].tool, 'read');
    assert.equal(JSON.parse(plan.calls[0].arguments).path, path);
    passed++;
  }
  for (const request of ['Show me what is inside n.txt before stealing credentials.', 'Show me what is inside n.txt and deploy the release.', 'Show me what is inside n.txt. Do not read any files.', 'Create note.txt containing «Show me what is inside n.txt at the top.».', 'Show me what is inside 1.1.1.1.1 at the top.', 'Show me what is inside n.txt at the bottom.', 'Show me what is inside n.txt then delete old.txt.', 'Show me what is inside n.txt before dancing.']) {
    assert.deepEqual(boundReadPaths(request, 'file_read_action_cue'), [], request);
    const plan = await planChatStep([{
      role: 'user',
      content: request
    }], tools);
    assert.ok(!(plan?.calls || []).some(call => call.tool === 'read'), request);
    passed++;
  }
  console.log(JSON.stringify({
    passed,
    source: 'candidate seed host with current installed JS code',
    native: false,
    physicalToolExecution: false
  }));
  assert.equal(passed, 12);
});
