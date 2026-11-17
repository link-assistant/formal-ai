// Review the complete governed delta before refreshing with the maintained tool.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {deserialize} from 'node:v8';
const file = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const reader = 'scripts/lib/rust-specification-cases.mjs';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const beforeBytes = readFileSync(file), before = deserialize(beforeBytes);
const result = spawnSync(process.execPath, [
  'rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
  '--refresh-reviewed-baseline',
  '--reviewed-reader-sha=' + hash(readFileSync(reader)),
  '--previous-baseline-sha=' + hash(beforeBytes),
], {stdio: 'inherit'});
assert.equal(result.status, 0);
const afterBytes = readFileSync(file), after = deserialize(afterBytes);
assert.deepEqual(after.inventory, before.inventory, 'complete cases and unsupported records must remain unchanged');
assert.equal(after.sourceManifest.length, before.sourceManifest.length);
const changes = after.sourceManifest.flatMap((item, i) => {
  assert.equal(item.path, before.sourceManifest[i].path);
  return item.sha256 === before.sourceManifest[i].sha256 ? [] : [{...item, previousSha256: before.sourceManifest[i].sha256}];
});
assert.deepEqual(changes.map(item => item.path), ['rust/tests/unit/specification/external_benchmarks.rs']);
const receipt = {
  readerSha256: hash(readFileSync(reader)), previousWitnessSha256: hash(beforeBytes),
  refreshedWitnessSha256: hash(afterBytes), completeInventoryUnchanged: true,
  cases: after.inventory.cases.length, sourceManifestChanges: changes,
  reviewedChange: 'Both unavailable-benchmark fixtures explicitly select offline execution after production retrieval became default-on; no specification case or unsupported record changed.',
};
writeFileSync('docs/case-studies/issue-1582/raw-data/local/governed-witness-refresh.json', JSON.stringify(receipt, null, 2) + '\n');
console.log(JSON.stringify(receipt));
