import { before, test } from 'node:test';
import assert from 'node:assert/strict';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { closedArithmeticDeclaration } from '../../../js/agentic/module_function.mjs';
import { ownedAdditiveLiteralFrame } from '../../../js/agentic/planner/owned_goals.mjs';
import { planChatStep } from '../../../js/agentic/planner.mjs';

before(async () => installNodeHost(new WorkerHost()));
const tools = ['read', 'write', 'edit', 'bash'];
const declaration = 'Add a function difference(left, right) that returns left - right to result.mjs.';

for (const operator of ['+', '-', '*', '/', '%']) {
  test('closed operator ownership: ' + operator, () => {
    const request = declaration.replace('left - right', 'left ' + operator + ' right');
    assert.equal(closedArithmeticDeclaration(request), true);
    assert.equal(ownedAdditiveLiteralFrame(request), null);
  });
}

for (const request of [
  'Do not write. ' + declaration,
  'Do not read. ' + declaration,
  declaration + ' Do not write.',
  declaration + ' Do not read.',
  declaration + ' Deploy it.',
  declaration + ' and perform an unknown task.',
  declaration + ' Read another.mjs first.',
  declaration + ' Run node --check result.mjs.',
  declaration + ' after future approval.',
  declaration.replace('to result.mjs', 'and deploy to result.mjs'),
  declaration.replace('Add', 'I will add'),
  declaration.replace('left - right', 'left - unknown'),
  declaration + '"',
  '"' + declaration,
]) {
  test('unconsumed scope remains refused: ' + request, async () => {
    assert.equal(closedArithmeticDeclaration(request), false);
    const plan = await planChatStep([{ role: 'user', content: request }], tools);
    const calls = plan?.calls ?? [];
    assert.equal(calls.some(call => ['write', 'edit', 'bash'].includes(call.tool)), false);
    if (!request.includes('Read another.mjs')) assert.equal(calls.length, 0);
  });
}

test('explicit quoted function prose is literal data', () => {
  const content = 'function difference(left, right) that returns left - right';
  const request = 'Append «' + content + '» to result.mjs.';
  assert.equal(closedArithmeticDeclaration(request), false);
  const literal = ownedAdditiveLiteralFrame(request);
  assert.equal(literal?.content, content);
  assert.equal(literal?.atEnd, true);
});
