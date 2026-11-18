import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {readGovernedCurrentPair} from '../../../scripts/governed-github-command-provider.mjs';
import {requireExactCurrentJob, requireExactCurrentGroupMember} from '../../../scripts/staged-caller-authority.mjs';

// Reuse the immutable public membership capture. Only redacted runner fields and
// representative ordered observation times are supplied, as in its owner test.
const capture = JSON.parse(readFileSync(new URL('../fixtures/release-fixture-membership-89a.json', import.meta.url), 'utf8'));
function fixture() {
  const [group, jobs] = structuredClone(capture.snapshots);
  const member = group.body.group_members.find(value => value.status === 'in_progress');
  const job = jobs.body.jobs.find(value => value.id === member.job_id);
  const expected = {...capture.context, head: capture.context.eventHead, runnerName: 'representative-runner'};
  job.runner_name = expected.runnerName;
  job.runner_id = 7;
  jobs.observedAt = '2026-10-21T13:24:48.153Z';
  group.observedAt = '2026-10-21T13:24:49.000Z';
  return {group, jobs, job, expected, descendantName: member.job_name,
    validateJobs: snapshot => requireExactCurrentJob(expected, member.job_name, snapshot)};
}
// Minimal recorded transport shape: read17 on main94f, 5s, SIGKILL, no bytes.
function timeout() {
  return {status: 0, provider: {attempts: [{attempt: 1, maximumAttempts: 1,
    timeoutMilliseconds: 5000, httpStatus: 0, processStatus: null,
    signal: 'SIGKILL', errorCode: 'ETIMEDOUT', bytes: 0}]}};
}
function membership(value, pair) {
  return requireExactCurrentGroupMember(value.expected, value.descendantName,
    value.group.body.group_name, pair.group, pair.jobs);
}

test('captured group timeout discards the partial pair and rereads jobs before group', () => {
  const old = fixture(), fresh = fixture(), calls = [];
  old.jobs.observedAt = '2026-10-21T13:24:40.000Z';
  const pair = readGovernedCurrentPair({deadline: 30000, now: () => 0,
    validateJobs: fresh.validateJobs,
    readJobs(attempt) {calls.push(['jobs', attempt]); return attempt === 1 ? old.jobs : fresh.jobs;},
    readGroup(attempt) {calls.push(['group', attempt]); return attempt === 1 ? timeout() : fresh.group;}});
  assert.deepEqual(calls, [['jobs', 1], ['group', 1], ['jobs', 2], ['group', 2]]);
  assert.equal(pair.jobs, fresh.jobs); assert.equal(pair.group, fresh.group);
  assert.notEqual(pair.jobs, old.jobs); membership(fresh, pair);
});

test('jobs transport failure skips its group and restarts the complete pair', () => {
  const value = fixture(), calls = [];
  const pair = readGovernedCurrentPair({deadline: 30000, now: () => 0,
    validateJobs: value.validateJobs,
    readJobs(attempt) {calls.push(['jobs', attempt]); return attempt === 1 ? timeout() : value.jobs;},
    readGroup(attempt) {calls.push(['group', attempt]); return value.group;}});
  assert.deepEqual(calls, [['jobs', 1], ['jobs', 2], ['group', 2]]); membership(value, pair);
});

test('repeated transport failures stop after at most two pairs', () => {
  for (const unavailableJobs of [false, true]) {
    const value = fixture(), calls = [];
    assert.throws(() => readGovernedCurrentPair({deadline: 30000, now: () => 0,
      validateJobs: value.validateJobs,
      readJobs(attempt) {calls.push(['jobs', attempt]); return unavailableJobs ? timeout() : value.jobs;},
      readGroup(attempt) {calls.push(['group', attempt]); return timeout();}}), /unavailable authenticatedGET current pair/);
    assert.deepEqual(calls, unavailableJobs ? [['jobs', 1], ['jobs', 2]] :
      [['jobs', 1], ['group', 1], ['jobs', 2], ['group', 2]]);
  }
});

