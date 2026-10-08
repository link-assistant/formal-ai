#!/usr/bin/env node
// The CI speed gate (PR #1188): keep CI fast to iterate on.
//
// The rule, from `data/meta/ci-speed.lino`:
//
// - no job runs longer than `limit-minutes` (30; 15 is the target): every job
//   declares a `timeout-minutes` at or under the limit, and so does every step,
//   and no job's measured maximum in `data/meta/ci-durations.lino` is above it;
// - long work starts first: a long job (measured median at or above
//   `long-job-minutes`) waits only on jobs whose outputs or artifacts it uses,
//   and test shards are planned longest-first by `scripts/plan-test-shards.mjs`
//   from recorded durations, never dealt out by listed index;
// - nothing is lost by sharding: the plan puts every test in exactly one shard.
//
// A job that cannot meet the limit yet is listed under `over-limit-job` with
// the cap it may keep and why. The list only shrinks: an entry whose job now
// meets the limit, or no longer exists, fails until it is removed.
//
// Usage:
//   node scripts/check-ci-speed.mjs            # check, exit 1 on a violation
//   node scripts/check-ci-speed.mjs --report   # also print each workflow's critical path

import { existsSync, readdirSync, readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

import { parseCiDurations, parseTestDurations } from './lib/ci-speed-durations.mjs';
import { parseSpeedPolicy } from './lib/ci-speed-policy.mjs';
import { durationLookup, isLongestFirst, partitionProblems, planShards } from './lib/ci-speed-shards.mjs';
import { readWorkflows } from './lib/ci-speed-workflows.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '..');

/** Index-order sharding: each deals tests out by position, not by duration. */
export const INDEX_ORDER_SHARDING = [
  /\(NR\s*-\s*1\)\s*%/,
  /\bposition\s*%\s*SHARD_TOTAL\b/,
  /--test-shard[= ]/,
  /--partition[= ](?:slice|count):/,
  // Playwright's `--shard=<i>/<n>` cuts the suite by test count in file order.
  /--shard[= ]"?[$\w{}]+\/[$\w{}]+/,
];

const exceptionFor = (policy, workflow, job) =>
  policy.exceptions.find((entry) => entry.workflow === workflow && entry.job === job);

/** Measured maximum and median per `<workflow>#<job id>`, over every matrix leg. */
export function measuredByJob(durations) {
  const measured = new Map();
  for (const workflow of durations.workflows) {
    for (const job of workflow.jobs) {
      const key = `${workflow.path}#${job.key}`;
      const previous = measured.get(key) ?? { maximum: 0, median: 0, display: job.display };
      measured.set(key, {
        maximum: Math.max(previous.maximum, job.maximum),
        median: Math.max(previous.median, job.median),
        display: job.maximum >= previous.maximum ? job.display : previous.display,
      });
    }
  }
  return measured;
}

/** Every declared cap at or under the limit, unless an exception allows it. */
export function capProblems(workflows, policy) {
  const problems = [];
  for (const workflow of workflows) {
    for (const job of workflow.jobs) {
      if (job.uses) continue;
      const where = `${workflow.path}: job \`${job.id}\``;
      if (job.timeout === null) {
        problems.push(`${where} declares no timeout-minutes, so GitHub's 360-minute default bounds it`);
        continue;
      }
      if (job.timeoutValues.length === 0) {
        problems.push(`${where} sets timeout-minutes to \`${job.timeout}\`, which resolves to no number`);
        continue;
      }
      const cap = Math.max(...job.timeoutValues, ...job.stepTimeouts);
      const exception = exceptionFor(policy, workflow.path, job.id);
      const allowed = exception ? Math.max(exception.timeoutMinutes, policy.limitMinutes) : policy.limitMinutes;
      if (cap > allowed) {
        problems.push(
          `${where} may run ${cap} minutes, above the ${allowed}-minute ${exception ? 'cap its exception records' : 'limit'}; ` +
            'split it into parallel shards or jobs of 15 minutes or less',
        );
      }
    }
  }
  return problems;
}

/** Every measured maximum at or under the limit, unless an exception covers the job. */
export function measuredProblems(durations, policy) {
  const problems = [];
  for (const [key, { maximum, display }] of measuredByJob(durations)) {
    const [workflow, job] = key.split('#');
    if (maximum > policy.limitMinutes && !exceptionFor(policy, workflow, job)) {
      problems.push(
        `${workflow}: job \`${job}\` (${display}) measured ${maximum} minutes, above the ${policy.limitMinutes}-minute limit`,
      );
    }
  }
  return problems;
}

/** Exceptions that no longer excuse anything, so the list only shrinks. */
export function staleExceptionProblems(workflows, durations, policy) {
  const problems = [];
  const measured = measuredByJob(durations);
  for (const exception of policy.exceptions) {
    const where = `${exception.workflow}: over-limit-job \`${exception.job}\``;
    const job = workflows.find((workflow) => workflow.path === exception.workflow)?.jobs.find((each) => each.id === exception.job);
    if (!exception.reason) problems.push(`${where} gives no reason`);
    if (!job) {
      problems.push(`${where} names a job the workflow no longer has; remove the exception`);
      continue;
    }
    const cap = Math.max(0, ...job.timeoutValues, ...job.stepTimeouts);
    const maximum = measured.get(`${exception.workflow}#${exception.job}`)?.maximum ?? 0;
    if (cap <= policy.limitMinutes && maximum <= policy.limitMinutes) {
      problems.push(`${where} now meets the ${policy.limitMinutes}-minute limit (cap ${cap}, measured ${maximum}); remove the exception`);
    }
  }
  return problems;
}

/** A job's text, with the workflow it calls when it is a reusable-workflow call. */
function jobText(root, job) {
  const called = job.uses?.match(/^\.\/(\.github\/workflows\/[\w.-]+\.ya?ml)$/)?.[1];
  return called && existsSync(join(root, called)) ? `${job.body}\n${readFileSync(join(root, called), 'utf8')}` : job.body;
}

/** Whether a job uses what a job it needs produces: an output, a result or an artifact. */
function usesNeed(root, job, needed) {
  const escaped = needed.id.replace(/[.*+?^${}()|[\]\\-]/g, '\\$&');
  if (new RegExp(`needs\\.${escaped}\\.|needs\\[['"]${escaped}['"]\\]`).test(job.body)) return true;
  const downloads = /download-artifact|uses: \.\/\.github\/actions\/download-/.test(jobText(root, job));
  const uploads = /upload-artifact|uses: \.\/\.github\/actions\/upload-/.test(needed.body);
  return downloads && uploads;
}

/** A long job waits only on jobs whose outputs, results or artifacts it uses. */
export function orderProblems(root, workflows, durations, policy) {
  const problems = [];
  const measured = measuredByJob(durations);
  for (const workflow of workflows) {
    const byId = new Map(workflow.jobs.map((job) => [job.id, job]));
    for (const job of workflow.jobs) {
      const median = measured.get(`${workflow.path}#${job.id}`)?.median ?? 0;
      if (median < policy.longJobMinutes) continue;
      for (const need of job.needs) {
        const needed = byId.get(need);
        if (needed && !usesNeed(root, job, needed)) {
          problems.push(
            `${workflow.path}: long job \`${job.id}\` (median ${median} minutes) waits on \`${need}\` but uses none of its ` +
              'outputs, results or artifacts; drop the need so the long job starts first',
          );
        }
      }
    }
  }
  return problems;
}

/** Text without shell and YAML comment lines, which may quote what they replace. */
const withoutComments = (text) => text.split('\n').filter((line) => !/^\s*#/.test(line)).join('\n');

/**
 * Shards are planned longest-first: no workflow or script deals tests out by
 * index, every sharded suite runs through the planner, and the planner splits
 * the recorded tests into sound, longest-first shards.
 */
export function shardingProblems(root, policy, testDurations) {
  const problems = [];
  const sources = [
    ...readdirSync(join(root, '.github/workflows')).filter((file) => /\.ya?ml$/.test(file)).map((file) => `.github/workflows/${file}`),
    ...readdirSync(join(root, 'scripts')).filter((file) => file.endsWith('.sh')).map((file) => `scripts/${file}`),
  ];
  for (const source of sources) {
    const text = withoutComments(readFileSync(join(root, source), 'utf8'));
    for (const pattern of INDEX_ORDER_SHARDING) {
      if (pattern.test(text)) {
        problems.push(`${source} shards tests by listed index (${pattern.source}); plan them with ${policy.shardPlanner} instead`);
      }
    }
  }
  for (const suite of policy.shardedSuites) {
    if (!existsSync(join(root, suite))) {
      problems.push(`sharded-suite ${suite} does not exist`);
    } else if (!withoutComments(readFileSync(join(root, suite), 'utf8')).includes(policy.shardPlanner)) {
      problems.push(`${suite} is a sharded suite that does not plan its shards with ${policy.shardPlanner}`);
    }
  }
  for (const [label, recorded] of testDurations) {
    const tests = [...recorded.keys(), 'unrecorded::first', 'unrecorded::second'];
    const secondsOf = durationLookup(recorded);
    for (let shardCount = 1; shardCount <= 12; shardCount += 1) {
      const { shards } = planShards(tests, shardCount, secondsOf);
      const lost = partitionProblems(tests, shards);
      if (lost.missing.length || lost.repeated.length || lost.extra.length || !isLongestFirst(shards, secondsOf)) {
        problems.push(`${label}: the ${shardCount}-shard plan loses, repeats or misorders tests`);
      }
    }
  }
  return problems;
}

/** Each workflow's longest `needs:` chain by measured median minutes. */
export function criticalPaths(workflows, durations) {
  const measured = measuredByJob(durations);
  return workflows
    .map((workflow) => {
      const byId = new Map(workflow.jobs.map((job) => [job.id, job]));
      const memo = new Map();
      const longest = (id, seen = new Set()) => {
        if (memo.has(id)) return memo.get(id);
        if (seen.has(id) || !byId.has(id)) return { minutes: 0, chain: [] };
        seen.add(id);
        const own = measured.get(`${workflow.path}#${id}`)?.median ?? 0;
        const before = byId
          .get(id)
          .needs.map((need) => longest(need, seen))
          .reduce((best, path) => (path.minutes > best.minutes ? path : best), { minutes: 0, chain: [] });
        const result = { minutes: before.minutes + own, chain: [...before.chain, `${id}(${own})`] };
        memo.set(id, result);
        return result;
      };
      const best = workflow.jobs
        .map((job) => longest(job.id))
        .reduce((left, right) => (right.minutes > left.minutes ? right : left), { minutes: 0, chain: [] });
      const wall = durations.workflows.find((entry) => entry.path === workflow.path);
      return { path: workflow.path, minutes: Math.round(best.minutes * 10) / 10, chain: best.chain, wallMedian: wall?.wallMedian };
    })
    .filter((path) => path.minutes > 0)
    .sort((left, right) => right.minutes - left.minutes);
}

export function checkCiSpeed(root = ROOT) {
  const policy = parseSpeedPolicy(readFileSync(join(root, 'data/meta/ci-speed.lino'), 'utf8'));
  const durations = parseCiDurations(readFileSync(join(root, policy.durations), 'utf8'));
  const workflows = readWorkflows(root);
  const testDurations = [policy.testDurations, policy.javascriptTestDurations, policy.playwrightTestDurations]
    .filter((path) => path && existsSync(join(root, path)))
    .map((path) => [path, parseTestDurations(readFileSync(join(root, path), 'utf8'))]);
  return {
    policy,
    workflows,
    durations,
    problems: [
      ...capProblems(workflows, policy),
      ...measuredProblems(durations, policy),
      ...staleExceptionProblems(workflows, durations, policy),
      ...orderProblems(root, workflows, durations, policy),
      ...shardingProblems(root, policy, testDurations),
    ],
  };
}

function main() {
  const { policy, workflows, durations, problems } = checkCiSpeed();
  if (process.argv.includes('--report')) {
    console.log('Critical path per workflow (measured median minutes along the longest needs chain; wall = run median):');
    for (const path of criticalPaths(workflows, durations)) {
      console.log(`  ${path.minutes}\twall ${path.wallMedian ?? '-'}\t${path.path}: ${path.chain.join(' -> ')}`);
    }
    for (const exception of policy.exceptions) {
      console.log(`  over limit: ${exception.workflow} ${exception.job} (cap ${exception.timeoutMinutes}): ${exception.reason}`);
    }
  }
  for (const problem of problems) console.log(`::error title=CI speed::${problem}`);
  if (problems.length) {
    console.log(`${problems.length} CI speed violation(s); the rule and its exceptions are in data/meta/ci-speed.lino (PR #1188).`);
    process.exit(1);
  }
  const jobs = workflows.reduce((sum, workflow) => sum + workflow.jobs.length, 0);
  console.log(
    `CI speed: ${jobs - policy.exceptions.length} of ${jobs} jobs in ${workflows.length} workflows meet the ` +
      `${policy.limitMinutes}-minute limit and ${policy.exceptions.length} carry a recorded exception; ` +
      'long jobs start first; test shards are planned longest-first.',
  );
}

if (process.argv[1] && pathToFileURL(resolve(process.argv[1])).href === import.meta.url) main();
