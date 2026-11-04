import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {deserialize} from 'node:v8';
import {pathToFileURL} from 'node:url';

const registry = 'scripts/lib/generated-native-programs.mjs';
const baseline = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/'
  + 'unicode-identifier-identity/governed-baseline.v8';
const reader = 'scripts/lib/rust-specification-cases.mjs';
const generator = 'scripts/lib/generate-native-specification-programs.mjs';
const oldRoot = '797bdfc344c69094db88f2e76ee80b386a95f1e0a0aa93257ee0dd4e0c1b029b';
const newRoot = '7a5810bcb594c76c0908fc9fd0365ad59513706260e23223ec621138743d3024';
const oldUnitRegistry = 'a4920565b407a50ecb97128ab7f0224bf33d3f70eac9b71593d273a42d3fe01d';
const newUnitRegistry = 'e865cca3fb2de22996a703221de87b5c9e790ff18357a3ac44c728334f58343f';
const reviewedFixtures = new Map([
  ['repo:rust/src/lib.rs', {before: oldRoot, after: newRoot, occurrences: 35}],
  ['repo:rust/tests/unit/mod.rs', {before: oldUnitRegistry, after: newUnitRegistry, occurrences: 4}]
]);
const expectedBaseline = '4f0e43299b8e63de8a8dfef176e1559f45fb604ad0c6ce2d7d5f6f12c067fa79';
const expectedRegistry = 'acf4a68fffb7661cdf10d0762919e128f9fc38e0ca84c1449cec576eae241ce7';
const expectedReader = '861c6c5b40cfc8f4e99ad0ebece2cc3c20f941c6a1103dd5e8e70813c40ee0ca';
const expectedGenerator = '73cbc8a5f657f6048766019d3e9f484cd2ad503d6234de197d93e7799b028d28';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const full = (repo, file) => path.join(repo, file);
function bytes(repo, file) {
  assert.equal(typeof file, 'string');
  assert(!path.isAbsolute(file) && !file.includes('\\') && !file.includes('\0'));
  assert(file.split('/').every(part => part && part !== '.' && part !== '..'));
  execFileSync('git', ['ls-files', '--error-unmatch', '--', file], {
    cwd: repo, timeout: 10000, maxBuffer: 65536, stdio: ['ignore', 'ignore', 'pipe']
  });
  const target = full(repo, file);
  assert(target.startsWith(repo + path.sep), 'Source path escaped checkout');
  assert(fs.lstatSync(target).isFile(), 'Source must be a regular file');
  assert.equal(fs.realpathSync(target), target, 'Source symlink traversal refused');
  return fs.readFileSync(target);
}

function replaceReviewedRoot(witnesses) {
  let count = 0;
  for (const witness of witnesses) {
    if (witness.path !== 'rust/src/lib.rs' && witness.file !== 'repo:rust/src/lib.rs') continue;
    assert.equal(witness.sha256, newRoot, 'Unexpected current root identity');
    witness.sha256 = oldRoot;
    count++;
  }
  return count;
}

function run(repo, args) {
  execFileSync(process.execPath, ['--max-old-space-size=384', ...args], {
    cwd: repo, encoding: 'utf8', timeout: 90000, maxBuffer: 1048576,
    env: {...process.env, FORMAL_AI_GENERATION_OBSERVATION_FILE: ''}
  });
}

