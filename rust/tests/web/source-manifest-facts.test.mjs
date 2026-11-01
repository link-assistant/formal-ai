import { observedManifestEvidence, supportedManifestClaims } from '../../../scripts/work-item-qualification/manifest-claims.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { observedManifestFacts } from '../../../js/agentic/source_manifest_facts.mjs';
const fixture = JSON.parse(readFileSync(new URL('../fixtures/source-manifest-facts.json', import.meta.url), 'utf8'));
for (const entry of fixture.cases) test(entry.name, () => {
  if (entry.rejected) {
    assert.throws(() => observedManifestFacts(entry.owner, entry.source));
    return;
  }
  const actual = observedManifestFacts(entry.owner, entry.source);
  const sourceIdentity = createHash('sha256').update(entry.source).digest('hex');
  for (const fact of actual) {
    assert.equal(fact.manifest, entry.owner);
    assert.equal(fact.sourceIdentity, sourceIdentity);
  }
  const withoutIdentity = actual.map(({
    manifest,
    sourceIdentity,
    ...fact
  }) => fact);
  if (entry.expected) assert.deepEqual(withoutIdentity, entry.expected);else for (const expected of entry.expectedSubset) assert.ok(withoutIdentity.some(fact => assertionEqual(fact, expected)), JSON.stringify(expected));
});
const assertionEqual = (left, right) => {
  try {
    assert.deepEqual(left, right);
    return true;
  } catch {
    return false;
  }
};
test('untrusted owner, UTF8 size and depth remain bounded', () => {
  for (const owner of ['../Cargo.toml', '/tmp/package.json', 'a//package.json', 'a/../package.json', 'manifest.yml']) assert.throws(() => observedManifestFacts(owner, '{}'));
  assert.throws(() => observedManifestFacts('package.json', ' '.repeat(262145)));
  assert.throws(() => observedManifestFacts('package.json', '['.repeat(66) + '0' + ']'.repeat(66)));
});
test('independent host parser checks literal declarations and refuses forged claims', () => {
  const entry = fixture.cases.find(row => row.name === 'npm literal declarations and unrelated fields');
  const evidence = observedManifestEvidence([{
    path: entry.owner,
    content: entry.source
  }]);
  assert.equal(evidence.observations[0].status, 'observed');
  const actual = observedManifestFacts(entry.owner, entry.source);
  for (const fact of actual) assert.ok(evidence.facts.some(row => assertionEqual(row, fact)));
  const fact = actual.find(row => row.name === 'river-tools');
  const block = claim => '\x60\x60\x60formal-ai-source-claim\n' + JSON.stringify(claim) + '\n\x60\x60\x60';
  assert.equal(supportedManifestClaims(block(fact), evidence.facts, evidence.observations)[0].status, 'verified-literal-declaration');
  assert.equal(supportedManifestClaims(block({
    ...fact,
    version: 'invented'
  }), evidence.facts, evidence.observations)[0].status, 'contradicted');
  assert.equal(supportedManifestClaims(block({
    ...fact,
    name: 'absent-module'
  }), evidence.facts, evidence.observations)[0].status, 'contradicted');
  assert.equal(supportedManifestClaims(block({
    ...fact,
    sourceIdentity: 'f'.repeat(64)
  }), evidence.facts, evidence.observations)[0].status, 'unresolved');
  assert.equal(supportedManifestClaims(block(fact), evidence.facts, [])[0].status, 'unresolved');
  const duplicate = '\x60\x60\x60formal-ai-source-claim\n{"kind":"manifest-declaration","kind":"cargo-feature"}\n\x60\x60\x60';
  assert.equal(supportedManifestClaims(duplicate, evidence.facts, evidence.observations)[0].status, 'unresolved');
  const invalid = observedManifestEvidence([{
    path: 'package.json',
    content: '{"dependencies":{"river":"1","river":"2"}}'
  }]);
  assert.equal(invalid.observations[0].status, 'unknown');
  assert.deepEqual(invalid.facts, []);
});
test('independent TOML implementation agrees on explicit target and feature facts', () => {
  const entry = fixture.cases.find(row => row.name === 'Cargo aliases targets options and feature graph');
  const evidence = observedManifestEvidence([{
    path: entry.owner,
    content: entry.source
  }]);
  if (evidence.observations[0].status === 'unknown' && process.env.CI !== 'true') {
    assert.equal(evidence.observations[0].reason, 'IndependentTomlParserUnavailable');
    return;
  }
  assert.equal(evidence.observations[0].status, 'observed');
  const facts = observedManifestFacts(entry.owner, entry.source);
  assert.equal(evidence.facts.length, facts.length);
  for (const fact of facts) assert.ok(evidence.facts.some(row => assertionEqual(row, fact)));
});
test('flat declaration counts terminate with an explicit budget error', () => {
  const source = '[dependencies]\n' + Array.from({
    length: 1025
  }, (_, index) => 'module-' + index + '="1"').join('\n');
  assert.throws(() => observedManifestFacts('Cargo.toml', source), /Budget/);
});
