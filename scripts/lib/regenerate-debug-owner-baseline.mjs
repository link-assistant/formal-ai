import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {deserialize} from 'node:v8';
import {scalarDifferenceReport} from './regenerate-governed-specification.mjs';

const registry = 'scripts/lib/generated-native-programs.mjs';
const baseline = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/'
  + 'unicode-identifier-identity/governed-baseline.v8';
const reader = 'scripts/lib/rust-specification-cases.mjs';
const baselineCli = 'rust/tests/web/rust-specification-unicode-identifiers.test.mjs';
const debugOwner = 'rust/tests/unit/specification/debug_session.rs';
const expectedBaseline = '1fbee3423873b3918b7c824d9f801e0b5bae24d64b8943e15b75504c67ad36cf';
const expectedRegistry = 'c4b481931b1261952a77e01117e12173792383d8859339e9e4a569286188de11';
const expectedReader = '861c6c5b40cfc8f4e99ad0ebece2cc3c20f941c6a1103dd5e8e70813c40ee0ca';
const oldDebug = '6872fff807e6a28f8b04afc8e404b19046e57a8f1d6e1602e94ed700567ec421';
const newDebug = '8da5534f8a41bab9b89b8dbd25bc93baa1d1e76f2c041c0f70a05e90b39dcaac';
const changedOwners = new Map([
  [debugOwner, newDebug],
  ['rust/src/solver_handlers/mod.rs', 'b4cacba6d71d0cfae2fec2aa323b260756f9d53a9b8771ae0c3e4eb2ce3fd9cc'],
  ['rust/src/solver_dispatch.rs', '97bf7bc66f850a409d74774a2ad1712acfcecab0358546aff784bcc6e2f2bb0e'],
  ['rust/src/quantity_constraint.rs', 'd0fa75d91fdf91efa734b03342dd33e698497a45cf8243a193000fa9b62c8b3a']
]);
const sha = value => createHash('sha256').update(value).digest('hex');
const git = (repo, args, maxBuffer = 65536) => execFileSync('git', args, {
  cwd: repo, encoding: 'utf8', timeout: 10000, maxBuffer
});

function sourceBytes(repo, file, limit = 16777216) {
  assert.equal(typeof file, 'string');
  assert(!path.isAbsolute(file) && !file.includes('\\') && !file.includes('\0'));
  assert(file.split('/').every(part => part && part !== '.' && part !== '..'));
  const target = path.join(repo, file);
  assert(target.startsWith(repo + path.sep));
  const stat = fs.lstatSync(target);
  assert(stat.isFile() && stat.size <= limit, 'Bounded regular source required');
  assert.equal(fs.realpathSync(target), target, 'Source symlink traversal refused');
  return fs.readFileSync(target);
}

function committedInput(repo, sourceCommit, file) {
  const committed = execFileSync('git', ['show', sourceCommit + ':' + file], {
    cwd: repo, timeout: 10000, maxBuffer: 16777216
  });
  const observed = sourceBytes(repo, file);
  assert.equal(sha(observed), sha(committed), 'Source differs from immutable Git blob: ' + file);
  return {path: file, bytes: observed.length, sha256: sha(observed)};
}

function assertInventory(before, after) {
  const normalized = structuredClone(after);
  try {
    for (const value of [before, after]) {
      assert.equal(value.inventory.total, 1184);
      assert.equal(value.inventory.cases.length, 221);
      assert.equal(value.inventory.unsupported.length, 963);
      const debug = value.inventory.unsupported.filter(item => item.id.startsWith(debugOwner + '::'));
      assert.equal(debug.length, 9);
      for (const item of debug)
        assert.deepEqual(Object.keys(item).sort(), ['id', 'legacyReason', 'reason']);
      for (const item of value.inventory.cases) {
        for (const owner of changedOwners.keys()) {
          assert(!item.id.startsWith(owner + '::'), 'Changed owner has a supported case');
          for (const fixture of item.program?.fixtures ?? [])
            assert.notEqual(fixture.file, 'repo:' + owner, 'Changed owner is a direct fixture');
          for (const witness of item.program?.sourceWitnesses ?? [])
            assert.notEqual(witness.path, owner, 'Changed owner is a program witness');
        }
      }
    }
    const oldRows = before.sourceManifest.filter(item => item.path === debugOwner);
    const newRows = normalized.sourceManifest.filter(item => item.path === debugOwner);
    assert.equal(oldRows.length, 1);
    assert.equal(newRows.length, 1);
    assert.equal(oldRows[0].sha256, oldDebug);
    assert.equal(newRows[0].sha256, newDebug);
    newRows[0].sha256 = oldDebug;
    assert.deepEqual(normalized, before, 'Unreviewed complete baseline delta');
  } catch (error) {
    console.error(JSON.stringify({phase: 'debug-owner-complete-baseline-differences',
      ...scalarDifferenceReport(before, normalized)}));
    throw error;
  }
}

