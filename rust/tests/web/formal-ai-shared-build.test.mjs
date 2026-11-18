// Issue #1486: Hive drafts at the same effective build identity produce the
// Formal AI binary at most once, and consumers reuse it only after verifying
// the producer, source identity, target and digest. Offline: the GitHub API is
// a recorded fake, so no Rust compilation, Docker or network is needed.
import assert from 'node:assert/strict';
import test from 'node:test';
import { readFileSync } from 'node:fs';
import {
  BUILD_ARTIFACT_PREFIX, CLAIM_ARTIFACT_PREFIX, PRODUCER_WORKFLOW, SharedBuildError, buildIdentity, buildManifest,
  canonicalJson, decide, resolveSharedBuild, verifyBuildProduct
} from '../../../scripts/lib/formal-ai-build-identity.mjs';

const repository = 'link-assistant/formal-ai';
const rustc = 'rustc 1.95.0 (abc 2026-09-01)\nbinary: rustc\ncommit-hash: ' + 'c'.repeat(40) + '\nhost: x86_64-unknown-linux-gnu\nrelease: 1.95.0\nLLVM version: 21.1.0\n';
const identity = buildIdentity({ sourceCommit: 'a'.repeat(40), sourceTree: 'b'.repeat(40), rustcVerbose: rustc, cargoLock: 'lock-v1' });
const binary = Buffer.from('formal-ai binary bytes');
const producer = { repository, runId: 100, runAttempt: 1 };
const manifest = buildManifest({ identity, binary, version: 'formal-ai 0.300.0\n', producer, builtAt: '2026-11-15T00:00:00Z' });
const run = (id, overrides = {}) => ({ id, path: PRODUCER_WORKFLOW, head_branch: 'main', event: 'issues', repository: { full_name: repository }, head_repository: { full_name: repository }, ...overrides });
const artifact = (id, prefix, runId, overrides = {}) => ({ id, name: prefix + identity.key, size_in_bytes: 10, expired: false, created_at: '2026-11-15T00:00:' + String(id).padStart(2, '0') + 'Z', workflow_run: { id: runId, head_branch: 'main', repository_id: 1, head_repository_id: 1 }, ...overrides });

test('the identity key covers source tree, toolchain, target, profile, features, lockfile and recipe', () => {
  assert.match(identity.key, /^[0-9a-f]{64}$/u);
  const same = buildIdentity({ sourceCommit: 'd'.repeat(40), sourceTree: 'b'.repeat(40), rustcVerbose: rustc, cargoLock: 'lock-v1' });
  assert.equal(same.key, identity.key, 'a commit with the same tree and inputs is the same build');
  for (const changed of [
    { sourceTree: 'e'.repeat(40) },
    { rustcVerbose: rustc.replace('release: 1.95.0', 'release: 1.96.0') },
    { cargoLock: 'lock-v2' },
    { target: 'aarch64-unknown-linux-gnu' },
    { features: 'default,provenance-gpu' },
    { profile: 'dev' },
    { recipe: ['cargo', 'build'] },
  ]) {
    const other = buildIdentity({ sourceCommit: 'a'.repeat(40), sourceTree: 'b'.repeat(40), rustcVerbose: rustc, cargoLock: 'lock-v1', ...changed });
    assert.notEqual(other.key, identity.key, JSON.stringify(changed));
  }
  assert.throws(() => buildIdentity({ sourceCommit: 'a'.repeat(40), sourceTree: 'b'.repeat(40), rustcVerbose: rustc, cargoLock: null }), /LockfileMissing/);
  assert.throws(() => buildIdentity({ sourceCommit: 'a'.repeat(40), sourceTree: 'b'.repeat(40), rustcVerbose: 'garbage', cargoLock: 'x' }), /ToolchainUnknown/);
  assert.equal(canonicalJson({ b: 1, a: [{ d: 2, c: 3 }] }), '{"a":[{"c":3,"d":2}],"b":1}');
});

