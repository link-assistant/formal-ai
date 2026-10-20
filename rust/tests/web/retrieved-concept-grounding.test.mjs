import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import test, { before } from 'node:test';
import { installHost } from '../../../js/agentic/host.mjs';
import { parseLino, REPO_ROOT } from '../../../js/server/lino.mjs';
import { ConceptGraph, formalizeDeeply } from '../../../js/agentic/crate/formalization_concept_links.mjs';
import { conceptFromSense, conceptIdentifier, conceptSenseContentIdentifier, findCue, readGloss, relationsIn } from '../../../js/agentic/crate/formalization_concepts.mjs';
import { structuresFor } from '../../../js/agentic/crate/formalization_structures.mjs';
import { normalizePrompt } from '../../../js/agentic/crate/engine.mjs';
import { sentences } from '../../../js/agentic/crate/formalization_segment.mjs';

before(() => installHost({ parseLino, readText: file => readFileSync(path.join(REPO_ROOT, file), 'utf8'), realm: {} }));
const sense = (surface, gloss = 'a category') => ({surface, lemma: surface, language: 'en', gloss,
  source_id: 'declared-test-lookup', source_url: 'https://example.invalid/capture', sha256: 'a'.repeat(64),
  license_name: 'test-fixture', depth: 100, content_id: 'untrusted-caller-id'});

test('retrieved captured senses preserve native attribution, genus and stable evidence identity', () => {
  const captured = JSON.parse(readFileSync(path.join(REPO_ROOT, 'rust/tests/fixtures/issue-1138-b1/expected-senses.json'), 'utf8'))[0];
  const input = { ...captured, source_id: captured.sourceId, source_url: captured.sourceUrl,
    license_name: captured.licenseName };
  assert.equal(conceptSenseContentIdentifier(input), captured.contentId);
  const concept = conceptFromSense(input);
  assert.equal(concept.genus, 'line');
  assert.equal(concept.gloss, captured.gloss);
  assert.equal(concept.source_url, captured.sourceUrl);
  assert.equal(concept.sha256, captured.sha256);
  assert.equal(concept.license_name, captured.licenseName);
  assert.equal(conceptIdentifier('  Красный кот! '), 'concept:красный-кот');
  assert.equal(conceptFromSense(sense('', 'a category')), null);
  assert.equal(conceptFromSense(sense('valid', '   ')), null);
});

test('successful lookup grounds concepts instead of throwing and recursive needs remain finite', () => {
  const calls = [];
  const lookup = { lookup(need) {
    calls.push(need.subject);
    if (need.subject === 'zebraneed' || need.subject === 'wombatneed') return { kind: 'found', senses: [sense(need.subject, 'a quuxchild')] };
    return { kind: 'not_found', consulted: [] };
  }};
  const graph = formalizeDeeply('zebraneed requires wombatneed.', 'doc:grounding', lookup, {}, 1);
  assert.deepEqual(graph.concepts.map(concept => concept.label), ['wombatneed', 'zebraneed']);
  assert.ok(graph.concepts.every(concept => concept.depth === 0));
  assert.equal(calls.filter(term => term === 'quuxchild').length, 1, 'same child is asked once across senses');
  const child = graph.needs.find(need => need.subject === 'quuxchild');
  assert.equal(child.depth, 1);
  assert.equal(child.state, 'unsatisfiable');
  assert.ok(child.raised_by.startsWith('need_'));
  assert.match(child.source_span, /^sense_[a-z0-9]+@2:11$/u);
  assert.ok(graph.needs.filter(need => need.state === 'satisfied').every(need => need.satisfied_by !== 'untrusted-caller-id'));
  assert.equal(graph.maxDepthReached(), 1);
  assert.equal(graph.relations.find(relation => relation.kind === 'requires')?.subject, 'concept:zebraneed');
  assert.equal(graph.relations.find(relation => relation.kind === 'requires')?.object, 'concept:wombatneed');
  assert.deepEqual(graph.groundedRatio(), [2, 3]);
  assert.match(graph.toLinksNotation(), /source_url "https:\/\/example.invalid\/capture"/u);
});

test('zero depth, cyclic senses and unrelated returned senses do not invent graph progress', () => {
  const calls = [];
  const lookup = { lookup(need) { calls.push(need.subject); return {kind: 'found', senses: [
    sense(need.subject, `a ${need.subject === 'zebraneed' ? 'wombatneed' : 'zebraneed'}`),
    sense('unrequestedterm', 'a category'),
  ]}; }};
  const zero = formalizeDeeply('zebraneed', 'doc:zero', lookup, {}, 0);
  assert.deepEqual(calls, ['zebraneed']);
  assert.equal(zero.concepts.length, 1);
  calls.length = 0;
  const cyclic = formalizeDeeply('zebraneed', 'doc:cycle', lookup, {}, 10);
  assert.deepEqual(calls, ['zebraneed', 'wombatneed']);
  assert.equal(cyclic.concepts.length, 2);
  assert.equal(cyclic.needs.length, 2);
  for (const value of [-1, Infinity, NaN, 1.5]) assert.throws(() => formalizeDeeply('zebraneed', 'doc:x', lookup, {}, value), /invalid_concept_depth/u);
});

test('relations require grounded terms on both sides with exact clause provenance', () => {
  const graph = new ConceptGraph({doc_id: 'doc:relation', concepts: [{id: 'concept:cat', label: 'cat'}]});
  assert.deepEqual(relationsIn(sentences('cat requires dog.')[0], graph), []);
  graph.entities.push({id: 'entity:dog', label: 'dog'});
  const relation = relationsIn(sentences('cat requires dog.')[0], graph).find(row => row.kind === 'requires');
  assert.equal(relation.subject, 'concept:cat'); assert.equal(relation.object, 'entity:dog');
  assert.equal(relation.source_span, 'doc:relation@0:17');
  assert.deepEqual(relationsIn(sentences('bobcat requires dog.')[0], graph), [], 'substring is not a grounded mention');
});

test('multilingual cues preserve source boundaries through Unicode case expansion', () => {
  const text = 'İx requires cat';
  assert.deepEqual(findCue(text, 'requires'), [3, 11]);
  assert.equal(findCue('İ', 'i'), null, 'half a folded scalar is not an original source token');
  assert.equal(findCue('format', 'a'), null);
  assert.deepEqual(readGloss('a word in which no letter repeats'), ['word', ['no letter repeats']]);
  const graph = new ConceptGraph({doc_id: 'doc:ru', concepts: [{id: 'cat',label:'кот'},{id:'dog',label:'собака'}]});
  assert.equal(relationsIn(sentences('кот требует собака.')[0], graph).find(row => row.kind === 'requires')?.language, 'ru');
});

test('structural meaning discovery applies native quantifier and reduction precedence', () => {
  const ids = prompt => structuresFor(normalizePrompt(prompt)).map(structure => structure.id);
  assert.ok(ids('print a value').includes('print_stdout'));
  assert.ok(ids('at least one item').includes('quantifier_any'));
  assert.ok(!ids('at least one of every pair').includes('quantifier_all'));
  assert.ok(!ids('the sum of every integer').includes('quantifier_all'));
});
