import assert from 'node:assert/strict';
import test from 'node:test';
import vm from 'node:vm';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { tokenize } from '../../../scripts/translate-es.mjs';
import { compactFactoryLayout, factoryOf } from '../../../scripts/generate-worker-crate-modules.mjs';

test('factory layout conserves literal and comment bytes and ASI newlines', () => {
  const source = '/* two\n\ncomment */\n\nconst raw = `a\n\nb`;\n\n// fixed\n\nconst pattern = /a+/u;\n\nfunction value() { return\n\nraw; }\n';
  const result = compactFactoryLayout(source, 12);
  assert.ok(result.includes('/* two\n\ncomment */'));
  assert.ok(result.includes('`a\n\nb`'));
  assert.ok(result.includes('// fixed\n'));
  assert.ok(result.includes('return\nraw;'));
  assert.equal(vm.runInNewContext(source + 'value()'), undefined);
  assert.equal(vm.runInNewContext(result + 'value()'), undefined);
  assert.equal(compactFactoryLayout(result, 12), result);
});

test('factory layout preserves under-budget bytes and refuses insufficient budgets', () => {
  const source = 'const x = 1;\n\nconst y = 2;\n';
  assert.equal(compactFactoryLayout(source, 3), source);
  assert.throws(() => compactFactoryLayout(source, 1), /cannot fit/);
  for (const value of [0, -1, 1.5, Infinity, Number.MAX_SAFE_INTEGER + 1]) {
    assert.throws(() => compactFactoryLayout(source, value), /Invalid/);
  }
});

test('current provenance factory fits its unchanged 47-line source-owned budget', async () => {
  const root = fileURLToPath(new URL('../../../', import.meta.url));
  const source = readFileSync(root + 'js/agentic/final_result.mjs', 'utf8');
  const budget = readFileSync(root + 'data/meta/worker-line-budget/formal_ai_worker_crate_final_result.lino', 'utf8');
  const ceiling = Number(budget.match(/^\s*ceiling (\d+)\s*$/mu)?.[1]);
  assert.equal(ceiling, 47);
  const normal = factoryOf('final_result.mjs', source);
  const compact = factoryOf('final_result.mjs', source, ceiling);
  assert.ok(compact.split('\n').length - Number(compact.endsWith('\n')) <= ceiling);
  assert.deepEqual(tokenize(compact), tokenize(normal));
  const realm = { self: {} };
  vm.runInNewContext(compact, realm);
  const payload = await import('../../../js/agentic/final_payload.mjs');
  const result = realm.self.FORMAL_AI_CRATE_FACTORIES['final_result.mjs'](name => {
    assert.equal(name, 'final_payload.mjs');
    return payload;
  });
  assert.equal(result.FinalPayloadRole, payload.FinalPayloadRole);
  const resolved = result.resolvedFinalAnswer('observed answer', result.FinalDisposition.Gap, 'actual-origin');
  const projected = result.projectPlan(resolved);
  assert.deepEqual(Object.keys(projected), ['kind', 'answer']);
  assert.equal(result.observedFinalResult(projected), resolved.result);
  assert.equal(result.canDeliverFinal(projected), false);
  projected.answer = 'changed';
  assert.equal(result.observedFinalResult(projected).disposition, result.FinalDisposition.Unknown);
});

test('generated export rows preserve order, aliases and exact executable tokens', () => {
  const source = Array.from({length: 8}, (_, index) => `const local${index} = ${index};`).join('\n')
    + '\nexport { ' + Array.from({length: 8}, (_, index) => `local${index} as public${index}`).join(', ') + ' };\n';
  const normal = factoryOf('aliases.mjs', source);
  const count = text => text.split('\n').length - Number(text.endsWith('\n'));
  const compact = factoryOf('aliases.mjs', source, count(normal) - 4);
  assert.deepEqual(tokenize(compact), tokenize(normal));
  const realm = { self: {} };
  vm.runInNewContext(compact, realm);
  const result = realm.self.FORMAL_AI_CRATE_FACTORIES['aliases.mjs'](() => { throw Error('no import'); });
  assert.deepEqual(Object.keys(result), Array.from({length: 8}, (_, index) => `public${index}`));
  for (let index = 0; index < 8; index++) assert.equal(result[`public${index}`], index);
});
