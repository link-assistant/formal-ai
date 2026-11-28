import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../js/server/worker-host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {planChatStep} from '../../js/agentic/planner.mjs';
import {shellCommandForTask, explicitPassthroughCommand} from '../../js/agentic/shell_command.mjs';

before(async () => { await installNodeHost(new WorkerHost()); });

const commands = [
  "env sh -c 'python3 /tmp/issue-1580-research/refresh-default-branch-for-current-batch.py'",
  "env node /tmp/research-check.mjs",
  "/usr/bin/env sh -c 'printf research'",
];

test('explicit environment commands retain research-shaped arguments verbatim', () => {
  for (const command of commands) {
    const request = 'Run ' + command;
    assert.equal(shellCommandForTask(request), command, request);
    assert.equal(explicitPassthroughCommand(request), command, request);
  }
});

test('the actual planner selects one exact shell call for environment commands', async () => {
  for (const command of commands) {
    const plan = await planChatStep([{role:'user',content:'Run ' + command}], ['exec_command','web_search']);
    assert.equal(plan.calls.length, 1, command);
    assert.equal(plan.calls[0].tool, 'exec_command', command);
    assert.equal(JSON.parse(plan.calls[0].arguments).command, command, command);
  }
});

test('environment commands still honor command prohibition', () => {
  const request = "Do not run env sh -c 'printf research'.";
  assert.equal(shellCommandForTask(request), null);
  assert.equal(explicitPassthroughCommand(request), null);
});
