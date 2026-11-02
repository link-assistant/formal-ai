import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {mkdtempSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {planChatStepResolved} from '../../../js/agentic/planner.mjs';
import {executeResult} from '../../../experiments/js_dogfood/drive.mjs';

before(async () => {
  await installNodeHost(new WorkerHost(), {sourceUsageRustGrammar: false});
});
const tools = ['write_file', 'run_shell_command'];
const content = 'Hello World';
const cases = [
  ['R916-04', 'hello.txt', 'Create a file hello.txt containing exactly: Hello World'],
  ['R916-05', 'greeting.txt', 'Create a file greeting.txt whose content is exactly: Hello World']
];
function append(messages, call, result) {
  const id = 'ladder-shell-' + messages.length;
  messages.push({
    role: 'assistant',
    tool_calls: [{
      id, type: 'function',
      function: {name: call.tool, arguments: call.arguments}
    }]
  }, {
    role: 'tool', tool_call_id: id, name: call.tool, ...result
  });
}
function execute(directory, call) {
  const canonical = {write_file: 'write', run_shell_command: 'bash'}[call.tool];
  assert.ok(canonical, 'only the two advertised ladder tools execute');
  return executeResult(directory, {...call, tool: canonical});
}
function one(plan, tool) {
  assert.equal(plan.kind, 'tool_calls');
  assert.equal(plan.calls.length, 1);
  assert.equal(plan.calls[0].tool, tool);
  return plan.calls[0];
}
function history(prompt) {
  return [{role: 'user', content: prompt}];
}
async function written(directory, prompt, target, names = tools, expected = content) {
  const messages = history(prompt);
  const call = one(await planChatStepResolved(messages, names), 'write_file');
  const args = JSON.parse(call.arguments);
  assert.equal(args.path ?? args.file_path ?? args.filePath, target);
  assert.equal(args.content, expected);
  append(messages, call, execute(directory, call));
  assert.equal(readFileSync(join(directory, target), 'utf8'), expected);
  return messages;
}
function sandbox(t) {
  const directory = mkdtempSync(join(tmpdir(), 'literal-shell-readback-'));
  t.after(() => rmSync(directory, {recursive: true, force: true}));
  return directory;
}
function notVerified(plan) {
  assert.notEqual(plan.result?.disposition, 'finding');
  if (plan.kind === 'final') {
    assert.doesNotMatch(plan.answer, /i created|created and verified|successfully created/i);
  }
}
function targetCat(plan, target) {
  const call = one(plan, 'run_shell_command');
  const command = JSON.parse(call.arguments).command;
  assert.equal(command,
    "printf %s '(' && cat -- '" + target + "' && printf %s ')'");
  return call;
}
for (const [id, target, prompt] of cases) {
  test(id + ' reports completion only after actual write and fresh exact shell readback', async t => {
    const directory = sandbox(t);
    const messages = await written(directory, prompt, target);
    const call = targetCat(await planChatStepResolved(messages, tools), target);
    append(messages, call, execute(directory, call));
    const final = await planChatStepResolved(messages, tools);
    assert.equal(readFileSync(join(directory, target), 'utf8'), content);
    assert.equal(final.kind, 'final');
    assert.equal(final.result?.disposition, 'finding');
    assert.match(final.answer.toLowerCase(),
      /completed the|created and verified|i created|i've created|i have created|successfully created|successfully wrote|verified it with|has been created/u);
    assert.equal(messages.filter(message => message.role === 'tool').length, 2);
  });
}
test('a physical write without Read or Run remains an honest unverified gap', async t => {
  const directory = sandbox(t), [, target, prompt] = cases[0];
  const messages = await written(directory, prompt, target, ['write_file']);
  const final = await planChatStepResolved(messages, ['write_file']);
  assert.equal(final.kind, 'final');
  assert.equal(final.result?.disposition, 'gap');
  notVerified(final);
  assert.equal(readFileSync(join(directory, target), 'utf8'), content);
});
const invalidResults = [
  ['nonzero exit', {content: 'Output: (Hello World)\nExit Code: 1', is_error: true}],
  ['truncated stdout', {content: JSON.stringify({
    stdout: '(' + content + ')', exit_code: 0, truncated: true
  })}],
  ['wrong stdout', {content: 'Output: (Hello world)\nExit Code: 0'}],
  ['extra newline stdout', {content: 'Output: (Hello World\n)\nExit Code: 0'}],
  ['missing status', {content: '(' + content + ')'}],
  ['provider failure', {content: JSON.stringify({
    stdout: '(' + content + ')', exit_code: 0, is_error: true
  })}]
];
for (const [label, result] of invalidResults) {
  test(label + ' cannot certify a physically written file', async t => {
    const directory = sandbox(t), [, target, prompt] = cases[0];
    const messages = await written(directory, prompt, target);
    const call = targetCat(await planChatStepResolved(messages, tools), target);
    execute(directory, call);
    append(messages, call, result);
    notVerified(await planChatStepResolved(messages, tools));
    assert.equal(readFileSync(join(directory, target), 'utf8'), content);
  });
}
test('a successful unrelated shell read cannot certify the target', async t => {
  const directory = sandbox(t), [, target, prompt] = cases[0];
  const messages = await written(directory, prompt, target);
  writeFileSync(join(directory, 'other.txt'), content);
  const call = {tool: 'run_shell_command', arguments: JSON.stringify({command: 'cat other.txt'})};
  append(messages, call, execute(directory, call));
  const next = await planChatStepResolved(messages, tools);
  notVerified(next);
  targetCat(next, target);
  assert.equal(readFileSync(join(directory, target), 'utf8'), content);
});
test('a matching shell observation before the write is stale', async t => {
  const directory = sandbox(t), [, target, prompt] = cases[0];
  writeFileSync(join(directory, target), content);
  const messages = history(prompt);
  const old = {tool: 'run_shell_command', arguments: JSON.stringify({command: 'cat ' + target})};
  append(messages, old, execute(directory, old));
  const call = one(await planChatStepResolved(messages, tools), 'write_file');
  append(messages, call, execute(directory, call));
  const next = await planChatStepResolved(messages, tools);
  notVerified(next);
  targetCat(next, target);
  assert.equal(readFileSync(join(directory, target), 'utf8'), content);
});

function qwenEnvelope(call, receipt) {
  const prefix = 'Output: ', suffix = '\nExit Code: 0';
  assert.ok(receipt.content.startsWith(prefix));
  assert.ok(receipt.content.endsWith(suffix));
  const stdout = receipt.content.slice(prefix.length, -suffix.length);
  return {
    content: [
      'Command: ' + JSON.parse(call.arguments).command,
      'Directory: (root)',
      // Faithful to the existing Python ladder: edge trim is transport behavior.
      'Output: ' + (stdout.trim() || '(empty)'),
      'Error: (none)',
      'Exit Code: 0',
      'Signal: 0',
      'Process Group PGID: 0'
    ].join('\n')
  };
}
for (const [id, target, prompt] of cases) {
  test(id + ' verifies actual bytes through the faithful qwen shell envelope', async t => {
    const directory = sandbox(t);
    const messages = await written(directory, prompt, target);
    const call = targetCat(await planChatStepResolved(messages, tools), target);
    append(messages, call, qwenEnvelope(call, execute(directory, call)));
    const final = await planChatStepResolved(messages, tools);
    assert.equal(final.kind, 'final');
    assert.equal(final.result?.disposition, 'finding');
    assert.equal(readFileSync(join(directory, target), 'utf8'), content);
  });
}
for (const [label, body] of [
  ['Unicode and edge newlines', '\nαβ🙂\n'],
  ['leading and trailing spaces', '  Hello World  '],
  ['JSON-looking body', '{"is_error":true,"exit_code":1}']
]) {
  test(label + ' survives qwen edge trimming inside the readback frame', async t => {
    const directory = sandbox(t), target = 'edge.txt';
    const prompt = 'Create edge.txt with the following exact content: «' + body + '».';
    const messages = await written(directory, prompt, target, tools, body);
    const call = targetCat(await planChatStepResolved(messages, tools), target);
    const receipt = qwenEnvelope(call, execute(directory, call));
    assert.ok(receipt.content.includes('Output: (' + body + ')\nError:'));
    append(messages, call, receipt);
    const final = await planChatStepResolved(messages, tools);
    assert.equal(final.kind, 'final');
    assert.equal(final.result?.disposition, 'finding');
    assert.equal(readFileSync(join(directory, target), 'utf8'), body);
  });
}