test('a verified product is reused; stale, poisoned, incomplete, failed and foreign products are rejected', () => {
  const verified = verifyBuildProduct({ identity, manifest, binary, producerRunId: 100, repository });
  assert.equal(verified.sha256, manifest.binary.sha256);
  const cases = [
    ['BuildIncomplete', { manifest: null }],
    ['BuildIncomplete', { binary: Buffer.alloc(0) }],
    ['BuildDigestMismatch', { binary: Buffer.from('poisoned binary bytes!') }],
    ['BuildDigestMismatch', { binary: Buffer.concat([binary, Buffer.from('x')]) }],
    ['BuildIdentityMismatch', { manifest: { ...manifest, key: 'f'.repeat(64) } }],
    ['BuildIdentityMismatch', { manifest: { ...manifest, inputs: { ...manifest.inputs, target: 'aarch64-unknown-linux-gnu' } } }],
    ['BuildProducerMismatch', { producerRunId: 101 }],
    ['BuildProducerMismatch', { repository: 'someone/fork' }],
    ['BuildFailed', { manifest: { ...manifest, buildOutcome: 'failure' } }],
    ['BuildManifestSchema', { manifest: { ...manifest, schema: 'other' } }],
  ];
  for (const [code, overrides] of cases) {
    assert.throws(() => verifyBuildProduct({ identity, manifest, binary, producerRunId: 100, repository, ...overrides }),
      error => error instanceof SharedBuildError && error.code === code, code + ' ' + JSON.stringify(Object.keys(overrides)));
  }
});

test('only an authenticated, unexpired main-branch producer of this workflow is reused', () => {
  const base = { key: identity.key, self: 300, repository, claims: [], waitedSeconds: 0, maxWaitSeconds: 600 };
  const good = artifact(5, BUILD_ARTIFACT_PREFIX, 100);
  assert.equal(decide({ ...base, builds: [good], runs: { 100: { run: run(100), buildJob: 'completed' } } }).mode, 'reuse');
  for (const [label, built, producerRun] of [
    ['expired', { ...good, expired: true }, run(100)],
    ['empty', { ...good, size_in_bytes: 0 }, run(100)],
    ['other workflow', good, run(100, { path: '.github/workflows/release.yml' })],
    ['branch', good, run(100, { head_branch: 'feature' })],
    ['pull request', good, run(100, { event: 'pull_request' })],
    ['fork', good, run(100, { head_repository: { full_name: 'someone/fork' } })],
    ['run missing', good, null],
    ['another identity', { ...good, name: BUILD_ARTIFACT_PREFIX + 'f'.repeat(64) }, run(100)],
  ]) {
    const decision = decide({ ...base, builds: [built], runs: { 100: { run: producerRun, buildJob: 'completed' } } });
    assert.equal(decision.mode, 'build', label);
  }
});

test('concurrent drafts elect one producer; the others wait, then reuse its artifact', () => {
  const claims = [artifact(2, CLAIM_ARTIFACT_PREFIX, 201), artifact(1, CLAIM_ARTIFACT_PREFIX, 200), artifact(3, CLAIM_ARTIFACT_PREFIX, 202)];
  const runs = { 200: { run: run(200), buildJob: 'in_progress' }, 201: { run: run(201), buildJob: 'in_progress' }, 202: { run: run(202, { event: 'workflow_dispatch' }), buildJob: 'queued' } };
  const modes = [200, 201, 202].map(self => decide({ key: identity.key, self, repository, builds: [], claims, runs, waitedSeconds: 30, maxWaitSeconds: 600 }).mode);
  assert.deepEqual(modes, ['build', 'wait', 'wait'], 'exactly one producer per identity');
  const built = artifact(9, BUILD_ARTIFACT_PREFIX, 200);
  for (const self of [201, 202]) {
    const decision = decide({ key: identity.key, self, repository, builds: [built], claims, runs, waitedSeconds: 400, maxWaitSeconds: 600 });
    assert.equal(decision.mode, 'reuse');
    assert.equal(decision.producerRunId, '200');
  }
});

