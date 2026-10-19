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

test('a backtick inside a quoted JavaScript string is literal shell data', async () => {
  const output='odd '+String.fromCharCode(96)+' literal';
  const code='process.stdout.write('+JSON.stringify(output)+')';
  const command="node -e '"+code+"'";
  const plan=await planChatStep([{role:'user',content:'Run '+command}],['bash']);
  assert.equal(plan.calls[0].tool,'bash');assert.equal(JSON.parse(plan.calls[0].arguments).command,command);
  const {execFileSync}=await import('node:child_process');assert.equal(execFileSync('/bin/sh',['-c',command],{encoding:'utf8'}),output);
});
test('semantic explanatory listing suffix and web search use their actual capability', () => {
  assert.equal(explicitPassthroughCommand('Run ls to list files here'),'ls');
  assert.equal(explicitPassthroughCommand('выполни веб поиск rust ownership'),null);
  const code='console.log('+JSON.stringify('выполни веб поиск rust ownership')+')';
  const opaque="node -e '"+code+"'";
  assert.equal(explicitPassthroughCommand('Run '+opaque),opaque);
});

// The unchanged R916-01 failure receipt must retain its actual command owner.
test('Spanish command references preserve the original failed receipt', async () => {
  const request = 'Ejecuta el comando y dime qué pasó, por favor';
  assert.equal(explicitPassthroughCommand(request), null);
  const messages = [
    { role: 'user', content: request },
    { role: 'assistant', content: null, tool_calls: [{
      id: 'original916', type: 'function',
      function: { name: 'bash', arguments: JSON.stringify({ command: 'cat hello.txt' }) },
    }] },
    { role: 'tool', tool_call_id: 'original916', name: 'bash', content: [
      'Command: cat hello.txt', 'Directory: (root)', 'Output: (empty)',
      'Error: cat: hello.txt: No such file or directory', 'Exit Code: 1', 'Signal: 0',
    ].join('\n') },
  ];
  for (const tools of [['bash'], ['bash', 'read_file']]) {
    const result = await planChatStep(messages, tools);
    assert.equal(result.kind, 'final', JSON.stringify(result));
    assert.match(result.answer, /cat hello.txt/u);
    assert.match(result.answer, /No such file or directory/u);
    assert.match(result.answer, /1/u);
  }
});
test('unknown executables and quoted Spanish command operands stay literal', () => {
  for (const command of [
    'frobnicator --version',
    "printf '%s' 'el comando y dime qué pasó'",
    'node -e "console.log(1)"',
  ]) assert.equal(explicitPassthroughCommand('Ejecuta ' + command), command);
});
test('a named real command keeps its actual executable and operands', () => {
  assert.equal(explicitPassthroughCommand('Ejecuta el comando git status'), 'git status');
});
