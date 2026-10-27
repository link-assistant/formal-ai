// Source-bound immutable claims prevent automatic cleanup retries after interruption.
import assert from 'node:assert/strict';
import {digest} from './generated-task-custody.mjs';
const repository = 'link-assistant/formal-ai';
export const effectOwnerArtifact = 'hive-real-effect-owner';
export const cleanupWorkflow = '.github/workflows/hive-original-delivery-finalize.yml';
const source = /^[a-f0-9]{40}$/u, hash = /^[a-f0-9]{64}$/u;
const cleanupRoots = ['scripts/finalize-owned-hive-delivery-runs.mjs',
  'scripts/request-owned-hive-cleanup.mjs',
  'scripts/lib/generated-task-cleanup-journal.mjs', 'scripts/finalize-generated-test-issues.mjs',
  'scripts/lib/generated-task-custody.mjs', 'scripts/close-finished-test-issues.mjs',
  'scripts/lib/hive-isolated-task.mjs', 'scripts/lib/hive-delivery-evidence.mjs',
  'scripts/collect-test-replays.mjs', 'scripts/collect-ci-failures.mjs',
  'scripts/lib/hive-delivery-journal.mjs', 'scripts/lib/hive-delivery-source.mjs',
  'scripts/github-artifact-by-identifier.mjs', '.github/workflows/hive-original-delivery-finalize.yml'];
