// Actual canonical operand binding; no compiler, external fetch or execution claim.
import assert from 'node:assert/strict';
import {before, test} from 'node:test';
import {WorkerHost} from '../../js/server/worker-host.mjs';
const host = new WorkerHost();
before(async () => { await host.boot(); });
const template = 'factorial={{n}}; end={{end_exclusive}}';
for (const [prompt, operand] of [
  ['Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए', null],
  ['Python में एक फ़ंक्शन लिखो जो 5 का फ़ैक्टोरियल लौटाए', 5],
  ['Write one function in Python that returns the factorial of n', null],
  ['Write one function in Python for factorial of five', 5],
  ['Write a Python function for factorial of one', 1],
  ['Write 2 functions in Python to calculate factorial of 5', 5],
]) {
  test(prompt, {timeout: 15000}, async () => {
    const result = await host.run(`bindCatalogProgram(${JSON.stringify(prompt)}, 'factorial', 'python', ${JSON.stringify(template)})`);
    if (operand === null) assert.equal(result, null, 'an artifact count cannot bind the unknown n');
    else assert.deepEqual(result, {
      code: `factorial=${operand}; end=${operand + 1}`,
      output: operand === 5 ? '120' : '1',
    });
  });
}

test('the whole request retains missing-operand guidance without executable code', {timeout: 15000}, async () => {
  const result = await host.solve('Python में एक फ़ंक्शन लिखो जो n का फ़ैक्टोरियल लौटाए');
  assert.equal(result.intent, 'write_program_skill_gap');
  assert.equal(result.content, 'इस अनुरोध के लिए कोई सत्यापित स्रोत-कथन प्राप्त नहीं हुआ, इसलिए कुछ भी नहीं लिखा गया।');
  assert.ok(result.evidence.includes('program_parameter:operand_unbound'));
  assert.doesNotMatch(result.content, /```(?:python|py)\b/u);
  assert.equal(result.programExecution ?? null, null);
});
