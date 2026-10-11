import assert from 'node:assert/strict';
import fs from 'node:fs';
import test from 'node:test';
import { WorkerHost } from '../../../js/server/worker-host.mjs';
import { installNodeHost } from '../../../js/agentic/node-host.mjs';
import { sha256Hex } from '../../../js/agentic/crate/source_fetch.mjs';
import { observedCallableRequest } from '../../../js/agentic/module_function.mjs';
import { observedCallableGoalLedger } from '../../../js/agentic/module_function/discovery.mjs';
import {
  observeSourceCallables,
  observedConditionalGraphs,
  observedGuardedGraph,
} from '../../../js/agentic/module_function/callable_catalog.mjs';
await installNodeHost(new WorkerHost());
const producer =
  "export function decide(input) { return input.state.mode === 'go' ? input.pool[input.state.offset] ?? null : null; }";
const consumer =
  "export function describe(row) { const lines = row.entries.map((field) => `${field.tag}:${field.atom.code}`); if (row.notes.length > 0) lines.push(`N=${row.notes.join(',')}`); return lines.length === 0 ? 'none' : lines.join(';'); }";
const observations = () => [
  { path: 'choice.mjs', content: producer },
  { path: 'caption.mjs', content: consumer },
];
const graph = () =>
  observedConditionalGraphs(observations()).find(
    (value) => value.schemas.result.kind === 'text'
  );
const declaration = (source) =>
  observeSourceCallables(source, 'input.mjs').declarations[0];

test('source-only constraints infer structurally different plain-data schemas without supplied fields', () => {
  const value = graph();
  assert.equal(value.kind, 'conditional-schema-graph');
  assert.equal(value.suppliedPerTaskSchemas, false);
  const input = value.schemas.input.options[0].fields;
  assert.deepEqual(input.state.fields.mode, { kind: 'literal', value: 'go' });
  assert.deepEqual(input.state.fields.offset, { kind: 'index' });
  assert.equal(input.pool.kind, 'array');
  assert.equal(
    value.schemas.consumed.fields.entries.item.fields.tag.kind,
    'scalar'
  );
  assert.equal(
    value.schemas.consumed.fields.entries.item.fields.atom.fields.code.kind,
    'scalar'
  );
  assert.equal(value.schemas.consumed.fields.notes.item.kind, 'scalar');
  assert.deepEqual(value.schemas.produced.value, value.schemas.consumed);
  assert.deepEqual(value.schemas.result, { kind: 'text' });
});

test('conditional schemas preserve unknown real caller, module and Goal effects', () => {
  const value = graph();
  assert.equal(value.effects, 'unknown');
  assert.equal(value.sourceEffects, 'conditional');
  assert.equal(value.goal, 'unbound');
  assert.equal(value.authored, false);
  assert.ok(value.preconditions.includes('SourceOwnedPlainDataSchema'));
  assert.ok(value.preconditions.includes('IntrinsicOwnership'));
  assert.deepEqual(value.moduleSyntax, ['unknown', 'unknown']);
  const imported = observations();
  imported[1].content = "import { foreign } from './other.mjs'; " + consumer;
  const conditional = observedConditionalGraphs(imported).find(
    (value) => value.schemas.result.kind === 'text'
  );
  assert.equal(conditional.moduleEffects[1], 'unknown');
  assert.equal(
    observedGuardedGraph(
      imported,
      { path: 'choice.mjs', exported: 'decide' },
      { path: 'caption.mjs', exported: 'describe' }
    ).reason,
    'ModuleSyntaxUnknown'
  );
});

test('renamed declarations, parameter/local bindings and exports keep source identities', () => {
  const data = observations();
  data[0].content =
    producer.replaceAll('decide', 'renamed').replaceAll('input', 'pick') +
    ' export { renamed as publicPick };';
  data[1].content = consumer
    .replaceAll('describe', 'other')
    .replaceAll('row', 'record')
    .replaceAll('lines', 'pieces')
    .replaceAll('field', 'entry');
  const values = observedConditionalGraphs(data).filter(
    (value) => value.schemas.result.kind === 'text'
  );
  assert.equal(values.length, 2);
  assert.ok(
    values.some((value) => value.bindings[0].exported === 'publicPick')
  );
  for (const value of values) {
    assert.equal(value.calls[0].moduleContentId, sha256Hex(data[0].content));
    assert.equal(value.calls[1].moduleContentId, sha256Hex(data[1].content));
    assert.deepEqual(value.schemas.consumed, graph().schemas.consumed);
  }
});

test('complete current bytes determine identity; stale original binding is refused', () => {
  const original = graph(),
    data = observations();
  data[0].content = '// 文\n' + data[0].content;
  const current = observedConditionalGraphs(data).find(
    (value) => value.schemas.result.kind === 'text'
  );
  assert.notEqual(
    current.calls[0].moduleContentId,
    original.calls[0].moduleContentId
  );
  assert.notEqual(current.calls[0].span.byteStart, current.calls[0].span.start);
  assert.equal(
    observedGuardedGraph(
      data,
      {
        path: 'choice.mjs',
        exported: 'decide',
        contentId: original.calls[0].moduleContentId,
      },
      { path: 'caption.mjs', exported: 'describe' }
    ).reason,
    'SourceChanged'
  );
});

