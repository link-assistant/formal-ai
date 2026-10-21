import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {prepareOwnedHiveRuns, applyOwnedHiveClaim, writeCleanupPreparation, writeCleanupOutcome} from '../../../scripts/finalize-owned-hive-delivery-runs.mjs';
import {prepareCleanupClaim, readScanCursor, makeScanProgress, cleanupClaimName,
  effectOwnerArtifact} from '../../../scripts/lib/generated-task-cleanup-journal.mjs';
const protocol = 'a'.repeat(40), cleanupProtocol = 'b'.repeat(40), selected = 'c'.repeat(40);
const deliveryHash = 'd'.repeat(64), inputHash = 'e'.repeat(64), cleanupHash = 'f'.repeat(64);
const producer = {run: 900, attempt: 1, protocolCommit: cleanupProtocol};
function fixture() {
  const calls = [], effects = [], owners = new Map(), artifacts = new Map(), documents = new Map();
  let claims = [], progress = [], listingOverride = null;
  function addOwner(run = 101, artifact = 11) {
    const owner = {schema: 'formal-ai-hive-effect-owner/v1', repository: 'link-assistant/formal-ai',
      run, attempt: 1, protocolCommit: protocol, selectedSource: selected, scope: 'pr-1329',
      policyHash: deliveryHash, inputHash, taskMode: 'historical-1364', tools: ['claude']};
    const metadata = {id: artifact, name: effectOwnerArtifact, expired: false, size_in_bytes: 100,
      digest: 'sha256:' + '1'.repeat(64), workflow_run: {id: run, head_sha: protocol}};
    owners.set(artifact, owner); artifacts.set(artifact, metadata); documents.set(artifact, owner); return metadata;
  }
  const metadata = addOwner();
  const reader = {
    async read(endpoint) {
      calls.push(endpoint);
      if (endpoint.includes('/actions/artifacts?')) {
        if (listingOverride) return listingOverride(endpoint);
        return {total_count: owners.size, artifacts: [...owners.keys()].map(id => artifacts.get(id))};
      }
      if (/actions\/artifacts\/\d+$/u.test(endpoint)) {
        const value = artifacts.get(Number(endpoint.split('/').at(-1))); if (!value) throw Error('missing artifact'); return value;
      }
      if (endpoint.endsWith('/git/ref/heads/main')) return {object: {sha: cleanupProtocol}};
      if (endpoint.includes('/compare/')) return {status: 'ahead', merge_base_commit: {sha: endpoint.split('/compare/')[1].split('...')[0]}};
      const match = /runs\/(\d+)\/attempts\/(\d+)/u.exec(endpoint);
      if (match) {
        const id = Number(match[1]), attempt = Number(match[2]);
        return {id, run_attempt: attempt, head_sha: id === producer.run ? cleanupProtocol : protocol,
          repository: {full_name: 'link-assistant/formal-ai'}, head_repository: {full_name: 'link-assistant/formal-ai'},
          path: id === producer.run ? '.github/workflows/hive-original-delivery-finalize.yml'
            : '.github/workflows/hive-original-delivery.yml', event: id === producer.run ? 'schedule' : 'workflow_run',
          status: 'completed', conclusion: 'success'};
      }
      throw Error('unexpected synthetic endpoint');
    },
    async pages(endpoint) {
      calls.push(endpoint);
      if (endpoint.includes('name=hive-cleanup-attempt-')) return claims;
      if (endpoint.includes('name=hive-cleanup-scan-progress-')) return progress.filter(row => endpoint.endsWith(row.name));
      const match = /runs\/(\d+)\/artifacts/u.exec(endpoint);
      if (match) return [...owners.keys()].map(id => artifacts.get(id)).filter(row => row.workflow_run.id === Number(match[1]));
      throw Error('unexpected synthetic pages');
    },
  };
  const dependencies = {readArtifact: async (_reader, _repository, artifact) => {
    calls.push('artifact-json/' + artifact.id); return documents.get(artifact.id); },
  deliveryPolicy: async () => ({policyHash: deliveryHash, inputHash}),
  cleanupPolicy: async () => ({policyHash: cleanupHash})};
  const finalize = async options => { effects.push(options); return {applied: {results: [{status: 'closed', comment: 'recorded'}]},
    cleanupReceipt: {schema: 'generated-e2e-cleanup-proof/v1', sourceRun: options.runId,
      sourceAttempt: options.expectedRunAttempt, protocolHead: options.expectedProtocolHead, complete: true}}; };
  function publishClaim(claim, id = 21) {
    const value = {id, name: cleanupClaimName(claim.sourceRun, claim.sourceAttempt), expired: false,
      size_in_bytes: 100, digest: 'sha256:' + '2'.repeat(64), workflow_run: {id: producer.run, head_sha: cleanupProtocol}};
    artifacts.set(id, value); documents.set(id, claim); claims.push(value); return value;
  }
  return {reader, metadata, producer, dependencies, finalize, calls, effects, owners, artifacts, documents,
    addOwner, publishClaim, setClaims: value => { claims = value; }, setProgress: value => { progress = value; },
    setListing: value => { listingOverride = value; }};
}
async function prepare(f) {
  return prepareCleanupClaim({reader: f.reader, metadata: f.metadata, producer, ...f.dependencies});
}
test('nonfinite scan bounds and invalid cursors reject before IO', async () => {
  for (const options of [{maxPages: Infinity}, {maxOwners: NaN}, {cursor: {page: 0, offset: 0}}, {now: Infinity}]) {
    const f = fixture(); await assert.rejects(prepareOwnedHiveRuns({reader: f.reader, producer,
      cursor: {page: 1, offset: 0}, ...f.dependencies, ...options})); assert.equal(f.calls.length, 0);
  }
});
test('prepare creates source-bound claim data without any close', async () => {
  const f = fixture(), result = await prepare(f);
  assert.equal(result.claimName, 'hive-cleanup-attempt-101-1');
  assert.equal(result.outcomeName, 'hive-cleanup-outcome-101-1');
  assert.equal(result.claim.cleanupPolicyHash, cleanupHash);
  assert.equal(result.claim.protocolHead, protocol); assert.equal(result.claim.selectedSourceFromOwner, selected);
  assert.equal(f.effects.length, 0);
});
test('no remote durable claim means no cleanup effect', async () => {
  const f = fixture(); await assert.rejects(applyOwnedHiveClaim({reader: f.reader, producer,
    claimArtifactId: 21, finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
});
test('interruption after claim upload permanently quarantines automatic repetition', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  const again = await prepare(f); assert.equal(again.eligible, false);
  assert.equal(again.reason, 'prior-claim-quarantined'); assert.equal(f.effects.length, 0);
});
test('apply authenticates exact claim and delegates expected owner attempt and protocol', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  const result = await applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    finalize: f.finalize, ...f.dependencies});
  assert.equal(result.state, 'completed'); assert.equal(f.effects.length, 1);
  assert.equal(f.effects[0].expectedRunAttempt, 1); assert.equal(f.effects[0].expectedProtocolHead, protocol);
  assert.equal(result.cleanupReceipt.protocolHead, protocol);
});
test('claim cannot migrate to another cleanup producer or policy', async () => {
  for (const variant of ['producer', 'policy']) {
    const f = fixture(), prepared = await prepare(f);
    if (variant === 'policy') prepared.claim.cleanupPolicyHash = '0'.repeat(64);
    f.publishClaim(prepared.claim);
    await assert.rejects(applyOwnedHiveClaim({reader: f.reader,
      producer: variant === 'producer' ? {...producer, attempt: 2} : producer,
      claimArtifactId: 21, finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
  }
});
test('concurrent immutable claims refuse all issue mutations', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim); f.publishClaim(prepared.claim, 22);
  await assert.rejects(applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
});
test('changed owner artifact identity after claim is refused', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  f.metadata.digest = 'sha256:' + '3'.repeat(64);
  await assert.rejects(applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21,
    finalize: f.finalize, ...f.dependencies})); assert.equal(f.effects.length, 0);
});
test('uncertain mutation outcome remains a claim and is never retried', async () => {
  const f = fixture(), prepared = await prepare(f); f.publishClaim(prepared.claim);
  const finalize = async options => { f.effects.push(options); throw Error('transport interrupted after effect'); };
  const result = await applyOwnedHiveClaim({reader: f.reader, producer, claimArtifactId: 21, finalize, ...f.dependencies});
  assert.equal(result.state, 'uncertain'); assert.equal((await prepare(f)).eligible, false);
  assert.equal(f.effects.length, 1);
});
test('bounded exact-name enumeration exposes continuation instead of losing older owners', async () => {
  const f = fixture(); f.addOwner(102, 12);
  const result = await prepareOwnedHiveRuns({reader: f.reader, producer, maxOwners: 1,
    cursor: {page: 1, offset: 0}, ...f.dependencies});
  assert.equal(result.owners.length, 1); assert.deepEqual(result.continuation, {page: 1, offset: 1});
  assert.equal(result.incomplete, true); assert.equal(f.effects.length, 0);
  assert(f.calls.some(endpoint => endpoint.includes('/actions/artifacts?name=hive-real-effect-owner')));
});
test('active and untrusted protocol owners never produce claims', async () => {
  for (const variant of ['active', 'untrusted']) {
    const f = fixture(), original = f.reader.read;
    f.reader.read = async endpoint => {
      const value = await original(endpoint);
      if (variant === 'active' && endpoint.includes('/runs/101/')) return {...value, status: 'in_progress'};
      if (variant === 'untrusted' && endpoint.includes('/compare/')) return {...value, status: 'diverged'};
      return value;
    };
    await assert.rejects(prepare(f)); assert.equal(f.effects.length, 0);
  }
});
test('authenticated durable progress advances scheduled discovery beyond its previous bounded window', async () => {
  const f = fixture();
  const progress = await makeScanProgress({reader: f.reader, producer, now: Date.parse('2026-10-21T12:00:00Z'),
    continuation: {page: 8, offset: 17}, ...f.dependencies});
  const metadata = {id: 31, name: progress.name, expired: false, size_in_bytes: 100,
    created_at: '2026-10-21T12:00:00Z', workflow_run: {id: producer.run, head_sha: cleanupProtocol}};
  f.documents.set(31, progress.document); f.setProgress([metadata]);
  assert.deepEqual(await readScanCursor({reader: f.reader, now: Date.parse('2026-10-21T13:00:00Z'), ...f.dependencies}),
    {page: 8, offset: 17});
  progress.document.cleanupPolicyHash = '0'.repeat(64);
  await assert.rejects(readScanCursor({reader: f.reader, now: Date.parse('2026-10-21T13:00:00Z'), ...f.dependencies}));
});

test('published claim and outcome filenames match exact sole artifact members', () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'cleanup-artifact-members-'));
  try {
    const claim = {schema: 'synthetic-local-file-only', sourceRun: 101, sourceAttempt: 1};
    const prepared = writeCleanupPreparation({owners: [{run: 101, attempt: 1, claim}], progress: {continuation: null}}, directory);
    assert.equal(path.basename(prepared.owners[0].claimPath), 'claim.json');
    assert.deepEqual(JSON.parse(fs.readFileSync(prepared.owners[0].claimPath)), claim);
    const outcome = {sourceRun: 101, sourceAttempt: 1, state: 'completed'};
    const published = writeCleanupOutcome(outcome, directory);
    assert.equal(path.basename(published.outcomePath), 'outcome.json');
    assert.deepEqual(JSON.parse(fs.readFileSync(published.outcomePath)), outcome);
    assert.throws(() => writeCleanupOutcome(outcome, directory));
    assert.throws(() => writeCleanupPreparation({owners: [{run: '../other', attempt: 1, claim}], progress: {}}, directory));
  } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});
