import assert from 'node:assert/strict';
import { before, test } from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';
import { composeGeneralChangePlan } from '../../../js/agentic/general_planner.mjs';
const tools = ['web_fetch', 'web_search', 'read_file', 'write_file', 'exec_command'];
before(async () => { await installNodeHost(new WorkerHost()); });

test('retrieval phrase owns its overlapping write cue and exact source path', async () => {
  for (const [task, target] of [
    ['escribe el contenido de sample.txt', 'sample.txt'],
    ['Escribe el contenido de notas-α.txt', 'notas-α.txt'],
    ['Hola, escribe el contenido de registro.txt', 'registro.txt'],
    ['输出内容 sample.txt', 'sample.txt'],
  ]) {
    const plan = await planChatStep([{ role: 'user', content: task }], tools);
    assert.equal(plan.kind, 'tool_calls', task);
    assert.equal(plan.calls.length, 1, task);
    assert.equal(plan.calls[0].tool, 'read_file', task);
    assert.equal(JSON.parse(plan.calls[0].arguments).path, target, task);
  }
});

test('a content-free write gains no read or physical write capability', async () => {
  for (const task of ['Create sample.txt', 'Write sample.txt']) {
    assert.equal(await planChatStep([{ role: 'user', content: task }], tools), null, task);
  }
});

test('explicit literal writes retain owned content instead of retrieval precedence', () => {
  for (const [task, expected] of [
    ['Create sample.txt containing «hello».', 'hello'],
    ['Escribe sample.txt con el contenido «hola».', 'hola'],
  ]) {
    const plan = composeGeneralChangePlan(task);
    assert.equal(plan.target, 'sample.txt', task);
    assert.equal(plan.content, expected, task);
    assert.equal(plan.mode, 'literal_file', task);
  }
});
