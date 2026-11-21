// Procedure refactors must retain the cases actually carried by JavaScript.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import test from 'node:test';
import {specificationCases, testFunctions, tokenize} from '../../scripts/lib/rust-specification-cases.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const owner = 'rust/tests/unit/specification/reasoning_paths_procedures.rs';
test('procedure coverage keeps the original JavaScript case and native test identities', () => {
  const inventory = specificationCases(root);
  const expected = [
    'telegraphic_how_requires_a_known_procedural_action',
    'greeting_prefixed_russian_connect_how_to_composes_procedure',
    'procedural_elaboration_followup_rebinds_to_prior_how_to',
    'procedural_elaboration_requires_a_prior_how_to',
    'procedural_elaboration_followup_covers_supported_languages',
  ];
  assert.deepEqual(inventory.cases.filter(entry => entry.id.startsWith(owner + '::'))
    .map(entry => entry.id.slice(owner.length + 2)), expected);
  const source = readFileSync(root + owner, 'utf8');
  const names = testFunctions(tokenize(source)).map(entry => entry.name);
  assert.equal(names.length, 16);
  assert.equal(new Set(names).size, names.length);
  for (const name of expected) assert(names.includes(name), 'native test identity disappeared: ' + name);
});
