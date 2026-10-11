import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';
const source = readFileSync(new URL('../../../js/worker/formal_ai_worker_meta_synthesis.js', import.meta.url), 'utf8');
// Full original source-owned enumerator preimage, before the call-local cache.
const reference = "function metaPrograms(fromType, length, mustUse = null) {\n  const primitives = metaSeed().primitives;\n  let frontier = [mustUse === null ? { steps: [], type: fromType } : { steps: [], type: fromType, uses: false }];\n  const out = [];\n  for (let size = 1; size <= length; size += 1) {\n    const next = [];\n    // A last-length program using none of `mustUse` is never built.\n    const extend = (program, step, type) => {\n      if (mustUse === null) {\n        next.push({ steps: program.steps.concat(step), type });\n        return;\n      }\n      const uses = program.uses || metaStepUsesAny(step, mustUse);\n      if (size === length && !uses) return;\n      next.push({ steps: program.steps.concat(step), type, uses });\n    };\n    for (const program of frontier) {\n      for (const primitive of primitives) {\n        const direct = metaApply(primitive, program.type);\n        if (direct) extend(program, { primitive, mapped: false }, direct);\n        if (program.type.startsWith(\"list_\")) {\n          const element = program.type.slice(5);\n          const mapped = metaApply(primitive, element);\n          if (mapped && !mapped.startsWith(\"list_\")) extend(program, { primitive, mapped: true }, `list_${mapped}`);\n        }\n      }\n      // A file is edited in place by any text transformation: read it,\n      // transform the text, write it back (\"rewrite_file\").\n      if (program.type === \"path\" || program.type === \"list_path\") {\n        for (const primitive of primitives) {\n          if (primitive.from !== \"text\" || primitive.to !== \"text\" || primitive.infer || primitive.takes.length) continue;\n          extend(program, { primitive, mapped: program.type === \"list_path\", rewrite: true }, program.type);\n        }\n      }\n      if (program.type.startsWith(\"list_\")) {\n        for (const measure of metaMeasures(program.type.slice(5))) {\n          for (const filter of metaSeed().filters) {\n            if (filter.measure === \"number\") extend(program, { primitive: measure, mapped: false, filter }, program.type);\n          }\n        }\n        // A filter comparing the element's content with a value.\n        for (const filter of metaSeed().filters) {\n          if (filter.measure === \"number\") continue;\n          for (const measure of metaMeasures(program.type.slice(5), filter.measure)) extend(program, { primitive: measure, mapped: false, filter }, program.type);\n        }\n        // A selector picks one element by its measure (\"the longest word\").\n        for (const measure of metaMeasures(program.type.slice(5))) {\n          for (const select of metaSeed().selectors) extend(program, { primitive: measure, mapped: false, select }, program.type.slice(5));\n        }\n      }\n    }\n    for (const program of next) if (mustUse === null || program.uses) out.push(program);\n    frontier = next;\n  }\n  return out;\n}";
function evaluator(seed, original = false) {
  const state = {seed, applications: 0};
  const context = vm.createContext({metaSeed: () => state.seed, META_BOUNDS: {measureLength: 3}});
  vm.runInContext(source, context);
  if (original) vm.runInContext(reference, context);
  vm.runInContext('const countedApply = metaApply; metaApply = (...values) => { applications++; return countedApply(...values); }; let applications = 0;', context);
  return {
    state, context,
    run: (type, length, required) => {
      context.inputType = type; context.inputLength = length; context.requiredOperations = required;
      const result = vm.runInContext('metaPrograms(inputType, inputLength, requiredOperations)', context);
      return {result, plain: JSON.parse(JSON.stringify(result)), applications: vm.runInContext('applications', context)};
    },
  };
}
function seed() {
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
for (const type of ['text', 'list_text', 'number', 'path', 'list_path']) {
  test(`complete ordered programs conserved for ${type}`, () => {
    for (const required of [null, new Set(), new Set(['read']), new Set(['map_each', 'filter'])]) {
      const original = evaluator(seed(), true), candidate = evaluator(seed());
      for (const length of [1, 2, 3]) {
        assert.deepEqual(candidate.run(type, length, required).plain, original.run(type, length, required).plain);
      }
    }
  });
}
test('same-call typed transitions reduce repeated pure applications without fewer programs', () => {
  const original = evaluator(seed(), true).run('list_text', 3, null);
  const candidate = evaluator(seed()).run('list_text', 3, null);
  assert.deepEqual(candidate.plain, original.plain);
  assert.ok(candidate.result.length > 0);
  assert.ok(candidate.applications < original.applications);
});
test('different calls observe changed seed types and newly learned operations', () => {
  const original = evaluator(seed(), true), candidate = evaluator(seed());
  assert.deepEqual(candidate.run('text', 2, null).plain, original.run('text', 2, null).plain);
  for (const evaluator_ of [original, candidate]) {
    evaluator_.state.seed = seed();
    evaluator_.state.seed.primitives.push({id: 'learned', from: 'text', to: 'path', infer: '', takes: [], code: ''});
  }
  const changed = candidate.run('text', 2, new Set(['learned']));
  assert.deepEqual(changed.plain, original.run('text', 2, new Set(['learned'])).plain);
  assert.ok(changed.result.some(program => program.steps.some(step => step.primitive.id === 'learned')));
});
test('step-object sharing matches original prefix ownership; fresh calls retain no result mutations', () => {
  const original = evaluator(seed(), true), candidate = evaluator(seed());
  const before = original.run('text', 2, null).result, after = candidate.run('text', 2, null).result;
  const left = before.flatMap(program => program.steps), right = after.flatMap(program => program.steps);
  assert.equal(left.length, right.length);
  for (let first = 0; first < left.length; first++) for (let second = 0; second < left.length; second++) {
    assert.equal(right[first] === right[second], left[first] === left[second]);
  }
  right[0].mapped = !right[0].mapped;
  assert.deepEqual(candidate.run('text', 2, null).plain, original.run('text', 2, null).plain);
});

const originalCompiler = "function metaCompile(primitive, parameter) {\n  const source = metaSubstitute(primitive.code, parameter);\n  const cached = metaCompiled.get(source);\n  if (cached) return cached;\n  // eslint-disable-next-line no-new-func -- the instruction set is seed data.\n  const fn = new Function(`return (${source});`)();\n  metaCompiled.set(source, fn);\n  return fn;\n}";
function compileModel(original) {
  const context = vm.createContext({metaCompiled: new Map()});
  vm.runInContext(source, context);
  if (original) vm.runInContext(originalCompiler, context);
  return context;
}
test('compiler cache preserves null and parametric sources, mutation, coercion and exact failures', () => {
  for (const code of ['(input) => input + 1', '(input) => input * {k}', '(input) => input + {k0}', '(input) => "{keep}"', '(input) => { throw Error("owned failure"); }']) {
    for (const parameter of [null, undefined, 3, [4]]) {
      const values = [true, false].map(original => {
        const context = compileModel(original); context.primitive = {code}; context.parameter = parameter;
        return vm.runInContext('(() => { try { return {source: metaCompile(primitive, parameter).toString()}; } catch(error) { return {error: error.name + ":" + error.message}; } })()', context);
      });
      assert.equal(JSON.stringify(values[0]), JSON.stringify(values[1]));
    }
  }
  for (const original of [true, false]) {
    const context = compileModel(original);
    vm.runInContext('let reads = 0; const primitive = {get code() {reads++; return "(input) => input + 1";}}; const first = metaCompile(primitive, null); const second = metaCompile(primitive, null);', context);
    assert.equal(vm.runInContext('reads', context), 2);
    assert.equal(vm.runInContext('first === second', context), true);
    vm.runInContext('const changing = {code:"(input) => input + 1"}; const prior = metaCompile(changing, null); changing.code="(input) => input + 2";', context);
    assert.equal(vm.runInContext('metaCompile(changing, null)(3)', context), 5);
    vm.runInContext('let literalCalls = 0; metaLiteral = () => {literalCalls++; throw Error("literal refusal");};', context);
    assert.throws(() => vm.runInContext('metaCompile({code:"(input) => input"}, null)', context), /literal refusal/);
    assert.equal(vm.runInContext('literalCalls', context), 1);
  }
});

test('compiler conserves changed-realm intrinsic exceptions and substitution calls', () => {
  for (const changed of [
    'String.prototype.includes = function () { throw Error("changed-realm-includes"); };',
    'const split = String.prototype.split; let splitCalls = 0; String.prototype.split = function (...args) { splitCalls++; return split.apply(this, args); };',
  ]) {
    const outcomes = [true, false].map(original => {
      const context = compileModel(original);
      vm.runInContext(changed, context);
      const value = vm.runInContext('metaCompile({code:"(input) => input"},null)(42)', context);
      const splitCalls = vm.runInContext('typeof splitCalls === "undefined" ? null : splitCalls', context);
      return {value, splitCalls};
    });
    assert.deepEqual(outcomes[1], outcomes[0]);
    assert.equal(outcomes[0].value, 42);
  }
});
