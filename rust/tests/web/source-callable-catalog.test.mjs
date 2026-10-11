import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import test from 'node:test';
import { observeCallableModule, bindObservedExport, validateObservedCallable } from '../../../scripts/self-translation/source-callable-catalog.mjs';

const observe = (text, name = 'module.mjs') => observeCallableModule(text, 'file:///observed/' + name);

test('source catalog excludes commented/string exports and keeps full Unicode spans', () => {
  const source = '// export function decoy() {}\nconst text = "export function fake() {}";\n// 文\nexport function real(input) { return input; }';
  const catalog = observe(source);
  assert.deepEqual(catalog.declarations.map((entry) => entry.name), ['real']);
  const entry = catalog.declarations[0];
  assert.equal(entry.source, 'export function real(input) { return input; }');
  assert.equal(source.slice(entry.span.start, entry.span.end), entry.source);
  assert.equal(Buffer.from(source).subarray(entry.span.byteStart, entry.span.byteEnd).toString(), entry.source);
  assert.notEqual(entry.span.byteStart, entry.span.start);
  assert.equal(validateObservedCallable(entry, source), true);
  assert.equal(validateObservedCallable(entry, source + '\n'), false);
  assert.equal(validateObservedCallable(entry, source.replace('return input', 'return null')), false);
});

test('source catalog resolves actual import/export aliases and refuses unobserved origin', () => {
  const original = observe('export function pick(value) { return value; }', 'original.mjs');
  const facade = observe("import { pick as chosen } from './original.mjs'; export { chosen as publicPick };", 'facade.mjs');
  const bound = bindObservedExport([original, facade], facade.moduleId, 'publicPick', 'readSelected');
  assert.equal(bound.kind, 'observed');
  assert.equal(bound.declaration.name, 'pick');
  assert.equal(bound.declaration.moduleId, original.moduleId);
  assert.equal(bound.import.exported, 'publicPick');
  assert.equal(bound.import.local, 'readSelected');
  assert.equal(bindObservedExport([facade], facade.moduleId, 'publicPick', 'readSelected').reason, 'module-unobserved');
  assert.equal(bindObservedExport([original], original.moduleId, 'pick', 'input', ['input']).reason, 'binding-collision');
});

test('source catalog does not certify types/effects or silently bind shadowed aliases', () => {
  const source = "import { read as input } from './reader.mjs'; export function run(input) { input.value = 4; return input; }";
  const entry = observe(source).declarations[0];
  assert.deepEqual(entry.shadowedImports, ['input']);
  assert.deepEqual(entry.contract, { inputs: null, result: null, callEffects: null, moduleEffects: null, status: 'unknown' });
  const invalid = observe("import { read as input } from './reader.mjs'; function input() {} export { input };");
  assert.equal(bindObservedExport([invalid], invalid.moduleId, 'input', 'reader').reason, 'ambiguous-binding');
  const forward = observe("export { read } from './reader.mjs';");
  assert.equal(forward.gaps[0].kind, 'unobserved-reexport');
  assert.equal(bindObservedExport([forward], forward.moduleId, 'read', 'reader').kind, 'gap');
});

test('real formalization callables retain observed bodies and unknown composition contracts', () => {
  for (const [path, name] of [['solver_formalization.mjs', 'selectedCandidate'], ['translation_formalization.mjs', 'candidateCompactSummary']]) {
    const url = new URL('../../../js/agentic/crate/' + path, import.meta.url);
    const source = fs.readFileSync(url, 'utf8');
    const catalog = observeCallableModule(source, url.href);
    const bound = bindObservedExport([catalog], catalog.moduleId, name, 'heldOutBinding');
    assert.equal(bound.kind, 'observed');
    assert.equal(bound.declaration.moduleSha256, createHash('sha256').update(source).digest('hex'));
    assert.equal(validateObservedCallable(bound.declaration, source), true);
    assert.equal(bound.declaration.contract.status, 'unknown');
    assert.deepEqual(bound.declaration.parameters, [name === 'selectedCandidate' ? 'selection' : 'candidate']);
  }
});
