import fs from 'node:fs';
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { recipeProgressAfterLatestUser } from '../../../js/agentic/command_reroute.mjs';
import { reportedExitCode, stepOutcome } from '../../../js/agentic/tool_result.mjs';
const observed = JSON.parse(fs.readFileSync(new URL('../fixtures/prerequisite-success-probe-stream.json', import.meta.url)));
before(async () => installNodeHost(new WorkerHost()));
const recipe = {
  path: 'Main.kt',
  source: '// Declared test source; no compiler is executed.',
  supporting_files: [],
  commands: ['sh tests/verify-output.sh']
};
function progress(content, {
  failed = false,
  command = recipe.commands[0]
} = {}) {
  const messages = [{
    role: 'user',
    content: 'Verify the declared program.'
  }, {
    role: 'assistant',
    content: '',
    tool_calls: [{
      id: 'write',
      function: {
        name: 'Write',
        arguments: JSON.stringify({
          path: recipe.path,
          content: recipe.source
        })
      }
    }]
  }, {
    role: 'tool',
    name: 'Write',
    tool_call_id: 'write',
    content: 'written'
  }, {
    role: 'assistant',
    content: '',
    tool_calls: [{
      id: 'run',
      function: {
        name: 'Bash',
        arguments: JSON.stringify({
          command
        })
      }
    }]
  }, {
    role: 'tool',
    name: 'Bash',
    tool_call_id: 'run',
    content,
    is_error: failed
  }];
  return recipeProgressAfterLatestUser(messages, 'Write', recipe, recipe.commands);
}
test('actual hosted probe failure followed by recovery data advances once without inventing an exit', () => {
  assert.equal(reportedExitCode(observed.content), null);
  assert.equal(stepOutcome(observed.content), 'unreported');
  const result = progress(observed.content);
  assert.equal(result.failure, null);
  assert.equal(result.commands_done, 1);
  assert.equal(result.command_outputs[0], observed.content);
});
test('held-out nested diagnostics are structured data rather than outer shell failure prose', () => {
  for (const text of ['error: invalid argument', 'permission denied', 'no such file or directory', 'English Русский हिन्दी 中文 Español']) {
    const output = [JSON.stringify({
      phase: 'probe',
      exit_code: 1,
      output: text
    }), JSON.stringify({
      phase: 'probe',
      exit_code: 0,
      output: 'ready'
    }), 'requested output'].join('\n');
    assert.equal(reportedExitCode(output), null);
    const result = progress(output);
    assert.equal(result.failure, null, text);
    assert.equal(result.commands_done, 1);
  }
});
test('outer process status and explicit transport failures retain authority over probe success', () => {
  for (const content of ['Exit Code: 1\n' + observed.content, JSON.stringify({
    exit_code: 2,
    stdout: observed.content
  }), JSON.stringify({
    error: 'permission denied',
    stdout: 'ready'
  })]) {
    const result = progress(content);
    assert.notEqual(result.failure, null);
    assert.equal(result.commands_done, 0);
  }
  assert.notEqual(progress(observed.content, {
    failed: true
  }).failure, null);
});
test('unstructured failures, malformed records and unrelated commands cannot become success evidence', () => {
  for (const text of ['Error: permission denied', 'Failed: compiler stopped', 'command timed out', '{broken error: invalid argument']) {
    const result = progress(observed.content + '\n' + text);
    assert.notEqual(result.failure, null, text);
    assert.equal(result.commands_done, 0);
  }
  const foreign = progress(observed.content, {
    command: 'printf unrelated'
  });
  assert.equal(foreign.commands_done, 0);
});
