import {before, test} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {WorkerHost, evaluate} from '../../../js/server/worker-host.mjs';
import {parseLino, childrenNamed, childValue} from '../../../js/server/lino.mjs';
import {encodeWord, parseLinks, textOf, writeLink} from '../../../js/logging/lino-codec.mjs';
import {gapsFromDocuments} from '../../../scripts/lib/checks-language-parity.mjs';
import {parseMeanings, parseSchema, findGaps} from '../../../scripts/audit-seed-metadata.mjs';

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
      assert.equal(result.content, vector.expected.content);
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

test('quantity lexical and response data cover five languages with faithful text and slots', () => {
  const sourcePath = 'data/seed/meanings-calculator.lino';
  const source = fs.readFileSync(new URL('../../../' + sourcePath, import.meta.url), 'utf8');
  assert.deepEqual(gapsFromDocuments([[sourcePath, source]]), []);
  const requiredSlots = new Map([
    ['quantity-constraint-bound-frame', ['bound_amount', 'bound_ref', 'bound_unit']],
    ['quantity-constraint-members-frame', ['members_ref', 'quantities']],
    ['quantity-constraint-comparison-frame', ['aggregation', 'query_ref', 'relation']],
    ['quantity-constraint-ownership-frame', ['query_ref', 'relation']]
  ]);
  const originalHeads = new Set([
  "quantity-constraint-bound-frame",
  "quantity-constraint-members-frame",
  "quantity-constraint-comparison-frame",
  "quantity-constraint-ownership-frame",
  "quantity-constraint-query-prefix",
  "quantity-constraint-bound-reference",
  "quantity-constraint-member-reference",
  "quantity-constraint-explicit-sum",
  "quantity-constraint-inclusive-bound",
  "quantity-constraint-list-conjunction"
]);
  const allReviewedHeads = new Set([...originalHeads, ...["quantity-constraint-within","quantity-constraint-exceeds","quantity-constraint-unknown","budget","capacity","amounts","quantities"]]);
  const document = parseLino(source);
  const schema = parseSchema(fs.readFileSync(new URL(
    '../../../data/meta/seed-metadata-schema.lino', import.meta.url), 'utf8'));
  const records = parseMeanings(sourcePath, source).filter(row => allReviewedHeads.has(row.name));
  assert.equal(records.length, 17);
  assert.deepEqual(findGaps(records, {...schema, completeSources: []}), []);
  for (const name of allReviewedHeads) {
    const node = document.children.find(row => row.name === name);
    assert(node, name);
    assert.deepEqual([...new Set(childrenNamed(node, 'lexeme').map(row => row.value))].sort(),
      ['en', 'es', 'hi', 'ru', 'zh'], name);
    for (const field of schema.requiredFields) assert(childValue(node, field).length > 0, name + ':' + field);
  }
  const owners = document.children.filter(node => originalHeads.has(node.name));
  assert.equal(owners.length, 10);
  for (const owner of owners) {
    const role = childValue(owner, 'role');
    for (const language of ['en', 'ru', 'hi', 'zh', 'es']) {
      const lexemes = childrenNamed(owner, 'lexeme').filter(node => node.value === language);
      assert.equal(lexemes.length, 1, role + ':' + language);
      const surfaces = childrenNamed(lexemes[0], 'surface').map(node => childValue(node, 'text'));
      assert(surfaces.length > 0);
      for (const surface of surfaces) {
        const slots = [...surface.matchAll(/\{([a-z_]+)\}/gu)].map(match => match[1]).sort();
        assert.deepEqual(slots, requiredSlots.get(role) ?? [], role + ':' + language);
        const encoded = writeLink([encodeWord('lexeme'), encodeWord(language), {quoted: surface}],
          {quotedText: 'json'});
        const decoded = parseLinks(encoded, {quotedText: 'json'});
        assert.equal(textOf(decoded[0][2]), surface);
      }
    }
  }
  const responses = parseLino(fs.readFileSync(new URL(
    '../../../data/seed/multilingual-responses-agentic.lino', import.meta.url), 'utf8'));
  for (const intent of ['quantity-constraint-within', 'quantity-constraint-exceeds',
    'quantity-constraint-unknown']) {
    const rows = childrenNamed(responses, 'response').filter(node => childValue(node, 'intent') === intent);
    assert.deepEqual(rows.map(node => childValue(node, 'language')).sort(), ['en', 'es', 'hi', 'ru', 'zh']);
    const english = childValue(rows.find(node => childValue(node, 'language') === 'en'), 'text');
    const slots = text => [...text.matchAll(/\{([a-z_]+)\}/gu)].map(match => match[1]).sort();
    for (const row of rows) {
      const text = childValue(row, 'text');
      assert(text.length > 0);
      assert.deepEqual(slots(text), slots(english));
      const encoded = writeLink([encodeWord(intent), {quoted: text}], {quotedText: 'json'});
      assert.equal(textOf(parseLinks(encoded, {quotedText: 'json'})[0][1]), text);
    }
  }

});

