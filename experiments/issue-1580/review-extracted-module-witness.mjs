// Supplied complete-object review: zero autonomous-authorship credit.
// Usage: node review-extracted-module-witness.mjs PREFIX [--refresh]
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {readFileSync, writeFileSync} from 'node:fs';
import {deserialize} from 'node:v8';
const [prefix, action] = process.argv.slice(2);
assert(prefix && (action === undefined || action === '--refresh'));
const sha = data => createHash('sha256').update(data).digest('hex');
const previousBytes = readFileSync(prefix + '.before.v8');
const candidateBytes = readFileSync(prefix + '.after.v8');
assert.equal(sha(previousBytes), '7c0bdcaceb4aa0777a5bd0c9a9d35402f70569ea1c2b505d94e2ee73bd25c92f');
const previous = deserialize(previousBytes);
const candidate = deserialize(candidateBytes);
const expected = structuredClone(previous);
const sources = new Map([
  ['rust/tests/unit/specification/chat_surface.rs', [
    'dd73a316f6992dfc213d44381781c1cf8be22e4fcf2502a8f16d482e72f4cd46',
    '637574f534e58db169568adbc010fac60d8666c73dff0e96fe178752552db9e0']],
  ['rust/tests/unit/specification/debug_session.rs', [
    '16de6028fb30474a3736cd219b346786513ef40fe18ec6e9b3bc968dbcc98dc7',
    '6b49fa0a0bee8fff9d9bd0b67937245fec664d1ee681a1b0b208ffb982c2c89d']]
]);
const changes = [];
for (const item of expected.inventory.cases) for (const fixture of item.program?.fixtures ?? []) {
  const path = fixture.file.slice('repo:'.length);
  const pair = sources.get(path);
  if (!pair) continue;
  assert.equal(fixture.file, 'repo:' + path);
  assert.equal(fixture.sha256, pair[0]);
  fixture.sha256 = pair[1];
  changes.push({kind: 'fixture', id: item.id, path, before: pair[0], after: pair[1]});
}
for (const source of expected.sourceManifest) {
  const pair = sources.get(source.path);
  if (!pair) continue;
  assert.equal(source.sha256, pair[0]);
  assert.equal(sha(readFileSync(source.path)), pair[1]);
  source.sha256 = pair[1];
  changes.push({kind: 'source', path: source.path, before: pair[0], after: pair[1]});
}
assert.equal(changes.filter(change => change.kind === 'fixture').length, 6);
assert.equal(changes.filter(change => change.kind === 'source').length, 2);
assert.deepEqual(candidate, expected, 'Every other complete object and source identity must be preserved');
assert.deepEqual(candidate.inventory.unsupported, previous.inventory.unsupported);
assert.equal(candidate.inventory.total, 1193);
assert.equal(candidate.inventory.cases.length, 223);
assert.equal(candidate.inventory.unsupported.length, 970);
assert.equal(candidate.sourceManifest.length, 141);
const readerSha = sha(readFileSync('scripts/lib/rust-specification-cases.mjs'));
assert.equal(readerSha, '861c6c5b40cfc8f4e99ad0ebece2cc3c20f941c6a1103dd5e8e70813c40ee0ca');
const witness = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/'
  + 'unicode-identifier-identity/governed-baseline.v8';
const report = {previousSha256: sha(previousBytes), candidateSha256: sha(candidateBytes),
  readerSha256: readerSha, changes, total: 1193, supported: 223, unsupported: 970,
  nativeSourceCount: 141, completeObjectsPreservedExceptEightReviewedHashes: true,
  autonomousAuthorshipCredit: 0, nativeBehaviorVerified: false, refreshed: false};
if (action === '--refresh') {
  assert.equal(sha(readFileSync(witness)), sha(previousBytes), 'Witness changed after complete review');
  execFileSync(process.execPath, ['--max-old-space-size=256',
    'rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
    '--refresh-reviewed-baseline', '--reviewed-reader-sha=' + readerSha,
    '--previous-baseline-sha=' + sha(previousBytes)], {stdio: 'inherit'});
  assert.deepEqual(deserialize(readFileSync(witness)), candidate);
  report.refreshedSha256 = sha(readFileSync(witness));
  report.refreshed = true;
}
writeFileSync(prefix + '.reviewed.json', JSON.stringify(report, null, 2) + '\n');
console.log(JSON.stringify(report, null, 2));
