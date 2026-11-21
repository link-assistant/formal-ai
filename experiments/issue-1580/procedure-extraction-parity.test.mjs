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

test('native constant coverage survives extraction into an ordinarily resolved helper module', () => {
  const owner = 'rust/tests/unit/specification/openai_compatibility';
  const inventory = specificationCases(root);
  assert(inventory.cases.some(entry => entry.id === owner + '.rs::canonical_model_id_is_formal_ai'),
    'the previously passing native constant case must remain carried to JavaScript');
  const parent = readFileSync(root + owner + '.rs', 'utf8');
  const registry = readFileSync(root + 'rust/tests/unit/specification/mod.rs', 'utf8');
  assert.match(registry, /^mod openai_compatibility;$/mu);
  assert.match(parent, /^mod streaming_diagnostics;$/mu);
  assert(!parent.includes('#[path = "openai_compatibility/streaming_diagnostics.rs"]'));
  const child = readFileSync(root + owner + '/streaming_diagnostics.rs', 'utf8');
  assert.match(child, /^pub\(super\) fn streaming_chat_completion_emits_reasoning_content_delta\(\)/mu);
});
