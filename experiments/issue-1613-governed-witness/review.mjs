// Compare governed specification snapshots before and after the default-branch merge.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { isDeepStrictEqual } from 'node:util';
import { deserialize } from 'node:v8';

const [branchSnapshot, mainSnapshot, mode] = process.argv.slice(2);
assert.ok(branchSnapshot && mainSnapshot, 'Supply the pre-merge branch and main witness files.');
assert.ok(mode === undefined || mode === '--release-merge');
const repo = new URL('../../', import.meta.url);
const manifest = 'rust/Cargo.toml';
const witness = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const before = deserialize(readFileSync(branchSnapshot));
const main = deserialize(readFileSync(mainSnapshot));
const after = deserialize(readFileSync(new URL(witness, repo)));

function changes(beforeRows, afterRows, identifier) {
  const left = new Map(beforeRows.map(record => [identifier(record), record]));
  const right = new Map(afterRows.map(record => [identifier(record), record]));
  return [...new Set([...left.keys(), ...right.keys()])].flatMap(id => {
    if (isDeepStrictEqual(left.get(id), right.get(id))) return [];
    const change = !left.has(id) ? 'added' : !right.has(id) ? 'removed' : 'updated';
    return [{ id, change }];
  });
}

function cargoWitnesses(snapshot) {
  const fixtures = snapshot.inventory.cases.flatMap(item => item.program?.fixtures ?? []);
  return [...new Set(fixtures.filter(fixture => fixture.file === `repo:${manifest}`).map(fixture => fixture.sha256))];
}

const lookupSpec = 'rust/tests/unit/specification/project_lookups.rs';
const roleplaySpec = 'rust/tests/unit/specification/prompt_variations_facts.rs';
const cargoSha256 = createHash('sha256').update(readFileSync(new URL(manifest, repo))).digest('hex');
const caseChanges = changes(before.inventory.cases, after.inventory.cases, record => record.id);
const unsupportedChanges = changes(before.inventory.unsupported, after.inventory.unsupported, record => record.id);
const sourceChanges = changes(before.sourceManifest, after.sourceManifest, record => record.path);

assert.deepEqual(cargoWitnesses(after), [cargoSha256]);
assert.deepEqual(cargoWitnesses(main), [cargoSha256]);
if (mode === '--release-merge') {
  assert.equal(cargoWitnesses(before).length, 1);
  assert.notEqual(cargoWitnesses(before)[0], cargoSha256);
  for (const item of before.inventory.cases) {
    for (const fixture of item.program?.fixtures ?? []) {
      if (fixture.file === `repo:${manifest}`) fixture.sha256 = cargoSha256;
    }
  }
  assert.deepEqual(before.inventory, after.inventory);
  assert.deepEqual(sourceChanges, []);
} else {
  assert.ok(caseChanges.length > 0 && caseChanges.every(record => record.id.startsWith(`${lookupSpec}::`)));
  assert.ok(unsupportedChanges.every(record => record.id.startsWith(`${lookupSpec}::`)));
  assert.deepEqual(sourceChanges.map(record => record.id).sort(), [lookupSpec, roleplaySpec].sort());
  assert.ok(sourceChanges.every(record => record.change === 'updated'));
}

console.log(JSON.stringify({
  oldCases: before.inventory.cases.length,
  newCases: after.inventory.cases.length,
  cargoSha256,
  caseChanges,
  unsupportedChanges,
  sourceChanges,
}, null, 2));
