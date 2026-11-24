// Supplied complete-object review; zero autonomous-authorship or execution credit.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {mkdirSync,readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {isAbsolute,relative,resolve} from 'node:path';
import {deserialize,serialize} from 'node:v8';
import {gzipSync} from 'node:zlib';
import {specificationCases} from '../../scripts/lib/rust-specification-cases.mjs';

const hash = data => createHash('sha256').update(data).digest('hex');
const root = process.cwd();
const output = process.argv[2];
assert.ok(output, 'usage: node review-extracted-solver-fixture.mjs OUTPUT_DIRECTORY');
const out = resolve(output);
mkdirSync(out, {recursive:true});
const file = 'rust/src/solver.rs';
const priorRevision = 'f8b372ad314c8052a00dbd2a3139dac0f45d0f78';
const priorSource = execFileSync('git', ['show', priorRevision + ':' + file]);
const currentSource = readFileSync(file);
const priorSha = '8205f9a8609ed4178f8093eecf36a6c88b60255afd53bc6f6935bb9d7c0e44b1';
const currentSha = '57ce0b8a381a6691d538a32d90242b942fee4ea7661b30c109c41ad2cf4d01fb';
assert.equal(hash(priorSource), priorSha);
assert.equal(hash(currentSource), currentSha);
const witness = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/unicode-identifier-identity/governed-baseline.v8';
const beforeRaw = readFileSync(witness);
assert.equal(hash(beforeRaw), 'f4a30ecd210b8b793e75d912cbdef0f4ed5f950203070e2d3969cedf2e97e76a');
const before = deserialize(beforeRaw);
const expected = deserialize(serialize(before));
const differences = [];
for (const [index,item] of expected.inventory.cases.entries()) {
  for (const [fixtureIndex,fixture] of (item.program?.fixtures ?? []).entries()) {
    if (fixture.file !== 'repo:' + file) continue;
    assert.deepEqual(Object.keys(fixture), ['file','sha256']);
    assert.equal(fixture.sha256, priorSha);
    fixture.sha256 = currentSha;
    differences.push({caseIndex:index,fixtureIndex,file,priorSha,currentSha});
  }
}
assert.equal(differences.length, 19);
const inventory = specificationCases(root);
const cases = structuredClone(inventory.cases);
for (const item of cases) for (const fixture of item.program?.fixtures ?? []) {
  if (!isAbsolute(fixture.file)) continue;
  const path = relative(root, fixture.file);
  assert.ok(!path.startsWith('..'));
  fixture.file = 'repo:' + path;
}
const sourceManifest = readdirSync('rust/tests/unit/specification', {recursive:true})
  .map(String).filter(path => path.endsWith('.rs')).sort().map(path => {
    const file = 'rust/tests/unit/specification/' + path;
    return {path:file,sha256:hash(readFileSync(file))};
  });
const candidate = {inventory:{...inventory,cases},sourceManifest};
assert.deepEqual(candidate, expected, 'only nineteen reviewed fixture hashes may differ');
assert.deepEqual(candidate.inventory.unsupported, before.inventory.unsupported);
assert.deepEqual(candidate.sourceManifest, before.sourceManifest);
assert.equal(inventory.total, 1189);
assert.equal(cases.length, 221);
assert.equal(inventory.unsupported.length, 968);
const readerSha = hash(readFileSync('scripts/lib/rust-specification-cases.mjs'));
assert.equal(readerSha, '861c6c5b40cfc8f4e99ad0ebece2cc3c20f941c6a1103dd5e8e70813c40ee0ca');
writeFileSync(resolve(out,'previous-witness.v8.gz'), gzipSync(beforeRaw, {mtime:0}));
const argv = ['--max-old-space-size=256','rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
  '--refresh-reviewed-baseline','--reviewed-reader-sha=' + readerSha,
  '--previous-baseline-sha=' + hash(beforeRaw)];
const stdout = execFileSync(process.execPath, argv, {cwd:root,maxBuffer:1024*1024});
writeFileSync(resolve(out,'refresh.stdout.gz'), gzipSync(stdout, {mtime:0}));
const afterRaw = readFileSync(witness);
assert.deepEqual(deserialize(afterRaw), expected);
const report = {priorRevision,file,priorSha256:priorSha,currentSha256:currentSha,
  readerSha256:readerSha,previousWitnessSha256:hash(beforeRaw),currentWitnessSha256:hash(afterRaw),
  total:inventory.total,supported:cases.length,unsupported:inventory.unsupported.length,
  differences,completeObjectDeltaReviewed:true,unsupportedObjectsUnchanged:true,
  nativeSourceManifestUnchanged:true,nativeBehaviorVerified:false,
  independentAuthorshipCredit:0,fullIssueComplete:false};
writeFileSync(resolve(out,'complete-delta.json'), JSON.stringify(report,null,2) + '\n');
console.log(JSON.stringify(report,null,2));
