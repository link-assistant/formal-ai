// Issue #1486: build the Formal AI binary once per effective build identity.
//
// The identity is the source tree, Rust toolchain, target, profile, features,
// binary, Cargo.lock digest and the build recipe. Every Hive draft for the same
// identity shares one authenticated build product:
//
//   1. each build job uploads a claim artifact named for the identity key;
//   2. the earliest live, authenticated claim is the only producer;
//   3. other consumers wait for that producer's build artifact, re-elect if it
//      fails, and fall back to their own build only after a bounded wait;
//   4. a build artifact is reused only when the producer run is this workflow
//      on main in this repository, the artifact is unexpired, and the downloaded
//      manifest and binary match the identity, producer and digest.
//
// Failed builds never publish a build artifact, and a stale, poisoned,
// incomplete or mismatched artifact is rejected, so the consumer builds itself.
import { createHash } from 'node:crypto';

export const BUILD_IDENTITY_SCHEMA = 'formal-ai-build-identity/v1';
export const BUILD_MANIFEST_SCHEMA = 'formal-ai-build-manifest/v1';
export const PRODUCER_WORKFLOW = '.github/workflows/hive-mind-draft.yml';
export const PRODUCER_EVENTS = Object.freeze(['issues', 'workflow_dispatch']);
export const PRODUCER_JOB = 'build';
export const BUILD_ARTIFACT_PREFIX = 'formal-ai-build-';
export const CLAIM_ARTIFACT_PREFIX = 'formal-ai-build-claim-';
export const BUILD_RECIPE = Object.freeze(['cargo', 'build', '--manifest-path', 'rust/Cargo.toml', '--target-dir', 'target', '--release', '--locked', '--bin', 'formal-ai']);

const HEX40 = /^[0-9a-f]{40}$/u;
const HEX64 = /^[0-9a-f]{64}$/u;

export class SharedBuildError extends Error {
  constructor(code, detail) {
    super(code + ': ' + detail);
    this.code = code;
  }
}

const sha256 = value => createHash('sha256').update(value).digest('hex');

/** Canonical JSON: object keys sorted at every depth. */
export function canonicalJson(value) {
  if (Array.isArray(value)) return '[' + value.map(canonicalJson).join(',') + ']';
  if (value && typeof value === 'object') {
    return '{' + Object.keys(value).sort().map(key => JSON.stringify(key) + ':' + canonicalJson(value[key])).join(',') + '}';
  }
  return JSON.stringify(value);
}

/** `rustc -vV` reduced to the fields that change the produced code. */
export function toolchainOf(rustcVerbose) {
  const field = name => new RegExp('^' + name + ': (.+)$', 'mu').exec(String(rustcVerbose ?? ''))?.[1]?.trim() ?? null;
  const toolchain = { release: field('release'), commitHash: field('commit-hash'), host: field('host'), llvm: field('LLVM version') };
  if (!toolchain.release || !toolchain.host) throw new SharedBuildError('ToolchainUnknown', 'rustc -vV did not report release and host');
  return toolchain;
}

/** The effective build identity and its key. */
export function buildIdentity({ sourceCommit, sourceTree, rustcVerbose, cargoLock, target = 'x86_64-unknown-linux-gnu', profile = 'release', features = 'default', bin = 'formal-ai', recipe = BUILD_RECIPE }) {
  if (!HEX40.test(sourceTree ?? '')) throw new SharedBuildError('SourceTreeUnknown', String(sourceTree));
  if (!HEX40.test(sourceCommit ?? '')) throw new SharedBuildError('SourceCommitUnknown', String(sourceCommit));
  if (cargoLock === undefined || cargoLock === null) throw new SharedBuildError('LockfileMissing', 'Cargo.lock is required for a --locked build');
  const inputs = {
    schema: BUILD_IDENTITY_SCHEMA,
    sourceTree,
    toolchain: toolchainOf(rustcVerbose),
    target,
    profile,
    features,
    bin,
    cargoLockSha256: sha256(cargoLock),
    recipeSha256: sha256(canonicalJson(recipe)),
  };
  return { key: sha256(canonicalJson(inputs)), inputs, sourceCommit };
}

