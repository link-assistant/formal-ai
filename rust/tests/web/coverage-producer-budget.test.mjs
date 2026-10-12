import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { workflowJobs } from '../../../scripts/lib/ci-speed-workflows.mjs';
import { parseSpeedPolicy } from '../../../scripts/lib/ci-speed-policy.mjs';

const workflow = readFileSync('.github/workflows/coverage.yml', 'utf8');
const producer = workflowJobs(workflow).find(job => job.id === 'coverage-build');
const policy = parseSpeedPolicy(readFileSync('data/meta/ci-speed.lino', 'utf8'));

test('the whole instrumented producer fits thirty minutes without an exception', () => {
  assert.equal(producer.timeout, '30');
  const budgets = [...producer.body.matchAll(/TEST_BUDGET_SECONDS: (\d+)/gu)]
    .map(match => Number(match[1]));
  assert.deepEqual(budgets, [1260]);
  assert.ok(budgets.reduce((sum, seconds) => sum + seconds, 0) <= 30 * 60 * 0.7);
  assert.equal(policy.exceptions.some(exception =>
    exception.workflow === '.github/workflows/coverage.yml' &&
    exception.job === 'coverage-build'), false);
});

test('the bounded compile retains instrumentation, every test target and upload guards', () => {
  assert.match(producer.body, /cargo llvm-cov show-env --sh/u);
  assert.match(producer.body, /eval "\$env_script"/u);
  assert.match(producer.body, /cargo test --manifest-path rust\/Cargo.toml --all-features --no-run/u);
  assert.match(producer.body, /--message-format=json-render-diagnostics > coverage-build\.json/u);
  assert.doesNotMatch(producer.body, /cargo test[^\n]*(?:--test |--lib|--bins|--release)/u);
  assert.match(producer.body, /\.profile\.test == true and \.executable != null/u);
  assert.match(producer.body, /wc -l < coverage-tests\.tsv/u);
  assert.match(producer.body, /tar -cf coverage-objects\.tar coverage-tests\.tsv -T coverage-objects\.txt/u);
  assert.match(producer.body, /name: coverage-objects\n/u);
  assert.match(producer.body, /if-no-files-found: error/u);
});
