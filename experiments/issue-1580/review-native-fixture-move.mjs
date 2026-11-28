// Explicit one-time review of the isolated publisher fixture extraction.
// Usage: node experiments/issue-1580/review-native-fixture-move.mjs PREFIX [--refresh]
// PREFIX is the complete before/after capture from issue-1584/review-witness.mjs.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {deserialize} from 'node:v8';

const [prefix, action] = process.argv.slice(2);
assert(prefix, 'Complete before/after capture prefix required');
assert(action === undefined || action === '--refresh');
const hash = value => createHash('sha256').update(value).digest('hex');
const previousBytes = readFileSync(prefix + '.before.v8');
const currentBytes = readFileSync(prefix + '.after.v8');
const before = deserialize(previousBytes);
const current = deserialize(currentBytes);
assert.equal(hash(previousBytes), 'aef751a9fbadf5c6ec8b5950dab2175e961370bfe7f41811e71effc023628132');
const reviewed = structuredClone(before);
const sources = new Map([
  ['rust/tests/unit/specification/prompt_variations_facts.rs', ['ff366fbb8f69f8d3bd762335bffb928ae4bd33ab751e3d06d39ec4915c265da3', 'e727bba32aba887a47cda501a929ea1797ecbb7db3c1eb57482ef5838c356244']],
  ['rust/tests/unit/specification/pronoun_topic_follow_up.rs', ['a744e3bbafff115590c1fdde77fa21ceeead641e116bdde94b693d72f74e4e30', '953866dba2523fbb90dea3d0dc602904b671b9b4fca1fe144a83c7a2a6264bc9']],
  ['rust/tests/unit/specification/synthesis.rs', ['e3780a042817d225cda69f33ef91f5c1d88842bb2c0d8065d2c58aaba9ea440e', 'e4c2209c736d707951dee2c040d1e14decf00c32e0d5485539a4b9693ffc998b']],
]);
const changes = [];
for (const source of reviewed.sourceManifest) {
  const pair = sources.get(source.path);
  if (!pair) continue;
  assert.equal(source.sha256, pair[0]);
  assert.equal(hash(readFileSync(source.path)), pair[1]);
  changes.push({kind: 'native-source', file: source.path, before: pair[0], after: pair[1]});
  source.sha256 = pair[1];
}
const fixtureFile = 'repo:rust/tests/unit/specification/pronoun_topic_follow_up.rs';
for (const item of reviewed.inventory.cases) for (const fixture of item.program?.fixtures ?? []) {
  if (fixture.file !== fixtureFile) continue;
  const pair = sources.get(fixture.file.slice('repo:'.length));
  assert.equal(fixture.sha256, pair[0]);
  changes.push({kind: 'program-fixture', id: item.id, file: fixture.file, before: pair[0], after: pair[1]});
  fixture.sha256 = pair[1];
}
assert.equal(changes.filter(item => item.kind === 'native-source').length, 3);
assert.equal(changes.filter(item => item.kind === 'program-fixture').length, 3);
// Compare the entire object graph after permitting only the six reviewed hashes.
assert.deepEqual(current, reviewed, 'Unexpected semantic, case, unsupported or source delta');
assert.equal(current.inventory.total, 1193);
assert.equal(current.inventory.cases.length, 223);
assert.equal(current.inventory.unsupported.length, 970);
const baseline = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const readerSha256 = hash(readFileSync('scripts/lib/rust-specification-cases.mjs'));
const receipt = {previousSha256: hash(previousBytes), proposedSha256: hash(currentBytes),
  readerSha256, completeObjectsCompared: true, changes, total: 1193, cases: 223,
  unsupported: 970, independentAuthorshipCredit: 0,
  nativeCompilationOrExecution: false, refreshed: false};
if (action === '--refresh') {
  assert.equal(hash(readFileSync(baseline)), hash(previousBytes), 'Governed witness changed since capture');
  execFileSync(process.execPath, ['rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
    '--refresh-reviewed-baseline', '--reviewed-reader-sha=' + readerSha256,
    '--previous-baseline-sha=' + hash(previousBytes)], {stdio: 'inherit'});
  assert.deepEqual(deserialize(readFileSync(baseline)), current);
  receipt.refreshedSha256 = hash(readFileSync(baseline));
  receipt.refreshed = true;
}
writeFileSync(prefix + '.reviewed.json', JSON.stringify(receipt, null, 2) + '\n');
console.log(JSON.stringify(receipt, null, 2));
