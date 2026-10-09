// Harmless GitHub runtime fixture: text tar only, scoped GET requests and private fixture lease.
// It deliberately never invokes the production transfer CLI, compiler, package or release authority.
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync, chmodSync } from 'node:fs';
import { resolve } from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { seal, importBundle } from './release-stage-transfer.mjs';
const sha = bytes => createHash('sha256').update(bytes).digest('hex');
const marker = 'fixture-no-native-compiler';
const text = 'Formal AI GitHub artifact runtime fixture: controlled text, never native executable.\n';
const root = process.cwd();
const directory = resolve(root, '.release-fixture');
const json = file => JSON.parse(readFileSync(file, 'utf8'));
const write = (name, value) => {
  mkdirSync(directory, {
    recursive: true
  });
  writeFileSync(resolve(directory, name), JSON.stringify(value, null, 2) + '\n');
};
const outputs = record => {
  for (const [key, value] of Object.entries(record)) {
    assert.ok(!String(value).includes('\n'));
    writeFileSync(process.env.GITHUB_OUTPUT, key + '=' + value + '\n', {
      flag: 'a'
    });
  }
};
const pause = milliseconds => new Promise(done => setTimeout(done, milliseconds));
function identity() {
  assert.equal(process.env.GITHUB_ACTIONS, 'true', 'CLI fixture is CI only');
  const repository = process.env.GITHUB_REPOSITORY;
  assert.match(repository, /^[\w.-]+\/[\w.-]+$/);
  assert.match(process.env.GITHUB_RUN_ID, /^[1-9][0-9]*$/);
  assert.match(process.env.GITHUB_RUN_ATTEMPT, /^[1-9][0-9]*$/);
  assert.ok(process.env.GITHUB_WORKFLOW_REF.startsWith(repository + '/.github/workflows/release-runtime-fixture.yml@'), 'fixture caller only');
  const source = execFileSync('git', ['rev-parse', 'HEAD'], {
    cwd: root,
    encoding: 'utf8'
  }).trim();
  const tree = execFileSync('git', ['rev-parse', 'HEAD^{tree}'], {
    cwd: root,
    encoding: 'utf8'
  }).trim();
  assert.equal(source, process.env.GITHUB_SHA, 'actual checked-out fixture source differs');
  if (process.env.FIXTURE_SOURCE) assert.equal(source, process.env.FIXTURE_SOURCE, 'independent caller source differs');
  if (process.env.FIXTURE_TREE) assert.equal(tree, process.env.FIXTURE_TREE, 'independent caller tree differs');
  assert.match(process.env.FIXTURE_EVENT_HEAD, /^[a-f0-9]{40}$/);
  return {
    repository,
    eventHead: process.env.FIXTURE_EVENT_HEAD,
    run: process.env.GITHUB_RUN_ID,
    attempt: process.env.GITHUB_RUN_ATTEMPT,
    source,
    tree,
    fixture: true,
    nativeExecutable: false,
    compilerInvoked: false,
    productionAuthority: false
  };
}
function group(value) {
  assert.equal(value, 'formal-ai-release-fixture-' + process.env.GITHUB_RUN_ID + '-' + process.env.GITHUB_RUN_ATTEMPT, 'only private per-run fixture group');
  return value;
}
async function api(route) {
  const context = identity();
  assert.ok(route.startsWith('/repos/' + context.repository + '/actions/'), 'scoped Actions GET only');
  const headers = {
    Accept: 'application/vnd.github+json',
    'X-GitHub-Api-Version': '2026-03-10'
  };
  if (process.env.GH_TOKEN) headers.Authorization = 'Bearer ' + process.env.GH_TOKEN;
  const response = await fetch('https://api.github.com' + route, {
    method: 'GET',
    headers,
    redirect: 'error',
    signal: AbortSignal.timeout(15000)
  });
  const bytes = Buffer.from(await response.arrayBuffer());
  assert.ok(bytes.length <= 4 * 1024 * 1024);
  let body = null;
  try {
    body = JSON.parse(bytes);
  } catch {/* Preserve the status and physical response digest. */}
  return {
    status: response.status,
    route,
    responseSha256: sha(bytes),
    body,
    observedAt: new Date().toISOString()
  };
}
export function validateQueueSnapshot(snapshot, expected, {
  requireContender = true
} = {}) {
  assert.equal(snapshot.status, 200);
  assert.equal(snapshot.body.group_name, expected.group);
  const members = snapshot.body.group_members;
  assert.ok(Array.isArray(members));
  assert.ok(members.some(member => String(member.run_id) === expected.run && member.status === 'in_progress'), 'actual active caller ancestor lease missing');
  if (requireContender) assert.ok(members.some(member => String(member.run_id) === expected.run && member.status === 'pending' && member.job_name === 'Fixture private queued contender' && Number.isSafeInteger(member.job_id)), 'independent pending contender missing');
  return true;
}
export function validateFixtureOutputs(active, inactive, source) {
  assert.equal(active.source, source);
  assert.equal(inactive.source, source);
  assert.equal(active.artifact, 'VerifiedTextTar');
  assert.equal(active.marker, 'fixture-text-artifact-roundtrip');
  assert.equal(inactive.artifact, 'SkippedInactive');
  assert.equal(inactive.marker, '');
}
async function queueObservation(final = false) {
  const context = identity();
  const name = group(process.env.FIXTURE_GROUP);
  const base = '/repos/' + context.repository + '/actions';
  const snapshots = [];
  const deadline = Date.now() + (final ? 90000 : 180000);
  while (Date.now() < deadline) {
    const state = await api(base + '/concurrency_groups/' + encodeURIComponent(name));
    snapshots.push(state);
    if ([401, 403, 422].includes(state.status)) return {
      observation: 'Unknown',
      reason: 'ActualQueueAPIUnavailable:' + state.status,
      snapshots
    };
    try {
      validateQueueSnapshot(state, {
        group: name,
        run: context.run
      });
      const jobs = await api(base + '/runs/' + context.run + '/jobs?filter=latest&per_page=100');
      snapshots.push(jobs);
      assert.equal(jobs.status, 200);
      const label = final ? 'Fixture final active' : 'Fixture lease holder active';
      const child = jobs.body.jobs.find(job => job.name.endsWith(label) && job.status === 'in_progress');
      assert.ok(child, 'actual executing descendant job absent');
      const ancestor = await api(base + '/concurrency_groups/' + encodeURIComponent(name) + '?ahead_of_job=' + child.id);
      snapshots.push(ancestor);
      validateQueueSnapshot(ancestor, {
        group: name,
        run: context.run
      }, {
        requireContender: false
      });
      return {
        observation: 'VerifiedPrivateFixtureLease',
        group: name,
        descendantJobId: child.id,
        snapshots
      };
    } catch {/* Pending scheduling or propagation remains an observation, never guessed success. */}
    await pause(2000);
  }
  return {
    observation: 'Unknown',
    reason: 'NoCompleteActualQueuedAncestorObservation',
    snapshots
  };
}
function work(name, callback) {
  const target = resolve(directory, name);
  mkdirSync(target, {
    recursive: true
  });
  const previous = process.cwd();
  process.chdir(target);
  try {
    return callback(target);
  } finally {
    process.chdir(previous);
  }
}
function produce() {
  const context = identity();
  assert.equal(process.env.GITHUB_JOB, 'auto_compile-release', 'fixed text fixture producer only');
  Object.assign(process.env, {
    EXPECT_SOURCE: context.source,
    EXPECT_TREE: context.tree,
    RELEASE_VERSION: '0.0.0',
    EXPECT_COMPILER: sha(marker)
  });
  work('producer', () => {
    mkdirSync('prepared-release/binary', {
      recursive: true
    });
    writeFileSync('prepared-release/binary/controlled-text.txt', text);
    chmodSync('prepared-release/binary/controlled-text.txt', 0o755);
    writeFileSync('prepared-release/fixture-truth.json', JSON.stringify(context) + '\n');
    seal('binary', ['prepared-release']);
  });
  outputs({
    source: context.source,
    tree: context.tree,
    compiler_marker_sha: sha(marker)
  });
  write('producer.json', {
    ...context,
    textSha256: sha(text),
    artifactService: 'PendingActualUpload'
  });
}
function consume() {
  const context = identity();
  assert.equal(process.env.FIXTURE_ACTIVE, 'true');
  assert.equal(process.env.EXPECT_COMPILER, sha(marker), 'fixture marker differs; not an actual compiler');
  Object.assign(process.env, {
    EXPECT_SOURCE: context.source,
    EXPECT_TREE: context.tree,
    RELEASE_VERSION: '0.0.0',
    EXPECT_PRODUCER: 'auto_compile-release'
  });
  work('consumer', () => {
    importBundle('binary');
    assert.equal(readFileSync('prepared-release/binary/controlled-text.txt', 'utf8'), text);
    assert.deepEqual(json('prepared-release/fixture-truth.json'), {
      ...context
    });
  });
  const receipt = {
    ...context,
    observation: 'VerifiedTextTar',
    expectedProducer: 'auto_compile-release',
    bundleSha256: process.env.EXPECT_BUNDLE,
    descriptorSha256: process.env.EXPECT_DESCRIPTOR,
    textSha256: sha(text)
  };
  write('consumer.json', receipt);
  outputs({
    artifact_observation: receipt.observation
  });
}
async function observeStart() {
  const context = identity();
  const name = 'release-fixture-lease-start-' + context.run + '-' + context.attempt;
  const snapshots = [];
  const deadline = Date.now() + 180000;
  while (Date.now() < deadline) {
    const result = await api('/repos/' + context.repository + '/actions/runs/' + context.run + '/artifacts?per_page=100');
    snapshots.push(result);
    assert.equal(result.status, 200, 'actual same-run artifact listing failed');
    const artifact = result.body.artifacts.find(item => item.name === name && !item.expired);
    if (artifact) {
      assert.equal(String(artifact.workflow_run.id), context.run);
      assert.equal(artifact.workflow_run.head_sha, context.eventHead);
      assert.ok(Number.isSafeInteger(artifact.id));
      write('observer.json', {
        ...context,
        observation: 'ActualLiveLeaseWitnessListed',
        artifact,
        snapshots
      });
      outputs({
        witness_id: artifact.id
      });
      return;
    }
    await pause(2000);
  }
  write('observer.json', {
    ...context,
    observation: 'MissingLiveLeaseWitness',
    snapshots
  });
  throw Error('Actual live lease witness was not observed; contender must not be queued');
}
function verifyStart() {
  const context = identity();
  const receipt = json(resolve(directory, 'observer-start/lease-start.json'));
  for (const [key, value] of Object.entries(context)) assert.deepEqual(receipt[key], value);
  assert.equal(receipt.producerJob, 'lease-holder');
  assert.equal(receipt.group, group(process.env.FIXTURE_GROUP));
}
async function finalize() {
  const context = identity();
  assert.ok(['true', 'false'].includes(process.env.FIXTURE_ACTIVE));
  const active = process.env.FIXTURE_ACTIVE === 'true';
  if (active) assert.equal(process.env.ARTIFACT_OBSERVATION, 'VerifiedTextTar', 'actual artifact consumer success required');
  const lease = active ? await queueObservation(true) : {
    observation: 'NotExercisedInactive',
    snapshots: []
  };
  const receipt = {
    ...context,
    active,
    artifact: active ? 'VerifiedTextTar' : 'SkippedInactive',
    marker: active ? 'fixture-text-artifact-roundtrip' : '',
    lease,
    finalObservedAt: new Date().toISOString()
  };
  write('final-' + (active ? 'active' : 'inactive') + '.json', receipt);
  outputs({
    fixture_source: context.source,
    artifact_observation: receipt.artifact,
    fixture_marker: receipt.marker,
    lease_observation: lease.observation
  });
}
async function collect() {
  const context = identity();
  const active = {
    source: process.env.ACTIVE_SOURCE,
    artifact: process.env.ACTIVE_ARTIFACT,
    marker: process.env.ACTIVE_MARKER
  };
  const inactive = {
    source: process.env.INACTIVE_SOURCE,
    artifact: process.env.INACTIVE_ARTIFACT,
    marker: process.env.INACTIVE_MARKER
  };
  validateFixtureOutputs(active, inactive, context.source);
  assert.equal(process.env.CONTENDER_RESULT, 'success');
  const jobs = await api('/repos/' + context.repository + '/actions/runs/' + context.run + '/jobs?filter=latest&per_page=100');
  assert.equal(jobs.status, 200);
  const final = jobs.body.jobs.find(job => job.name.endsWith('Fixture final active'));
  const contender = jobs.body.jobs.find(job => job.name === 'Fixture private queued contender');
  assert.ok(final && contender);
  assert.equal(final.conclusion, 'success');
  assert.equal(contender.conclusion, 'success');
  assert.ok(Date.parse(contender.started_at) >= Date.parse(final.completed_at), 'contender started before the final callee job completed');
  const receipt = {
    ...context,
    active,
    inactive,
    actualJobs: jobs,
    leaseObservation: process.env.ACTIVE_LEASE,
    actualArtifactObservation: 'VerifiedTextTar',
    actualOutputsObservation: 'VerifiedActiveAndInactive',
    clockPrecision: 'GitHub job timestamps are seconds; raw queue/ancestor receipts are separate',
    productionReleaseProof: false,
    coldProductionTimingProof: false
  };
  write('collected.json', receipt);
  console.log(JSON.stringify({
    fixture: true,
    artifact: receipt.actualArtifactObservation,
    outputs: receipt.actualOutputsObservation,
    privateLease: receipt.leaseObservation,
    productionReleaseProof: false,
    nativeExecutable: false,
    compilerInvoked: false
  }));
}
async function main(mode) {
  assert.ok(['produce', 'consume', 'lease-start', 'wait-queue', 'observe-start', 'verify-start', 'finalize', 'contender', 'collect'].includes(mode), 'fixed fixture modes only');
  if (mode === 'produce') return produce();
  if (mode === 'consume') return consume();
  if (mode === 'observe-start') return observeStart();
  if (mode === 'verify-start') return verifyStart();
  if (mode === 'lease-start') {
    const context = identity();
    assert.equal(process.env.GITHUB_JOB, 'lease-holder');
    write('lease-start.json', {
      ...context,
      producerJob: 'lease-holder',
      group: group(process.env.FIXTURE_GROUP),
      startedAt: new Date().toISOString()
    });
    return;
  }
  if (mode === 'wait-queue') {
    const context = identity();
    const result = await queueObservation();
    write('lease-queue.json', {
      ...context,
      ...result
    });
    outputs({
      lease_observation: result.observation
    });
    return;
  }
  if (mode === 'finalize') return finalize();
  if (mode === 'contender') {
    write('contender.json', {
      ...identity(),
      privateGroup: group(process.env.FIXTURE_GROUP),
      startedAt: new Date().toISOString()
    });
    return;
  }
  return collect();
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main(process.argv[2]);
