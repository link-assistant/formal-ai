import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {ciAdmissionDecision, declaredScheduledWorkflows, isGatingCiRun} from '../../../scripts/lib/hive-delivery-admission.mjs';
import {precheckPublishedCi} from '../../../scripts/lib/hive-delivery-release.mjs';

const capture = JSON.parse(readFileSync(new URL('../fixtures/generated-e2e/scheduled-claim-race.json', import.meta.url), 'utf8'));
const scheduledWorkflows = declaredScheduledWorkflows([{path: capture.backgroundRun.path,
  document: {on: {schedule: capture.trustedSchedule}}}]);
const options = extra => ({source: capture.source, expected: capture.releaseRuns.map(r => r.path),
  runs: [...capture.releaseRuns, capture.backgroundRun], complete: true, ...extra});
const reader = runs => ({pages: async () => runs});

test('actual Agent claim failure is reproduced and fixed using authenticated schedule declarations', async () => {
  const old = ciAdmissionDecision(options());
  assert.equal(old.admitted, false);
  assert.equal(old.reason, capture.diagnostic.reason);
  assert.deepEqual(old.blockedBy, capture.diagnostic.blockedBy);
  const fixed = ciAdmissionDecision(options({scheduledWorkflows}));
  assert.equal(fixed.admitted, true);
  assert.deepEqual(fixed.ci.map(r => r.id).sort(), capture.releaseRuns.map(r => r.id).sort());
  const early = await precheckPublishedCi(reader(options().runs), capture.repository, capture.source, null, {scheduledWorkflows});
  assert.equal(early.admitted, true);
  assert.deepEqual(early.ci, fixed.ci);
});
test('scheduled observations never stand in for a required commit check', () => {
  const result = ciAdmissionDecision(options({scheduledWorkflows,
    expected: [...options().expected, capture.backgroundRun.path]}));
  assert.equal(result.admitted, false); assert.match(result.reason, /missing applicable CI/u);
});
test('unknown schedules, manual runs and push checks still refuse pending admission', () => {
  for (const change of [{path: '.github/workflows/unknown-security.yml'}, {event: 'push'}, {event: 'workflow_dispatch'}]) {
    const run = {...capture.backgroundRun, ...change};
    assert.equal(isGatingCiRun(run, {scheduledWorkflows}), true);
    const result = ciAdmissionDecision(options({scheduledWorkflows, runs: [...capture.releaseRuns, run]}));
    assert.equal(result.admitted, false); assert.equal(result.blockedBy.run, run.id);
  }
});
test('required publishers must still exist, finish and pass at the exact source', () => {
  for (const change of [{status: 'in_progress', conclusion: null}, {conclusion: 'failure'}, {conclusion: 'cancelled'}]) {
    const result = ciAdmissionDecision(options({scheduledWorkflows,
      runs: [{...capture.releaseRuns[0], ...change}, ...capture.releaseRuns.slice(1), capture.backgroundRun]}));
    assert.equal(result.admitted, false); assert.equal(result.blockedBy.run, capture.releaseRuns[0].id);
  }
  assert.equal(ciAdmissionDecision(options({scheduledWorkflows, runs: capture.releaseRuns.slice(1)})).admitted, false);
  assert.throws(() => ciAdmissionDecision(options({scheduledWorkflows,
    runs: [...capture.releaseRuns, {...capture.backgroundRun, head_sha: 'f'.repeat(40)}]})), /wrong-source/u);
});
test('schedule classification requires the trusted workflow source and a valid cron declaration', () => {
  assert.deepEqual(declaredScheduledWorkflows([{path: capture.backgroundRun.path, document: {on: {schedule: []}}}]), []);
  assert.deepEqual(declaredScheduledWorkflows([{path: capture.backgroundRun.path, document: {on: {schedule: [{}]}}}]), []);
  assert.throws(() => declaredScheduledWorkflows([{path: '../../untrusted.yml', document: {on: {schedule: capture.trustedSchedule}}}]));
});
