import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';

const vectors = JSON.parse(fs.readFileSync(
  new URL('../fixtures/quantity-aggregate-comparison.json', import.meta.url), 'utf8'));
let wasmAttempts = 0;
const forbidden = () => { wasmAttempts++; throw Error('ProductWasmForbiddenForQuantityUnit'); };
const host = new WorkerHost({WebAssembly: {
  instantiate: forbidden, compile: forbidden, instantiateStreaming: forbidden,
  compileStreaming: forbidden, validate: forbidden,
  Module: class {constructor() { forbidden(); }},
  Instance: class {constructor() { forbidden(); }},
  Memory: class {constructor() { forbidden(); }}
}});
before(async () => { await host.boot(); });

function frameFor(result) {
  const row = result.evidence.find(value => value.startsWith('quantity_constraint:frame:'));
  return row ? JSON.parse(row.slice('quantity_constraint:frame:'.length)) : null;
}
function assertRawSpans(question, frame) {
  assert.equal(frame.source, question);
  const bytes = Buffer.from(question, 'utf8');
  const slice = span => bytes.subarray(span.start, span.end).toString('utf8');
  assert.equal(frame.sourceUtf8Bytes, bytes.length);
  assert.equal(slice(frame.aggregationProof.span), frame.aggregationProof.surface);
  assert.equal(slice(frame.comparisonProof.span), frame.comparisonProof.surface);
  for (const value of [...frame.terms, frame.bound]) {
    assert.equal(slice(value.span), value.literal);
    assert.equal(slice(value.unitSpan), value.unitSurface);
  }
}

function validDecline(result, vector) {
  if (['quantity_constraint_unknown', 'calculation_error', 'unknown'].includes(result.intent)) return true;
  return vector.id === 'incompatible-units'
    && result.intent === 'unit_incompatibility'
    && result.evidence.includes('trace:impulse:' + vector.question)
    && result.evidence.includes('unit_incompatibility:kg:mass:vs:ms:time')
    && /different physical dimensions and cannot be converted into each other/u.test(result.content)
    && !/[0-9=]/u.test(result.content)
    && !result.evidence.some(value => value.startsWith('unit_conversion:'));
}

for (const vector of vectors.cases) {
  test('whole QA: ' + vector.id, async () => {
    const result = await host.solve(vector.question);
    if (vector.expected.kind === 'comparison') {
      const expectedIntent = vector.expected.holds ? 'quantity_constraint_within' : 'quantity_constraint_exceeds';
      assert.equal(result.intent, expectedIntent);
      assert(result.evidence.includes('quantity_constraint:verdict:' + vector.expected.holds));
      assert(result.content.includes(String(vector.expected.total)));
      assert(result.content.includes(String(vector.expected.bound)));
      assert(result.content.includes(vector.expected.unit));
      const frame = frameFor(result);
      assert.equal(frame.status, 'supported');
      assert.equal(frame.aggregation, 'sum');
      assert.equal(frame.aggregationProof.kind, 'explicit-sum-cue');
      assert.equal(frame.relation, 'le');
      assertRawSpans(vector.question, frame);
    } else {
      assert(validDecline(result, vector),
        'unsupported comparison must decline, not return an unrelated numeric answer: ' + result.intent);
      assert.equal(typeof result.content, 'string');
      assert(result.content.length > 0);
      assert(!result.evidence.some(value => value.startsWith('quantity_constraint:verdict:')));
      const frame = frameFor(result);
      if (result.intent === 'quantity_constraint_unknown') {
        assert.equal(frame.status, 'unknown');
        assert.equal(frame.source, vector.question);
        assert.equal(typeof frame.reason, 'string');
      }
    }
    assert.equal(wasmAttempts, 0);
  });
}

test('ordinary arithmetic remains unchanged through whole QA', async () => {
  const result = await host.solve(vectors.arithmeticRegression.question);
  assert.equal(result.intent, 'calculation');
  assert.equal(result.content, vectors.arithmeticRegression.expectedContent);
  assert.equal(wasmAttempts, 0);
});

test('raw UTF-8 spans survive non-ASCII whitespace without normalized offsets', async () => {
  const vector = vectors.cases.find(row => row.id === 'explicit-total-renamed-amounts');
  const question = vector.question.replaceAll(' ', '\u00a0');
  const result = await host.solve(question);
  assert.equal(result.intent, 'quantity_constraint_within');
  const frame = frameFor(result);
  assertRawSpans(question, frame);
  assert(frame.bound.span.start > question.indexOf(frame.bound.literal));
  assert.equal(wasmAttempts, 0);
});

test('dimension decline requires actual source and matching dimension evidence', () => {
  const vector = vectors.cases.find(row => row.id === 'incompatible-units');
  const refusal = {
    intent: 'unit_incompatibility',
    content: 'Different physical dimensions and cannot be converted into each other.',
    evidence: []
  };
  assert.equal(validDecline(refusal, vector), false);
  refusal.evidence = ['trace:impulse:' + vector.question,
    'unit_incompatibility:kg:length:vs:ms:time'];
  assert.equal(validDecline(refusal, vector), false);
  refusal.evidence[1] = 'unit_incompatibility:kg:mass:vs:ms:time';
  refusal.content = '1000 ms = 1 second';
  assert.equal(validDecline(refusal, vector), false);
  refusal.content = 'different physical dimensions and cannot be converted into each other.';
  assert.equal(validDecline(refusal, {...vector, id: 'unrelated-control'}), false);
});

for (const question of [
  'Is the sum operation associative?',
  'Is their combined total a useful column name?'
]) {
  test('aggregate concept remains outside comparison ownership: ' + question, async () => {
    const result = await host.solve(question);
    assert.equal(frameFor(result), null);
    assert(!result.intent.startsWith('quantity_constraint_'));
    assert(!result.evidence.some(value => value.startsWith('quantity_constraint:verdict:')));
    assert.equal(wasmAttempts, 0);
  });
}
