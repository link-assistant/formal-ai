// Review the sole governed source delta for the four-example native benchmark.
// Supplied review and edits receive zero independent authorship credit.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {deserialize} from 'node:v8';

const [prefix, action] = process.argv.slice(2);
assert(prefix && (action === undefined || action === '--refresh'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const previousBytes = readFileSync(prefix + '.before.v8');
const currentBytes = readFileSync(prefix + '.after.v8');
assert.equal(hash(previousBytes), '36a18e8456ef7d2dfa6023248eee9e1916c68b4384d51d79f90e4cfa23524f88');
const previous = deserialize(previousBytes), current = deserialize(currentBytes);
const reviewed = structuredClone(previous);
const file = 'rust/tests/unit/specification/benchmarks.rs';
const source = reviewed.sourceManifest.find(item => item.path === file);
assert.equal(source.sha256, 'd258e048c3dc81f26df48a1167170065c0f71bacea59b307b8e7dc3d65284fed');
const after = '075df58926f318d54e2f9b11bf1c27790edca6344552dea72ceeef771913e80c';
assert.equal(hash(readFileSync(file)), after);
source.sha256 = after;
assert.deepEqual(current, reviewed, 'Only the reviewed benchmark source hash may change');
assert.equal(current.inventory.total, 1193);
assert.equal(current.inventory.cases.length, 223);
assert.equal(current.inventory.unsupported.length, 970);
const baseline = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const readerSha256 = hash(readFileSync('scripts/lib/rust-specification-cases.mjs'));
const receipt = {previousSha256: hash(previousBytes), proposedSha256: hash(currentBytes),
  readerSha256, file, after, completeObjectsCompared: true, total: 1193,
  supported: 223, unsupported: 970, independentAuthorshipCredit: 0,
  nativeCompilationOrExecution: false, refreshed: false};
if (action === '--refresh') {
  assert.equal(hash(readFileSync(baseline)), hash(previousBytes));
  execFileSync(process.execPath, ['rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
    '--refresh-reviewed-baseline', '--reviewed-reader-sha=' + readerSha256,
    '--previous-baseline-sha=' + hash(previousBytes)], {stdio: 'inherit'});
  assert.deepEqual(deserialize(readFileSync(baseline)), current);
  receipt.refreshed = true;
  receipt.refreshedSha256 = hash(readFileSync(baseline));
}
writeFileSync(prefix + '.reviewed.json', JSON.stringify(receipt, null, 2) + '\n');
console.log(JSON.stringify(receipt, null, 2));
