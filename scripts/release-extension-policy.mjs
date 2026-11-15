// Scratch policy adapter. Authenticated API/verifier boundaries must be supplied by a real driver.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const identifier = value => assert.match(String(value),
 /^[1-9][0-9]*$/u);
const commit = value => assert.match(value,
 /^[a-f0-9]{40}$/u);
export function verifyAuthoritativeTransfer(expected,
 evidence,
 verifySignedBytes) {
  assert.match(expected.repository,
 /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  assert.match(expected.tag,
 /^v[0-9]+\.[0-9]+\.[0-9]+$/u);
  for (const field of ['sourceCommit',
 'sourceTree',
 'protocolCommit']) commit(expected[field]);
  identifier(expected.runId);
  identifier(expected.attempt);
  const run = evidence.run;
  assert.equal(String(run.id),
 String(expected.runId));
  assert.equal(run.run_attempt,
 expected.attempt);
  assert.equal(run.repository.full_name,
 expected.repository);
  assert.equal(run.head_repository.full_name,
 expected.repository);
  assert.equal(run.head_branch,
 'main');
  assert.equal(run.path,
 '.github/workflows/desktop-release.yml');
  assert.ok(['workflow_run',
 'release',
 'workflow_dispatch'].includes(run.event));
  assert.equal(run.status,
 'completed');
  assert.ok(!['cancelled',
 'skipped'].includes(run.conclusion));
  assert.equal(run.head_sha,
 expected.protocolCommit,
 'workflow source pin differs');
  for (const name of ['Package VS Code extension (.vsix)',
 'Publish SHA256SUMS.txt + provenance']) {
    const jobs = evidence.jobs.filter(job => job.name === name);
    assert.equal(jobs.length,
 1);
    assert.equal(jobs[0].run_id,
 run.id);
    assert.equal(jobs[0].run_attempt,
 expected.attempt);
    assert.equal(jobs[0].status,
 'completed');
    assert.equal(jobs[0].conclusion,
 'success');
    identifier(jobs[0].id);
  }
  assert.equal(evidence.consumerChecks.id,
 expected.consumerCheckJobId);
  assert.equal(evidence.consumerChecks.run_id,
 expected.consumerRunId);
  assert.equal(evidence.consumerChecks.run_attempt,
 expected.consumerAttempt);
  identifier(expected.consumerCheckJobId);
  identifier(expected.consumerRunId);
  identifier(expected.consumerAttempt);
  assert.equal(evidence.consumerChecks.status,
 'completed');
  assert.equal(evidence.consumerChecks.conclusion,
 'success');
  assert.equal(evidence.release.tag_name,
 expected.tag);
  assert.equal(evidence.release.draft,
 false);
  assert.equal(evidence.release.prerelease,
 false);
  assert.equal(evidence.tagCommit.sha,
 expected.sourceCommit);
  assert.equal(evidence.tagCommit.commit.tree.sha,
 expected.sourceTree);
  const source = JSON.parse(new TextDecoder('utf-8',
 {
    fatal: true
  }).decode(evidence.sourceBytes));
  assert.equal(source.version,
 1);
  assert.equal(source.release_tag,
 expected.tag);
  assert.equal(source.source_commit,
 expected.sourceCommit);
  assert.equal(source.source_tree,
 expected.sourceTree);
  assert.equal(source.producer_run,
 String(run.id));
  const filename = 'formal-ai-vscode-' + expected.tag.slice(1) + '.vsix';
  const digest = hash(evidence.artifactBytes),

    assets = evidence.release.assets.filter(asset => asset.name === filename);
  assert.equal(assets.length,
 1);
  identifier(assets[0].id);
  assert.equal(assets[0].state,
 'uploaded');
  assert.equal(assets[0].size,
 evidence.artifactBytes.length);
  assert.equal(assets[0].digest,
 'sha256:' + digest);
  const rows = new TextDecoder('utf-8',
 {
    fatal: true
  }).decode(evidence.checksumBytes).trim().split(/\r?\n/u);
  const checksums = new Map();
  for (const row of rows) {
    const m = /^([a-f0-9]{64}) [ *]([^/\\\r\n]+)$/u.exec(row);
    assert.ok(m);
    if (m[2] === filename) assert.ok(!checksums.has(m[2]),
      'duplicate extension checksum');
    checksums.set(m[2],
 m[1]);
  }
  assert.equal(checksums.get(filename),
 digest);
  const subjectProducers = [];
  for (const [name, bytes, jobName] of [
    [filename, evidence.artifactBytes, 'Package VS Code extension (.vsix)'],
    ['formal-ai-native-source-' + expected.tag.slice(1) + '.json', evidence.sourceBytes, 'Publish SHA256SUMS.txt + provenance']
  ]) {
    const request = {name, bytes, repository: expected.repository,
      signerWorkflow: expected.repository + '/.github/workflows/desktop-release.yml',
      signerDigest: expected.protocolCommit};
    const verified = verifySignedBytes(request);
    const attempts = signedSubjectAttempts(expected, request, verified);
    subjectProducers.push(verifySubjectProducer(expected, evidence, request, attempts, jobName));
  }
  return {
    name: filename,

    sha256: digest,

    bytes: Buffer.from(evidence.artifactBytes),

    producerRun: run.id,

    producerAttempt: subjectProducers[0].attempt,

    sourceProducerAttempt: subjectProducers[1].attempt,

    latestWorkflowAttempt: expected.attempt,

    subjectProducers,

    sourceCommit: expected.sourceCommit,

    sourceTree: expected.sourceTree,

    publicationCommandExecuted: false,

    driverAuthenticationNotProvedByThisPurePolicy: true
  };
}

function signedIdentityFailure(expected, request, records, reason, candidateAttempts = []) {
  const observed = (Array.isArray(records) ? records : []).slice(0, 8).map(record => {
    const result = record?.verificationResult;
    const invocation = result?.signature?.certificate?.runInvocationURI;
    const parsed = /^https:\/\/github\.com\/[^/]+\/[^/]+\/actions\/runs\/([1-9][0-9]*)\/attempts\/([1-9][0-9]*)$/u.exec(invocation ?? '');
    const subjects = Array.isArray(result?.statement?.subject) ? result.statement.subject : [];
    const digest = subjects.find(subject => subject?.name === request.name)?.digest?.sha256;
    return {runId: parsed ? parsed[1] : null, attempt: parsed ? parsed[2] : null,
      sha256: typeof digest === 'string' && /^[a-f0-9]{64}$/u.test(digest) ? digest : null};
  });
  assert.fail('signed producer identity mismatch: ' + JSON.stringify({
    subject: request.name, sha256: hash(request.bytes), runId: expected.runId,
    latestWorkflowAttempt: expected.attempt, reason, candidateAttempts: candidateAttempts.slice(0, 8), observed
  }));
}

/** Discover exact cryptographically verified identities; an older attempt alone grants no authority. */
export function signedSubjectAttempts(expected, request, verifiedRecords) {
  assert.match(expected.repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  identifier(expected.runId);identifier(expected.attempt);commit(expected.protocolCommit);
  assert(Number.isSafeInteger(expected.attempt) && expected.attempt > 0);
  assert.match(request.name, /^[A-Za-z0-9][A-Za-z0-9._-]{0,159}$/u);
  assert(Buffer.isBuffer(request.bytes));
  if (!Array.isArray(verifiedRecords) || verifiedRecords.length === 0 || verifiedRecords.length > 32)
    signedIdentityFailure(expected, request, [], 'missing-or-unbounded-verified-records');
  const prefix = 'https://github.com/' + expected.repository + '/actions/runs/' + expected.runId + '/attempts/';
  const identity = 'https://github.com/' + expected.repository + '/.github/workflows/desktop-release.yml@refs/heads/main';
  const attempts = new Set();
  for (const record of verifiedRecords) {
    const statement = record?.verificationResult?.statement;
    const certificate = record?.verificationResult?.signature?.certificate;
    const invocation = certificate?.runInvocationURI;
    if (typeof invocation !== 'string' || !invocation.startsWith(prefix)) continue;
    const suffix = invocation.slice(prefix.length);
    if (!/^[1-9][0-9]*$/u.test(suffix)) continue;
    const attempt = Number(suffix);
    if (!Number.isSafeInteger(attempt) || attempt > expected.attempt) continue;
    const subjects = Array.isArray(statement?.subject) ? statement.subject.filter(subject => subject.name === request.name) : [];
    if (certificate.buildSignerURI === identity
      && certificate.buildSignerDigest === expected.protocolCommit
      && certificate.sourceRepositoryURI === 'https://github.com/' + expected.repository
      && certificate.issuer === 'https://token.actions.githubusercontent.com'
      && certificate.runnerEnvironment === 'github-hosted'
      && statement?.predicateType === 'https://slsa.dev/provenance/v1'
      && statement.predicate?.runDetails?.metadata?.invocationId === invocation
      && subjects.length === 1 && subjects[0].digest?.sha256 === hash(request.bytes)) attempts.add(attempt);
  }
  if (attempts.size === 0 || attempts.size > 8)
    signedIdentityFailure(expected, request, verifiedRecords, 'missing-or-unbounded-exact-signed-identity');
  return [...attempts].sort((left, right) => left - right);
}

function verifySubjectProducer(expected, evidence, request, attempts, jobName) {
  const selected = attempts.includes(expected.attempt) ? expected.attempt : attempts.length === 1 ? attempts[0] : null;
  if (selected === null) signedIdentityFailure(expected, request, [], 'ambiguous-nonlatest-producer-attempts', attempts);
  if (evidence.producerProofs === undefined) {
    assert.equal(selected, expected.attempt, 'historical signed subject requires exact authenticated producer proof');
    return {name: request.name, sha256: hash(request.bytes), runId: expected.runId, attempt: selected,
      jobId: evidence.jobs.find(job => job.name === jobName).id};
  }
  assert(Array.isArray(evidence.producerProofs) && evidence.producerProofs.length === 2,
    'exact per-subject producer proofs required');
  const matches = evidence.producerProofs.filter(proof => proof.name === request.name);
  assert.equal(matches.length, 1, 'missing or ambiguous subject producer proof');
  const proof = matches[0], run = proof.run, inventory = proof.jobs;
  assert.equal(proof.attempt, selected, 'proof must match the preferred exact signed producer attempt');
  assert.equal(String(run.id), String(expected.runId));assert.equal(run.run_attempt, selected);
  assert.equal(run.repository.full_name, expected.repository);assert.equal(run.head_repository.full_name, expected.repository);
  assert.equal(run.head_branch, 'main');assert.equal(run.path, '.github/workflows/desktop-release.yml');
  assert(['workflow_run', 'release', 'workflow_dispatch'].includes(run.event));
  assert.equal(run.status, 'completed');
  assert(['success', 'failure', 'neutral', 'timed_out', 'action_required', 'stale'].includes(run.conclusion));
  assert.equal(run.head_sha, expected.protocolCommit, 'signed producer source differs');
  assert(inventory && Array.isArray(inventory.jobs) && inventory.jobs.length <= 100);
  assert.equal(inventory.total_count, inventory.jobs.length, 'producer proof job inventory must be complete');
  const ids = new Set();
  for (const job of inventory.jobs) {
    identifier(job.id);assert(!ids.has(job.id), 'duplicate producer job identity');ids.add(job.id);
    assert.equal(String(job.run_id), String(expected.runId));assert.equal(job.run_attempt, selected);
    assert.equal(job.status, 'completed');
    if (job.head_sha !== undefined) assert.equal(job.head_sha, expected.protocolCommit);
  }
  const jobs = inventory.jobs.filter(job => job.name === jobName);
  assert.equal(jobs.length, 1, 'one exact signed-subject producer job required');
  assert.equal(jobs[0].conclusion, 'success', 'signed-subject producer job must succeed');
  return {name: request.name, sha256: hash(request.bytes), runId: expected.runId, attempt: selected, jobId: jobs[0].id};
}