export function cleanupClaimName(run, attempt) {
  assert.ok(Number.isSafeInteger(run) && run > 0 && Number.isSafeInteger(attempt) && attempt > 0);
  return `hive-cleanup-attempt-${run}-${attempt}`;
}
export function cleanupOutcomeName(run, attempt) {
  cleanupClaimName(run, attempt); return `hive-cleanup-outcome-${run}-${attempt}`;
}
export function progressName(day) {
  assert.match(day, /^\d{4}-\d\d-\d\d$/u); return `hive-cleanup-scan-progress-${day}`;
}
export function validateCleanupCursor(cursor) {
  assert.ok(Number.isSafeInteger(cursor?.page) && cursor.page > 0 && cursor.page <= 10000);
  assert.ok(Number.isSafeInteger(cursor?.offset) && cursor.offset >= 0 && cursor.offset < 100);
  return cursor;
}
async function dependencies(options) {
  return {readArtifact: options.readArtifact ?? (await import('./hive-delivery-journal.mjs')).artifactJson,
    deliveryPolicy: options.deliveryPolicy ?? (await import('./hive-delivery-source.mjs')).policyIdentity,
    cleanupPolicy: options.cleanupPolicy ?? cleanupPolicyIdentity};
}
async function trustedProtocol(reader, protocol, cache) {
  assert.match(protocol, source);
  if (!cache.has(protocol)) {
    const main = await reader.read(`repos/${repository}/git/ref/heads/main`); assert.match(main.object?.sha, source);
    const ancestry = await reader.read(`repos/${repository}/compare/${protocol}...${main.object.sha}`);
    assert.ok(['identical', 'ahead'].includes(ancestry.status), 'protocol is not trusted main history');
    assert.equal(ancestry.merge_base_commit?.sha, protocol); cache.set(protocol, true);
  }
}
const cleanupIdentities = new WeakMap();
export async function cleanupPolicyIdentity(reader, ownedRepository, protocol) {
  assert.equal(ownedRepository, repository); assert.match(protocol, source);
  let identities = cleanupIdentities.get(reader);
  if (!identities) { identities = new Map(); cleanupIdentities.set(reader, identities); }
  if (identities.has(protocol)) return identities.get(protocol);
  assert.ok(identities.size < 128, 'bounded historical policy identities');
  const pending = computeCleanupPolicyIdentity(reader, protocol);
  identities.set(protocol, pending);
  return pending;
}
async function computeCleanupPolicyIdentity(reader, protocol) {
  const {sourceBytes, declaredPolicyRoots} = await import('./hive-delivery-source.mjs');
  const policyPath = 'scripts/lib/generated-task-cleanup-journal.mjs';
  const declared = declaredPolicyRoots(await sourceBytes(reader, repository, protocol, policyPath), 'cleanupRoots', policyPath);
  const roots = [];
  for (const path of declared) roots.push({path, sha256: digest(await sourceBytes(reader, repository, protocol, path))});
  return {policyHash: digest(JSON.stringify({schema: 'formal-ai-hive-cleanup-policy/v1', roots}))};
}
export async function authenticateCleanupProducer({reader, producer, metadata = null,
  completed = false, cache = new Map()}) {
  assert.ok(Number.isSafeInteger(producer?.run) && producer.run > 0);
  assert.ok(Number.isSafeInteger(producer.attempt) && producer.attempt > 0); assert.match(producer.protocolCommit, source);
  const run = await reader.read(`repos/${repository}/actions/runs/${producer.run}/attempts/${producer.attempt}`);
  assert.equal(run.id, producer.run); assert.equal(run.run_attempt, producer.attempt);
  assert.equal(run.repository?.full_name, repository); assert.equal(run.head_repository?.full_name, repository);
  assert.equal(run.path?.split('@')[0], cleanupWorkflow); assert.equal(run.head_sha, producer.protocolCommit);
  assert.ok(['schedule', 'workflow_run', 'workflow_dispatch'].includes(run.event));
  if (completed) assert.equal(run.status, 'completed');
  if (metadata) {
    assert.equal(metadata.workflow_run?.id, producer.run); assert.equal(metadata.workflow_run.head_sha, producer.protocolCommit);
  }
  await trustedProtocol(reader, producer.protocolCommit, cache); return run;
}
export async function authenticateEffectOwner({reader, metadata, cache = new Map(), ...options}) {
  const services = await dependencies(options);
  assert.equal(metadata.name, effectOwnerArtifact); assert.equal(metadata.expired, false);
  assert.ok(Number.isSafeInteger(metadata.id) && metadata.id > 0);
  assert.ok(metadata.size_in_bytes > 0 && metadata.size_in_bytes <= 131072); assert.match(metadata.digest, /^sha256:[a-f0-9]{64}$/u);
  const owner = await services.readArtifact(reader, repository, metadata, 'owner.json');
  assert.equal(owner.schema, 'formal-ai-hive-effect-owner/v1'); assert.equal(owner.repository, repository);
  assert.ok(Number.isSafeInteger(owner.run) && owner.run > 0 && Number.isSafeInteger(owner.attempt) && owner.attempt > 0);
  assert.equal(owner.run, metadata.workflow_run?.id); assert.match(owner.protocolCommit, source);
  assert.match(owner.selectedSource, source); assert.match(owner.policyHash, hash); assert.match(owner.inputHash, hash);
  assert.match(owner.scope, /^(?:pr-[1-9]\d*|release-v\d+\.\d+\.\d+)$/u);
  assert.equal(owner.taskMode, 'historical-1364', 'only reviewed effect policy');
  assert.ok(Array.isArray(owner.tools) && owner.tools.length > 0 && owner.tools.length <= 3
    && new Set(owner.tools).size === owner.tools.length
    && owner.tools.every(tool => ['claude', 'codex', 'agent'].includes(tool)));
  const run = await reader.read(`repos/${repository}/actions/runs/${owner.run}/attempts/${owner.attempt}`);
  assert.equal(run.id, owner.run); assert.equal(run.run_attempt, owner.attempt);
  assert.equal(run.repository?.full_name, repository); assert.equal(run.head_repository?.full_name, repository);
  assert.equal(run.path?.split('@')[0], '.github/workflows/hive-original-delivery.yml');
  assert.ok(['workflow_run', 'workflow_dispatch'].includes(run.event)); assert.equal(run.status, 'completed');
  assert.equal(run.head_sha, owner.protocolCommit); assert.equal(metadata.workflow_run.head_sha, owner.protocolCommit);
  await trustedProtocol(reader, owner.protocolCommit, cache);
  const identity = await services.deliveryPolicy(reader, repository, owner.protocolCommit);
  assert.equal(identity.policyHash, owner.policyHash); assert.equal(identity.inputHash, owner.inputHash);
  return owner;
}
async function uniqueOwnerArtifact(reader, metadata, owner, readArtifact) {
  const values = await reader.pages(`repos/${repository}/actions/runs/${owner.run}/artifacts?name=${effectOwnerArtifact}`, 'artifacts');
  assert.ok(values.length > 0 && values.length <= 16, 'bounded owner artifact census');
  const matching = [];
  for (const candidate of values) {
    assert.equal(candidate.name, effectOwnerArtifact);
    const record = candidate.id === metadata.id ? owner : await readArtifact(reader, repository, candidate, 'owner.json');
    assert.equal(record.schema, 'formal-ai-hive-effect-owner/v1'); assert.equal(record.run, owner.run);
    if (record.attempt === owner.attempt) matching.push(candidate);
  }
  assert.equal(matching.length, 1, 'ambiguous effect-owner artifact'); assert.deepEqual(matching[0], metadata);
}
export async function prepareCleanupClaim({reader, metadata, producer, cache = new Map(), expectedClaim = null, ...options}) {
  const services = await dependencies(options);
  const owner = await authenticateEffectOwner({reader, metadata, cache, ...services});
  await uniqueOwnerArtifact(reader, metadata, owner, services.readArtifact);
  await authenticateCleanupProducer({reader, producer, cache});
  const claimName = cleanupClaimName(owner.run, owner.attempt);
  const previous = await reader.pages(`repos/${repository}/actions/artifacts?name=${claimName}`, 'artifacts');
  assert.ok(previous.length <= 16, 'bounded immutable cleanup claims');
  // Discovery may cheaply retain an existing claim; queued rechecks must still authenticate their full input.
  const retained = previous.length ? {eligible: false, run: owner.run, attempt: owner.attempt, reason: 'prior-claim-quarantined'} : null;
  if (retained && expectedClaim === null) return retained;
  const identity = await services.cleanupPolicy(reader, repository, producer.protocolCommit); assert.match(identity.policyHash, hash);
  const claim = {schema: 'formal-ai-hive-cleanup-claim/v1', repository, producer,
    cleanupPolicyHash: identity.policyHash, ownerArtifactId: metadata.id, ownerArtifactDigest: metadata.digest,
    sourceRun: owner.run, sourceAttempt: owner.attempt, protocolHead: owner.protocolCommit,
    selectedSourceFromOwner: owner.selectedSource};
  if (expectedClaim !== null) assert.deepEqual(claim, expectedClaim, 'queued cleanup claim changed before publication');
  // Unknown, expired or interrupted claims cannot become permission for another mutation.
  if (retained) return retained;
  return {eligible: true, run: owner.run, attempt: owner.attempt, claimName,
    outcomeName: cleanupOutcomeName(owner.run, owner.attempt), claim};
}
export async function revalidateCleanupClaim({reader, claim, producer, ...options}) {
  assert.equal(claim?.schema, 'formal-ai-hive-cleanup-claim/v1'); assert.equal(claim.repository, repository);
  assert.deepEqual(claim.producer, producer, 'queued claim belongs to this exact cleanup invocation');
  assert.ok(Number.isSafeInteger(claim.ownerArtifactId) && claim.ownerArtifactId > 0);
  assert.match(claim.ownerArtifactDigest, /^sha256:[a-f0-9]{64}$/u);
  const metadata = await reader.read(`repos/${repository}/actions/artifacts/${claim.ownerArtifactId}`);
  assert.equal(metadata.digest, claim.ownerArtifactDigest);
  return prepareCleanupClaim({...options, reader, metadata, producer, expectedClaim: claim});
}
export async function applyCleanupClaim({reader, claimArtifactId, producer, client, finalize, ...options}) {
  assert.ok(Number.isSafeInteger(claimArtifactId) && claimArtifactId > 0);
  const services = await dependencies(options), cache = new Map();
  const metadata = await reader.read(`repos/${repository}/actions/artifacts/${claimArtifactId}`);
  const claim = await services.readArtifact(reader, repository, metadata, 'claim.json');
  assert.equal(claim.schema, 'formal-ai-hive-cleanup-claim/v1'); assert.equal(claim.repository, repository);
  assert.deepEqual(claim.producer, producer, 'claim belongs to this exact cleanup invocation');
  await authenticateCleanupProducer({reader, producer, metadata, cache});
  const identity = await services.cleanupPolicy(reader, repository, producer.protocolCommit);
  assert.equal(claim.cleanupPolicyHash, identity.policyHash);
  const name = cleanupClaimName(claim.sourceRun, claim.sourceAttempt); assert.equal(metadata.name, name);
  const claims = await reader.pages(`repos/${repository}/actions/artifacts?name=${name}`, 'artifacts');
  assert.equal(claims.length, 1, 'concurrent or ambiguous cleanup claims quarantined'); assert.deepEqual(claims[0], metadata);
  const ownerMetadata = await reader.read(`repos/${repository}/actions/artifacts/${claim.ownerArtifactId}`);
  assert.equal(ownerMetadata.digest, claim.ownerArtifactDigest);
  const owner = await authenticateEffectOwner({reader, metadata: ownerMetadata, cache, ...services});
  await uniqueOwnerArtifact(reader, ownerMetadata, owner, services.readArtifact);
  assert.equal(owner.run, claim.sourceRun); assert.equal(owner.attempt, claim.sourceAttempt);
  assert.equal(owner.protocolCommit, claim.protocolHead); assert.equal(owner.selectedSource, claim.selectedSourceFromOwner);
  finalize ??= (await import('../finalize-generated-test-issues.mjs')).finalizeGeneratedRun;
  const outcome = {schema: 'owned-hive-cleanup-outcome/v1', claimArtifactId, sourceRun: owner.run,
    sourceAttempt: owner.attempt, producer, cleanupPolicyHash: claim.cleanupPolicyHash,
    outcomeName: cleanupOutcomeName(owner.run, owner.attempt)};
  try {
    const result = await finalize({runId: owner.run, expectedRunAttempt: owner.attempt,
      expectedProtocolHead: owner.protocolCommit, apply: true, client});
    const uncertain = result.applied?.results.some(row => row.status === 'mutation-uncertain'
      || row.comment === 'uncertain-not-retried') === true;
    return {...outcome, state: uncertain ? 'uncertain' : result.cleanupReceipt?.complete === true ? 'completed'
      : result.cleanupReceipt?.noEffects === true ? 'not-started' : 'retained',
      cleanupReceipt: result.cleanupReceipt};
  } catch (error) {
    return {...outcome, state: error.code === 'OWNER_CHANGED_BEFORE_CLEANUP' ? 'retained' : 'uncertain',
      reason: error.code === 'OWNER_CHANGED_BEFORE_CLEANUP' ? 'owner-changed-before-cleanup' : 'finalizer-interrupted-or-unknown'};
  }
}
export async function readScanCursor({reader, now = Date.now(), ...options}) {
  assert.ok(Number.isFinite(now)); const services = await dependencies(options), cache = new Map();
  const policies = new Map();
  const dates = [0, 1].map(offset => new Date(now - offset * 86400000).toISOString().slice(0, 10));
  const records = [];
  for (const day of dates) {
    // A cursor is a discovery hint, never permission to mutate an owner. Read a
    // finite candidate window instead of enumerating an ever-growing journal.
    // Every selected cursor still authenticates its complete artifact/producer;
    // every owner independently requires its unique immutable claim at apply.
    const listing = await reader.read(`repos/${repository}/actions/artifacts?name=${progressName(day)}&per_page=100&page=1`);
    assert.ok(Number.isSafeInteger(listing.total_count) && listing.total_count >= 0);
    const values = listing.artifacts;
    assert.ok(Array.isArray(values) && values.length <= 100 && values.length <= listing.total_count);
    assert.ok(listing.total_count === 0 || values.length > 0, 'nonempty progress history needs a candidate window');
    assert.equal(new Set(values.map(value => value.id)).size, values.length, 'unique progress candidates');
    for (const metadata of values) records.push({day, metadata});
  }
  records.sort((left, right) => Date.parse(right.metadata.created_at) - Date.parse(left.metadata.created_at)
    || right.metadata.id - left.metadata.id);
  for (const {day, metadata} of records) {
    try {
      assert.equal(metadata.name, progressName(day));
      const progress = await services.readArtifact(reader, repository, metadata, 'progress.json');
      assert.equal(progress.schema, 'formal-ai-hive-cleanup-progress/v1'); assert.equal(progress.day, day);
      await authenticateCleanupProducer({reader, producer: progress.producer, metadata, completed: true, cache});
      const protocol = progress.producer.protocolCommit;
      if (!policies.has(protocol)) policies.set(protocol, Promise.resolve().then(() => services.cleanupPolicy(reader, repository, protocol)));
      const identity = await policies.get(protocol);
      assert.equal(progress.cleanupPolicyHash, identity.policyHash);
      return progress.continuation === null ? {page: 1, offset: 0} : validateCleanupCursor(progress.continuation);
    } catch (error) {
      if (error?.code === 'GITHUB_READ_FAILED' && error.httpStatus !== 404) throw error;
      /* An unauthenticated cursor never grants progress or source authority. */
    }
  }
  assert.equal(records.length, 0, 'progress journal exists without an authenticated cursor');
  return {page: 1, offset: 0};
}
export async function makeScanProgress({reader, producer, continuation, now = Date.now(), ...options}) {
  assert.ok(Number.isFinite(now)); if (continuation !== null) validateCleanupCursor(continuation);
  const services = await dependencies(options); await authenticateCleanupProducer({reader, producer});
  const identity = await services.cleanupPolicy(reader, repository, producer.protocolCommit);
  const day = new Date(now).toISOString().slice(0, 10);
  return {name: progressName(day), document: {schema: 'formal-ai-hive-cleanup-progress/v1',
    day, producer, cleanupPolicyHash: identity.policyHash, continuation}};
}