for (const [language, question] of [
  [
    "ru",
    "The budget is 100 ms. The amounts are 30 ms and 40 ms. Верно ли, что сумма в пределах бюджета?"
  ],
  [
    "hi",
    "The budget is 100 ms. The amounts are 30 ms and 40 ms. क्या योग बजट के भीतर है?"
  ],
  [
    "zh",
    "The budget is 100 ms. The amounts are 30 ms and 40 ms. 是否总和不超过预算？"
  ],
  [
    "es",
    "The budget is 100 ms. The amounts are 30 ms and 40 ms. ¿es la suma menor o igual que el presupuesto?"
  ]
]) {
  test('new lexical data does not enable an unpaired non-English quantity route: ' + language, async () => {
    const result = await host.solve(question);
    assert(!result.intent.startsWith('quantity_constraint'));
    assert(!result.evidence.some(value => value.startsWith('quantity_constraint:')));
    assert.equal(wasmAttempts, 0);
  });
}

test('parser grammar lexemes stay scoped outside ordinary semantic indexes', () => {
  const text = evaluate(host.context, "crateModule('crate/text_formalization.mjs')");
  const seed = evaluate(host.context, "crateModule('crate/seed_meanings.mjs')");
  const scoped = seed.meaningsWithRole(text.SCOPED_GRAMMAR_ROLE);
  const expected = [
    'quantity-constraint-bound-frame',
    'quantity-constraint-members-frame',
    'quantity-constraint-comparison-frame',
    'quantity-constraint-ownership-frame',
    'quantity-constraint-query-prefix',
    'quantity-constraint-bound-reference',
    'quantity-constraint-member-reference',
    'quantity-constraint-explicit-sum',
    'quantity-constraint-inclusive-bound',
    'quantity-constraint-list-conjunction'
  ];
  for (const slug of expected) assert(scoped.some(entry => entry.slug === slug), slug);
  for (const language of ['en', 'ru', 'hi', 'zh', 'es']) {
    const index = text.meaningIndex(language);
    for (const entry of scoped) {
      assert(!Array.from(index.values()).includes(entry.slug), language + ':' + entry.slug);
    }
  }
  assert(seed.wordsForRoleInLanguages('quantity-constraint-query-prefix', ['en']).includes('is the'));
  assert(seed.wordsForRoleInLanguages('quantity-constraint-bound-reference', ['en']).includes('budget'));
  assert(!text.functionWords('en').has('budget'));
  assert(!text.functionWords('en').has('capacity'));
  assert.equal(wasmAttempts, 0);
});

test('ordinary Moon copula keeps its semantic identity beside quantity grammar', () => {
  const text = evaluate(host.context, "crateModule('crate/text_formalization.mjs')");
  const page = evaluate(host.context, "crateModule('crate/page_formalization.mjs')");
  assert.equal(text.meaningIndex('en').get('is the'), 'wikidata_property_instance_of');
  const statement = text.clauseStatement('The Moon is the only natural satellite of Earth.', 'en', null, false);
  assert.equal(page.statementNotation(statement), "(moon wikidata_property_instance_of unknown:only unknown:natural unknown:satellite name:earth)");
  assert.equal(wasmAttempts, 0);
});
