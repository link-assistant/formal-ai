import assert from 'node:assert/strict';
import { test } from 'node:test';
import vm from 'node:vm';
import { exportedNames, factoryOf, importClosure } from '../../../scripts/generate-worker-crate-modules.mjs';

test('a named re-export retains its public name and dependency in a generated factory', () => {
  const source = "export { readLines as lines } from './reader.mjs';\nexport const label = 'rows';\n";
  assert.deepEqual(exportedNames(source), ['label', 'lines']);
  assert.deepEqual(importClosure(['root.mjs'], (name) => name === 'root.mjs' ? source : 'export function readLines() {}\n'), ['reader.mjs', 'root.mjs']);
  const realm = { self: {} };
  vm.runInNewContext(factoryOf('root.mjs', source), realm);
  const readLines = () => ['one', 'two'];
  const result = realm.self.FORMAL_AI_CRATE_FACTORIES['root.mjs']((name) => {
    assert.equal(name, 'reader.mjs');
    return { readLines };
  });
  assert.equal(result.lines, readLines);
  assert.equal(result.label, 'rows');
});
