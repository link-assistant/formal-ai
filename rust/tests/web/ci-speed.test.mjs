// PR #1188 (CI-SPEED): CI stays fast to iterate on, and that is enforced.
//
// The rule (data/meta/ci-speed.lino, docs/ci-cd/speed.md): no job or step runs
// longer than 30 minutes, long jobs wait only on what they use, and test
// shards are planned longest-first from recorded durations with every test in
// exactly one shard. These cases pin the pieces of scripts/check-ci-speed.mjs
// and scripts/plan-test-shards.mjs on small inputs, then the real tree.

import { describe, test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

import {
  INDEX_ORDER_SHARDING, capProblems, checkCiSpeed, criticalPaths, measuredProblems, orderProblems, staleExceptionProblems,
} from '../../../scripts/check-ci-speed.mjs';
import { parseCiDurations, parseTestDurations, renderCiDurations } from '../../../scripts/lib/ci-speed-durations.mjs';
import { parseSpeedPolicy } from '../../../scripts/lib/ci-speed-policy.mjs';
import {
  durationLookup, isLongestFirst, parseReserved, partitionProblems, planShards,
} from '../../../scripts/lib/ci-speed-shards.mjs';
import { jobIdForDisplayName, workflowJobs, workflowTriggers } from '../../../scripts/lib/ci-speed-workflows.mjs';
import { parseReportTime } from '../../../experiments/formal_ai_subagent/ci-durations.mjs';

const ROOT = join(dirname(fileURLToPath(import.meta.url)), '../../..');

const WORKFLOW = `name: Example
on: # a comment
  pull_request:
  push:
jobs:
  detect:
    name: Detect
    timeout-minutes: 5
  lint:
    name: 'Lint (issue #920)'
    timeout-minutes: 10
  build:
    needs: detect
    timeout-minutes: 20
    steps:
      - uses: actions/upload-artifact@v7
  test:
    name: Test \${{ matrix.part }}
    needs: [detect, lint,
      build]
    timeout-minutes: \${{ matrix.cap }}
    strategy:
      matrix:
        include:
          - { part: one, cap: 25 }
          - { part: two, cap: 45 }
    steps:
      - uses: actions/download-artifact@v8
      - run: echo "\${{ needs.detect.outputs.changed }}"
  call:
    needs:
      - build
    uses: ./.github/workflows/other.yml
  free:
    runs-on: ubuntu-latest
`;

const POLICY = parseSpeedPolicy(`ci-speed
  limit-minutes 30
  long-job-minutes 10
  workflow "example.yml"
    over-limit-job test
      timeout-minutes 45
      reason "Two parts, one of them long."
`);

const workflowOf = (text = WORKFLOW) => ({ path: 'example.yml', name: 'Example', triggers: workflowTriggers(text), jobs: workflowJobs(text) });
const durationsOf = (rows) => ({ workflows: [{ path: 'example.yml', jobs: rows }] });

describe('the shard plan', () => {
  const seconds = new Map([['a::slow', 300], ['a::medium', 120], ['b::medium', 110], ['b::quick', 5]]);
  const tests = [...seconds.keys(), 'c::new-one', 'c::new-two'];
  const secondsOf = durationLookup(seconds);

  test('puts every test in exactly one shard, longest first', () => {
    for (let shardCount = 1; shardCount <= 7; shardCount += 1) {
      const { shards } = planShards(tests, shardCount, secondsOf);
      assert.deepEqual(partitionProblems(tests, shards), { missing: [], repeated: [], extra: [] });
      assert.ok(isLongestFirst(shards, secondsOf));
    }
  });

  test('gives the slowest test a shard of its own and balances the rest', () => {
    const { shards, load } = planShards(tests, 2, secondsOf);
    assert.deepEqual(shards[0], ['a::slow']);
    assert.ok(Math.max(...load) <= 300 + 1e-9);
  });

  test('is the same on every machine whatever the input order', () => {
    const forward = planShards(tests, 3, secondsOf).shards;
    const backward = planShards([...tests].reverse(), 3, secondsOf).shards;
    assert.deepEqual(forward, backward);
  });

  test('gives a shard with other work fewer tests', () => {
    const { shards } = planShards(tests, 2, secondsOf, parseReserved('1=400', 2));
    assert.ok(!shards[0].includes('a::slow'));
  });

  test('reads `<group>\\t<test>` lines by the test name', () => {
    assert.equal(durationLookup(seconds)('unit\ta::slow'), 300);
  });

  test('reports what a broken partition loses or repeats', () => {
    assert.deepEqual(partitionProblems(['x', 'y', 'z'], [['x', 'x'], ['w']]), {
      missing: ['y', 'z'], repeated: ['x'], extra: ['w'],
    });
  });

  test('the command line prints each shard and checks the partition', () => {
    const input = `${tests.join('\n')}\n`;
    const printed = [1, 2, 3].flatMap((shard) => spawnSync(process.execPath, [
      join(ROOT, 'scripts/plan-test-shards.mjs'), '--shard', String(shard), '--of', '3',
    ], { input, encoding: 'utf8' }).stdout.split('\n').filter(Boolean));
    assert.deepEqual([...printed].sort(), [...tests].sort());
    const check = spawnSync(process.execPath, [join(ROOT, 'scripts/plan-test-shards.mjs'), '--of', '3', '--check'], { input, encoding: 'utf8' });
    assert.equal(check.status, 0, check.stderr);
  });
});

describe('the workflow reader', () => {
  const jobs = new Map(workflowJobs(WORKFLOW).map((job) => [job.id, job]));

  test('reads names, needs in every shape, and matrix timeouts', () => {
    assert.equal(jobs.get('lint').name, 'Lint (issue #920)');
    assert.deepEqual(jobs.get('build').needs, ['detect']);
    assert.deepEqual(jobs.get('test').needs, ['detect', 'lint', 'build']);
    assert.deepEqual(jobs.get('call').needs, ['build']);
    assert.deepEqual(jobs.get('test').timeoutValues, [25, 45]);
    assert.equal(jobs.get('call').uses, './.github/workflows/other.yml');
    assert.deepEqual(workflowTriggers(WORKFLOW), ['pull_request', 'push']);
  });

  test('attributes a measured display name to its job id', () => {
    const workflow = workflowOf();
    assert.equal(jobIdForDisplayName(workflow, 'Test two'), 'test');
    assert.equal(jobIdForDisplayName(workflow, 'Lint (issue #920)'), 'lint');
    assert.equal(jobIdForDisplayName(workflow, 'call / Inner job'), 'call');
    assert.equal(jobIdForDisplayName(workflow, 'Removed job'), null);
  });
});

describe('the speed rule', () => {
  test('flags a missing cap and a cap over the limit, and honours a recorded exception', () => {
    const problems = capProblems([workflowOf()], POLICY);
    assert.equal(problems.length, 1);
    assert.match(problems[0], /job `free` declares no timeout-minutes/);
    const strict = capProblems([workflowOf()], { ...POLICY, exceptions: [] });
    assert.ok(strict.some((problem) => /job `test` may run 45 minutes/.test(problem)));
  });

  test('flags a measured maximum over the limit without an exception', () => {
    const durations = durationsOf([{ display: 'Build', key: 'build', median: 20, maximum: 31 }]);
    assert.match(measuredProblems(durations, POLICY)[0], /job `build` \(Build\) measured 31 minutes/);
  });

  test('an exception that excuses nothing any more fails until removed', () => {
    const fixed = WORKFLOW.replace('{ part: two, cap: 45 }', '{ part: two, cap: 30 }');
    const problems = staleExceptionProblems([workflowOf(fixed)], durationsOf([]), POLICY);
    assert.match(problems[0], /now meets the 30-minute limit/);
  });

  test('a long job may wait only on jobs whose outputs or artifacts it uses', () => {
    const durations = durationsOf([{ display: 'Test one', key: 'test', median: 12, maximum: 14 }]);
    const problems = orderProblems(ROOT, [workflowOf()], durations, POLICY);
    assert.equal(problems.length, 1);
    assert.match(problems[0], /long job `test` .* waits on `lint`/);
  });

  test('index-order sharding is recognised in each spelling', () => {
    for (const text of [
      `awk '(NR - 1) % n == i - 1'`,
      'if [ $((position % SHARD_TOTAL)) -eq 0 ]',
      'node --test --test-shard="$SHARD/$SHARDS" a.test.mjs',
      'cargo nextest run --partition slice:1/4',
      'npx playwright test --config=playwright.local.config.js --shard="${SHARD}/3"',
      'npx playwright test --shard=2/3',
    ]) {
      assert.ok(INDEX_ORDER_SHARDING.some((pattern) => pattern.test(text)), text);
    }
    // The planner's own flags are not index-order sharding.
    const planned = 'node scripts/plan-test-shards.mjs --shard "$SHARD" --of 3 < specs';
    assert.ok(!INDEX_ORDER_SHARDING.some((pattern) => pattern.test(planned)), planned);
  });

  test('the critical path follows the longest needs chain by median', () => {
    const durations = durationsOf([
      { display: 'Detect', key: 'detect', median: 1, maximum: 1 },
      { display: 'Build', key: 'build', median: 10, maximum: 11 },
      { display: 'Test one', key: 'test', median: 8, maximum: 9 },
    ]);
    const [path] = criticalPaths([workflowOf()], durations);
    assert.equal(path.minutes, 19);
    assert.deepEqual(path.chain, ['detect(1)', 'build(10)', 'test(8)']);
  });
});

describe('the measured records', () => {
  test('ci-durations round-trips through its renderer', () => {
    const workflows = [{
      path: '.github/workflows/x.yml', name: 'X', runs: 3, wallMedian: 12.5, wallMaximum: 20,
      jobs: [{ display: 'Test (a / b)', key: 'test', median: 9.5, maximum: 11, samples: 3 }],
    }];
    const text = renderCiDurations({ comment: ['A record.'], header: { issue: 1188, branch: 'b' }, workflows });
    const parsed = parseCiDurations(text);
    assert.equal(parsed.header.branch, 'b');
    assert.deepEqual(parsed.workflows[0].jobs[0], workflows[0].jobs[0]);
    assert.equal(parsed.workflows[0].wallMaximum, 20);
  });

  test('test durations read quoted names; report-time lines become seconds', () => {
    const recorded = parseTestDurations('test_durations\n  test "a::b"\n    seconds 2.5\n');
    assert.equal(recorded.get('a::b'), 2.5);
    const log = '2026-10-08T09:00:00Z test a::b ... ok <1.250s>\ntest c::d ... FAILED <0.500s>\ntest e ... ok\n';
    assert.deepEqual([...parseReportTime(log)], [['a::b', 1.25], ['c::d', 0.5]]);
  });
});

test('the repository meets its own CI speed rule', () => {
  const { problems, policy } = checkCiSpeed(ROOT);
  assert.deepEqual(problems, []);
  assert.equal(policy.limitMinutes, 30);
  for (const exception of policy.exceptions) {
    assert.ok(exception.reason.length > 40, `${exception.workflow} ${exception.job} explains itself`);
  }
});
