import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync, readdirSync, mkdtempSync, rmSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import YAML from 'yaml';
const fixture = new URL('./fixtures/hosted-javascript-5ce44145/', import.meta.url);
const read = file => readFileSync(new URL(file, fixture), 'utf8');
const hash = bytes => createHash('sha256').update(bytes).digest('hex');

test('the terminal 5ce cohort retains every observed owner and both budget warnings', () => {
  const identity = JSON.parse(read('identity.json'));
  const run = JSON.parse(read('run-observation.json'));
  const raw = read('durations.jsonl'), rows = raw.trim().split('\n').map(JSON.parse);
  assert.equal(run.id, 45262786254);
  assert.equal(run.head_sha, identity.sourceCommit);
  assert.equal(run.conclusion, 'success');
  assert.equal(identity.testedMergeCommit, 'accd42df3b138d547c2d39667cc444ef21e65d70');
  assert.equal(identity.completeObservations, 922);
  assert.equal(rows.length, 922);
  assert.deepEqual(rows.map(row => row.file).sort(), readdirSync('rust/tests/web')
    .filter(name => name.endsWith('.test.mjs')).map(name => 'rust/tests/web/' + name).sort());
  assert(rows.every(row => row.complete && row.result === 'completed'
    && identity.jobs.some(job => job.job === row.job && job.conclusion === 'success')));
  assert.deepEqual(identity.warnings.map(row => row.shard), [2, 9]);
  assert.equal(hash(raw), identity.durationsSha256);
  const prior = read('prior-scheduling-floor.lino');
  assert.equal(hash(prior), identity.priorSchedulingFloorSha256);
  assert.equal(prior, readFileSync('data/meta/javascript-test-durations.lino', 'utf8'),
    'capacity repair must preserve every existing scheduling weight and fallback');
  assert.equal(identity.executionProof, false);
});

test('the smallest finite capacity repair fits all retained observations without extending a budget', {timeout: 15000}, () => {
  const directory = mkdtempSync(join(tmpdir(), 'javascript-capacity-'));
  try {
    const output = join(directory, 'report.json');
    const child = spawnSync(process.execPath,
      ['--max-old-space-size=128', 'experiments/issue-1580/probe-hosted-javascript-capacity.mjs', output],
      {encoding: 'utf8', timeout: 10000, maxBuffer: 65536});
    assert.equal(child.error, undefined);
    assert.equal(child.status, 0, child.stderr);
    const report = JSON.parse(readFileSync(output, 'utf8'));
    assert.equal(report.candidateLimit, 8);
    assert.equal(report.candidates.length, 8);
    assert.equal(report.sourceOwners, 922);
    assert.equal(report.executionProof, false);
    const job = YAML.parse(readFileSync('.github/workflows/layered-ci.yml', 'utf8')).jobs.js;
    assert.deepEqual(job.strategy.matrix.shard, Array.from({length: report.selectedCount}, (_, index) => index + 1));
    assert.equal(report.selectedCount, 10);
    assert.equal(report.selectedCount, Math.min(...report.candidates.filter(row => row.fits).map(row => row.count)));
    for (const step of job.steps.filter(step => step.env?.SHARDS !== undefined))
      assert.equal(step.env.SHARDS, report.selectedCount);
    const execute = job.steps.find(step => step.name === 'Run this shard of the JavaScript suites');
    assert.equal(execute.env.TEST_BUDGET_SECONDS, 360);
    assert.equal(execute.env.TEST_WARN_RATIO_PERCENT, 75);
    const before = report.candidates.find(row => row.count === 9);
    assert.equal(before.fits, false, 'negative control must reproduce the warning condition');
    const old = before.results.find(row => row.name.endsWith('5ce44145'));
    assert.deepEqual(old.loads.map((seconds, index) => seconds > 270 ? index + 1 : null).filter(Boolean), [2, 9]);
    const after = report.candidates.find(row => row.count === report.selectedCount);
    assert.equal(after.results.length, 13);
    assert(Object.values(after.constraints).every(Boolean));
    assert(after.conservativeQueues.every(seconds => seconds < 360));
    assert(after.results.every(row => row.maximumSeconds <= 270));
  } finally {
    rmSync(directory, {recursive: true, force: true});
  }
});
