import assert from 'node:assert/strict';
import test from 'node:test';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { resolve } from 'node:path';
import { selectedCandidate } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/solver_formalization.mjs';
import { candidateCompactSummary } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/translation_formalization.mjs';

const sourceIdentities = [
  ['js/agentic/crate/solver_formalization.mjs', '46d2b2b7b006935a1a1cb03d180faf63ba71dfa1f758cb16ca1cc925286d7c8c'],
  ['js/agentic/crate/translation_formalization.mjs', 'f4161079a1e37708915bef76e0ddab96a4104745668c051a3d0b0f7330db822b'],
];
const repository = '/Users/konard/Code/Archive/link-assistant/formal-ai';
const anchor = (id) => ({ kind: 'wikidata_item', id, label: 'irrelevant label', source: 'fixture', score: 1 });
const candidate = (slots, unresolved_terms = [], score = 1) => ({
  source_text: 'fixture source', language: 'en', slots, unresolved_terms, score,
});
const decoy = candidate([{ role: 'subject', surface: 'decoy', anchor: anchor('Q1') }], [], 1000);
const chosen = candidate([
  { role: 'subject', surface: 'apple', anchor: anchor('Q89') },
  { role: 'predicate', surface: 'nutrition', anchor: { ...anchor('P5972'), kind: 'wikidata_property' } },
], ['rind', 'पीला']);
const empty = candidate([]);
const reordered = candidate([
  { role: 'object', surface: 'first', anchor: anchor('Q2') },
  { role: 'subject', surface: 'second', anchor: anchor('Q3') },
]);
const selection = (candidates, decision) => ({ candidates, probabilities: candidates.map(() => 0), decision });
const deepFreeze = (value) => {
  if (value && typeof value === 'object') {
    Object.values(value).forEach(deepFreeze);
    Object.freeze(value);
  }
  return value;
};
export const acceptanceCases = [
  { name: 'selected index wins over highest score and first position', input: selection([decoy, chosen], { kind: 'selected', index: 1 }), expected: 'subject=Q89 predicate=P5972 unresolved=rind|पीला' },
  { name: 'selected empty candidate is a string', input: selection([empty], { kind: 'selected', index: 0 }), expected: 'empty' },
  { name: 'slot order is preserved', input: selection([decoy, empty, reordered], { kind: 'selected', index: 2 }), expected: 'object=Q2 subject=Q3' },
  { name: 'unresolved terms without slots are retained', input: selection([candidate([], ['文', 'a b'])], { kind: 'selected', index: 0 }), expected: 'unresolved=文|a b' },
  { name: 'clarification has no selected summary', input: selection([decoy, chosen], { kind: 'clarify', top: 1, runnerUp: 0, margin: 0, epsilon: 1 }), expected: null },
  { name: 'no-candidate decision is null', input: selection([], { kind: 'no_candidate' }), expected: null },
  { name: 'out-of-range selected index is null', input: selection([decoy], { kind: 'selected', index: 3 }), expected: null },
];
export const heldOutCases = [
  { id: 'alias-a', destination: 'nested/selection-caption.mjs', exportName: 'describeChosen', parameterName: 'pick', sourceBindings: { selectedCandidate: 'chosenRecord', candidateCompactSummary: 'shortRecord' }, fixtureNames: ['selected index wins over highest score and first position', 'clarification has no selected summary'] },
  { id: 'alias-b', destination: 'reports/caption.mjs', exportName: 'candidateCaption', parameterName: 'result', sourceBindings: { selectedCandidate: 'recordForDecision', candidateCompactSummary: 'renderRecord' }, fixtureNames: ['slot order is preserved', 'unresolved terms without slots are retained', 'out-of-range selected index is null'] },
];

test('fixture: source preimages remain exact', () => {
  for (const [path, expected] of sourceIdentities) {
    const bytes = readFileSync(resolve(repository, path));
    assert.equal(createHash('sha256').update(bytes).digest('hex'), expected, path);
  }
});
test('fixture: selector returns the chosen object without cloning or mutation', () => {
  const input = deepFreeze(structuredClone(acceptanceCases[0].input));
  const before = JSON.stringify(input);
  assert.strictEqual(selectedCandidate(input), input.candidates[1]);
  assert.equal(JSON.stringify(input), before);
  assert.equal(selectedCandidate(acceptanceCases[4].input), null);
  assert.equal(selectedCandidate(acceptanceCases[6].input), null);
});
test('fixture: summaries agree with independently stated strings', () => {
  assert.equal(candidateCompactSummary(deepFreeze(structuredClone(chosen))), 'subject=Q89 predicate=P5972 unresolved=rind|पीला');
  assert.equal(candidateCompactSummary(deepFreeze(structuredClone(empty))), 'empty');
  assert.equal(candidateCompactSummary(deepFreeze(structuredClone(reordered))), 'object=Q2 subject=Q3');
});
for (const fixture of acceptanceCases) {
  test('composition: ' + fixture.name, async () => {
    const target = process.env.SELECTED_SUMMARY_MODULE || new URL('./repair-summary.mjs', import.meta.url).pathname;
    const exportName = process.env.SELECTED_SUMMARY_EXPORT || 'selectedSummary';
    const module = await import(pathToFileURL(resolve(target)).href);
    assert.equal(typeof module[exportName], 'function', 'required named export');
    const input = deepFreeze(structuredClone(fixture.input));
    const before = JSON.stringify(input);
    assert.strictEqual(module[exportName](input), fixture.expected);
    assert.equal(JSON.stringify(input), before, 'input must remain unchanged');
  });
}
