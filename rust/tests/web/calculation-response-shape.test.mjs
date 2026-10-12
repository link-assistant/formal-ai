import assert from 'node:assert/strict';
import test from 'node:test';
import {WorkerHost} from '../../../js/server/worker-host.mjs';

const host = new WorkerHost();

for (const [expression, expected] of [
  ['31 + 6', '37'],
  ['-7 * 4', '-28'],
  ['1 / 8', '0.125'],
  ['2^128', '340282366920938463463374607431768211456'],
]) {
  test(`a complete numeric response clause preserves ${expression}`, async () => {
    const request = `Calculate ${expression}. Return only the numeric result.`;
    const response = await host.solve(request);
    assert.equal(response.content, expected);
    assert.equal(response.intent, 'calculation');
    assert.ok(response.evidence.some((entry) => entry.startsWith('calculation_derivation:')));
    assert.ok(response.solverEvents || response.rawSolverEvents);
  });
}

for (const request of [
  'Calculate 31 + 6. Return only the numeric result. Explain why.',
  'Calculate 31 + 6 and delete files. Return only the numeric result.',
  'Calculate 31 + 6. "Return only the numeric result."',
  'Calculate 31 + 6. Return only code.',
  'Explain Calculate 31 + 6. Return only the numeric result.',
]) {
  test(`an unowned or conflicting clause does not mint numeric output: ${request}`, async () => {
    const response = await host.solve(request);
    assert.notEqual(response.content, '37');
  });
}

test('the unshaped calculation retains its expression and value', async () => {
  const response = await host.solve('Calculate 31 + 6.');
  assert.equal(response.content, '31 + 6 = 37');
});

test('numeric presentation never replaces an evaluation error', async () => {
  const response = await host.solve('Calculate 1 / 0. Return only the numeric result.');
  assert.notEqual(response.content, 'Infinity');
  assert.notEqual(response.content, 'NaN');
  assert.notEqual(response.content, '0');
});

test('nonprimitive shape operands are refused before coercion callbacks', async () => {
  const observation = await host.run(`(() => {
    let coercions = 0;
    const operand = {toString() { coercions += 1; return '37'; }};
    const request = 'Calculate 31 + 6. Return only the numeric result.';
    const result = numericCalculationResponse(request, '31 + 6', operand);
    return {result, coercions};
  })()`);
  assert.deepEqual(observation, {result: null, coercions: 0});
});