/** The manifest a successful producer publishes next to the binary. */
export function buildManifest({ identity, binary, version, producer, builtAt }) {
  if (!binary?.length) throw new SharedBuildError('BuildIncomplete', 'no binary bytes');
  if (!producer?.runId || !producer?.runAttempt || !producer?.repository) throw new SharedBuildError('ProducerUnknown', JSON.stringify(producer));
  return {
    schema: BUILD_MANIFEST_SCHEMA,
    key: identity.key,
    inputs: identity.inputs,
    sourceCommit: identity.sourceCommit,
    binary: { name: identity.inputs.bin, sha256: sha256(binary), bytes: binary.length, version: String(version ?? '').trim() },
    producer: { repository: producer.repository, workflow: PRODUCER_WORKFLOW, runId: String(producer.runId), runAttempt: String(producer.runAttempt) },
    builtAt: builtAt ?? new Date().toISOString(),
    buildOutcome: 'success',
  };
}

/**
 * Authenticate a producer run from the Actions API: this repository's draft
 * workflow, started on main by an issue or a dispatch, not a fork.
 */
export function producerProblems(run, repository) {
  const problems = [];
  if (!run) return ['producer run not found'];
  if (run.path !== PRODUCER_WORKFLOW) problems.push('workflow ' + run.path);
  if (run.head_branch !== 'main') problems.push('branch ' + run.head_branch);
  if (!PRODUCER_EVENTS.includes(run.event)) problems.push('event ' + run.event);
  if (run.repository?.full_name !== repository) problems.push('repository ' + run.repository?.full_name);
  if (run.head_repository?.full_name !== repository) problems.push('head repository ' + run.head_repository?.full_name);
  return problems;
}

/** Problems with a listed build or claim artifact before it is downloaded. */
export function artifactProblems(artifact, name, repository) {
  const problems = [];
  if (artifact?.name !== name) problems.push('name ' + artifact?.name);
  if (artifact?.expired) problems.push('expired');
  if (!(artifact?.size_in_bytes > 0)) problems.push('empty');
  if (!artifact?.workflow_run?.id) problems.push('no producer run');
  if (artifact?.workflow_run?.head_branch && artifact.workflow_run.head_branch !== 'main') problems.push('branch ' + artifact.workflow_run.head_branch);
  if (repository && artifact?.workflow_run?.repository_id && artifact?.workflow_run?.head_repository_id
    && artifact.workflow_run.repository_id !== artifact.workflow_run.head_repository_id) problems.push('fork producer');
  return problems;
}

/**
 * Verify a downloaded build product: the manifest names this identity, the
 * authenticated producer run, and the exact binary bytes.
 */
export function verifyBuildProduct({ identity, manifest, binary, producerRunId, repository }) {
  if (!manifest) throw new SharedBuildError('BuildIncomplete', 'manifest missing');
  if (!binary?.length) throw new SharedBuildError('BuildIncomplete', 'binary missing');
  if (manifest.schema !== BUILD_MANIFEST_SCHEMA) throw new SharedBuildError('BuildManifestSchema', String(manifest.schema));
  if (manifest.buildOutcome !== 'success') throw new SharedBuildError('BuildFailed', String(manifest.buildOutcome));
  if (manifest.key !== identity.key || canonicalJson(manifest.inputs) !== canonicalJson(identity.inputs)) {
    throw new SharedBuildError('BuildIdentityMismatch', 'manifest ' + manifest.key + ' is not ' + identity.key);
  }
  if (manifest.producer?.workflow !== PRODUCER_WORKFLOW || manifest.producer?.repository !== repository || String(manifest.producer?.runId) !== String(producerRunId)) {
    throw new SharedBuildError('BuildProducerMismatch', JSON.stringify(manifest.producer) + ' is not run ' + producerRunId + ' of ' + repository);
  }
  if (!HEX64.test(manifest.binary?.sha256 ?? '')) throw new SharedBuildError('BuildIncomplete', 'manifest has no binary digest');
  const digest = sha256(binary);
  if (digest !== manifest.binary.sha256 || binary.length !== manifest.binary.bytes) {
    throw new SharedBuildError('BuildDigestMismatch', 'binary ' + digest + ' (' + binary.length + ' bytes) is not ' + manifest.binary.sha256);
  }
  return { key: identity.key, sha256: digest, bytes: binary.length, producerRunId: String(producerRunId), sourceCommit: manifest.sourceCommit };
}

