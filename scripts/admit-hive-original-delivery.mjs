#!/usr/bin/env node
// Hosted read-only source and CI admission; effect attempts consume its authenticated result.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { isDeepStrictEqual } from 'node:util';
import { policyIdentity, readWorkflowPolicy } from './lib/hive-delivery-source.mjs';
import { planClients, artifactJson } from './lib/hive-delivery-journal.mjs';
import { wakeupName, wakeupRecord, validateWakeup, nextContinuation, maxContinuations } from './lib/hive-delivery-deferral.mjs';
import { observePublishedAdmission, latePublishedBindingDecision } from './lib/hive-delivery-release.mjs';
import { boundedCommand } from '../js/server/bounded-command.mjs';
import { createTreeReader } from './lib/hive-delivery-trees.mjs';
import { authenticatePullRequest, applicableWorkflows, ciAdmission, ciAdmissionDecision, deliveryWorkflow, admissionPolicy } from './lib/hive-delivery-admission.mjs';
const maximumBytes = 4 * 1024 * 1024;
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export function githubReader({
  repository,
  transport = boundedCommand,
  environment = process.env,
  diagnostics = record => console.error('GitHub read diagnostic ' + JSON.stringify(record))
} = {}) {
  assert.match(repository ?? '', /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  let requests = 0, restRequests = 0, graphqlRequests = 0,
    byteCount = 0;
  let terminalReadFailure = null;
  const deadline = Date.now() + 150000;
  const request = async (arguments_, kind, operation) => {
    assert.ok(requests < 512, 'bounded admission API requests');
    if (terminalReadFailure) throw terminalReadFailure;
    const stableArguments = [...arguments_];
    const requestSha256 = hash(Buffer.from(JSON.stringify(stableArguments)));
    for (let attempt = 1; attempt <= 2; attempt++) {
      assert.ok(requests < 512, 'bounded admission API requests');
      const remaining = deadline - Date.now();
      assert.ok(remaining > 0, 'admission deadline');
      requests++;
      if (kind === 'graphql') graphqlRequests++; else restRequests++;
      const result = await transport('gh', [...stableArguments], {
        env: environment,
        timeout: Math.min(20000, remaining),
        stdoutLimitBytes: maximumBytes,
        stderrLimitBytes: 65536
      });
      if (result.status !== 0 || result.signal !== null || result.error || result.timed_out || result.cleanup_complete !== true) {
        const stdout = Buffer.from(result.stdout ?? ''), stderr = Buffer.from(result.stderr ?? '');
        const http = /(?:^|[\s(])HTTP\s+([1-5][0-9]{2})(?=[:\s)]|$)/u.exec(stderr.toString());
        const httpStatus = http ? Number(http[1]) : null;
        const errorCode = result.error?.code;
        // Only unchanged, read-only REST calls may repeat once. Failed bodies,
        // 4xx denials, GraphQL costs and incomplete process cleanup never retry.
        const retrying = kind === 'rest' && attempt === 1 && result.status === 1 && result.signal === null &&
          !result.error && !result.timed_out && result.cleanup_complete === true && stdout.length === 0 &&
          (httpStatus === null || httpStatus >= 500) && requests < 512 && Date.now() < deadline;
        const diagnostic = {
          kind, operation, requestSha256, attempt, maximumAttempts: kind === 'rest' ? 2 : 1,
          request: requests, status: result.status, signal: result.signal,
          timedOut: result.timed_out === true, cleanupComplete: result.cleanup_complete === true,
          errorCode: typeof errorCode === 'string' && /^[A-Z0-9_]{1,64}$/u.test(errorCode) ? errorCode : null,
          httpStatus, retrying,
          stdoutBytes: stdout.length, stderrBytes: stderr.length,
          stdoutSha256: hash(stdout), stderrSha256: hash(stderr)
        };
        diagnostics(diagnostic);
        if (retrying) continue;
        const failure = new Error('GitHub read failed ' + JSON.stringify(diagnostic));
        failure.code = 'GITHUB_READ_FAILED'; failure.httpStatus = httpStatus;
        // Missing individual candidates may be skipped. Unavailable authority
        // ends this reader's observation; a bulk scan cannot hammer other paths.
        if (httpStatus !== 404) terminalReadFailure = failure;
        throw failure;
      }
      const body = Buffer.from(result.stdout);
      assert.ok(body.length <= maximumBytes, 'admission response bytes');
      byteCount += body.length;
      assert.ok(byteCount <= 64 * 1024 * 1024, 'admission aggregate bytes');
      return body;
    }
    throw new Error('bounded admission read attempts exhausted');
  };
  const bytes = async (endpoint, {
    accept = null
  } = {}) => {
    assert.ok(accept === null || accept === 'application/octet-stream');
    assert.ok(endpoint.startsWith('repos/' + repository + '/') && !endpoint.split('?')[0].split('/').some(value => value === '.' || value === '..'),
 'owned GitHub endpoint');
    const operation = /\/commits\/[^/?]+\/pulls(?:\?|$)/u.test(endpoint) ? 'commit-pull-requests' : 'owned-rest-read';
    return request(['api', '--hostname', 'github.com', ...(accept ? ['-H', 'Accept: ' + accept] : []), endpoint], 'rest', operation);
  };
  const treeReader = createTreeReader(repository, async query => JSON.parse(new TextDecoder('utf-8', {fatal: true}).decode(
    await request(['api', '--hostname', 'github.com', 'graphql', '-f', 'query=' + query], 'graphql', 'owned-tree-query'))));
  const read = async endpoint => JSON.parse(new TextDecoder('utf-8', {
    fatal: true
  }).decode(await bytes(endpoint)));
  const pages = async (endpoint, member) => {
    const values = [];
    let count;
    for (let page = 1; page <= 20; page++) {
      const result = await read(endpoint + (endpoint.includes('?') ? '&' : '?') + 'per_page=100&page=' + page);
      const items = member ? result[member] : result;
      assert.ok(Array.isArray(items) && items.length <= 100);
      if (member) {
        assert.ok(Number.isSafeInteger(result.total_count) && result.total_count <= 2000);
        count ??= result.total_count;
        assert.equal(count, result.total_count, 'API census changed');
      }
      values.push(...items);
      assert.ok(values.length <= 2000);
      if (items.length < 100 || count !== undefined && values.length === count) {
        if (count !== undefined) assert.equal(values.length, count, 'partial API census');
        return values;
      }
    }
    throw new Error('admission pagination limit');
  };
  return {
    read,
    bytes,
    pages,
    readTree: treeReader.readTree,
    statistics: () => ({
      requests, restRequests, graphqlRequests, graphqlPoints: treeReader.statistics().points,
      bytes: byteCount
    })
  };
}
export async function selectPullRequest({
  repository,
  eventName,
  event,
  reader
}) {
  let trigger = null,
    number;
  if (eventName === 'workflow_dispatch') {
    assert.match(event.inputs?.['pull-request'] ?? '', /^[1-9]\d*$/u);
    number = Number(event.inputs['pull-request']);
  } else {
    assert.equal(eventName, 'workflow_run');
    assert.ok(Number.isSafeInteger(event.workflow_run?.id));
    trigger = await reader.read('repos/' + repository + '/actions/runs/' + event.workflow_run.id);
    assert.equal(trigger.id, event.workflow_run.id);
    assert.equal(trigger.repository.full_name, repository);
    assert.equal(trigger.head_repository.full_name, repository);
    assert.equal(trigger.status, 'completed');
    assert.notEqual(trigger.path, deliveryWorkflow);
    const candidates = await reader.pages('repos/' + repository + '/commits/' + trigger.head_sha + '/pulls');
    const current = candidates.filter(value => value.state === 'open' && !value.merged_at && value.head.repo?.full_name === repository && value.head.sha === trigger.head_sha);
    if (current.length === 0) return null;
    assert.equal(current.length, 1, 'ambiguous current pull-request source');
    number = current[0].number;
  }
  const pullRequest = await reader.read('repos/' + repository + '/pulls/' + number);
  const requestedHead = trigger?.head_sha ?? pullRequest.head.sha;
  return authenticatePullRequest({
    repository,
    number,
    requestedHead,
    pullRequest,
    trigger
  });
}
/** Fetch trigger source before selected checkout; an absent applicable workflow cannot silently disappear. */
export async function observeAdmission({
  repository,
  eventName,
  event,
  reader
}) {
  const selected = await selectPullRequest({
    repository,
    eventName,
    event,
    reader
  });
  if (selected === null) return {
    admitted: false,
    reason: 'not-current-open-pull-request'
  };
  const pullRequest = await reader.read('repos/' + repository + '/pulls/' + selected.number);
  const changed = await reader.pages('repos/' + repository + '/pulls/' + selected.number + '/files');
  assert.equal(changed.length, pullRequest.changed_files, 'incomplete changed-path census');
  assert.ok(changed.length > 0 && changed.length <= 2000);
  const workflows = await readWorkflowPolicy(reader, repository, selected.source);
  const expected = applicableWorkflows(workflows, {
    event: 'pull_request',
    baseBranch: selected.baseBranch,
    files: changed.map(value => value.filename)
  });
  const runs = await reader.pages('repos/' + repository + '/actions/runs?head_sha=' + selected.source, 'workflow_runs');
  for (const run of runs) {
    assert.equal(run.repository.full_name, repository);
    assert.equal(run.head_repository.full_name, repository);
  }
  let proof;
  try {
    proof = ciAdmission({
      source: selected.source,
      expected,
      runs,
      complete: true
    });
  } catch (error) {
    return {
      admitted: false,
      reason: error.message,
      selected,
      expected
    };
  }
  // Refuse a head update or file-count change during the complete multi-read census.
  const latest = await reader.read('repos/' + repository + '/pulls/' + selected.number);
  authenticatePullRequest({
    repository,
    number: selected.number,
    requestedHead: selected.source,
    pullRequest: latest
  });
  assert.equal(latest.base.sha, selected.base, 'pull-request base changed during admission');
  assert.equal(latest.changed_files, pullRequest.changed_files);
  return {
    schema: admissionPolicy,
    admitted: true,
    selected,
    expected,
    ci: proof,
    authorshipCredit: 0
  };
}
/**
 * Revalidate source and all CI immediately before credentials; source-qualified obligations remain immutable.
 * Ordinary non-admission is typed (#1485): a pending prerequisite keeps its blockedBy run and a changed
 * candidate is 'source-changed'. Malformed or unauthenticated data still throws.
 */
export async function recheckAdmission({
  selection,
  repository,
  eventName,
  event,
  reader
}) {
  assert.equal(selection.schema, admissionPolicy);
  assert.equal(selection.admitted, true);
  const changed = {admitted: false, reason: 'source-changed'};
  if (selection.selected.event === 'published-release') {
    const verifiedRunId = selection.publicDownloadVerification.run;
    const latest = await reader.read('repos/' + repository + '/actions/workflows/verify-release-downloads.yml/runs?per_page=1');
    assert.ok(Array.isArray(latest.workflow_runs) && latest.workflow_runs.length === 1, 'bounded latest producer metadata');
    if (latest.workflow_runs[0].id !== verifiedRunId) return {...changed, supersededBy: latest.workflow_runs[0].id};
    const current = await observePublishedAdmission({
      repository,
      reader,
      verifiedRunId
    });
    if (!current.admitted) return {
      admitted: false,
      reason: current.reason,
      ...(current.blockedBy ? {blockedBy: current.blockedBy} : {})
    };
    if (!isDeepStrictEqual(current.selected, selection.selected) || !isDeepStrictEqual(current.publishedRuntime, selection.publishedRuntime)) return changed;
    return {
      ...selection,
      ci: current.ci
    };
  }
  if (eventName === 'workflow_dispatch') {
    // A closed, merged or moved pull request is a changed candidate, not malformed custody.
    const current = await reader.read('repos/' + repository + '/pulls/' + selection.selected.number);
    if (current.state !== 'open' || current.merged !== false || current.head?.sha !== selection.selected.source) return changed;
  }
  const selected = await selectPullRequest({
    repository,
    eventName,
    event,
    reader
  });
  if (selected === null || !isDeepStrictEqual(selected, selection.selected)) return changed;
  const runs = await reader.pages('repos/' + repository + '/actions/runs?head_sha=' + selected.source, 'workflow_runs');
  for (const run of runs) {
    assert.equal(run.repository.full_name, repository);
    assert.equal(run.head_repository.full_name, repository);
  }
  assert.ok(Array.isArray(selection.expected) && selection.expected.length > 0, 'admitted CI obligations');
  if (runs.length === 0) return {admitted: false, reason: 'missing applicable CI ' + selection.expected[0], selected};
  const decision = ciAdmissionDecision({
    source: selected.source,
    expected: selection.expected,
    runs,
    complete: true
  });
  if (!decision.admitted) return {...decision, selected};
  return {
    ...selection,
    ci: decision.ci
  };
}
/** A late event is only a read-only wakeup; it cannot select a different release source. */
export function isLatePublishedCompletion(trigger, repository) {
  return trigger?.repository?.full_name === repository && trigger?.head_repository?.full_name === repository
    && trigger.status === 'completed' && trigger.conclusion === 'success'
    && trigger.event === 'push' && trigger.head_branch === 'main'
    && Number.isSafeInteger(trigger.id) && trigger.id > 0
    && Number.isSafeInteger(trigger.run_attempt) && trigger.run_attempt > 0
    && /^[a-f0-9]{40}$/u.test(trigger.head_sha ?? '')
    && typeof trigger.path === 'string';
}
const wakeupRoles = ['claude', 'agent', 'codex', 'admission'];
export const wakeupMember = role => 'wakeup-' + role + '.json';
/**
 * Event-driven continuation (#1485). When a run that blocked a published candidate
 * completes, find the wakeup records that named it. Each record is a tiny artifact
 * produced by this exact delivery workflow from a protocol on main. The resumed
 * admission is a full admission of the same immutable candidate: a superseded
 * producer or a different selection is a refusal with a new attempt identity, and a
 * failed, cancelled or skipped prerequisite is a refusal, never a pass.
 */
export async function observeDeferredWakeups({repository, reader, trigger}) {
  if (trigger.head_branch !== 'main' || trigger.status !== 'completed' || trigger.path === deliveryWorkflow
    || ['pull_request', 'pull_request_target'].includes(trigger.event)) return null;
  const main = await reader.read('repos/' + repository + '/git/ref/heads/main');
  const wakeups = [];
  for (const role of wakeupRoles) {
    const artifacts = await reader.pages('repos/' + repository + '/actions/artifacts?name=' + encodeURIComponent(wakeupName(trigger.id, role)), 'artifacts');
    assert.ok(Array.isArray(artifacts), 'wakeup artifact census');
    // Equivalent records for one blocking run are bounded; the newest few suffice.
    for (const metadata of artifacts.filter(value => value.expired === false).slice(0, 4)) {
      assert.equal(metadata.name, wakeupName(trigger.id, role));
      const producer = await reader.read('repos/' + repository + '/actions/runs/' + metadata.workflow_run.id);
      assert.equal(producer.id, metadata.workflow_run.id);
      assert.equal(producer.repository.full_name, repository);
      assert.equal(producer.head_repository.full_name, repository);
      assert.equal(producer.path, deliveryWorkflow, 'wakeup produced by the delivery workflow');
      assert.ok(['workflow_run', 'workflow_dispatch'].includes(producer.event));
      assert.equal(producer.head_sha, metadata.workflow_run.head_sha);
      const ancestry = await reader.read('repos/' + repository + '/compare/' + producer.head_sha + '...' + main.object.sha);
      assert.ok(['ahead', 'identical'].includes(ancestry.status), 'wakeup producer protocol from main');
      const value = validateWakeup(await artifactJson(reader, repository, metadata, wakeupMember(role)), {repository, trigger});
      assert.equal(value.run, producer.id);
      assert.equal(value.role, role);
      wakeups.push(value);
    }
  }
  if (wakeups.length === 0) return null;
  // Only the newest public producer can still be admitted; older candidates are superseded below.
  const candidate = wakeups.map(value => value.candidate).sort((left, right) => right.verifiedRun - left.verifiedRun)[0];
  const generation = Math.max(...wakeups.map(value => value.generation));
  const resume = {trigger: trigger.id, triggerAttempt: trigger.run_attempt, generation,
    roles: [...new Set(wakeups.filter(value => isDeepStrictEqual(value.candidate, candidate)).map(value => value.role))].sort()};
  if (trigger.conclusion !== 'success') return {admitted: false, reason: 'prerequisite-failed', candidate, resume,
    blockedBy: {run: trigger.id, attempt: trigger.run_attempt, path: trigger.path, event: trigger.event,
      status: 'completed', conclusion: trigger.conclusion}};
  const latest = await reader.read('repos/' + repository + '/actions/workflows/verify-release-downloads.yml/runs?per_page=1');
  assert.ok(Array.isArray(latest.workflow_runs) && latest.workflow_runs.length === 1, 'bounded latest producer metadata');
  if (latest.workflow_runs[0].id !== candidate.verifiedRun) return {admitted: false, reason: 'source-changed',
    requiresNewAttemptIdentity: true, supersededBy: latest.workflow_runs[0].id, candidate, resume};
  const published = await observePublishedAdmission({repository, reader, verifiedRunId: candidate.verifiedRun});
  if (published.admitted && (published.selected.source !== candidate.source || published.selected.scope !== candidate.scope)) {
    return {admitted: false, reason: 'source-changed', requiresNewAttemptIdentity: true, candidate, resume};
  }
  return {...published, resume};
}
/** Wakeup for an admission that deferred a published candidate on a pending run, bounded by generation. */
export function admissionWakeup(result, {repository, run, attempt, observedAt}) {
  if (result.admitted || !result.candidate || !result.blockedBy || result.blockedBy.status === 'completed') return null;
  const generation = nextContinuation(result);
  if (generation === null) return null;
  return wakeupRecord({repository, run, attempt, role: 'admission', candidate: result.candidate,
    prerequisites: [result.blockedBy], observedAt, generation});
}
/** Only an authenticated public producer selects release bytes; late CI only rechecks eligibility. */
export async function observeEligibleAdmission(options) {
  const {
    repository,
    eventName,
    event,
    reader
  } = options;
  let result = await observeAdmission(options);
  if (!result.admitted && result.reason === 'not-current-open-pull-request' && eventName === 'workflow_run') {
    const trigger = await reader.read('repos/' + repository + '/actions/runs/' + event.workflow_run.id);
    assert.equal(trigger.repository.full_name, repository);
    assert.equal(trigger.head_repository.full_name, repository);
    const resumed = trigger.path === '.github/workflows/verify-release-downloads.yml' ? null
      : await observeDeferredWakeups({repository, reader, trigger});
    if (trigger.path === '.github/workflows/verify-release-downloads.yml' && trigger.status === 'completed' && trigger.conclusion === 'success') result = await observePublishedAdmission({
      repository,
      reader,
      verifiedRunId: trigger.id
    });
    else if (resumed) result = resumed;
    else if (isLatePublishedCompletion(trigger, repository)) {
      const latest = await reader.read('repos/' + repository + '/actions/workflows/verify-release-downloads.yml/runs?per_page=1');
      assert.ok(Array.isArray(latest.workflow_runs) && latest.workflow_runs.length <= 1, 'bounded latest producer metadata');
      if (latest.workflow_runs.length === 0) return {admitted: false, reason: 'no-public-download-producer'};
      const verified = latest.workflow_runs[0];
      assert.ok(Number.isSafeInteger(verified.id) && verified.id > 0, 'latest producer identity');
      assert.equal(verified.repository.full_name, repository);
      assert.equal(verified.head_repository.full_name, repository);
      assert.equal(verified.path, '.github/workflows/verify-release-downloads.yml');
      assert.ok(['completed', 'queued', 'in_progress', 'requested', 'waiting', 'pending'].includes(verified.status), 'latest producer status metadata');
      if (verified.status === 'completed') assert.ok(['success', 'failure', 'neutral', 'cancelled', 'skipped', 'timed_out', 'action_required', 'stale', 'startup_failure'].includes(verified.conclusion), 'latest producer conclusion metadata');
      else assert.equal(verified.conclusion, null, 'latest nonterminal conclusion metadata');
      if (verified.status !== 'completed') return {admitted: false, reason: 'latest-public-download-producer-pending'};
      if (verified.conclusion !== 'success') return {admitted: false, reason: 'latest-public-download-producer-not-green'};
      const published = await observePublishedAdmission({ repository, reader, verifiedRunId: verified.id, lateCompletionHead: trigger.head_sha });
      result = published.admitted ? latePublishedBindingDecision(trigger, published) : published;
    }
  }
  return result;
}
/** Read-only admission has no authority to execute a client or mutate GitHub. */
export async function main(environment = process.env) {
  assert.equal(environment.GITHUB_ACTIONS, 'true');
  const repository = environment.GITHUB_REPOSITORY;
  assert.ok(environment.GITHUB_WORKFLOW_REF?.startsWith(repository + '/' + deliveryWorkflow + '@'));
  assert.match(environment.GITHUB_WORKFLOW_SHA ?? '', /^[a-f0-9]{40}$/u);
  // Failed-job-only reruns cannot inherit prior-attempt binary custody.
  assert.equal(environment.GITHUB_RUN_ATTEMPT, '1', 'start a new admitted workflow execution for a failed client');
  const event = JSON.parse(fs.readFileSync(environment.GITHUB_EVENT_PATH, 'utf8'));
  const reader = githubReader({
    repository,
    environment
  });
  const trusted = await reader.read('repos/' + repository + '/git/ref/heads/main');
  const ancestry = await reader.read('repos/' + repository + '/compare/' + environment.GITHUB_WORKFLOW_SHA + '...' + trusted.object.sha);
  assert.ok(['ahead', 'identical'].includes(ancestry.status), 'only default-branch installed protocol may admit effects');
  const options = {
    repository,
    eventName: environment.GITHUB_EVENT_NAME,
    event,
    reader
  };
  const mode = process.argv[2] ?? 'admit';
  assert.ok(['admit', 'recheck'].includes(mode));
  let result;
  if (mode === 'recheck') {
    const selection = JSON.parse(fs.readFileSync(environment.HIVE_ADMISSION_SELECTION, 'utf8'));
    assert.equal(selection.producer.run, environment.GITHUB_RUN_ID);
    assert.equal(selection.producer.attempt, environment.GITHUB_RUN_ATTEMPT);
    result = await recheckAdmission({
      ...options,
      selection
    });
  } else {
    result = await observeEligibleAdmission(options);
    if (result.admitted) {
      Object.assign(result, await policyIdentity(reader, repository, environment.GITHUB_WORKFLOW_SHA));
      result.clients = await planClients({
        reader,
        selection: result.selected,
        policyHash: result.policyHash,
        inputHash: result.inputHash,
        currentRun: Number(environment.GITHUB_RUN_ID)
      });
    }
  }
  result.producer = {
    repository,
    run: environment.GITHUB_RUN_ID,
    attempt: environment.GITHUB_RUN_ATTEMPT,
    protocol: environment.GITHUB_WORKFLOW_SHA
  };
  let wakeup = null;
  if (mode === 'admit') {
    if (!result.admitted && result.candidate && result.blockedBy && result.blockedBy.status !== 'completed'
      && nextContinuation(result) === null) {
      result.reason = 'continuation-budget-exhausted';
      result.continuationLimit = maxContinuations;
    }
    wakeup = admissionWakeup(result, {repository, run: Number(environment.GITHUB_RUN_ID),
      attempt: Number(environment.GITHUB_RUN_ATTEMPT), observedAt: Date.now()});
    if (wakeup) {
      result.wakeup = {name: wakeupName(wakeup.prerequisites[0].run, 'admission'), generation: wakeup.generation};
      fs.writeFileSync(path.join(path.dirname(environment.HIVE_ADMISSION_OUTPUT), wakeupMember('admission')),
        JSON.stringify(wakeup, null, 2) + '\n', {flag: 'wx', mode: 0o600});
    }
  }
  const destination = environment.HIVE_ADMISSION_OUTPUT;
  assert.ok(destination && path.isAbsolute(destination));
  fs.mkdirSync(path.dirname(destination), {
    recursive: true
  });
  fs.writeFileSync(destination, JSON.stringify({
    ...result,
    statistics: reader.statistics()
  }, null, 2) + '\n', {
    flag: 'wx',
    mode: 0o600
  });
  if (environment.GITHUB_OUTPUT) {
    const lines = ['admitted=' + result.admitted, 'wakeup_name=' + (result.wakeup?.name ?? '')];
    if (result.admitted) {
      const tools = Object.entries(result.clients).filter(([, decision]) => decision.action === 'execute').map(([tool]) => tool);
      lines.push('source=' + result.selected.source,
 'scope=' + result.selected.scope,
 'tools=' + JSON.stringify(tools),
 'execute=' + (tools.length > 0),
 'published=' + !!result.publishedRuntime,
 'input_hash=' + result.inputHash,
 'policy_hash=' + result.policyHash);
    }
    fs.appendFileSync(environment.GITHUB_OUTPUT, lines.join('\n') + '\n');
  }
  console.log(JSON.stringify({
    admitted: result.admitted,
    source: result.selected?.source ?? null,
    reason: result.reason ?? null,
    authorshipCredit: 0
  }));
  return result;
}
if (import.meta.url === pathToFileURL(process.argv[1] ?? '').href) await main();
