#!/usr/bin/env node
// Hosted read-only source and CI admission; effect attempts consume its authenticated result.
import fs from 'node:fs';
import path from 'node:path';
import assert from 'node:assert/strict';
import { createHash } from 'node:crypto';
import { pathToFileURL } from 'node:url';
import { policyIdentity, readWorkflowPolicy } from './lib/hive-delivery-source.mjs';
import { planClients } from './lib/hive-delivery-journal.mjs';
import { observePublishedAdmission } from './lib/hive-delivery-release.mjs';
import { boundedCommand } from '../js/server/bounded-command.mjs';
import { authenticatePullRequest, applicableWorkflows, ciAdmission, deliveryWorkflow, admissionPolicy } from './lib/hive-delivery-admission.mjs';
const maximumBytes = 4 * 1024 * 1024;
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
export function githubReader({
  repository,
  transport = boundedCommand,
  environment = process.env
} = {}) {
  assert.match(repository ?? '', /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  let requests = 0,
    byteCount = 0;
  const deadline = Date.now() + 150000;
  const bytes = async (endpoint, {
    accept = null
  } = {}) => {
    assert.ok(accept === null || accept === 'application/octet-stream');
    assert.ok(endpoint.startsWith('repos/' + repository + '/') && !endpoint.split('?')[0].split('/').some(value => value === '.' || value === '..'),
 'owned GitHub endpoint');
    assert.ok(++requests <= 512, 'bounded admission API requests');
    const remaining = deadline - Date.now();
    assert.ok(remaining > 0, 'admission deadline');
    const result = await transport('gh', ['api', '--hostname', 'github.com', ...(accept ? ['-H', 'Accept: ' + accept] : []), endpoint], {
      env: environment,
      timeout: Math.min(20000, remaining),
      stdoutLimitBytes: maximumBytes,
      stderrLimitBytes: 65536
    });
    assert.equal(result.status, 0, 'GitHub read failed');
    assert.equal(result.signal, null);
    assert.ok(!result.error && !result.timed_out && result.cleanup_complete === true, 'complete bounded GitHub read required');
    const body = Buffer.from(result.stdout);
    byteCount += body.length;
    assert.ok(byteCount <= 64 * 1024 * 1024, 'admission aggregate bytes');
    return body;
  };
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
    statistics: () => ({
      requests,
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
/** Revalidate source and all CI immediately before credentials; source-qualified obligations remain immutable. */
export async function recheckAdmission({
  selection,
  repository,
  eventName,
  event,
  reader
}) {
  assert.equal(selection.schema, admissionPolicy);
  assert.equal(selection.admitted, true);
  if (selection.selected.event === 'published-release') {
    const current = await observePublishedAdmission({
      repository,
      reader,
      verifiedRunId: selection.publicDownloadVerification.run
    });
    assert.equal(current.admitted, true);
    assert.deepEqual(current.selected, selection.selected);
    assert.deepEqual(current.publishedRuntime, selection.publishedRuntime);
    return {
      ...selection,
      ci: current.ci
    };
  }
  const selected = await selectPullRequest({
    repository,
    eventName,
    event,
    reader
  });
  assert.deepEqual(selected, selection.selected, 'source changed after preparation');
  const runs = await reader.pages('repos/' + repository + '/actions/runs?head_sha=' + selected.source, 'workflow_runs');
  for (const run of runs) {
    assert.equal(run.repository.full_name, repository);
    assert.equal(run.head_repository.full_name, repository);
  }
  try {
    return {
      ...selection,
      ci: ciAdmission({
        source: selected.source,
        expected: selection.expected,
        runs,
        complete: true
      })
    };
  } catch (error) {
    return {
      admitted: false,
      reason: error.message,
      selected
    };
  }
}
/** Only the exact completed public-download producer can select a published runtime. */
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
    if (trigger.path === '.github/workflows/verify-release-downloads.yml' && trigger.status === 'completed' && trigger.conclusion === 'success') result = await observePublishedAdmission({
      repository,
      reader,
      verifiedRunId: trigger.id
    });
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
    const lines = ['admitted=' + result.admitted];
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
