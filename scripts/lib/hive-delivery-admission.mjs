// Pure admission policy. Authenticated API observations are supplied by the hosted adapter.
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
export const deliveryWorkflow = '.github/workflows/hive-original-delivery.yml';
export const admissionPolicy = 'formal-ai-hive-1364-admission/v1';
const commit = value => assert.match(value ?? '', /^[a-f0-9]{40}$/u);
const positive = value => assert.ok(Number.isSafeInteger(value) && value > 0);
const strings = value => {
  assert.ok(Array.isArray(value) && value.length <= 3000);
  assert.ok(value.every(item => typeof item === 'string' && item.length <= 1024));
  return value;
};
/** Match the bounded GitHub branch/path glob subset used by maintained triggers. Unknown syntax refuses. */
export function githubPattern(pattern, value) {
  assert.equal(typeof pattern, 'string');
  assert.ok(pattern.length > 0 && pattern.length <= 512);
  let expression = '^';
  for (let index = 0; index < pattern.length; index++) {
    const character = pattern[index];
    if (character === '*') {
      if (pattern[index + 1] === '*') {
        index++;
        if (pattern[index + 1] === '/') {
          index++;
          expression += '(?:.*/)?';
        } else expression += '.*';
      } else expression += '[^/]*';
    } else if (character === '?') expression += '[^/]';else if (character === '[') {
      const end = pattern.indexOf(']', index + 1);
      assert.ok(end > index + 1);
      const members = pattern.slice(index + 1, end);
      assert.match(members, /^[A-Za-z0-9_-]+$/u);
      expression += '[' + members + ']';
      index = end;
    } else {
      assert.ok(!['!', '+', '\\', '{', '}'].includes(character), 'unsupported GitHub filter');
      expression += character.replace(/[.*+?^$()|\[\]\\]/gu, '\\$&');
    }
  }
  return new RegExp(expression + '$', 'u').test(value);
}
function filtered(values, include, exclude) {
  assert.ok(include === undefined || exclude === undefined, 'conflicting filters');
  if (exclude !== undefined) return !values.every(value => strings(exclude).some(pattern => githubPattern(pattern, value)));
  if (include === undefined) return true;
  const patterns = strings(include);
  assert.ok(patterns.length > 0 && patterns.some(pattern => !pattern.startsWith('!')));
  return values.some(value => {
    let matched = false;
    for (const pattern of patterns) {
      if (githubPattern(pattern.startsWith('!') ? pattern.slice(1) : pattern, value)) matched = !pattern.startsWith('!');
    }
    return matched;
  });
}
/** Derive missing-run obligations from trusted workflow source, not the currently visible run list. */
export function applicableWorkflows(workflows, {
  event,
  baseBranch,
  files
}) {
  assert.ok(['pull_request', 'push'].includes(event));
  strings(files);
  assert.ok(files.length <= 3000);
  assert.ok(Array.isArray(workflows) && workflows.length > 0 && workflows.length <= 128);
  const expected = [];
  for (const workflow of workflows) {
    assert.match(workflow.path, /^\.github\/workflows\/[A-Za-z0-9_-]+\.ya?ml$/u);
    if (workflow.path === deliveryWorkflow) continue;
    const triggers = workflow.document.on;
    const policy = typeof triggers === 'string' ? triggers === event ? {} : undefined : Array.isArray(triggers) ? triggers.includes(event) ? {} : undefined : triggers?.[event];
    if (policy === undefined) continue;
    const configuration = policy ?? {};
    assert.equal(typeof configuration, 'object');
    assert.ok(!Array.isArray(configuration));
    for (const key of Object.keys(configuration)) assert.ok(['branches',
 'branches-ignore',
 'paths',
 'paths-ignore',
 'types',
 'tags',
 'tags-ignore'].includes(key),
 'unsupported trigger field');
    if (event === 'push' && (configuration.tags !== undefined || configuration['tags-ignore'] !== undefined) && configuration.branches === undefined && configuration['branches-ignore'] === undefined) continue;
    if (configuration.types && !strings(configuration.types).some(type => ['opened', 'synchronize', 'reopened'].includes(type))) continue;
    if (!filtered([baseBranch], configuration.branches, configuration['branches-ignore'])) continue;
    if (!filtered(files, configuration.paths, configuration['paths-ignore'])) continue;
    expected.push(workflow.path);
  }
  assert.ok(expected.length > 0, 'no applicable CI workflows');
  return expected.sort();
}
/** Authenticate the exact source before any checkout of that source or credentials for effects. */
export function authenticatePullRequest({
  repository,
  number,
  requestedHead,
  pullRequest,
  trigger
}) {
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  positive(number);
  commit(requestedHead);
  assert.equal(pullRequest.number, number);
  assert.equal(pullRequest.state, 'open');
  assert.equal(pullRequest.merged, false);
  assert.equal(pullRequest.base.repo.full_name, repository);
  assert.equal(pullRequest.head.repo.full_name, repository);
  assert.equal(pullRequest.head.sha, requestedHead, 'stale pull-request head');
  commit(pullRequest.base.sha);
  if (trigger) {
    assert.equal(trigger.repository.full_name, repository);
    assert.equal(trigger.head_repository.full_name, repository);
    assert.equal(trigger.head_sha, requestedHead);
    assert.equal(trigger.status, 'completed');
    assert.ok(trigger.path !== deliveryWorkflow, 'self trigger');
  }
  return {
    repository,
    scope: 'pr-' + number,
    number,
    source: requestedHead,
    base: pullRequest.base.sha,
    branch: pullRequest.head.ref,
    baseBranch: pullRequest.base.ref,
    event: 'pull_request'
  };
}
/** Every applicable workflow and every other observed workflow must be green at the exact source. */
export function ciAdmission({
  source,
  expected,
  runs,
  complete
}) {
  commit(source);
  assert.equal(complete, true, 'incomplete API census');
  strings(expected);
  assert.ok(expected.length > 0);
  assert.ok(Array.isArray(runs) && runs.length > 0 && runs.length <= 2000);
  const latest = new Map();
  for (const run of runs) {
    assert.equal(run.head_sha, source, 'wrong-source CI record');
    positive(run.id);
    positive(run.run_attempt);
    assert.equal(typeof run.path, 'string');
    if ([deliveryWorkflow, '.github/workflows/hive-original-delivery-finalize.yml'].includes(run.path)) continue;
    const key = run.path + '\0' + run.event,
      previous = latest.get(key);
    if (!previous || run.id > previous.id || run.id === previous.id && run.run_attempt > previous.run_attempt) latest.set(key, run);
  }
  for (const workflow of expected) assert.ok([...latest.values()].some(run => run.path === workflow), 'missing applicable CI ' + workflow);
  for (const run of latest.values()) {
    assert.equal(run.status, 'completed', 'other CI pending');
    assert.equal(run.conclusion, 'success', 'other CI not green');
  }
  return [...latest.values()].map(run => ({
    id: run.id,
    attempt: run.run_attempt,
    path: run.path,
    source
  })).sort((left, right) => left.id - right.id);
}
/** Validate every stronger acceptance clause, bound to independently authenticated source and image. */
export function requireQualifiedDelivery(receipt, {
  inputHash,
  tool,
  source,
  imageDigest
}) {
  assert.equal(receipt.schema, 'formal-ai-issue-1364-delivery/v1');
  assert.equal(receipt.complete, true);
  assert.equal(receipt.inputSha256, inputHash);
  assert.equal(receipt.tool, tool);
  assert.equal(receipt.source, source);
  assert.match(imageDigest, /^sha256:[a-f0-9]{64}$/u);
  assert.equal(receipt.imageDigest, imageDigest);
  commit(receipt.taskPRhead);
  assert.equal(receipt.firstPass.exitCode, 0);
  assert.equal(receipt.secondPass.exitCode, 0);
  for (const key of ['clean', 'verifierPassed', 'workflowCiGreen', 'allObligations']) assert.equal(receipt.firstPass[key], true, 'first pass ' + key);
  for (const key of ['clean', 'noWrite', 'noCommit']) assert.equal(receipt.secondPass[key], true, 'second pass ' + key);
  return receipt;
}
/** Success reuse names the tested source honestly; uncertain or partial attempts never authorize another write. */
export function clientAdmission({
  repository,
  scope,
  tool,
  source,
  policyHash,
  inputHash,
  currentRun,
  history,
  complete
}) {
  commit(source);
  positive(currentRun);
  assert.ok(['claude', 'agent', 'codex'].includes(tool));
  assert.equal(complete, true, 'incomplete attempt history');
  for (const value of [policyHash, inputHash]) assert.match(value, /^[a-f0-9]{64}$/u);
  assert.ok(Array.isArray(history) && history.length <= 256);
  const attempts = history.filter(record => record.repository === repository && record.scope === scope && record.tool === tool && record.policyHash === policyHash && record.inputHash === inputHash);
  const identities = new Set();
  for (const record of attempts) {
    positive(record.run);
    positive(record.attempt);
    commit(record.testedHead);
    assert.equal(record.authenticated, true, 'unauthenticated attempt');
    const identity = record.run + ':' + record.attempt;
    assert.ok(!identities.has(identity), 'duplicate attempt receipt');
    identities.add(identity);
  }
  const success = attempts.find(record => record.state === 'success' && record.producerConclusion === 'success' && record.originalOraclePassed === true && record.qualification === admissionPolicy);
  if (success) {
    requireQualifiedDelivery(success.acceptance, {
      inputHash,
      tool,
      source: success.testedHead,
      imageDigest: success.imageDigest
    });
    return {
      action: 'reuse',
      testedHead: success.testedHead,
      run: success.run,
      attempt: success.attempt
    };
  }
  if (attempts.some(record => record.run === currentRun)) return {
    action: 'refuse',
    reason: 'already-claimed'
  };
  if (attempts.some(record => record.state !== 'failure' || record.producerConclusion !== 'failure' || record.cleanupComplete !== true)) return {
    action: 'refuse',
    reason: 'uncertain-or-active-attempt'
  };
  if (attempts.length >= 2) return {
    action: 'refuse',
    reason: 'finite-attempt-budget'
  };
  return {
    action: 'execute',
    testedHead: source,
    ordinal: attempts.length + 1
  };
}