function assertCompleteInventory(before, after) {
  const normalized = structuredClone(after);
  assert.equal(after.inventory.total, before.inventory.total);
  assert.equal(after.inventory.cases.length, before.inventory.cases.length, 'No case loss');
  assert.equal(after.inventory.unsupported.length, before.inventory.unsupported.length);
  assert.deepEqual(after.sourceManifest, before.sourceManifest, 'Native specification source changed');
  let changed = 0;
  const occurrences = new Map();
  const expectedOccurrences = new Map([...reviewedFixtures.keys()].map(file => [file, 0]));
  for (const item of before.inventory.cases) {
    for (const fixture of item.program?.fixtures ?? []) {
      if (expectedOccurrences.has(fixture.file))
        expectedOccurrences.set(fixture.file, expectedOccurrences.get(fixture.file) + 1);
    }
  }
  for (let index = 0; index < before.inventory.cases.length; index++) {
    const oldCase = before.inventory.cases[index];
    const newCase = normalized.inventory.cases[index];
    assert.equal(newCase.id, oldCase.id, 'Case identity/order changed');
    const fixtures = newCase.program?.fixtures ?? [];
    for (let fixture = 0; fixture < fixtures.length; fixture++) {
      const oldFixture = oldCase.program?.fixtures?.[fixture];
      const approved = reviewedFixtures.get(oldFixture?.file);
      if (!approved) continue;
      assert.equal(fixtures[fixture].file, oldFixture.file);
      assert.equal(oldFixture.sha256, approved.before);
      assert.equal(fixtures[fixture].sha256, approved.after);
      fixtures[fixture].sha256 = approved.before;
      occurrences.set(oldFixture.file, (occurrences.get(oldFixture.file) ?? 0) + 1);
      changed++;
    }
  }
  assert.equal(changed, 39, 'Reviewed complete fixture occurrence count');
  for (const [file, approved] of reviewedFixtures)
    assert.equal(expectedOccurrences.get(file), approved.occurrences, 'Sealed old owner occurrence count');
  for (const [file, expected] of expectedOccurrences)
    assert.equal(occurrences.get(file), expected, 'Every reviewed old occurrence must carry new identity');
  assert.deepEqual(normalized, before, 'Unreviewed complete inventory delta');
  return changed;
}

