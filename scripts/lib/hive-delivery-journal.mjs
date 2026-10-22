import { readDeliveryLedger } from './hive-delivery-ledger.mjs';
// Authenticated tiny Actions artifacts retain real client attempts across pushes.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { readArtifactZip } from '../github-artifact-by-identifier.mjs';
import { admissionPolicy, deliveryWorkflow, clientAdmission } from './hive-delivery-admission.mjs';
import { policyIdentity } from './hive-delivery-source.mjs';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
export const finalizerWorkflow = '.github/workflows/hive-original-delivery-finalize.yml';
export function journalName(scope, tool, kind, policyHash, attempt) {
  assert.match(scope, /^(?:pr-[1-9]\d*|release-v\d+\.\d+\.\d+)$/u);
  assert.ok(['claude', 'agent', 'codex'].includes(tool));
  assert.ok(['claim', 'result'].includes(kind));
  assert.match(policyHash, /^[a-f0-9]{64}$/u);
  assert.ok([1, 2].includes(attempt));
  return 'hive-' + kind + '-' + scope + '-' + tool + '-' + policyHash + '-' + attempt;
}
/** Artifact ID, full ZIP digest, exact sole member, producer run and immutable metadata are checked. */
export async function artifactJson(reader, repository, metadata, member, {
  archiveLimit = 131072,
  memberLimit = 65536
} = {}) {
  assert.ok(archiveLimit > 0 && archiveLimit <= 2 * 1024 * 1024 && memberLimit > 0 && memberLimit <= 1024 * 1024);
  assert.ok(Number.isSafeInteger(metadata.id) && metadata.id > 0);
  assert.equal(metadata.expired, false);
  assert.ok(metadata.size_in_bytes > 0 && metadata.size_in_bytes <= archiveLimit, 'tiny journal archive required');
  const endpoint = 'repos/' + repository + '/actions/artifacts/' + metadata.id;
  const actual = await reader.read(endpoint);
  assert.deepEqual(actual, metadata, 'artifact metadata changed');
  const archive = await reader.bytes(endpoint + '/zip');
  assert.equal(archive.length, metadata.size_in_bytes);
  assert.equal(metadata.digest, 'sha256:' + digest(archive), 'complete artifact byte custody');
  const entries = readArtifactZip(archive);
  assert.equal(entries.length, 1);
  assert.equal(entries[0].name, member);
  assert.equal(entries[0].directory, false);
  assert.ok(entries[0].bytes.length <= memberLimit);
  assert.deepEqual(await reader.read(endpoint), metadata, 'artifact changed during transfer');
  return JSON.parse(new TextDecoder('utf-8', {
    fatal: true
  }).decode(entries[0].bytes));
}
export function authenticateJournalProducer(run, claim, metadata, repository) {
  assert.equal(run.repository.full_name, repository);
  assert.equal(run.head_repository.full_name, repository);
  assert.equal(run.path, deliveryWorkflow);
  assert.ok(['workflow_run', 'workflow_dispatch'].includes(run.event));
  assert.equal(run.id, claim.run);
  assert.equal(run.run_attempt, claim.attempt);
  assert.equal(metadata.workflow_run.id, run.id);
  assert.equal(metadata.workflow_run.head_sha, run.head_sha);
  assert.equal(run.head_sha, claim.protocolCommit, 'producer protocol source');
  assert.match(claim.testedHead, /^[a-f0-9]{40}$/u);
  assert.equal(claim.repository, repository);
  assert.equal(claim.qualification, admissionPolicy);
}
function exactResult(value, claim) {
  assert.equal(value.schema, 'formal-ai-hive-attempt-result/v1');
  for (const key of ['repository',
 'scope',
 'tool',
 'run',
 'attempt',
 'testedHead',
 'protocolCommit',
 'policyHash',
 'inputHash',
 'imageDigest']) assert.equal(value[key],
 claim[key],
 'result bound to claim ' + key);
  assert.ok(['success', 'failure', 'uncertain'].includes(value.state));
  return value;
}
/** The caller supplies policy/input identity derived from trusted source, never from previous receipts. */
export async function authenticateDurableProducer({
  reader,
  repository,
  claim,
  result = null,
  requirePublisher = true
}) {
  const run = await reader.read('repos/' + repository + '/actions/runs/' + claim.run + '/attempts/' + claim.attempt);
  authenticateJournalProducer(run, claim, {
    workflow_run: {
      id: run.id,
      head_sha: run.head_sha
    }
  }, repository);
  const main = await reader.read('repos/' + repository + '/git/ref/heads/main');
  const ancestry = await reader.read('repos/' + repository + '/compare/' + claim.protocolCommit + '...' + main.object.sha);
  assert.ok(['ahead', 'identical'].includes(ancestry.status));
  const identity = await policyIdentity(reader, repository, claim.protocolCommit);
  assert.equal(identity.policyHash, claim.policyHash);
  assert.equal(identity.inputHash, claim.inputHash);
  const record = {
    ...claim,
    authenticated: true,
    state: 'claimed',
    producerConclusion: run.conclusion ?? null,
    cleanupComplete: false
  };
  if (result) {
    const jobs = await reader.pages('repos/' + repository + '/actions/runs/' + claim.run + '/attempts/' + claim.attempt + '/jobs', 'jobs');
    const verification = jobs.filter(value => value.name === 'Independent original Hive ' + claim.tool + ' verification');
    assert.equal(verification.length, 1);
    const publisher = jobs.filter(value => value.name === 'Persist durable Hive ' + claim.tool + ' verification');
    if (requirePublisher) assert.equal(publisher.length, 1);
    if (verification[0].status === 'completed' && (!requirePublisher || publisher[0].status === 'completed' && publisher[0].conclusion === 'success')) return {
      ...record,
      ...result,
      producerConclusion: verification[0].conclusion
    };
  }
  return record;
}
export async function observeClientHistory({
  reader,
  repository,
  scope,
  tool,
  policyHash,
  inputHash
}) {
  const durable = await readDeliveryLedger({
    reader,
    repository,
    scope,
    tool,
    policyHash,
    inputHash
  });
  if (durable) {
    // Missing or changed producer proof throws and quarantines; Actions expiry never opens this occupied key.
    return [await authenticateDurableProducer({
      reader,
      repository,
      claim: durable.claim,
      result: durable.result
    })];
  }
  // A deleted/missing durable record cannot erase surviving evidence of a consumed attempt.
  for (const attempt of [1, 2]) {
    const artifacts = await reader.pages('repos/' + repository + '/actions/artifacts?name=' + encodeURIComponent(journalName(scope,
 tool,
 'claim',
 policyHash,
 attempt)),
 'artifacts');
    assert.equal(artifacts.length, 0, 'Actions claim exists but its durable record is missing; quarantine');
  }
  return [];
}
export async function planClients({
  reader,
  selection,
  policyHash,
  inputHash,
  currentRun
}) {
  const decisions = {};
  for (const tool of ['claude', 'agent', 'codex']) {
    const history = await observeClientHistory({
      reader,
      repository: selection.repository,
      scope: selection.scope,
      tool,
      policyHash,
      inputHash
    });
    decisions[tool] = clientAdmission({
      repository: selection.repository,
      scope: selection.scope,
      tool,
      source: selection.source,
      policyHash,
      inputHash,
      currentRun,
      history,
      complete: true
    });
  }
  return decisions;
}
