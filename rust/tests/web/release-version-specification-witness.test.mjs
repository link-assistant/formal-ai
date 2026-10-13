import test from 'node:test';
import assert from 'node:assert/strict';
import { serialize, deserialize } from 'node:v8';
import { createHash } from 'node:crypto';
import { validateVersionWitness } from '../../../scripts/refresh-release-specification-witness.mjs';
const beforeCargo = '[package]\nname = "formal-ai"\nversion = "0.352.1"\n',
  afterCargo = beforeCargo.replace('0.352.1', '0.353.0');
const hash = text => createHash('sha256').update(text).digest('hex');
function witnesses() {
  const before = {
    inventory: {
      total: 1182,
      cases: Array.from({
        length: 4
      }, (_, id) => ({
        id,
        checks: [{
          same: true
        }],
        program: {
          fixtures: [{
            file: 'repo:rust/Cargo.toml',
            sha256: hash(beforeCargo)
          }]
        }
      })),
      unsupported: [{
        id: 'unknown',
        reason: 'preserved'
      }]
    },
    sourceManifest: [{
      path: 'native.rs',
      sha256: 'source'
    }]
  };
  const after = structuredClone(before);
  for (const item of after.inventory.cases) item.program.fixtures[0].sha256 = hash(afterCargo);
  return {
    before,
    after
  };
}
test('all actual Cargo witnesses change together while all other fields remain exact', () => {
  const {
    before,
    after
  } = witnesses();
  assert.equal(validateVersionWitness(serialize(before), serialize(after), beforeCargo, afterCargo), 4);
});
for (const mutation of ['case', 'assertion', 'unsupported', 'scope', 'native', 'stale', 'drop']) test('version refresh refuses ' + mutation + ' drift', () => {
  const {
    before,
    after
  } = witnesses();
  if (mutation === 'case') after.inventory.cases[0].id = 'replaced';
  if (mutation === 'assertion') after.inventory.cases[0].checks = [];
  if (mutation === 'unsupported') after.inventory.unsupported[0].reason = 'accepted';
  if (mutation === 'scope') after.inventory.total = 1;
  if (mutation === 'native') after.sourceManifest[0].sha256 = 'new';
  if (mutation === 'stale') before.inventory.cases[0].program.fixtures[0].sha256 = 'wrong';
  if (mutation === 'drop') after.inventory.cases.pop();
  assert.throws(() => validateVersionWitness(serialize(before), serialize(after), beforeCargo, afterCargo));
});
test('dependency changes cannot become version witness maintenance', () => {
  const {
    before,
    after
  } = witnesses();
  assert.throws(() => validateVersionWitness(serialize(before), serialize(after), beforeCargo, afterCargo + '[dependencies]\nevil="1"\n'));
  assert.throws(() => validateVersionWitness(serialize(before), serialize(after), beforeCargo, beforeCargo));
});
