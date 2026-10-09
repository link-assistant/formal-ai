import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
import { observeSourceCallables, observedGuardedGraph } from '../../../js/agentic/module_function/callable_catalog.mjs';
const observe = (source, path = 'source.mjs') => observeSourceCallables(source, path);
const declaration = (source) => observe(source).declarations[0];
const firstSource = "export function choose(value) { return value === null ? null : 'ready'; } export { choose as publicChoice };";
const secondSource = "export function render(item) { return item === 'ready'; }";
const observations = () => [{ path: 'producer.mjs', content: firstSource }, { path: 'consumer.mjs', content: secondSource }];
const first = { path: 'producer.mjs', exported: 'publicChoice', contentId: sha256Hex(firstSource) };
const second = { path: 'consumer.mjs', exported: 'render', contentId: sha256Hex(secondSource) };

test('complete Unicode declaration spans retain source and both coordinate systems', () => {
  const source = '// 文\nexport function describe(input) { return input; }';
  const entry = declaration(source);
  assert.equal(entry.source, source.slice(entry.span.start, entry.span.end));
  assert.equal(entry.source, Buffer.from(source).subarray(entry.span.byteStart, entry.span.byteEnd).toString());
  assert.notEqual(entry.span.byteStart, entry.span.start);
  assert.equal(entry.identity.moduleContentId, sha256Hex(source));
  assert.equal(entry.identity.declarationContentId, sha256Hex(entry.source));
  assert.deepEqual(entry.contract.result, { kind: 'parameter', name: 'input' });
});

test('bounded bodies establish Optional and scalar constraints without caller-supplied types', () => {
  const catalog = observe(firstSource);
  assert.deepEqual(catalog.declarations[0].contract.result, { kind: 'optional', value: { kind: 'text' } });
  assert.equal(catalog.declarations[0].contract.status, 'supported');
  assert.equal(catalog.declarations[0].contract.callEffects, 'none');
  assert.equal(catalog.moduleEffects, 'none');
  assert.deepEqual(catalog.exports, [{ local: 'choose', exposed: 'choose' }, { local: 'choose', exposed: 'publicChoice' }]);
});

test('an actual observed pair produces a guarded typed graph, without a wrapper or write', () => {
  const before = JSON.stringify(observations());
  const graph = observedGuardedGraph(observations(), first, second);
  assert.equal(graph.kind, 'guarded-call-graph');
  assert.deepEqual(graph.guard, { kind: 'nonnull', call: 0 });
  assert.deepEqual(graph.result, { kind: 'optional', value: { kind: 'boolean' } });
  assert.deepEqual(graph.bindings, [{ path: 'producer.mjs', exported: 'publicChoice' }, { path: 'consumer.mjs', exported: 'render' }]);
  assert.equal(graph.calls[0].moduleContentId, sha256Hex(firstSource));
  assert.equal(graph.authored, false);
  assert.equal(JSON.stringify(observations()), before);
  assert.equal('source' in graph, false);
});

test('nested Optional type parameters are instantiated from the observed operand', () => {
  const source = 'export function pass(item) { return item === null ? null : item; }';
  const data = observations(); data[1].content = source;
  const graph = observedGuardedGraph(data, first, { path: 'consumer.mjs', exported: 'pass' });
  assert.deepEqual(graph.result, { kind: 'optional', value: { kind: 'text' } });
});

test('stale identities and unobserved exports cannot produce a graph', () => {
  const data = observations(); data[0].content += '\n';
  assert.equal(observedGuardedGraph(data, first, second).reason, 'SourceChanged');
  assert.equal(observedGuardedGraph(observations(), { ...first, exported: 'notObserved' }, second).reason, 'ExportUnresolved');
  assert.equal(observedGuardedGraph([], first, second).reason, 'ModuleUnobserved');
});

test('unknown calls, field schemas, input writes and local mutation remain contract gaps', () => {
  for (const [source, reason] of [
    ['export function f(x) { return external(x); }', 'UnobservedCallEffect'],
    ['export function f(x) { return x.value; }', 'MissingStructuralSchema'],
    ['export function f(x) { x.value = 3; return x; }', 'UnsupportedStatement'],
    ['export function f(x) { const out = []; out.push(x); return out; }', 'UnsupportedStatement'],
  ]) {
    const entry = declaration(source);
    assert.equal(entry.contract.status, 'unknown');
    assert.equal(entry.contract.callEffects, 'unknown');
    assert.equal(entry.contract.gap.reason, reason);
    const data = observations(); data[1].content = source;
    assert.equal(observedGuardedGraph(data, first, { path: 'consumer.mjs', exported: 'f' }).reason, 'ModuleSyntaxUnknown');
  }
});

