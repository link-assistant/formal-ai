import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { workflowJobs } from '../../../scripts/lib/ci-speed-workflows.mjs';

const workflow = readFileSync('.github/workflows/coverage.yml', 'utf8');
function checkBootstrap(source) {
  const producer = workflowJobs(source).find(job => job.id === 'browser-coverage-shard');
  assert.ok(producer);
  assert.equal(producer.timeout, '15');
  assert.match(producer.body, /shard: \[1, 2, 3, 4, 5, 6\]/u);
  assert.match(producer.body, /uses: oven-sh\/setup-bun@v2[\s\S]*?bun-version-file: \.bun-version/u);
  const install = producer.body.indexOf('run: bun install --frozen-lockfile --ignore-scripts');
  const measure = producer.body.indexOf('npm run coverage:web -- run');
  assert.ok(install >= 0 && measure > install);
  assert.match(producer.body, /if-no-files-found: error/u);
  assert.match(source, /run: rust-script scripts\/check-coverage-ratchet.rs --only browser/u);
}

test('every browser shard installs the actual frozen dependencies before its complete test inventory', () => {
  checkBootstrap(workflow);
  for (const modified of [
    workflow.replace('run: bun install --frozen-lockfile --ignore-scripts', 'run: true'),
    workflow.replace('bun-version-file: .bun-version', 'bun-version: latest'),
    workflow.replace('shard: [1, 2, 3, 4, 5, 6]', 'shard: [1, 2, 3, 4, 5]'),
  ]) assert.throws(() => checkBootstrap(modified));
});
