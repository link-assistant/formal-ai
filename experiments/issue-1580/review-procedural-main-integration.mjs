// Supplied merge review; no autonomous authorship or native-runtime credit.
import assert from 'node:assert/strict';
import {readFileSync, writeFileSync, readdirSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {deserialize, serialize} from 'node:v8';
import {isAbsolute, relative, resolve} from 'node:path';
import {execFileSync} from 'node:child_process';
import {specificationCases} from '../../scripts/lib/rust-specification-cases.mjs';
import {scalarDifferenceReport} from '../../scripts/lib/regenerate-governed-specification.mjs';
const [directory] = process.argv.slice(2);
assert(directory, 'existing review directory required');
const root = process.cwd();
const sha = value => createHash('sha256').update(value).digest('hex');
const witness = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/'
  + 'unicode-identifier-identity/governed-baseline.v8';
const beforeBytes = readFileSync(resolve(directory, 'ours.v8'));
const before = deserialize(beforeBytes);
const inventory = specificationCases(root);
const cases = structuredClone(inventory.cases);
for (const item of cases) for (const fixture of item.program?.fixtures ?? []) {
  if (!isAbsolute(fixture.file)) continue;
  const path = relative(root, fixture.file);
  assert(!path.startsWith('..'));
  fixture.file = 'repo:' + path;
}
const sourceManifest = readdirSync('rust/tests/unit/specification', {recursive: true})
  .map(String).filter(path => path.endsWith('.rs')).sort().map(name => {
    const path = 'rust/tests/unit/specification/' + name;
    return {path, sha256: sha(readFileSync(path))};
  });
const candidate = {inventory: {...inventory, cases}, sourceManifest};
assert.equal(candidate.inventory.total, before.inventory.total);
assert.deepEqual(candidate.inventory.cases.map(item => item.id),
  before.inventory.cases.map(item => item.id));
assert.deepEqual(candidate.inventory.unsupported.map(item => item.id),
  before.inventory.unsupported.map(item => item.id));
// All 18 scalar differences were reviewed before this explicit refresh path.
assert.equal(sha(beforeBytes), '787108c54dbb62b7548a29b453627d91ed0357dd9e02af903871a6056b6aefa3');
const readerSha = sha(readFileSync('scripts/lib/rust-specification-cases.mjs'));
assert.equal(readerSha, '861c6c5b40cfc8f4e99ad0ebece2cc3c20f941c6a1103dd5e8e70813c40ee0ca');
const theirs = deserialize(readFileSync(resolve(directory, 'theirs.v8')));
const expected = structuredClone(before);
const caseId = before.inventory.cases[184].id;
expected.inventory.cases[184] = structuredClone(theirs.inventory.cases.find(item => item.id === caseId));
assert(expected.inventory.cases[184], 'the main branch must supply the complete changed case');
const paths = new Set(['rust/Cargo.toml',
  'rust/tests/unit/specification/reasoning_paths_procedures.rs',
  'rust/tests/unit/specification/reasoning_paths_procedures/fixtures.rs']);
for (const item of expected.inventory.cases) for (const fixture of item.program?.fixtures ?? []) {
  const path = fixture.file.slice('repo:'.length);
  if (paths.has(path)) fixture.sha256 = sha(readFileSync(path));
}
for (const source of expected.sourceManifest)
  if (paths.has(source.path)) source.sha256 = sha(readFileSync(source.path));
assert.deepEqual(candidate, expected, 'complete inventory must preserve both branches except reviewed deltas');
assert.deepEqual(candidate.inventory.unsupported, before.inventory.unsupported);
assert.equal(candidate.inventory.total, 1193);
assert.equal(candidate.inventory.cases.length, 223);
assert.equal(candidate.inventory.unsupported.length, 970);
assert.equal(candidate.sourceManifest.length, 141);
const report = scalarDifferenceReport(before, candidate);
assert.equal(report.differences, 18);
assert.equal(report.allDifferencesSha256, 'd743d9e6225501a59b6477fb558401fb35e5267d117c44982c08e2cfdef2f24e');
// V8 encodings may differ while the full deserialized object stays identical.
const reviewedCandidate = readFileSync(resolve(directory, 'candidate-pre-refresh.v8'));
assert.equal(sha(reviewedCandidate), 'b207575f777500a8fe8e70052d42c1303e71a6193692f0a3a60dfac186369119');
assert.deepEqual(candidate, deserialize(reviewedCandidate));
const result = {beforeSha256: sha(beforeBytes), candidateSha256: sha(serialize(candidate)),
  readerSha256: readerSha, ...report, completeObjectsCompared: true,
  mainCaseId: caseId, nativeBehaviorVerified: false, autonomousAuthorshipCredit: 0, refreshed: false};
if (process.argv.includes('--refresh')) {
  assert.equal(sha(readFileSync(witness)), sha(beforeBytes));
  execFileSync(process.execPath, ['--max-old-space-size=192',
    'rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
    '--refresh-reviewed-baseline', '--reviewed-reader-sha=' + readerSha,
    '--previous-baseline-sha=' + sha(beforeBytes)], {stdio: 'inherit'});
  assert.deepEqual(deserialize(readFileSync(witness)), candidate);
  result.refreshed = true;
}
writeFileSync(resolve(directory, 'candidate.v8'), serialize(candidate));
writeFileSync(resolve(directory, 'witness-reviewed.json'), JSON.stringify(result, null, 2) + '\n');
console.log(JSON.stringify({differences: report.differences, completeObjectsCompared: true,
  caseId, refreshed: result.refreshed}));