test('real module initialization and transitive import effects remain separate obligations', () => {
  const data = observations(); data[1].content += ' globalThis.called = 1;';
  assert.equal(observedGuardedGraph(data, first, { ...second, contentId: undefined }).reason, 'ModuleContractGap');
  const facade = "import { render as aliased } from './consumer.mjs'; export { aliased as publicRender };";
  data[1].content = secondSource; data.push({ path: 'facade.mjs', content: facade });
  assert.equal(observedGuardedGraph(data, first, { path: 'facade.mjs', exported: 'publicRender' }).reason, 'MissingContract');
  assert.equal(observedGuardedGraph([data[0], data[2]], first, { path: 'facade.mjs', exported: 'publicRender' }).reason, 'ModuleUnobserved');
});

test('ASI line terminators, incompatible branches and unsupported async are not certified', () => {
  for (const [source, reason] of [
    ['export function f(x) { return\n x; }', 'ReturnLineTerminator'],
    ['export function f(x) { return /* 文\n */ x; }', 'ReturnLineTerminator'],
    ['export function f(x) { return x === null ? true : 3; }', 'IncompatibleBranches'],
    ['export async function f(x) { return x; }', 'AsyncResult'],
  ]) assert.equal(declaration(source).contract.gap.reason, reason);
});

test('duplicate bindings and malformed balanced source do not masquerade as provenance', () => {
  assert.equal(observe('export function f(x) { return x; } function f(x) { return x; }').gaps[0].reason, 'DuplicateBinding');
  assert.equal(observe('export function f(x) { return x;').gaps[0].reason, 'LexicalFailure');
  assert.equal(declaration('export function f(x, x) { return x; }').contract.gap.reason, 'UnsupportedParameters');
});

test('actual original callees retain exact declarations and honest unsupported contract reasons', () => {
  for (const [file, name, reason] of [['solver_formalization.mjs', 'selectedCandidate', 'MissingStructuralSchema'],
    ['translation_formalization.mjs', 'candidateCompactSummary', 'UnsupportedStatement']]) {
    const source = fs.readFileSync(new URL('../../../js/agentic/crate/' + file, import.meta.url), 'utf8');
    const catalog = observe(source, file), entry = catalog.declarations.find((candidate) => candidate.name === name);
    assert.ok(entry, name);
    assert.equal(entry.identity.moduleContentId, sha256Hex(source));
    assert.equal(entry.contract.gap.reason, reason);
    assert.equal(entry.contract.status, 'unknown');
    assert.equal(catalog.moduleEffects, 'unknown');
  }
});

test('production discovery records supported graphs but cannot certify an unbound requested goal', async () => {
  const { WorkerHost } = await import('../../../js/server/worker-host.mjs');
  const { installNodeHost } = await import('../../../js/agentic/node-host.mjs');
  const { planModuleFunctionStep } = await import('../../../js/agentic/module_function.mjs');
  await installNodeHost(new WorkerHost());
  const prompt = 'Add exported caption(pick) to output.mjs. Read producer.mjs and consumer.mjs. Run node --test gate.mjs.';
  const messages = [{ role: 'user', content: prompt }];
  for (const [path, content] of [['output.mjs', '// existing destination'], ['producer.mjs', firstSource],
    ['consumer.mjs', secondSource], ['gate.mjs', '// immutable acceptance']]) {
    messages.push({ role: 'assistant', content: '', tool_calls: [{ id: path, type: 'function',
      function: { name: 'read', arguments: JSON.stringify({ path }) } }] });
    messages.push({ role: 'tool', tool_call_id: path, content,
      source_read: { path, success: true, complete: true, format: 'raw' } });
  }
  const plan = await planModuleFunctionStep(prompt, messages, ['read', 'write', 'bash']);
  assert.equal(plan.result.disposition, 'gap');
  assert.equal(plan.result.discovery.detail.goal, 'unbound');
  assert.ok(plan.result.discovery.detail.graphs.length > 0);
  assert.equal(plan.result.discovery.authored, false);
  assert.equal(plan.result.discovery.verified, false);
  assert.equal(plan.result.discovery.detail.graphs.every((graph) => graph.authored === false), true);
});