test('the genuine mixed equality failure is conservatively refused rather than falsely interpreted', () => {
  for (const expression of [
    'value.a === value.b !== false',
    'value.a === value.b === value.c',
    'value.a > value.b > value.c',
  ]) {
    assert.equal(
      declaration(`export function relation(value) { return ${expression}; }`)
        .contract.conditionalIR.reason,
      'UnsupportedComparisonChain'
    );
  }
  assert.equal(
    declaration(
      'export function relation(value) { return (value.a === value.b) !== false; }'
    ).contract.conditionalIR.status,
    'parsed'
  );
});

test('ASI, invalid bindings, unsafe literals, duplicate exports and async do not become conditional programs', () => {
  for (const [source, reason] of [
    [
      "export function f(input) { const p = input.rows.map((row) => row); return\n p.join(' '); }",
      'ReturnLineTerminator',
    ],
    [
      "export function f(input) { const for = input.rows.map((row) => row); return for.join(' '); }",
      'UnsupportedLocalBinding',
    ],
    [
      "export function f(input) { const p = input.rows.map((for) => for); return p.join(' '); }",
      'UnsupportedMapper',
    ],
    [
      'export function f(input) { return 9007199254740992; }',
      'UnsafeNumericLiteral',
    ],
    ['export function f(input) { return input.run(); }', 'UnknownMethod'],
  ])
    assert.equal(declaration(source).contract.conditionalIR.reason, reason);
  assert.equal(
    declaration('export async function f(input) { return input; }').contract
      .conditionalIR,
    undefined
  );
  const data = observations();
  data[1].content += ' export { describe };';
  assert.deepEqual(observedConditionalGraphs(data), []);
});

test('caller mutation and unknown coercion constraints do not become pure graph operands', () => {
  const data = observations();
  data[1].content =
    "export function change(row) { if (row.notes.length > 0) row.notes.push('x'); return 'done'; }";
  assert.deepEqual(observedConditionalGraphs(data), []);
  assert.equal(
    declaration('export function f(input) { input.value = 3; return input; }')
      .contract.conditionalIR.reason,
    'UnsupportedStatement'
  );
  const schema = graph().schemas.consumed;
  assert.equal(schema.fields.entries.item.fields.tag.kind, 'scalar');
  assert.equal('getterEffects' in schema, false);
  assert.equal(graph().effects, 'unknown');
});

test('actual original source operands yield conditional schema constraints with preserved module uncertainty', () => {
  const paths = ['solver_formalization.mjs', 'translation_formalization.mjs'];
  const data = paths.map((name) => ({
    path: name,
    content: fs.readFileSync(
      new URL('../../../js/agentic/crate/' + name, import.meta.url),
      'utf8'
    ),
  }));
  const values = observedConditionalGraphs(data).filter(
    (value) => value.schemas.result.kind === 'text'
  );
  assert.equal(values.length, 1);
  assert.equal(
    values[0].schemas.consumed.fields.slots.item.fields.role.kind,
    'scalar'
  );
  assert.equal(
    values[0].schemas.consumed.fields.unresolved_terms.item.kind,
    'scalar'
  );
  assert.deepEqual(values[0].moduleEffects, ['unknown', 'unknown']);
  assert.equal(values[0].goal, 'unbound');
  assert.equal(values[0].authored, false);
});

test('independent original request clauses retain UTF-8/UTF-16 evidence and unbound statuses', () => {
  const task =
    'Add caption(pick) to output.mjs. Read left.mjs and right.mjs. Return a description or null. Run node --test accept.test.mjs. Preserve the 文 inputs.';
  const request = observedCallableRequest(task);
  const ledger = observedCallableGoalLedger(request, [
    { role: 'user', content: task },
  ]);
  assert.equal(ledger.sourceIdentity, sha256Hex(task));
  assert.equal(ledger.bytes, Buffer.byteLength(task));
  assert.equal(ledger.semantics, 'unbound');
  assert.equal(ledger.sourceBinding, 'unverified');
  assert.equal(ledger.authored, false);
  assert.ok(
    ledger.needs.some(
      (need) =>
        need.kind === 'declared-verification' &&
        need.command === request.command
    )
  );
  assert.ok(ledger.clauses.some((clause) => clause.returnAction));
  for (const clause of ledger.clauses) {
    assert.equal(
      clause.source,
      Buffer.from(task)
        .subarray(clause.span.byteStart, clause.span.byteEnd)
        .toString()
    );
    assert.equal(clause.source, task.slice(clause.span.start, clause.span.end));
    assert.equal(clause.contentId, sha256Hex(clause.source));
    assert.equal(clause.status, 'unbound');
  }
});

test('source-owned literal indexes and numeric-index intersections need no field-name specialization', () => {
  const data = observations();
  data[0].content =
    'export function first(value) { return value.flag === true ? value.pool[0] ?? null : null; }';
  const literal = observedConditionalGraphs(data).find(
    (value) => value.schemas.result.kind === 'text'
  );
  assert.equal(
    literal.schemas.input.options[0].fields.pool.item.kind,
    'record'
  );
  assert.equal('state' in literal.schemas.input.options[0].fields, false);
  data[0].content =
    'export function indexed(value) { return value.offset > 0 ? value.pool[value.offset] ?? null : null; }';
  const indexed = observedConditionalGraphs(data).find(
    (value) => value.schemas.result.kind === 'text'
  );
  assert.equal(indexed.schemas.input.fields.offset.kind, 'index');
});