test('a failed producer is never reused and the next live claim takes over', () => {
  const claims = [artifact(1, CLAIM_ARTIFACT_PREFIX, 200), artifact(2, CLAIM_ARTIFACT_PREFIX, 201), artifact(3, CLAIM_ARTIFACT_PREFIX, 202)];
  const runs = { 200: { run: run(200), buildJob: 'completed' }, 201: { run: run(201), buildJob: 'in_progress' }, 202: { run: run(202), buildJob: 'in_progress' } };
  const modes = [201, 202].map(self => decide({ key: identity.key, self, repository, builds: [], claims, runs, waitedSeconds: 60, maxWaitSeconds: 600 }).mode);
  assert.deepEqual(modes, ['build', 'wait']);
  const forged = artifact(0, CLAIM_ARTIFACT_PREFIX, 199);
  const decision = decide({ key: identity.key, self: 201, repository, builds: [], claims: [forged, ...claims], runs: { ...runs, 199: { run: run(199, { head_branch: 'attacker' }), buildJob: 'in_progress' } }, waitedSeconds: 60, maxWaitSeconds: 600 });
  assert.equal(decision.mode, 'build', 'an unauthenticated claim cannot hold the build');
  assert.equal(decision.rejected.length, 1);
  const expired = decide({ key: identity.key, self: 202, repository, builds: [], claims, runs, waitedSeconds: 600, maxWaitSeconds: 600 });
  assert.deepEqual([expired.mode, expired.reason], ['build', 'wait-expired'], 'a stuck producer cannot block a draft past the bounded wait');
});

test('the resolution loop waits for the producer through the recorded API', async () => {
  let clock = 0;
  let polls = 0;
  const api = async path => {
    if (path.includes('/actions/artifacts?')) {
      if (path.includes(encodeURIComponent(CLAIM_ARTIFACT_PREFIX))) return { artifacts: [artifact(1, CLAIM_ARTIFACT_PREFIX, 200), artifact(2, CLAIM_ARTIFACT_PREFIX, 201)] };
      polls += 1;
      return { artifacts: polls >= 3 ? [artifact(7, BUILD_ARTIFACT_PREFIX, 200)] : [] };
    }
    const id = Number(/runs\/(\d+)/u.exec(path)[1]);
    return path.includes('/jobs') ? { jobs: [{ name: 'build', status: 'in_progress' }, { name: 'draft', status: 'queued' }] } : run(id);
  };
  const lines = [];
  const decision = await resolveSharedBuild({ key: identity.key, self: 201, repository, api, sleep: async ms => { clock += ms; }, now: () => clock, log: line => lines.push(line) });
  assert.equal(decision.mode, 'reuse');
  assert.equal(decision.artifact.id, 7);
  assert.equal(decision.waitedSeconds, 55);
  assert.equal(lines.filter(line => line.startsWith('waiting for the producer run 200')).length, 2);
});

test('the draft workflow claims, resolves, verifies and publishes the shared build around one cargo build', () => {
  const workflow = readFileSync(new URL('../../../.github/workflows/hive-mind-draft.yml', import.meta.url), 'utf8');
  const build = workflow.slice(workflow.indexOf('\n  build:'), workflow.indexOf('\n  draft:'));
  const order = ['formal-ai-shared-build.mjs identity', 'formal-ai-build-claim-', 'formal-ai-shared-build.mjs resolve', 'run-id: ${{ steps.shared-build.outputs.producer_run_id }}',
    'formal-ai-shared-build.mjs verify', 'cargo build', 'formal-ai-shared-build.mjs manifest', 'name: formal-ai-build-${{ steps.build-identity.outputs.key }}'];
  const positions = order.map(text => build.indexOf(text));
  assert.ok(positions.every(position => position > 0), JSON.stringify(Object.fromEntries(order.map((text, index) => [text, positions[index]]))));
  assert.deepEqual([...positions].sort((left, right) => left - right), positions);
  assert.match(build, /actions: read/u);
  assert.match(build, /id: build-command\n\s+name: Build current main binary in CI\n\s+if: steps\.decide\.outputs\.should_run == 'true' && steps\.shared-verify\.outputs\.reused != 'true'/u);
  assert.match(build, /Publish the authenticated build product once per identity\n\s+if: steps\.build-command\.outcome == 'success'/u, 'only a successful build is published');
  assert.match(build, /id: shared-verify[\s\S]*?continue-on-error: true/u, 'a rejected product falls back to a fresh build');
  assert.match(build, /FORMAL_AI_BINARY_SOURCE: \$\{\{ steps\.shared-verify\.outputs\.reused == 'true' && 'shared-build' \|\| 'build' \}\}/u);
  assert.match(build, /hive-mind-main-binary-\$\{\{ github\.run_id \}\}-\$\{\{ github\.run_attempt \}\}/u, 'per-issue binary custody stays per run');
});