/** Cross-repository task admission is only the exact source-controlled historical replay entry. */
export function requireHistoricalTask({
  manifest,
  manifestBytes,
  inputHash,
  tool,
  evidence,
  repository,
  branch
}) {
  assert.deepEqual(manifest, JSON.parse(new TextDecoder('utf-8', {
    fatal: true
  }).decode(manifestBytes)), 'parsed canonical input');
  assert.equal(manifest.schema, 'formal-ai-issue-1364-client-replay-inputs/v1');
  assert.equal(createHash('sha256').update(manifestBytes).digest('hex'), inputHash, 'canonical input byte identity');
  const entry = manifest.clients[tool];
  assert.ok(entry);
  assert.match(entry.bodySha256 ?? '', /^[a-f0-9]{64}$/u, 'trusted public task body identity');
  if (Object.hasOwn(entry, 'body')) {
    assert.equal(createHash('sha256').update(entry.body).digest('hex'), entry.bodySha256, 'exact public task body');
  }
  const issue = /^https:\/\/github\.com\/([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+)\/issues\/([1-9]\d*)$/u.exec(entry.issueUrl);
  const pull = /^https:\/\/github\.com\/([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+)\/pull\/([1-9]\d*)$/u.exec(entry.pullRequestUrl);
  assert.ok(issue && pull);
  assert.equal(issue[1], pull[1]);
  assert.equal(repository, issue[1]);
  assert.equal(branch, entry.branch);
  assert.equal(evidence.issueUrl, entry.issueUrl);
  assert.equal(evidence.pullRequestUrl, entry.pullRequestUrl);
  return entry;
}
