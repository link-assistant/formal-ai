// PR #1188 G116-G118: path operands never supply operation or text-unit cues.
import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';

let semanticShellCommandForTask;
let planChatStep;
before(async () => {
  await installNodeHost(new WorkerHost());
  ({ semanticShellCommandForTask } = await import('../../../js/agentic/shell_command.mjs'));
  ({ planChatStep } = await import('../../../js/agentic/planner.mjs'));
});

test('file names containing operation and text-unit words remain copy operands', () => {
  for (const word of ['rename', 'move', 'delete', 'remove', 'word', 'line', 'text', 'javascript']) {
    for (const prefix of ['', '/tmp/']) {
      const source = prefix + word + '-source.txt';
      const target = prefix + word + '-destination.mjs';
      for (const [opening, closing] of [['', ''], ['«', '»'], ['“', '”']]) {
        const prompt = 'Copy ' + opening + source + closing + ' to ' + opening + target + closing;
        assert.equal(semanticShellCommandForTask(prompt), 'cp ' + source + ' ' + target, prompt);
      }
    }
  }
});

test('bare seeded delete verbs resolve a file and never text within a file', () => {
  for (const verb of ['Delete', 'Remove', 'удали', 'удалить', 'हटाओ', '删除', '移除', 'elimina', 'borra']) {
    assert.equal(semanticShellCommandForTask(verb + ' malformed-regression.test.mjs'), 'rm malformed-regression.test.mjs');
  }
  for (const prompt of [
    'Delete the word rename from note.txt',
    'Remove the line «rename source.txt» from note.txt',
    'Move lines 2-3 of note.txt to the end of other.txt',
  ]) assert.equal(semanticShellCommandForTask(prompt), null, prompt);
});

test('the actual planner copies a cue-bearing file and verifies that the source survives', async () => {
  const source = 'rename-shards.txt';
  const target = 'rename-shards.mjs';
  const files = new Map([[source, 'original content\n']]);
  const messages = [{ role: 'user', content: 'Copy ' + source + ' to ' + target }];
  const commands = [];
  let answer;
  for (let step = 0; step < 16; step += 1) {
    const plan = await planChatStep(messages, ['bash']);
    assert.ok(plan, 'the instruction is planned');
    if (plan.kind === 'final') { answer = plan.answer; break; }
    const call = plan.calls[0];
    assert.equal(call.tool, 'bash');
    const { command } = JSON.parse(call.arguments);
    commands.push(command);
    let result = '';
    const presence = /^test (! )?-e (\S+)$/u.exec(command);
    if (presence && files.has(presence[2]) === Boolean(presence[1])) result = 'Output: \nError: \nExit Code: 1';
    if (command === 'cp ' + source + ' ' + target) files.set(target, files.get(source));
    const identifier = 'call_' + step;
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id: identifier, type: 'function', function: { name: call.tool, arguments: call.arguments } }] });
    messages.push({ role: 'tool', tool_call_id: identifier, content: result });
  }
  assert.equal(files.get(source), 'original content\n');
  assert.equal(files.get(target), files.get(source));
  assert.ok(commands.includes('cp ' + source + ' ' + target));
  assert.ok(commands.includes('test -e ' + source));
  assert.ok(!commands.some((command) => command.startsWith('mv ')));
  assert.ok(answer, 'verified copy has a final answer');
});
