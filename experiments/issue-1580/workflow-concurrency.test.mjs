import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {spawnSync} from 'node:child_process';
import YAML from 'yaml';
import {checkSource, checkRepository} from '../../scripts/check-workflow-concurrency.mjs';

const source = (queue, cancellation = false, scope = 'workflow') => YAML.stringify({
  name: 'Queue validity control', on: 'push',
  ...(scope === 'workflow' ? {concurrency: {group: 'control', queue, 'cancel-in-progress': cancellation}} : {}),
  jobs: {control: {'runs-on': 'ubuntu-24.04',
    ...(scope === 'job' ? {concurrency: {group: 'control', queue, 'cancel-in-progress': cancellation}} : {}),
    steps: [{run: 'true'}]}}
});

for (const scope of ['workflow', 'job']) {
  test(scope + ' permits both documented queue values with cancellation disabled', () => {
    for (const queue of ['single', 'max']) {
      const result = checkSource('control.yml', source(queue, false, scope));
      assert.deepEqual(result.problems, []);
      assert.equal(result.observations.length, 1);
      assert.equal(result.observations[0].qualification, 'literal');
    }
  });
  test(scope + ' rejects unknown literal values and non-string queue nodes', () => {
    for (const queue of ['unlimited', 'Max', '', null, true, 100, ['max'], {value: 'max'}, '${{ broken']) {
      const result = checkSource('control.yml', source(queue, false, scope));
      assert.equal(result.problems.length, 1, JSON.stringify(queue));
      assert.match(result.problems[0].message, /must be max, single, or an expression/u);
    }
  });
  test(scope + ' rejects the documented max and true combination including quoted true', () => {
    for (const cancellation of [true, 'true'])
      assert.match(checkSource('control.yml', source('max', cancellation, scope)).problems[0].message,
        /cannot be combined/u);
    assert.deepEqual(checkSource('control.yml', source('single', true, scope)).problems, []);
  });
  test(scope + ' preserves unknown expression outcomes rather than rejecting them as literals', () => {
    for (const [queue, cancellation] of [
      ['${{ inputs.queue }}', true], ['max', '${{ inputs.cancel }}'],
      ['ma${{ inputs.suffix }}', false]
    ]) {
      const result = checkSource('control.yml', source(queue, cancellation, scope));
      assert.deepEqual(result.problems, []);
      assert.equal(result.observations[0].qualification, 'requires-runtime-evaluation');
    }
  });
}

test('omitted queue, scalar concurrency, and unrelated nested data retain actionlint ownership', () => {
  for (const concurrency of [undefined, 'release', {group: 'release'}, {group: 'release', queu: 'max'}]) {
    const result = checkSource('control.yml', YAML.stringify({concurrency,
      env: {queue: 'unlimited'}, jobs: {control: {steps: [{run: 'echo queue: max'}]}}}));
    assert.deepEqual(result.problems, []);
    assert.equal(result.observations.length, 0);
  }
});

test('malformed documents and duplicate queue keys fail closed', () => {
  for (const text of ['concurrency: [', '- workflow', 'concurrency:\n  queue: max\n  queue: single\n'])
    assert(checkSource('control.yml', text).problems.length > 0);
});

test('real command refuses invalid workflow sources and an empty repository', {timeout: 10000}, () => {
  const root = mkdtempSync(join(tmpdir(), 'formal-ai-workflow-queue-'));
  try {
    const directory = join(root, '.github/workflows');
    mkdirSync(directory, {recursive: true});
    const invoke = () => spawnSync(process.execPath, ['scripts/check-workflow-concurrency.mjs', root], {encoding: 'utf8'});
    assert.equal(invoke().status, 1);
    for (const [queue, cancellation, expected] of [
      ['max', false, 0], ['unlimited', false, 1], ['max', true, 1], [['single', 'max'], false, 1]
    ]) {
      writeFileSync(join(directory, 'control.yml'), source(queue, cancellation));
      const result = invoke();
      assert.equal(result.status, expected, result.stdout + result.stderr);
    }
  } finally { rmSync(root, {recursive: true, force: true}); }
});

test('all current workflow sources have explicit queue qualification', () => {
  const result = checkRepository();
  assert(result.files.length > 0);
  assert(result.observations.length > 0);
  assert.deepEqual(result.problems, []);
});

test('actual linter job admits the companion check and controls with locked parser inputs', () => {
  const workflow = YAML.parse(readFileSync('.github/workflows/workflows.yml', 'utf8'));
  const steps = workflow.jobs.actionlint.steps;
  assert(steps.some(step => step.run === 'node scripts/check-workflow-concurrency.mjs'));
  assert(steps.some(step => step.run === 'node --test --test-concurrency=1 experiments/issue-1580/workflow-concurrency.test.mjs'));
  assert(steps.some(step => step.uses === 'actions/setup-node@v7' && step.with['node-version'] === '22'));
  assert(steps.some(step => step.run?.includes('npm ci --ignore-scripts --no-audit --no-fund --prefix scripts/documentation-policy-dependencies')));
  const sparse = steps[0].with['sparse-checkout'];
  for (const path of ['/scripts/check-workflow-concurrency.mjs', '/experiments/issue-1580/workflow-concurrency.test.mjs',
    '/scripts/documentation-policy-dependencies/', '/package.json']) assert(sparse.includes(path), path);
  for (const trigger of [workflow.on.push, workflow.on.pull_request]) {
    for (const path of ['scripts/check-workflow-concurrency.mjs', 'experiments/issue-1580/workflow-concurrency.test.mjs',
      'scripts/documentation-policy-dependencies/**']) assert(trigger.paths.includes(path), path);
  }
  assert(steps.some(step => step.uses?.startsWith('docker://rhysd/actionlint@sha256:')));
  assert(steps.some(step => step.run?.includes('shellcheck-canary.yml')));
});
