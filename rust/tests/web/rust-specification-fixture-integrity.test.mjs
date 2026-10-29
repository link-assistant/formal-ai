// Cheap source binding; full reader/complete-object parity remains on main.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync} from 'node:fs';
import {deserialize} from 'node:v8';
import test from 'node:test';

const root = new URL('../../../', import.meta.url);
const witness = deserialize(readFileSync(new URL('experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8', root)));
const hash = file => createHash('sha256').update(readFileSync(new URL(file, root))).digest('hex');

test('every governed repository fixture matches its actual current source bytes', () => {
  const fixtures = new Map();
  for (const item of witness.inventory.cases) for (const fixture of item.program?.fixtures ?? []) {
    assert.match(fixture.file, /^repo:(?:rust|desktop|js|vscode)\//u);
    const file = fixture.file.slice('repo:'.length);
    assert(!file.split('/').includes('..'), 'fixture must stay inside repository source');
    if (fixtures.has(file)) assert.equal(fixtures.get(file), fixture.sha256, 'contradictory fixture: ' + file);
    fixtures.set(file, fixture.sha256);
  }
  assert(fixtures.size > 0);
  for (const [file, expected] of fixtures) {
    assert.equal(hash(file), expected, 'refresh and review the complete governed witness after changing ' + file);
  }
});

test('governed authoritative native source hashes match without parsing or compiling Rust', () => {
  assert(witness.sourceManifest.length > 0);
  for (const source of witness.sourceManifest) {
    assert.match(source.path, /^rust\/tests\/unit\/specification\/(?!.*\.\.)[^\0]+\.rs$/u);
    assert.equal(hash(source.path), source.sha256, 'review governed native source: ' + source.path);
  }
});
