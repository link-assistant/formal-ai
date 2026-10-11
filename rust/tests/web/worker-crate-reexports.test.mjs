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


test('local export aliases retain the existing callable binding', () => {
  const source = "const original = (value) => value + 1;\nexport { original as exposed };\n";
  assert.deepEqual(exportedNames(source), ['exposed']);
  const realm = { self: {} };
  vm.runInNewContext(factoryOf('local.mjs', source), realm);
  const result = realm.self.FORMAL_AI_CRATE_FACTORIES['local.mjs'](() => { throw new Error('no dependency expected'); });
  assert.equal(result.exposed(41), 42);
});

test('an imported binding can be exposed through a local export clause', () => {
  const source = "import { readLines } from './reader.mjs';\nexport {\n  readLines as lines,\n};\n";
  assert.deepEqual(exportedNames(source), ['lines']);
  assert.deepEqual(importClosure(['root.mjs'], (name) => name === 'root.mjs' ? source : 'export function readLines() {}\n'), ['reader.mjs', 'root.mjs']);
  const realm = { self: {} };
  vm.runInNewContext(factoryOf('root.mjs', source), realm);
  const readLines = () => ['actual dependency'];
  const result = realm.self.FORMAL_AI_CRATE_FACTORIES['root.mjs']((name) => { assert.equal(name, 'reader.mjs'); return { readLines }; });
  assert.equal(result.lines, readLines);
});

test('unsupported module syntax is rejected before browser artifacts can be written', () => {
  assert.throws(() => factoryOf('unsupported.mjs', 'export default 42;\n'), SyntaxError);
});
