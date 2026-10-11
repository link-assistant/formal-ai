import { readFileSync } from 'node:fs';
import test from 'node:test';
import assert from 'node:assert/strict';
import { validateCallerAdmission, observeCallerAdmission } from '../../../scripts/release-fixture-caller-admission.mjs';
import { validateDiagnosticDescendant } from '../../../scripts/release-fixture-lease-diagnostics.mjs';

const context = Object.freeze({repository: 'owner/repo', run: '123', attempt: '2', eventHead: 'a'.repeat(40)});
const base = '/repos/owner/repo/actions';
const groupName = 'formal-ai-release-fixture-123-2';
const childName = 'Active reusable fixture / Fixture lease holder active';
function syntheticSnapshots() {
  const job = {id: 42, name: childName, run_id: 123, run_attempt: 2, head_sha: context.eventHead,
    status: 'queued', completed_at: null, url: 'https://api.github.com' + base + '/jobs/42'};
  return [
    {status: 200, route: base + '/runs/123', observedAt: '2026-10-11T00:00:00Z',
      body: {id: 123, run_attempt: 2, head_sha: context.eventHead,
        path: '.github/workflows/release-runtime-fixture.yml', status: 'in_progress'}},
    {status: 200, route: base + '/runs/123/jobs?filter=latest&per_page=100',
      observedAt: '2026-10-11T00:00:01Z', body: {total_count: 1, jobs: [job]}},
    {status: 200, route: base + '/concurrency_groups/' + groupName,
      observedAt: '2026-10-11T00:00:02Z', body: {group_name: groupName, total_count: 1,
        group_members: [{run_id: 123, job_id: 42, job_name: childName, status: 'in_progress',
          job_url: job.url, run_url: 'https://api.github.com' + base + '/runs/123'}]}}
  ];
}

test('synthetic admission permits queuing but refuses executing-lease promotion', () => {
  const snapshots = syntheticSnapshots();
  const receipt = validateCallerAdmission(...snapshots, context);
  assert.equal(receipt.observation, 'ObservedCallerAdmissionForContender');
  assert.equal(receipt.descendantStatus, 'queued');
  assert.equal(receipt.executingLeaseProof, false);
  assert.equal(receipt.productionAuthority, false);
  assert.throws(() => validateDiagnosticDescendant({status: 200, body: snapshots[1].body.jobs[0]},
    {...context, id: 42, name: childName, runnerName: 'synthetic-runner'}));
});

test('synthetic admitted executing status still carries no lease proof', () => {
  const snapshots = syntheticSnapshots();
  snapshots[1].body.jobs[0].status = 'in_progress';
  assert.equal(validateCallerAdmission(...snapshots, context).executingLeaseProof, false);
});

test('synthetic foreign, ambiguous, truncated, terminal and stale admissions refuse', () => {
  const mutations = [
    values => values[0].status = 403,
    values => values[0].route += '/foreign',
    values => values[0].body.id = 124,
    values => values[0].body.run_attempt = 1,
    values => values[0].body.head_sha = 'b'.repeat(40),
    values => values[0].body.path = '.github/workflows/release.yml',
    values => values[0].body.status = 'queued',
    values => values[1].body.jobs.push(structuredClone(values[1].body.jobs[0])),
    values => values[1].body.total_count = 2,
    values => values[1].body.jobs[0].name = 'foreign caller',
    values => values[1].body.jobs[0].run_id = 124,
    values => values[1].body.jobs[0].run_attempt = 1,
    values => values[1].body.jobs[0].head_sha = 'b'.repeat(40),
    values => values[1].body.jobs[0].status = 'completed',
    values => values[1].body.jobs[0].completed_at = '2026-10-11T00:00:01Z',
    values => values[1].body.jobs[0].url = 'https://foreign.invalid',
    values => values[2].body.group_name = 'formal-ai-release-fixture-123-1',
    values => values[2].body.group_members[0].status = 'pending',
    values => values[2].body.group_members[0].job_id = 43,
    values => values[2].body.group_members[0].run_id = 124,
    values => values[2].body.group_members[0].job_url = 'https://foreign.invalid',
    values => values[2].body.group_members.push(structuredClone(values[2].body.group_members[0])),
    values => values[2].body.total_count = 2,
    values => values[2].body.group_members.push({job_name: 'Fixture private queued contender'}),
    values => values[2].observedAt = '2026-10-11T00:00:06Z',
    values => values[1].observedAt = '2026-10-10T23:59:59Z'
  ];
  for (const mutate of mutations) {
    const values = syntheticSnapshots();
    mutate(values);
    assert.throws(() => validateCallerAdmission(...values, context));
  }
});

