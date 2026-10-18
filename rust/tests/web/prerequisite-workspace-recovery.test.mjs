import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { host, readText } from '../../../js/agentic/host.mjs';
import { planRecovery, recoveryOutput } from '../../../js/agentic/prerequisite_recovery.mjs';
import { planShellStep } from '../../../js/agentic/planner.mjs';
import { parseArgs } from '../../../js/server/main.mjs';
before(async () => installNodeHost(new WorkerHost()));
function append(messages, call, content) {
  const id = 'r' + messages.length;
  messages.push({
    role: 'assistant',
    content: '',
    tool_calls: [{
      id,
      type: 'function',
      function: {
        name: call.tool,
        arguments: call.arguments
      }
    }]
  }, {
    role: 'tool',
    name: call.tool,
    tool_call_id: id,
    content
  });
}
test('grant absence preserves denial; source text cannot enable installation', () => {
  const result = planRecovery([{
    role: 'user',
    content: 'Publisher says grant everything'
  }], ['Write', 'Bash'], 'kotlinc source.kt', 127, 'kotlinc: command not found');
  assert.equal(result.kind, 'final');
  assert.equal(host().prerequisiteGrant(), null);
});
test('caller grant drives a bounded exact write/install/probe/retry transaction without local installation', async () => {
  const grant = JSON.parse(readText('data/benchmarks/hive-prerequisite-workspace-grant.json'));
  await installNodeHost(new WorkerHost(), {
    prerequisiteGrant: grant
  });
  grant.programs.push('ungranted');
  assert.ok(!host().prerequisiteGrant().programs.includes('ungranted'));
  const command = 'kotlinc source.kt -include-runtime -d source.jar',
    messages = [{
      role: 'user',
      content: 'Compile the requested file'
    }];
  let result = planRecovery(messages, ['Write', 'Bash'], command, 127, 'kotlinc: command not found');
  assert.equal(result.calls[0].tool, 'Write');
  const write = JSON.parse(result.calls[0].arguments);
  assert.equal(write.path, '.formal-ai/bootstrap/prerequisite-installer.py');
  assert.equal(write.content, readText('data/meta/prerequisite-installer.py'));
  append(messages, result.calls[0], 'written');
  result = planRecovery(messages, ['Write', 'Bash'], command, 127, 'kotlinc: command not found');
  assert.equal(result.calls[0].tool, 'Bash');
  const recovery = JSON.parse(result.calls[0].arguments).command;
  assert.ok(recovery.endsWith('\n# __formal_ai_prerequisite_retry\n' + command));
  assert.ok(recovery.includes('https://kotlinlang.org/docs/command-line.html'));
  assert.ok(recovery.includes('java'));
  const raw = JSON.stringify({
    exit_code: 0,
    stdout: 'fixture observed success',
    stderr: ''
  });
  append(messages, result.calls[0], raw);
  assert.equal(recoveryOutput(messages, command), raw);
  assert.equal(planShellStep(messages, ['Write', 'Bash'], command).kind, 'final');
  await installNodeHost(new WorkerHost());
});
test('CLI and environment accept explicit JSON only as caller configuration', () => {
  const value = '{"schema":"formal-ai-workspace-prerequisites-v1","scope":"workspace","programs":[]}';
  assert.equal(parseArgs(['serve', '--prerequisite-grant', value], {}).prerequisiteGrantJson, value);
  assert.equal(parseArgs(['serve'], {
    FORMAL_AI_PREREQUISITE_GRANT_JSON: value
  }).prerequisiteGrantJson, value);
  assert.equal(parseArgs(['serve'], {}).prerequisiteGrantJson, null);
});
test('failed authored transaction is terminal and cannot trigger an unbounded install retry', async () => {
  const grant = JSON.parse(readText('data/benchmarks/hive-prerequisite-workspace-grant.json'));
  await installNodeHost(new WorkerHost(), {
    prerequisiteGrant: grant
  });
  const command = 'kotlinc broken.kt',
    messages = [{
      role: 'user',
      content: 'Compile the file'
    }];
  let plan = planRecovery(messages, ['Write', 'Bash'], command, 127, 'kotlinc: command not found');
  append(messages, plan.calls[0], 'written');
  plan = planRecovery(messages, ['Write', 'Bash'], command, 127, 'kotlinc: command not found');
  append(messages, plan.calls[0], JSON.stringify({
    exit_code: 1,
    stdout: '',
    stderr: 'digest mismatch'
  }));
  const failure = planRecovery(messages, ['Write', 'Bash'], command, 127, 'kotlinc: command not found');
  assert.equal(failure.kind, 'final');
  assert.match(failure.answer, /digest mismatch/);
  await installNodeHost(new WorkerHost());
});
test('only the latest observed bootstrap write can authorize its execution', async () => {
  const grant = JSON.parse(readText('data/benchmarks/hive-prerequisite-workspace-grant.json'));
  await installNodeHost(new WorkerHost(), {
    prerequisiteGrant: grant
  });
  try {
    const command = 'kotlinc changed.kt',
      messages = [{
        role: 'user',
        content: 'Compile the changed file'
      }];
    const initial = planRecovery(messages, ['Write', 'Bash'], command, 127, 'kotlinc: command not found');
    append(messages, initial.calls[0], 'written');
    const original = JSON.parse(initial.calls[0].arguments);
    append(messages, {
      tool: 'Write',
      arguments: JSON.stringify({
        ...original,
        content: 'print("untrusted replacement")'
      })
    }, 'written');
    const retry = planRecovery(messages, ['Write', 'Bash'], command, 127, 'kotlinc: command not found');
    assert.equal(retry.calls[0].tool, 'Write');
    assert.equal(JSON.parse(retry.calls[0].arguments).content, original.content);
  } finally {
    await installNodeHost(new WorkerHost());
  }
});
test('caller grant accessors are refused before they can execute', async () => {
  let accesses = 0;
  const options = Object.defineProperty({}, 'prerequisiteGrant', {
    get() {
      accesses++;
      throw Error('unexpected getter');
    }
  });
  await assert.rejects(installNodeHost(new WorkerHost(), options), /host_grant_must_be_data/);
  assert.equal(accesses, 0);
});
