import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';

const source = readFileSync(new URL('../../../js/worker/formal_ai_worker_meta_synthesis.js', import.meta.url), 'utf8');
const historicalTest = readFileSync(new URL('./meta-program-transition-cache.test.mjs', import.meta.url), 'utf8');
const historical = JSON.parse(/const reference = ("(?:[^"\\]|\\.)*");\n/u.exec(historicalTest)[1]);

function fixture() {
  return {
    primitives: [
      {id: 'identity', from: 'text', to: 'text', infer: '', takes: [], code: ''},
      {id: 'count', from: 'text', to: 'number', infer: '', takes: [], code: ''},
      {id: 'words', from: 'text', to: 'list_text', infer: '', takes: [], code: ''},
      {id: 'read', from: 'path', to: 'text', infer: '', takes: [], code: ''},
      {id: 'first', from: 'list_any', to: 'text', infer: '', takes: [], code: ''},
    ],
    filters: [{id: 'filter', measure: 'number'}, {id: 'contains', measure: 'text'}],
    selectors: [{id: 'maximum'}],
  };
}
function evaluator(seed, original = false) {
  const context = vm.createContext({metaSeed: () => seed, META_BOUNDS: {measureLength: 3}});
  vm.runInContext(source, context);
  if (original) vm.runInContext(historical, context);
  return context;
}
function identityGraph(programs) {
  const objects = new Map();
  return Array.from(programs, program => Array.from(program.steps, step => {
    if (!objects.has(step)) objects.set(step, objects.size);
    return objects.get(step);
  }));
}

for (const type of ['text', 'list_text', 'number', 'path', 'list_path']) {
  test('demand enumeration preserves historical sequence and eager prefix identity for ' + type, () => {
    for (const operations of [null, [], ['read'], ['map_each', 'filter']]) {
      for (const length of [1, 2, 3]) {
        const original = evaluator(fixture(), true), candidate = evaluator(fixture());
        for (const context of [original, candidate]) {
          Object.assign(context, {inputType: type, inputLength: length, operations});
          vm.runInContext('const required = operations === null ? null : new Set(operations);', context);
        }
        const expected = vm.runInContext('metaPrograms(inputType, inputLength, required)', original);
        const eager = vm.runInContext('metaPrograms(inputType, inputLength, required)', candidate);
        const streamed = vm.runInContext('Array.from(metaProgramIterator(inputType, inputLength, required))', candidate);
        assert.deepEqual(JSON.parse(JSON.stringify(eager)), JSON.parse(JSON.stringify(expected)));
        assert.deepEqual(JSON.parse(JSON.stringify(streamed)), JSON.parse(JSON.stringify(expected)));
        assert.deepEqual(identityGraph(eager), identityGraph(expected));
      }
    }
  });
}

test('the final frontier allocates only demanded candidates and return stops further work', () => {
  const seed = {primitives: Array.from({length: 32}, (_, index) => ({
    id: 'operation-' + index, from: 'text', to: 'text', infer: '', takes: [], code: '',
  })), filters: [], selectors: []};
  const context = evaluator(seed);
  vm.runInContext([
    'let concatenations = 0;',
    'const originalConcat = Array.prototype.concat;',
    'Array.prototype.concat = function (...values) {',
    '  concatenations++;',
    '  return originalConcat.apply(this, values);',
    '};',
    'const iterator = metaProgramIterator("text", 3);',
  ].join('\n'), context);
  assert.equal(vm.runInContext('concatenations', context), 0);
  for (let index = 0; index < 32 + 32 * 32; index++) {
    assert.equal(vm.runInContext('iterator.next().done', context), false);
  }
  assert.equal(vm.runInContext('concatenations', context), 32 + 32 * 32);
  assert.equal(vm.runInContext('iterator.next().value.steps.length', context), 3);
  assert.equal(vm.runInContext('concatenations', context), 32 + 32 * 32 + 1);
  assert.equal(vm.runInContext('iterator.return().done', context), true);
  assert.equal(vm.runInContext('iterator.next().done', context), true);
  assert.equal(vm.runInContext('concatenations', context), 32 + 32 * 32 + 1);
});

test('meaning search solves without the eager compatibility collector', () => {
  const context = evaluator(fixture());
  assert.equal(vm.runInContext('Array.isArray(metaPrograms("text", 1))', context), true);
  vm.runInContext([
    'META_BOUNDS.programLength = 4;',
    'metaPrograms = () => { throw Error("EagerMeaningEnumeration"); };',
    'function metaClauseOrder(program, fromType) {',
    '  return {types: [fromType], violations: 0, objectMismatch: 0};',
    '}',
    'const found = metaSynthesizeFromMeaning([["identity"]],',
    '  new Map([["identity", 1]]), {emit() {}}, [],',
    '  [[{operation: "identity", score: 1}]], [], [], {});',
  ].join('\n'), context);
  assert.equal(vm.runInContext('found.fromType', context), 'text');
  assert.equal(vm.runInContext('found.steps.length', context), 1);
  assert.equal(vm.runInContext('found.steps[0].primitive.id', context), 'identity');
});
