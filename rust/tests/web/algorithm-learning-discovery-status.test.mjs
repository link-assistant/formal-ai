import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { captureCommand, shellQuote, commandPayload } from '../../../js/agentic/algorithm_learning.mjs';
await installNodeHost(new WorkerHost());
test('actual diagnostic stderr proves discovery status while source bytes and failed receipts stay refused', () => {
  const program = "process.stderr.write('Algorithm discovery diagnostic\\n');";
  const command = shellQuote(process.execPath) + ' -e ' + shellQuote(program);
  const wrapper = captureCommand(command);
  const actual = spawnSync('/bin/sh', ['-c', wrapper], {
    encoding: 'utf8'
  });
  assert.equal(actual.status, 0);
  const receipt = JSON.parse(actual.stdout);
  assert.equal(receipt.stderr, 'Algorithm discovery diagnostic\n');
  const messages = raw => [{
    role: 'user',
    content: 'observe actual discovery process'
  }, {
    role: 'assistant',
    tool_calls: [{
      id: 'real',
      type: 'function',
      function: {
        name: 'Bash',
        arguments: JSON.stringify({
          command: wrapper
        })
      }
    }]
  }, {
    role: 'tool',
    name: 'Bash',
    tool_call_id: 'real',
    content: raw
  }];
  assert.equal(commandPayload(messages(actual.stdout), wrapper, null, true), '');
  assert.equal(commandPayload(messages(actual.stdout), wrapper, null), null);
  const negatives = {
    exit_code: 7,
    signal: 'SIGTERM',
    complete: false,
    truncated: true,
    timed_out: true,
    aborted: true,
    error: 'failure',
    stdout: 5,
    stderr: 5,
    is_error: true,
    isError: true,
    ok: false,
    success: false,
    stream_complete: false,
    schema: 'unknown'
  };
  for (const [field, value] of Object.entries(negatives)) assert.equal(commandPayload(messages(JSON.stringify({
    ...receipt,
    [field]: value
  })), wrapper, null, true), null, field);
  for (const field of ['exit_code', 'signal', 'complete', 'truncated', 'timed_out', 'aborted', 'error', 'stdout', 'stderr']) {
    const value = {
      ...receipt
    };
    delete value[field];
    assert.equal(commandPayload(messages(JSON.stringify(value)), wrapper, null, true), null, 'missing ' + field);
  }
  for (const changed of [messages(actual.stdout).map(m => m.role === 'tool' ? {
    ...m,
    tool_call_id: 'wrong'
  } : m), messages(actual.stdout).map(m => m.role === 'tool' ? {
    ...m,
    name: 'Read'
  } : m), messages(actual.stdout).map(m => m.role === 'tool' ? {
    ...m,
    is_error: true
  } : m), [...messages(actual.stdout), {
    role: 'user',
    content: 'new task'
  }]]) assert.equal(commandPayload(changed, wrapper, null, true), null);
  assert.equal(commandPayload(messages(JSON.stringify({
    ...receipt,
    command: 'wrong'
  })), wrapper, null, true), null);
  const failure = spawnSync('/bin/sh', ['-c', captureCommand(shellQuote(process.execPath) + " -e " + shellQuote("process.stderr.write('real failure');process.exit(7)"))], {
    encoding: 'utf8'
  });
  assert.equal(failure.status, 7);
  assert.equal(JSON.parse(failure.stdout).exit_code, 7);
});
