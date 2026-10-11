import assert from 'node:assert/strict';
import test from 'node:test';
import vm from 'node:vm';
import { compactFactoryLayout } from '../../../scripts/generate-worker-crate-modules.mjs';

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
