// Issue #1580: calculator question cues must reach algebra substitution.
import assert from 'node:assert/strict';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';

const host = new WorkerHost();
for (const [prompt, expected] of [
  ['If x = 3 and y = 4, what is x*x + 2*y?', '17'],
  ['If x = 4 and y = 6, what is x*x + 2*y?', '28'],
  ['Given y = 4 and x = 3, evaluate (x^2 + 2y).', '17'],
  ['If x = 2 and y = 5, then what is the value of (x^4 + 2y^2) / 6?', '11'],
]) {
  test(`bound algebra operands evaluate: ${prompt}`, async () => {
    const response = await host.solve(prompt);
    assert.equal(response.intent, 'algebra_substitution', response.content);
    assert.ok(response.content.endsWith('= ' + expected + '.'), response.content);
    assert.ok(response.evidence.some((entry) => entry.startsWith('composition:substitution:')));
    assert.ok(response.evidence.some((entry) => entry.startsWith('composition:evaluation:')
      && entry.endsWith('= ' + expected)), JSON.stringify(response.evidence));
  });
}

for (const prompt of [
  'What is x*x + 2*y?',
  'If x = 3, what is x*x + 2*z?',
  'If x = 3 and y = 4, explain algebra.',
  'If x = 3 and y = 4.',
]) {
  test(`incomplete algebra operands do not claim substitution: ${prompt}`, async () => {
    const response = await host.solve(prompt);
    assert.notEqual(response.intent, 'algebra_substitution');
  });
}
