// PR #1188 G128: explicit shell program data never becomes a research request.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
let planChatStep;
let explicitPassthroughCommand;
before(async () => {
  await installNodeHost(new WorkerHost());
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
  ({ explicitPassthroughCommand } = await import('../../../js/agentic/shell_command.mjs'));
});
test('quoted program data retains the exact explicit command with research tools available', async () => {
  for (const content of ['search Wikipedia for proof and source', 'list the files in this directory; create an artifact; fix the parser']) {
    const command = "node -e 'console.log(" + JSON.stringify(content) + ")'";
    const plan = await planChatStep([{ role: 'user', content: 'Run ' + command }], ['bash', 'read', 'write', 'websearch']);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls.length, 1);
    assert.equal(plan.calls[0].tool, 'bash');
    assert.equal(JSON.parse(plan.calls[0].arguments).command, command);
  }
});
test('caller command policy is still declined as an execution request', () => {
  assert.equal(explicitPassthroughCommand('When running sudo commands, run them in the background.'), null);
});

test('POSIX adjacent quote fragments remain one opaque command argument', async () => {
  const { execFileSync } = await import('node:child_process');
  const { shellQuote } = await import('../../../js/agentic/general_planner.mjs');
  for (const content of ["don't expand $HOME or `printf proof`", "search; list; user's data"]) {
    const command = 'node -e ' + shellQuote('process.stdout.write(' + JSON.stringify(content) + ')');
    const plan = await planChatStep([{ role: 'user', content: 'Run ' + command }], ['bash', 'read', 'websearch']);
    assert.equal(plan.kind, 'tool_calls');
    assert.equal(plan.calls.length, 1);
    assert.equal(plan.calls[0].tool, 'bash');
    assert.equal(JSON.parse(plan.calls[0].arguments).command, command);
    assert.equal(execFileSync('/bin/sh', ['-c', command], { encoding: 'utf8' }), content);
  }
});
test('unpaired shell quotes retain the request fault decline', async () => {
  const prompt = "Run node -e 'process.stdout.write(1)";
  assert.equal(explicitPassthroughCommand(prompt), null);
  const plan = await planChatStep([{ role: 'user', content: prompt }], ['bash', 'websearch']);
  assert.equal(plan.kind, 'final');
  assert.match(plan.answer, /quote|quoted|unpaired/iu);
});