export async function regenerateSpecificationWitness({repo, sourceCommit, pin}) {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Hosted-only source generation');
  assert.equal(sha(bytes(repo, baseline)), expectedBaseline);
  assert.equal(sha(bytes(repo, registry)), expectedRegistry);
  assert.equal(sha(bytes(repo, reader)), expectedReader);
  assert.equal(sha(bytes(repo, generator)), expectedGenerator);
  assert.equal(sha(bytes(repo, 'rust/src/lib.rs')), newRoot);
  assert.equal(sha(bytes(repo, 'rust/tests/unit/mod.rs')), newUnitRegistry);
  const oldBaseline = deserialize(bytes(repo, baseline));
  const oldPrograms = (await import(pathToFileURL(full(repo, registry)))).GENERATED_NATIVE_PROGRAMS;
  const inputs = new Set([reader, generator, 'rust/src/lib.rs',
    'scripts/regenerate-targeted-js-rust.mjs', 'scripts/lib/regenerate-governed-specification.mjs',
    'rust/tests/web/rust-specification-unicode-identifiers.test.mjs', 'package.json', 'bun.lock', '.bun-version']);
  for (const program of oldPrograms) {
    for (const witness of program.sourceWitnesses ?? []) inputs.add(witness.path);
  }
  for (const source of oldBaseline.sourceManifest) inputs.add(source.path);
  for (const item of oldBaseline.inventory.cases) {
    for (const fixture of item.program?.fixtures ?? []) {
      assert.match(fixture.file, /^repo:(?:rust|desktop|js|vscode)\//u);
      inputs.add(fixture.file.slice('repo:'.length));
    }
  }
  assert(!inputs.has(registry) && !inputs.has(baseline), 'Generated outputs cannot be invariant inputs');
  const sourceInputs = [...inputs].sort().map(file => {
    const observed = bytes(repo, file);
    const committed = execFileSync('git', ['show', 'HEAD:' + file], {
      cwd: repo, timeout: 10000, maxBuffer: 16777216
    });
    assert.equal(sha(observed), sha(committed), 'Source input differs from immutable checkout');
    return {path: file, sha256: sha(observed)};
  });
  run(repo, [generator]);
  run(repo, [generator, '--check']);
  const generatedUrl = pathToFileURL(full(repo, registry));
  generatedUrl.search = '?generated-source';
  const newPrograms = (await import(generatedUrl)).GENERATED_NATIVE_PROGRAMS;
  assert.equal(newPrograms.length, oldPrograms.length, 'Generated callable inventory changed');
  const normalizedPrograms = structuredClone(newPrograms);
  let registryChanges = 0;
  for (let index = 0; index < oldPrograms.length; index++) {
    assert.equal(newPrograms[index].path, oldPrograms[index].path);
    const oldWitnesses = oldPrograms[index].sourceWitnesses ?? [];
    for (let witness = 0; witness < (normalizedPrograms[index].sourceWitnesses ?? []).length; witness++) {
      const value = normalizedPrograms[index].sourceWitnesses[witness];
      if (value.sha256 === oldWitnesses[witness]?.sha256) continue;
      assert.equal(oldWitnesses[witness]?.path, 'rust/src/lib.rs');
      assert.equal(oldWitnesses[witness].sha256, oldRoot);
      registryChanges += replaceReviewedRoot([value]);
    }
  }
  assert(registryChanges > 0);
  assert.deepEqual(normalizedPrograms, oldPrograms, 'Generated AST/program changed beyond reviewed root witness');
  run(repo, ['rust/tests/web/rust-specification-unicode-identifiers.test.mjs',
    '--refresh-reviewed-baseline', '--reviewed-reader-sha=' + expectedReader,
    '--previous-baseline-sha=' + expectedBaseline]);
  const refreshed = deserialize(bytes(repo, baseline));
  const changedFixtureOccurrences = assertCompleteInventory(oldBaseline, refreshed);
  for (const input of sourceInputs) assert.equal(sha(bytes(repo, input.path)), input.sha256);
  const changed = execFileSync('git', ['diff', '--name-only', '--'], {
    cwd: repo, encoding: 'utf8', timeout: 10000, maxBuffer: 65536
  }).trim().split('\n').filter(Boolean).sort();
  assert.deepEqual(changed, [registry, baseline].sort(), 'Unreviewed generated tracked output');
  const output = full(repo, 'translation-regeneration');
  assert(!fs.existsSync(output), 'Artifact directory must start absent');
  const files = [registry, baseline].map(file => ({path: file,
    bytes: bytes(repo, file).length, sha256: sha(bytes(repo, file))}));
  assert(files[0].bytes <= 262144 && files[1].bytes <= 3145728, 'Source artifact size bound');
  const manifest = {kind: 'specification-witness', source: sourceCommit, upstream: pin,
    upstreamRole: 'prepared workflow context; not invoked by specification generator',
    sourceGeneratorIdentity: {repositoryCommit: sourceCommit,
      generator: {path: generator, sha256: expectedGenerator},
      reader: {path: reader, sha256: expectedReader},
      implementation: 'repository-owned source AST program generator and complete typed specification reader'},
    run: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT,
    inputsScope: 'selected pinned source witnesses; immutable source commit binds other generator reads',
    inputs: sourceInputs, before: {registry: expectedRegistry, baseline: expectedBaseline},
    reviewedRoot: {path: 'rust/src/lib.rs', before: oldRoot, after: newRoot},
    reviewedFixtureChanges: [...reviewedFixtures].map(([file, approved]) => ({
      path: file.slice('repo:'.length), ...approved})),
    generatedProgramsUnchangedExceptReviewedRootWitness: true,
    completeInventoryUnchangedExceptReviewedFixtureWitnesses: true,
    generatedProgramCount: oldPrograms.length, registryChanges,
    total: refreshed.inventory.total, cases: refreshed.inventory.cases.length,
    unsupported: refreshed.inventory.unsupported.length, changedFixtureOccurrences, files};
  const manifestBytes = Buffer.from(JSON.stringify(manifest, null, 2) + '\n');
  assert(manifestBytes.length <= 262144);
  assert(files.reduce((sum, file) => sum + file.bytes, manifestBytes.length) <= 3670016);
  for (const file of files) {
    const destination = path.join(output, file.path);
    fs.mkdirSync(path.dirname(destination), {recursive: true});
    fs.copyFileSync(full(repo, file.path), destination);
  }
  fs.writeFileSync(path.join(output, 'manifest.json'), manifestBytes);
  console.log(JSON.stringify({phase: 'specification-source-artifact-ready', manifest}));
}