const order = (left, right) => String(left.created_at).localeCompare(String(right.created_at)) || left.id - right.id;

/**
 * One resolution step. `observe` returns the listed build artifacts, the
 * listed claims and, for every producer run, its run and build-job status.
 * The answer is `reuse` (an authenticated artifact), `build` (this run is the
 * earliest live claim, or the wait expired) or `wait`.
 */
export function decide({ key, self, repository, builds, claims, runs, waitedSeconds, maxWaitSeconds }) {
  const rejected = [];
  const authentic = artifact => {
    const run = runs[artifact.workflow_run?.id];
    const problems = [...artifactProblems(artifact, artifact.name, repository), ...producerProblems(run?.run, repository)];
    if (problems.length) rejected.push({ id: artifact.id, name: artifact.name, run: artifact.workflow_run?.id ?? null, problems });
    return problems.length === 0;
  };
  const build = builds.filter(artifact => artifact.name === BUILD_ARTIFACT_PREFIX + key).filter(authentic).sort(order).at(-1);
  if (build) return { mode: 'reuse', artifact: build, producerRunId: String(build.workflow_run.id), rejected };
  const live = claims.filter(artifact => artifact.name === CLAIM_ARTIFACT_PREFIX + key).filter(authentic)
    .filter(artifact => String(artifact.workflow_run.id) === String(self) || ['queued', 'in_progress', 'waiting', 'pending', 'requested'].includes(runs[artifact.workflow_run.id]?.buildJob))
    .sort(order);
  const leader = live[0] ? String(live[0].workflow_run.id) : String(self);
  if (leader === String(self)) return { mode: 'build', reason: 'producer', leader, rejected };
  if (waitedSeconds >= maxWaitSeconds) return { mode: 'build', reason: 'wait-expired', leader, rejected };
  return { mode: 'wait', leader, rejected };
}

/** Bounded resolution loop over an injected GitHub API. */
export async function resolveSharedBuild({ key, self, repository, api, sleep, now, settleSeconds = 15, pollSeconds = 20, maxWaitSeconds = 600, log = () => {} }) {
  const started = now();
  await sleep(settleSeconds * 1000);
  for (;;) {
    const list = async prefix => (await api('/repos/' + repository + '/actions/artifacts?per_page=100&name=' + encodeURIComponent(prefix + key)))?.artifacts ?? [];
    const builds = await list(BUILD_ARTIFACT_PREFIX);
    const claims = await list(CLAIM_ARTIFACT_PREFIX);
    const runs = {};
    for (const id of new Set([...builds, ...claims].map(artifact => artifact.workflow_run?.id).filter(Boolean))) {
      const run = await api('/repos/' + repository + '/actions/runs/' + id).catch(() => null);
      const jobs = run ? await api('/repos/' + repository + '/actions/runs/' + id + '/jobs?filter=latest&per_page=100').catch(() => null) : null;
      runs[id] = { run, buildJob: jobs?.jobs?.find(job => job.name === PRODUCER_JOB)?.status ?? null };
    }
    const waitedSeconds = Math.round((now() - started) / 1000);
    const decision = { ...decide({ key, self, repository, builds, claims, runs, waitedSeconds, maxWaitSeconds }), waitedSeconds };
    for (const rejected of decision.rejected) log('rejected ' + rejected.name + ' #' + rejected.id + ' from run ' + rejected.run + ': ' + rejected.problems.join(', '));
    if (decision.mode !== 'wait') return decision;
    log('waiting for the producer run ' + decision.leader + ' of build ' + key + ' (' + waitedSeconds + 's)');
    await sleep(pollSeconds * 1000);
  }
}