test('pair retries share the original deadline and cannot renew an exhausted budget', () => {
  const value = fixture(), calls = []; let clock = 19000;
  assert.throws(() => readGovernedCurrentPair({deadline: 30000, now: () => clock,
    validateJobs: value.validateJobs,
    readJobs(attempt) {calls.push(['jobs', attempt]); clock += 1000; return value.jobs;},
    readGroup(attempt) {calls.push(['group', attempt]); clock += 5000; return attempt === 1 ? timeout() : value.group;}}), /aggregate observation deadline expired/);
  assert.deepEqual(calls, [['jobs', 1], ['group', 1], ['jobs', 2], ['group', 2]]);
  calls.length = 0; clock = 0;
  assert.throws(() => readGovernedCurrentPair({deadline: 30000, now: () => clock,
    validateJobs: value.validateJobs,
    readJobs() {calls.push('jobs'); return value.jobs;},
    readGroup() {calls.push('group'); clock = 30000; return timeout();}}), /aggregate observation deadline expired/);
  assert.deepEqual(calls, ['jobs', 'group']);
});

test('successful group response arriving after the aggregate deadline is refused', () => {
  const value = fixture(); let clock = 0;
  assert.throws(() => readGovernedCurrentPair({deadline: 30000, now: () => clock,
    validateJobs: value.validateJobs, readJobs: () => value.jobs,
    readGroup() {clock = 30000; return value.group;}}), /aggregate observation deadline expired/);
});

test('denied missing service and malformed responses do not trigger transport retries', () => {
  for (const failed of [{status: 401}, {status: 403}, {status: 404}, {status: 503},
    {status: 0, provider: {attempts: [{httpStatus: 200, errorCode: null}]}},
    {status: 0, provider: {attempts: [{httpStatus: 0, errorCode: null}]}}]) {
    const value = fixture(), calls = [];
    assert.throws(() => readGovernedCurrentPair({deadline: 30000, now: () => 0,
      validateJobs: value.validateJobs,
      readJobs() {calls.push('jobs'); return value.jobs;},
      readGroup() {calls.push('group'); return failed;}}));
    assert.deepEqual(calls, ['jobs', 'group']);
  }
});

test('inactive or malformed current job identity fails before group or a second jobs read', () => {
  for (const mutate of [value => value.job.status = 'completed', value => value.job.completed_at = '2026-10-21T13:24:47Z',
    value => value.job.run_attempt++, value => value.job.head_sha = 'b'.repeat(40),
    value => value.job.runner_id = null, value => value.job.url = 'https://foreign.invalid',
    value => value.jobs.body.total_count++, value => value.jobs.body.jobs = null]) {
    const value = fixture(), replacement = fixture(), calls = []; mutate(value);
    assert.throws(() => readGovernedCurrentPair({deadline: 30000, now: () => 0,
      validateJobs: value.validateJobs,
      readJobs(attempt) {calls.push('jobs'); return attempt === 1 ? value.jobs : replacement.jobs;},
      readGroup(attempt) {calls.push('group'); return attempt === 1 ? timeout() : replacement.group;}}));
    assert.deepEqual(calls, ['jobs']);
  }
});

test('semantic group and stale observation failures never become pair retries', () => {
  for (const mutate of [value => value.group.body.total_count++,
    value => value.group.body.group_members.find(member => member.status === 'in_progress').job_id++,
    value => value.group.observedAt = '2026-10-21T13:24:54.000Z']) {
    const value = fixture(), calls = []; mutate(value);
    const pair = readGovernedCurrentPair({deadline: 30000, now: () => 0,
      validateJobs: value.validateJobs,
      readJobs() {calls.push('jobs'); return value.jobs;},
      readGroup() {calls.push('group'); return value.group;}});
    assert.throws(() => membership(value, pair)); assert.deepEqual(calls, ['jobs', 'group']);
  }
});

test('current job validation is mandatory before any transport', () => {
  let calls = 0;
  assert.throws(() => readGovernedCurrentPair({deadline: 30000, now: () => 0,
    readJobs() {calls++;}, readGroup() {calls++;}}), /independent current job validation required/);
  assert.equal(calls, 0);
});