test('balanced but invalid binding/export forms do not certify a module contract', () => {
  for (const source of ['export function super(x) { return x; }',
    'export function valid(implements) { return true; }',
    'export function valid(x) { return true; } export { missing };']) {
    assert.equal(observe(source).moduleEffects, 'unknown', source);
  }
  const source = 'export function f(x) { return\u2028x; }';
  assert.equal(declaration(source).contract.gap.reason, 'UnsupportedStatement');
  assert.equal(declaration(source).contract.status, 'unknown');
  const commented = 'export function f(x) { return /*\u2028*/ x; }';
  assert.equal(declaration(commented).contract.gap.reason, 'ReturnLineTerminator');
});


test('deferred unknown callable effects cannot contaminate module initialization', () => {
  const source = firstSource + ' function deferred(value) { return external(value); }';
  const catalog = observe(source);
  assert.equal(catalog.moduleEffects, 'none');
  assert.equal(catalog.declarations[1].contract.callEffects, 'unknown');
  const data = observations(); data[0].content = source;
  assert.equal(catalog.moduleSyntax, 'unknown');
  assert.equal(observedGuardedGraph(data, { ...first, contentId: undefined }, second).reason, 'ModuleSyntaxUnknown');
});

test('actual initialization and import regions retain independent complete byte witnesses', () => {
  const source = '// 文\n' + secondSource + ' globalThis.flag = 1;';
  const catalog = observe(source, 'consumer.mjs');
  const record = catalog.initialization[0];
  assert.equal(record.kind, 'unclassified');
  assert.equal(record.effects, 'unknown');
  assert.equal(record.source, 'globalThis.flag = 1;');
  assert.equal(source.slice(record.span.start, record.span.end), record.source);
  assert.equal(Buffer.from(source).subarray(record.span.byteStart, record.span.byteEnd).toString(), record.source);
  assert.equal(record.contentId, sha256Hex(record.source));
  assert.equal(catalog.moduleEffects, 'unknown');
  const imported = observe("import { render } from './consumer.mjs'; export { render };", 'facade.mjs');
  assert.equal(imported.initialization[0].kind, 'import');
  assert.equal(imported.initialization[0].source, "import { render } from './consumer.mjs';");
  assert.equal(imported.moduleEffects, 'unknown');
});

test('structural return access witnesses retain dynamic indices without schema or purity certification', () => {
  const source = '// 文\nexport function select(input) { return input.items[input.position] ?? null; }';
  const contract = declaration(source).contract;
  assert.equal(contract.status, 'unknown');
  assert.equal(contract.callEffects, 'unknown');
  assert.equal(contract.gap.reason, 'MissingStructuralSchema');
  assert.deepEqual(contract.structuralRequirements.map((entry) => entry.source), ['input.items[input.position]', 'input.position']);
  assert.deepEqual(contract.structuralRequirements[0].selectors, [{ property: 'items' }, { index: 'input.position' }]);
  for (const requirement of contract.structuralRequirements) {
    assert.equal(requirement.status, 'unproved');
    assert.equal(source.slice(requirement.span.start, requirement.span.end), requirement.source);
    assert.equal(Buffer.from(source).subarray(requirement.span.byteStart, requirement.span.byteEnd).toString(), requirement.source);
  }
});

test('unknown closures and structural writes cannot acquire read-only contracts', () => {
  for (const source of ['export function f(input) { return input.value = 1; }',
    'export function f(input) { return values.map(input => input.value); }']) {
    const contract = declaration(source).contract;
    assert.equal(contract.status, 'unknown');
    assert.equal(contract.callEffects, 'unknown');
    assert.deepEqual(contract.structuralRequirements, []);
  }
});


test('an unvalidated balanced body cannot certify an otherwise supported module import', () => {
  const source = firstSource + ' function broken(value) { return value++++; }';
  const catalog = observe(source);
  assert.equal(catalog.moduleEffects, 'none');
  assert.equal(catalog.moduleSyntax, 'unknown');
  const data = observations(); data[0].content = source;
  assert.equal(observedGuardedGraph(data, { ...first, contentId: undefined }, second).reason, 'ModuleSyntaxUnknown');
});
