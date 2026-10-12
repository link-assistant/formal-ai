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
    const m = /^([a-f0-9]{64})  ([^/\\\r\n]+)$/u.exec(row);
    assert.ok(m);
    assert.ok(!checksums.has(m[2]));
    checksums.set(m[2],
 m[1]);
  }
  assert.equal(checksums.get(filename),
 digest);
  const invocation = 'https://github.com/' + expected.repository + '/actions/runs/' + run.id + '/attempts/' + expected.attempt;
  for (const [name,
 bytes] of [[filename,
 evidence.artifactBytes],
 ['formal-ai-native-source-' + expected.tag.slice(1) + '.json',
 evidence.sourceBytes]]) {
    const verified = verifySignedBytes({
      name,

      bytes,

      repository: expected.repository,

      signerWorkflow: expected.repository + '/.github/workflows/desktop-release.yml',

      signerDigest: expected.protocolCommit
    });
    assert.ok(Array.isArray(verified) &&
 verified.length > 0);
    assert.ok(verified.some(record => {
      const statement = record.verificationResult?.statement;
      const certificate = record.verificationResult?.signature?.certificate;
      const identity = 'https://github.com/' + expected.repository + '/.github/workflows/desktop-release.yml@refs/heads/main';
      return certificate?.runInvocationURI === invocation &&
 certificate.buildSignerURI === identity &&
 certificate.buildSignerDigest === expected.protocolCommit &&
 certificate.sourceRepositoryURI === 'https://github.com/' + expected.repository &&
 certificate.issuer === 'https://token.actions.githubusercontent.com' &&
 certificate.runnerEnvironment === 'github-hosted' &&
 statement?.predicateType === 'https://slsa.dev/provenance/v1' &&
 statement.predicate?.runDetails?.metadata?.invocationId === invocation &&
 statement.subject?.some(subject => subject.name === name &&
 subject.digest?.sha256 === hash(bytes));
    }),
 'missing exact signed subject/run/attempt');
  }
  return {
    name: filename,

    sha256: digest,

    bytes: Buffer.from(evidence.artifactBytes),

    producerRun: run.id,

    producerAttempt: expected.attempt,

    sourceCommit: expected.sourceCommit,

    sourceTree: expected.sourceTree,

    publicationCommandExecuted: false,

    driverAuthenticationNotProvedByThisPurePolicy: true
  };
}
