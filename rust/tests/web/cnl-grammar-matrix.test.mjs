// Generalized CNL meta test (issue #1584): instead of listing outputs by hand,
// enumerate every declared meaning graph, every language grammar, every
// template style and every output configuration, and check the properties
// that must hold for all of them (see support/cnl-matrix.mjs):
//   1. inventory  - every language has the 65 primes, every term and every output style;
//   2. uniqueness - no two concepts share an explication in one register;
//   3. coverage   - every output style can say every answerable graph;
//   4. inversion  - every realization parses back to the graph it came from;
//   5. compliance - every feasible profile x density projection is compliant and keeps its meaning;
//   6. round trip - free-form -> Links Notation -> dense -> primes -> cnl-default keeps the graph;
//   7. dialogue   - every phrasing of one question gets the same answer, ambiguity asks first.
// One language at a time: node experiments/issue-1584/check-language.mjs <language>.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {controlledDialogue} from '../../../js/agentic/crate/cnl_output.mjs';
import {
  complianceFailures, complianceMatrix, DENSITIES, infeasibilityLedger, languageProperties, LEDGER_PATH, policy,
} from './support/cnl-matrix.mjs';

const languages = policy.languages.map(language => language.id);

test('every supported language has a grammar', () => {
  assert.deepEqual(Object.keys(policy.grammars).sort(), [...languages].sort());
});

for (const language of languages) {
  test(`${language}: inventory, uniqueness, coverage, inversion, round trip and dialogue hold`, () => {
    const failures = languageProperties(language);
    for (const [property, list] of Object.entries(failures)) assert.deepEqual(list.slice(0, 10), [], `${language} ${property}`);
  });
}

// The oracle validates the whole search domain exhaustively, independently of
// the projection's ordered search: if any candidate is compliant, the projection
// must be compliant; if none is, the combination is infeasible and the projection
// must still keep the named profile whenever some candidate keeps it. Infeasible
// combinations are not hidden behind a ratio: they must match the reviewed ledger
// (regenerate with `node experiments/issue-1584/feasibility.mjs --write`).
test('compliance: every feasible projection is compliant and the infeasible rest matches the ledger', () => {
  const rows = complianceMatrix();
  assert.deepEqual(complianceFailures(rows).slice(0, 20), []);
  assert.deepEqual(infeasibilityLedger(rows), JSON.parse(readFileSync(LEDGER_PATH, 'utf8')));
});

test('in-dialog switching is deterministic for every configuration', () => {
  const prompt = 'Is a cat an animal?';
  for (const profile of policy.profiles) {
    for (const density of DENSITIES) {
      const configuration = {output_standard: profile.id, lexical_density: density};
      const first = controlledDialogue(prompt, [], configuration, policy).content;
      assert.equal(controlledDialogue(prompt, [], {}, policy).intent, 'cnl_answer');
      assert.equal(controlledDialogue(prompt, [], configuration, policy).content, first);
    }
  }
});
