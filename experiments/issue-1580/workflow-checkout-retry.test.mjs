// Execute the real terminal guard. Network acquisition itself remains hosted.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import YAML from 'yaml';

test('workflow source checkout retries a stalled connection within its original allowance', () => {
  const job = YAML.parse(readFileSync('.github/workflows/workflows.yml', 'utf8')).jobs.actionlint;
  const attempts = job.steps.filter(step => step.uses === 'actions/checkout@v7');
  assert.equal(attempts.length, 2);
  const [initial, retry] = attempts;
  assert.equal(initial.id, 'actionlint-checkout');
  assert.equal(retry.id, 'actionlint-checkout-retry');
  assert.equal(initial['continue-on-error'], true);
  assert.equal(retry['continue-on-error'], true);
  assert.equal(initial['timeout-minutes'], 1);
  assert.equal(retry['timeout-minutes'], 2);
  assert.equal(initial['timeout-minutes'] + retry['timeout-minutes'], 3);
  assert.equal(job['timeout-minutes'], 10);
  assert.deepEqual(retry.with, initial.with);
  assert.deepEqual(retry.env, initial.env);
  assert.equal(initial.with['persist-credentials'], false);
  assert.equal(initial.with.filter, 'blob:none');
  assert.equal(retry.if, "${{ !cancelled() && steps.actionlint-checkout.outcome == 'failure' }}");
  const guard = job.steps.find(step => step.name === 'Require a successful workflow source checkout');
  assert(guard);
  assert.equal(job.steps.indexOf(guard), job.steps.indexOf(retry) + 1);
  assert.deepEqual(guard.env, {
    INITIAL_OUTCOME: '${{ steps.actionlint-checkout.outcome }}',
    RETRY_OUTCOME: '${{ steps.actionlint-checkout-retry.outcome }}',
  });
  for (const [first, second, expected] of [
    ['success', 'skipped', 0], ['failure', 'success', 0],
    ['failure', 'failure', 1], ['failure', 'skipped', 1],
    ['skipped', 'skipped', 1], ['cancelled', 'cancelled', 1],
  ]) {
    const observed = spawnSync('bash', ['-e', '-o', 'pipefail', '-c', guard.run], {
      encoding: 'utf8', maxBuffer: 8192, timeout: 5000,
      env: {...process.env, INITIAL_OUTCOME: first, RETRY_OUTCOME: second, BASH_ENV: '/dev/null'},
    });
    assert.equal(observed.error, undefined);
    assert.equal(observed.status, expected, `${first}/${second}: ${observed.stderr}`);
    if (expected) assert.match(observed.stderr, /failed after two bounded attempts/u);
  }
});