export async function regenerateDebugOwnerBaseline({repo, sourceCommit, pin}) {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'Hosted-only baseline refresh');
  assert.equal(process.env.RUNNER_OS, 'Linux');
  assert.match(sourceCommit, /^[a-f0-9]{40}$/u);
  assert.equal(sourceCommit, process.env.GITHUB_SHA);
  assert.equal(git(repo, ['rev-parse', 'HEAD']).trim(), sourceCommit);
  assert.match(process.env.GITHUB_RUN_ID ?? '', /^[1-9][0-9]*$/u);
  assert.equal(process.env.GITHUB_RUN_ATTEMPT, '1', 'Fresh hosted attempt required');
  assert.equal(pin, 'eb0574d5528babd3b4aadece81590d8e2230b99b');
  assert.equal(git(repo, ['diff', '--name-only', '--']), '');
  assert.equal(git(repo, ['diff', '--cached', '--name-only', '--']), '');
  const output = path.join(repo, 'translation-regeneration');
  assert(!fs.existsSync(output), 'Fresh artifact directory required');
  const originalRegistry = sourceBytes(repo, registry, 262144);
  const originalBaseline = sourceBytes(repo, baseline, 3145728);
  assert.equal(sha(originalRegistry), expectedRegistry);
  assert.equal(sha(originalBaseline), expectedBaseline);
  assert.equal(sha(sourceBytes(repo, reader)), expectedReader);
  for (const [file, expected] of changedOwners)
    assert.equal(sha(sourceBytes(repo, file)), expected);
  const oldBaseline = deserialize(originalBaseline);
  for (const owner of changedOwners.keys())
    assert(!originalRegistry.includes(Buffer.from(owner)), 'Changed owner occurs in unchanged registry');
  const inputs = new Set([registry, baseline, reader, baselineCli,
    'scripts/regenerate-targeted-js-rust.mjs', 'scripts/lib/regenerate-debug-owner-baseline.mjs',
    'scripts/lib/regenerate-governed-specification.mjs', '.github/workflows/layered-ci.yml',
    'scripts/check-fast-governance.mjs', 'scripts/check-language-parity.mjs',
    'scripts/check-minimal-core-boundary.mjs', 'scripts/check-closure-audit.py',
    'scripts/generate-role-registry.py', 'scripts/audit-seed-metadata.mjs',
    'scripts/generate-response-parity-debt.mjs', 'package.json', 'bun.lock', '.bun-version',
    ...changedOwners.keys()]);
  for (const source of oldBaseline.sourceManifest) inputs.add(source.path);
  for (const item of oldBaseline.inventory.cases) {
    for (const fixture of item.program?.fixtures ?? []) {
      assert.match(fixture.file, /^repo:(?:rust|desktop|js|vscode)\//u);
      inputs.add(fixture.file.slice('repo:'.length));
    }
  }
  const sourceInputs = [...inputs].sort().map(file => committedInput(repo, sourceCommit, file));
  const env = {...process.env, FORMAL_AI_GENERATION_OBSERVATION_FILE: ''};
  const governance = execFileSync(process.execPath,
    ['--max-old-space-size=384', 'scripts/check-fast-governance.mjs'], {
      cwd: repo, env, encoding: 'utf8', timeout: 135000, maxBuffer: 1048576
    });
  process.stdout.write(governance);
  assert.equal(git(repo, ['diff', '--name-only', '--']), '', 'Governance changed source');
  execFileSync(process.execPath, ['--max-old-space-size=384', baselineCli,
    '--refresh-reviewed-baseline', '--reviewed-reader-sha=' + expectedReader,
    '--previous-baseline-sha=' + expectedBaseline], {
    cwd: repo, env, timeout: 90000, maxBuffer: 1048576
  });
  const refreshedBytes = sourceBytes(repo, baseline, 3145728);
  const refreshed = deserialize(refreshedBytes);
  assertInventory(oldBaseline, refreshed);
  assert.deepEqual(sourceBytes(repo, registry, 262144), originalRegistry);
  for (const input of sourceInputs) {
    if (input.path !== baseline) assert.equal(sha(sourceBytes(repo, input.path)), input.sha256);
  }
  assert.equal(git(repo, ['rev-parse', 'HEAD']).trim(), sourceCommit);
  assert.equal(git(repo, ['diff', '--cached', '--name-only', '--']), '');
  assert.deepEqual(git(repo, ['diff', '--name-only', '--']).trim().split('\n'), [baseline]);
  const files = [
    {path: registry, bytes: originalRegistry.length, sha256: expectedRegistry},
    {path: baseline, bytes: refreshedBytes.length, sha256: sha(refreshedBytes)}
  ];
  const manifest = {kind: 'debug-owner-baseline', source: sourceCommit,
    run: process.env.GITHUB_RUN_ID, attempt: process.env.GITHUB_RUN_ATTEMPT,
    upstream: pin, upstreamRole: 'prepared workflow context; not invoked by baseline refresh',
    inputsScope: 'selected tracked input witnesses; immutable commit binds other reads; not an exhaustive dynamic read set',
    inputs: sourceInputs, before: {registry: expectedRegistry, baseline: expectedBaseline},
    sourceGeneratorIdentity: {repositoryCommit: sourceCommit,
      command: baselineCli, reader: {path: reader, sha256: expectedReader}},
    governance: {command: 'scripts/check-fast-governance.mjs', childTimeoutMs: 135000,
      requiredBeforeRefresh: true, exitStatus: 0},
    reviewedSourceManifest: {path: debugOwner, before: oldDebug, after: newDebug},
    registryCopiedUnchanged: true, completeBaselineUnchangedExceptReviewedSourceManifest: true,
    total: refreshed.inventory.total, cases: refreshed.inventory.cases.length,
    unsupported: refreshed.inventory.unsupported.length, files};
  const manifestBytes = Buffer.from(JSON.stringify(manifest, null, 2) + '\n');
  assert(manifestBytes.length <= 262144);
  assert(files.reduce((sum, file) => sum + file.bytes, manifestBytes.length) <= 3670016);
  for (const file of files) {
    const destination = path.join(output, file.path);
    fs.mkdirSync(path.dirname(destination), {recursive: true});
    fs.writeFileSync(destination, file.path === registry ? originalRegistry : refreshedBytes, {flag: 'wx'});
  }
  fs.writeFileSync(path.join(output, 'manifest.json'), manifestBytes, {flag: 'wx'});
  console.log(JSON.stringify({phase: 'debug-owner-baseline-artifact-ready', manifest}));
}
