// Execute the real workflow's terminal guard; checkout itself remains hosted.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import test from 'node:test';
import YAML from 'yaml';
import {admission} from '../../scripts/ci-pull-request-admission.mjs';

const workflow = YAML.parse(readFileSync('.github/workflows/native-type-check.yml', 'utf8'));
for (const name of ['clippy', 'contract-build', 'contract-family']) {
  test(`native ${name} retries checkout once within its original allowance`, () => {
    const job = workflow.jobs[name];
    const attempts = job.steps.filter(step => step.uses === 'actions/checkout@v7');
    assert.equal(attempts.length, 2);
    const [initial, retry] = attempts;
    assert.equal(initial.id, 'native-checkout');
    assert.equal(retry.id, 'native-checkout-retry');
    assert.equal(initial['continue-on-error'], true);
    assert.equal(retry['continue-on-error'], true);
    assert.equal(initial['timeout-minutes'] + retry['timeout-minutes'], 4);
    assert.equal(job['timeout-minutes'], 20);
    assert.deepEqual(retry.with, initial.with);
    assert.deepEqual(retry.env, initial.env);
    assert.equal(initial.with['persist-credentials'], false);
    assert.equal(initial.with.filter, 'blob:none');
    assert.equal(initial.with['fetch-depth'], 1);
    assert.equal(initial.with['sparse-checkout-cone-mode'], false);
    assert.equal(retry.if, name === 'clippy'
      ? "${{ !cancelled() && inputs.compile-inputs-changed && steps.native-checkout.outcome == 'failure' }}"
      : "${{ !cancelled() && steps.native-checkout.outcome == 'failure' }}");
    const guard = job.steps.find(step => step.name === 'Require a successful native checkout');
    assert(guard);
    assert.equal(job.steps.indexOf(guard), job.steps.indexOf(retry) + 1);
    assert.deepEqual(guard.env, {
      INITIAL_OUTCOME: '${{ steps.native-checkout.outcome }}',
      RETRY_OUTCOME: '${{ steps.native-checkout-retry.outcome }}',
    });
    if (name === 'clippy') {
      assert.equal(initial.if, '${{ inputs.compile-inputs-changed }}');
      assert.equal(guard.if, initial.if);
    }
    for (const [first, second, expected] of [
      ['success', 'skipped', 0], ['failure', 'success', 0],
      ['failure', 'failure', 1], ['failure', 'skipped', 1],
      ['skipped', 'skipped', 1], ['cancelled', 'cancelled', 1],
    ]) {
      const observed = spawnSync('bash', ['-e', '-o', 'pipefail', '-c', guard.run], {
        encoding: 'utf8', maxBuffer: 8192,
        env: {...process.env, INITIAL_OUTCOME: first, RETRY_OUTCOME: second, BASH_ENV: '/dev/null'},
      });
      assert.equal(observed.error, undefined);
      assert.equal(observed.status, expected, `${first}/${second}: ${observed.stderr}`);
      if (expected) assert.match(observed.stderr, /failed after two bounded attempts/u);
    }
  });
}

test('the native dynamic meta replay admits its host, package and loader inputs', () => {
  for (const path of [
    'rust/tests/web/dynamic-meta.test.mjs', 'rust/tests/web/support/structured-log-fixture.mjs',
    'js/server/dynamic-meta/host.mjs', 'js/server/dynamic-meta/runtime.mjs',
    'js/server/main.mjs', 'js/server/worker-host.mjs', 'js/server/response.mjs',
    'js/agentic/crate/links_substitution_query.mjs', 'js/worker/formal_ai_worker.js',
    'js/seed/parser.js', 'js/parser-cache-primordials.js', 'js/seed_loader.js',
    'js/worker-bootstrap-responses.js', 'js/worker-modules.js',
    'experiments/dynamic-meta/package.json', 'experiments/dynamic-meta/instructions.md',
    'experiments/js_dogfood/drive.mjs', 'package.json', 'bun.lock', '.bun-version',
  ]) assert.equal(admission([path])['native-type-check'], true, path);
  for (const path of ['js/app.js', 'ts/server/main.mts', 'experiments/unrelated.mjs',
    'docs/unrelated.md', 'rust/tests/web/unrelated.test.mjs'])
    assert.equal(admission([path])['native-type-check'], false, path);
});
