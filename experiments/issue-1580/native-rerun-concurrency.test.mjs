// Replays the native cancellation observed in run 45136124038 on 2026-11-23.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {Script} from 'node:vm';
import YAML from 'yaml';

const workflow = YAML.parse(readFileSync('.github/workflows/native-type-check.yml', 'utf8'));
const current = workflow.concurrency;
const context = (run, attempt = 1, event = 'pull_request', action = 'synchronize') => ({
  github: {workflow: 'CI/CD Pipeline', ref: 'refs/pull/1581/merge', event_name: event,
    event: {action}, run_id: run, run_attempt: attempt},
});
function evaluate(source, inputs) {
  if (typeof source !== 'string') return source;
  const expression = source.trim().replace(/^\$\{\{\s*|\s*\}\}$/gu, '');
  return new Script(expression).runInNewContext({...inputs, format: (pattern, ...values) =>
    pattern.replace(/\{(\d+)\}/gu, (_, index) => {
      assert.ok(Number(index) < values.length, 'format argument must exist');
      return String(values[Number(index)]);
    })}, {timeout: 100});
}
function lease(policy, inputs) {
  return {
    group: policy.group.replace(/\$\{\{([\s\S]*?)\}\}/gu, (_, expression) =>
      String(evaluate(expression, inputs))),
    cancel: Boolean(evaluate(policy['cancel-in-progress'], inputs)),
  };
}
function preserved(policy) {
  const initial = lease(policy, context(45136124038));
  const historicalRerun = lease(policy, context(45085971679, 2));
  assert.notEqual(initial.group, historicalRerun.group,
    'a historical failed-job rerun must not enter the current synchronization group');
  assert.equal(historicalRerun.cancel, false,
    'replaying historical work must not supersede current source');
}
test('historical failed-job reruns cannot cancel the current native run', {timeout: 2000}, () => {
  preserved(current);
});
test('new PR commits still supersede initial native work', {timeout: 2000}, () => {
  const old = lease(current, context(11));
  const fresh = lease(current, context(12));
  assert.equal(old.group, fresh.group);
  assert.equal(fresh.cancel, true);
});
test('rerun attempts retain separate groups and finish without superseding one another', {timeout: 2000}, () => {
  const contexts = [context(11, 2), context(11, 3), context(12, 2)];
  const leases = contexts.map(value => lease(current, value));
  assert.equal(new Set(leases.map(value => value.group)).size, contexts.length);
  for (const value of leases) assert.equal(value.cancel, false);
});
test('GitHub string and numeric attempt contexts agree', {timeout: 2000}, () => {
  for (const attempt of [1, 2, 3]) {
    assert.deepEqual(lease(current, context(11, String(attempt))), lease(current, context(11, attempt)));
  }
});
test('opening, dispatch and main events keep independent native groups', {timeout: 2000}, () => {
  for (const [event, action] of [['pull_request', 'opened'], ['workflow_dispatch', undefined], ['push', undefined]]) {
    const leases = [context(11, 1, event, action), context(12, 1, event, action), context(11, 2, event, action)]
      .map(value => lease(current, value));
    assert.equal(new Set(leases.map(value => value.group)).size, leases.length);
    for (const value of leases) assert.equal(value.cancel, false);
  }
});
test('the original synchronization policy reproduces the observed cancellation', {timeout: 2000}, () => {
  const original = {
    group: "${{ github.workflow }}-${{ github.ref }}-native-type-check${{ (github.event_name != 'pull_request' || github.event.action != 'synchronize') && format('-{0}', github.run_id) || '' }}",
    'cancel-in-progress': "${{ github.event_name == 'pull_request' && github.event.action == 'synchronize' }}",
  };
  assert.throws(() => preserved(original), /historical failed-job rerun/u);
});
test('native descendants inherit the reviewed workflow cancellation policy', {timeout: 2000}, () => {
  for (const [name, job] of Object.entries(workflow.jobs)) {
    assert.equal(job.concurrency, undefined, name + ' needs independent cancellation review');
  }
});