test('synthetic bounded GET orchestration preserves routes and never dispatches work', async () => {
  const values = syntheticSnapshots();
  const calls = [];
  const receipt = await observeCallerAdmission(async (route, timeout) => {
    calls.push({route, timeout});
    return values[calls.length - 1];
  }, context, Date.now() + 1000);
  assert.deepEqual(calls.map(value => value.route), values.map(value => value.route));
  assert.ok(calls.every(value => value.timeout > 0 && value.timeout <= 1000));
  assert.equal(receipt.executingLeaseProof, false);
  await assert.rejects(observeCallerAdmission(() => {throw Error('must not call');}, context, Date.now() - 1));
});

import { observeActiveAncestor } from '../../../scripts/release-runtime-fixture.mjs';
function syntheticPhysicalReceipts() {
  const physical = {id: 42, name: childName, run_id: 123, run_attempt: 2, head_sha: context.eventHead,
    status: 'in_progress', runner_name: 'synthetic-runner', runner_id: 7,
    url: 'https://api.github.com' + base + '/jobs/42'};
  return [{status: 200, body: {jobs: [structuredClone(physical)]}},
    {status: 200, body: physical},
    {status: 200, body: {group_name: groupName, group_members: [{run_id: 123, status: 'in_progress'}]}}];
}
async function syntheticPhysicalAttempt(values, runnerName = 'synthetic-runner') {
  const calls = [];
  const receipt = await observeActiveAncestor({
    get: async route => {calls.push(route); return values[calls.length - 1];},
    context, name: groupName, label: 'Fixture lease holder active', snapshots: [], runnerName
  });
  return {calls, receipt};
}

test('synthetic physical execution guard binds exact current runner, source, run and attempt', async () => {
  const result = await syntheticPhysicalAttempt(syntheticPhysicalReceipts());
  assert.equal(result.receipt.descendantJobId, 42);
  assert.deepEqual(result.calls, [base + '/runs/123/jobs?filter=latest&per_page=100',
    base + '/jobs/42', base + '/concurrency_groups/' + groupName + '?ahead_of_job=42']);
});

test('synthetic queued, foreign and missing physical execution cannot use admission as a witness', async () => {
  for (const mutate of [
    values => values[1].status = 403,
    values => values[1].body.id = 43,
    values => values[1].body.run_id = 124,
    values => values[1].body.run_attempt = 1,
    values => values[1].body.head_sha = 'b'.repeat(40),
    values => values[1].body.name = 'unrelated descendant',
    values => values[1].body.runner_name = 'foreign-runner',
    values => values[1].body.runner_id = 0,
    values => values[1].body.status = 'queued',
    values => values[1].body.url = 'https://foreign.invalid'
  ]) {
    const values = syntheticPhysicalReceipts();
    mutate(values);
    await assert.rejects(syntheticPhysicalAttempt(values));
  }
  await assert.rejects(syntheticPhysicalAttempt(syntheticPhysicalReceipts(), null));
  await assert.rejects(syntheticPhysicalAttempt(syntheticPhysicalReceipts(), ''));
});

test('production queue call site supplies missing-runner refusal rather than optional omission', () => {
  const source = readFileSync(new URL('../../../scripts/release-runtime-fixture.mjs', import.meta.url), 'utf8');
  assert.ok(source.includes('runnerName: process.env.RUNNER_NAME ?? null'));
  assert.ok(source.includes("assert.equal(typeof runnerName, 'string')"));
});
