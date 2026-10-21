import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { qualifiedTranscriptFrames } from '../../../js/agentic/qualified_tool_observation.mjs';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { readText } from '../../../js/agentic/host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
let worker, grant;
before(async () => {
  worker = new WorkerHost();
  await installNodeHost(worker);
  grant = JSON.parse(readText('data/meta/workspace-prerequisite-grant.json'));
});
function frames(name) {
  const messages = [{
    role: 'assistant',
    tool_calls: [{
      id: 'unique',
      type: 'function',
      function: {
        name: 'bash',
        arguments: '{"command":"true"}'
      }
    }]
  }, {
    role: 'tool',
    tool_call_id: 'unique',
    content: 'Exit code 0\n',
    ...(name === undefined ? {} : {
      name
    })
  }];
  return messages;
}
test('optional names bind only unique declared IDs and never provider execution authority', () => {
  for (const name of [undefined, null, 'bash']) {
    const frame = qualifiedTranscriptFrames(frames(name), 0)[0];
    assert.equal(frame.binding, 'exact');
    assert.equal(frame.succeeded, true);
    assert.deepEqual([frame.provider, frame.approval, frame.effects], ['Unknown', 'Unknown', 'Unknown']);
  }
  for (const name of ['', 42, 'Bash', 'write']) {
    const frame = qualifiedTranscriptFrames(frames(name), 0)[0];
    assert.equal(frame.binding, 'contradicted');
    assert.equal(frame.succeeded, false);
  }
});
test('unbound, duplicated, stale and conflicting nameless receipts never certify a declaration', () => {
  for (const id of [undefined, 'unknown', '']) {
    const messages = frames();
    messages[1].tool_call_id = id;
    const frame = qualifiedTranscriptFrames(messages, 0)[0];
    assert.equal(frame.binding, 'unknown');
    assert.equal(frame.succeeded, false);
  }
  for (const duplicate of ['declaration', 'receipt']) {
    const messages = frames();
    messages.push(structuredClone(messages[duplicate === 'declaration' ? 0 : 1]));
    assert(qualifiedTranscriptFrames(messages, 0).every(frame => frame.binding === 'contradicted' && !frame.succeeded));
  }
  const messages = frames();
  assert.deepEqual(qualifiedTranscriptFrames(messages, 1), []);
  messages[1].name = 'write';
  delete messages[1].tool_call_id;
  assert.equal(qualifiedTranscriptFrames(messages, 0)[0].binding, 'unknown');
});
function record(messages, call, content) {
  const id = 'agent-' + messages.length;
  let args = JSON.parse(call.arguments);
  if (call.tool === 'write') {
    args = {
      filePath: '/tmp/owned-workspace/' + args.path,
      content: args.content
    };
  }
  messages.push({
    role: 'assistant',
    content: '',
    tool_calls: [{
      id,
      type: 'function',
      function: {
        name: call.tool,
        arguments: JSON.stringify(args)
      }
    }]
  }, {
    role: 'tool',
    tool_call_id: id,
    content
  });
}
test('actual Agent nameless empty writes and absolute filePath recover an owned missing Scala compiler', async () => {
  await installNodeHost(worker, {
    prerequisiteGrant: grant
  });
  const tools = ['bash', 'write', 'read', 'edit', 'webfetch', 'websearch'];
  const messages = [{
    role: 'user',
    content: 'Write a Scala program that prints "A held-out result" to stdout.'
  }];
  let plan;
  for (let index = 0; index < 8; index++) {
    plan = await planChatStep(messages, tools);
    assert.equal(plan.kind, 'tool_calls');
    if (plan.calls[0].tool !== 'write') break;
    record(messages, plan.calls[0], '');
  }
  assert.equal(plan.calls[0].tool, 'bash');
  assert.equal(JSON.parse(plan.calls[0].arguments).command, 'sh tests/verify-output.sh');
  record(messages, plan.calls[0], 'Exit code 127\ntests/verify-output.sh:22:scalac:not found\n\n__formal_ai_owned_process_failed_0__\n');
  const bootstrap = await planChatStep(messages, tools);
  assert.equal(bootstrap.kind, 'tool_calls');
  assert.equal(bootstrap.calls[0].tool, 'write');
  assert.equal(JSON.parse(bootstrap.calls[0].arguments).path, '.formal-ai/bootstrap/prerequisite-installer.py');
  record(messages, bootstrap.calls[0], '');
  const retry = await planChatStep(messages, tools);
  assert.equal(retry.calls[0].tool, 'bash');
  const command = JSON.parse(retry.calls[0].arguments).command;
  assert(command.includes('https://api.github.com/repos/scala/scala3/releases/latest'));
  assert(command.endsWith('\n# __formal_ai_prerequisite_retry\nsh tests/verify-output.sh'));
  await installNodeHost(worker, {
    prerequisiteGrant: null
  });
  const denied = await planChatStep(messages.slice(0, -2), tools);
  assert.equal(denied.kind, 'final');
  assert.match(denied.answer, /127|grant|FORMAL_AI_INSTALL_GRANT/u);
});
