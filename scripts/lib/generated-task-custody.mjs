// Custody grants cleanup of generated tests only; it never certifies feature delivery.
import {digest} from './native-session-projections.mjs';
import {isolatedFixturePrefix} from './hive-isolated-task.mjs';
export {digest};

export const originalWorkflow = '.github/workflows/hive-original-delivery.yml';
export const generatedLabels = Object.freeze(['e2e-task', 'no-formal-ai-draft']);
// Full bodies derived as closed data from this exact public generator, never by executing upstream code.
export const generatedTaskGenerator = Object.freeze({repository: 'link-assistant/hive-mind',
  commit: '8f22b87c515adece0427cb59e429531f35d7760a', path: 'scripts/hello-world-task.lib.mjs',
  gitBlobSha1: '7f3158f0aec8d09a8199d54676b020c7dd05f59f',
  sha256: '1a934b63738628a2c2310fead3205d3ee6d1cb257338cf058b17179010b60d9b'});
const completeBodyHashes = new Map([
  ['Kotlin', new Set(['57e31e88511375b7bf29e3e023d2c30ce7238c0cd71df0f1bcb685f8fa7b6653',
    '59af36c4b1b27bc55db61be9a80005e67d593ecc797f966a6595191c60e1688b'])],
  ['Scala', new Set(['639ba239291b20e4c04fec2cb524d0797e55d705dd4745a25c45ee3f420cfd30',
    '7e82e40760bbb8cde2ab27ed74d2cf82edc9c4f3481885a03a0cf23d0b334b90'])],
  ['Rust', new Set(['7d0759fe103c20e59afed08d3af25b3367fa813913cdc266d59f9a7a2cf36823',
    'f04af3d39ed1b1856a0e526d81937360b20e739fcef2e9329c02d2551808645a'])],
]);
export function generatorFromSourceInputs(text, sourceHead) {
  if (typeof text !== 'string' || Buffer.byteLength(text) > 65536 || !/^[a-f0-9]{40}$/u.test(sourceHead ?? '')) return null;
  let inputs; try { inputs = JSON.parse(text); } catch { return null; }
  if (inputs.schema !== 'formal-ai-hive-original-delivery-inputs-v1'
    || inputs.hiveRepository !== generatedTaskGenerator.repository || inputs.hiveCommit !== generatedTaskGenerator.commit) return null;
  return {...generatedTaskGenerator, inputSourceHead: sourceHead, inputSha256: digest(text),
    inputSourcePath: 'data/benchmarks/hive-original-delivery-inputs.json'};
}
const terminal = new Set(['success', 'failure', 'cancelled', 'skipped', 'timed_out', 'neutral', 'action_required']);
export function issueSnapshot(issue) {
  return {
    number: issue.number, state: issue.state, author: issue.user?.login,
    title: issue.title, createdAt: issue.created_at, updatedAt: issue.updated_at,
    bodySha256: digest(issue.body ?? ''), comments: issue.comments ?? 0,
    labels: (issue.labels ?? []).map(label => typeof label === 'string' ? label : label.name).sort(),
  };
}
export function sameIssueSnapshot(expected, actual) {
  return JSON.stringify(expected) === JSON.stringify(issueSnapshot(actual));
}
export function generatedIssue(issue) {
  const snapshot = issueSnapshot(issue);
  const language = /^Implement Hello World in (Kotlin|Scala|Rust)$/u.exec(snapshot.title ?? '')?.[1];
  return snapshot.state === 'open' && snapshot.author === 'github-actions[bot]'
    && snapshot.comments === 0 && generatedLabels.every(label => snapshot.labels.includes(label))
    && completeBodyHashes.get(language)?.has(snapshot.bodySha256) === true;
}
export function custodyFromLog({repository, run, job, log, generator}) {
  const issueNumbers = new Set(), finalIssueUrls = new Set();
  const isolatedFixtures = [];
  for (const line of String(log.text ?? '').split(/\r?\n/u)) {
    const clean = line.replace(/^\d{4}-\d\d-\d\dT[\d:.]+Z\s*/u, '');
    const match = /^\s*Final issue URL: https:\/\/github\.com\/([^/]+\/[^/]+)\/issues\/([1-9]\d*)\s*$/u.exec(clean);
    if (match) finalIssueUrls.add(`https://github.com/${match[1]}/issues/${match[2]}`);
    if (match?.[1] === repository) issueNumbers.add(Number(match[2]));
    if (clean.startsWith(isolatedFixturePrefix)) {
      const value = clean.slice(isolatedFixturePrefix.length);
      try {
        const fixture = Buffer.byteLength(value) <= 32768 ? JSON.parse(value) : null;
        isolatedFixtures.push(fixture);
        const resource = fixture?.resource;
        if (Number.isSafeInteger(resource?.issueNumber) && resource.issueNumber > 0
          && resource.issueUrl === `https://github.com/${repository}/issues/${resource.issueNumber}`) {
          issueNumbers.add(resource.issueNumber); finalIssueUrls.add(resource.issueUrl);
        }
      } catch { isolatedFixtures.push(null); }
    }
  }
  return {
    schema: 'generated-e2e-issue-custody/v1', repository, run: run.id, job: job.id,
    sourceHead: run.head_sha, workflow: run.path?.split('@')[0],
    generator,
    issueNumbers: [...issueNumbers].sort((a, b) => a - b), finalIssueUrls: [...finalIssueUrls].sort(),
    ...(isolatedFixtures.length ? {isolatedFixtures} : {}),
    logSha256: digest(log.text ?? ''), logBytes: log.bytesReceived ?? Buffer.byteLength(log.text ?? ''),
    complete: log.ok === true && log.truncated !== true && log.downloadTruncated !== true
      && log.timedOut !== true && log.streamComplete === true && log.tailComplete === true,
  };
}
// Server-reported skipped effect gates prove a no-op, never successful delivery.
export function effectWasNotStarted(job, custody) {
  const gates = ['Claim one bounded client effect after fresh CI and source revalidation',
    'Persist effect claim before any task effects', 'Run actual Hive solve and original assertions'];
  if (job.status !== 'completed' || !terminal.has(job.conclusion)
    || custody.complete !== true || custody.job !== job.id
    || custody.issueNumbers.length !== 0 || custody.finalIssueUrls.length !== 0
    || !Array.isArray(job.steps) || job.steps.length === 0
    || job.steps.some(step => step.status !== 'completed' || !terminal.has(step.conclusion))) return false;
  const positions = gates.map(name => job.steps.flatMap((step, index) => step.name === name ? [index] : []));
  return positions.every(values => values.length === 1 && job.steps[values[0]].conclusion === 'skipped')
    && positions[0][0] < positions[1][0] && positions[1][0] < positions[2][0];
}
export function cleanupDecision({repository, issue, run, jobs, custody,
  allowOwnedCancelled = false, expectedSourceHead = null}) {
  const refuse = reason => ({eligible: false, reason});
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u.test(repository ?? '')) return refuse('invalid-repository');
  if (!generatedIssue(issue) || issue.html_url !== `https://github.com/${repository}/issues/${issue.number}`) return refuse('not-owned-generated-issue');
  if (run.repository?.full_name !== repository || run.path?.split('@')[0] !== originalWorkflow) return refuse('foreign-workflow');
  if (!Number.isSafeInteger(run.run_attempt) || run.run_attempt < 1) return refuse('unknown-owner-run-attempt');
  if (!/^[a-f0-9]{40}$/u.test(run.head_sha ?? '') || expectedSourceHead && run.head_sha !== expectedSourceHead) return refuse('changed-source-head');
  if (run.status !== 'completed' || !terminal.has(run.conclusion)) return refuse('owner-run-active-or-unknown');
  if (run.conclusion === 'cancelled' && !allowOwnedCancelled) return refuse('historical-cancelled-unproven');
  if (!Array.isArray(jobs) || jobs.length === 0 || jobs.some(job => job.status !== 'completed' || !terminal.has(job.conclusion))) return refuse('test-jobs-active-or-unknown');
  const job = jobs.find(candidate => candidate.id === custody?.job);
  if (!job || !/^Actual Hive (?:claude|codex|agent) original delivery$/u.test(job.name)) return refuse('foreign-client-job');
  if (job.conclusion === 'cancelled' && !allowOwnedCancelled) return refuse('historical-cancelled-unproven');
  if (!Array.isArray(job.steps) || job.steps.length === 0 || job.steps.some(step => step.status !== 'completed' || !terminal.has(step.conclusion))) return refuse('test-steps-active-or-unknown');
  const created = Date.parse(issue.created_at), start = Date.parse(job.started_at), end = Date.parse(job.completed_at);
  if (![created, start, end].every(Number.isFinite) || created < start || created > end) return refuse('creation-outside-owned-job');
  if (custody?.issueNumbers?.length !== 1 || custody.issueNumbers[0] !== issue.number
    || custody.finalIssueUrls?.length !== 1 || custody.finalIssueUrls[0] !== issue.html_url) return refuse('ambiguous-source-custody');
  if (Object.entries(generatedTaskGenerator).some(([key, value]) => custody.generator?.[key] !== value)
    || custody.generator.inputSourceHead !== run.head_sha
    || custody.generator.inputSourcePath !== 'data/benchmarks/hive-original-delivery-inputs.json'
    || !/^[a-f0-9]{64}$/u.test(custody.generator.inputSha256 ?? '')) return refuse('unproved-actual-generator-source');
  if (custody?.schema !== 'generated-e2e-issue-custody/v1' || custody.repository !== repository
    || custody.run !== run.id || custody.sourceHead !== run.head_sha || custody.workflow !== originalWorkflow
    || custody.complete !== true || !/^[a-f0-9]{64}$/u.test(custody.logSha256 ?? '')
    || !custody.issueNumbers?.includes(issue.number)) return refuse('missing-complete-source-custody');
  const succeeded = run.conclusion === 'success' && job.conclusion === 'success';
  return {eligible: true, reason: succeeded ? 'completed' : 'not_planned',
    outcome: `${run.conclusion}/${job.conclusion}`, run: run.id, job: job.id,
    runUrl: `https://github.com/${repository}/actions/runs/${run.id}`,
    sourceHead: run.head_sha, logSha256: custody.logSha256, snapshot: issueSnapshot(issue)};
}
