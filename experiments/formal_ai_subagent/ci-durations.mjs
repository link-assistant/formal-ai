#!/usr/bin/env node
// Measure how long CI jobs and tests take, from GitHub Actions, with `gh`.
//
// The CI speed rule (PR #1188, `data/meta/ci-speed.lino`): no job runs longer
// than 30 minutes, 15 is the target, and the long work starts first. The gate
// `scripts/check-ci-speed.mjs` reads what this script records:
//
// - jobs (default): every workflow's recent completed runs on a branch, per
//   job the median and maximum minutes, written to `data/meta/ci-durations.lino`;
// - `--tests`: per Rust test, the seconds the latest test jobs printed with
//   libtest's `--report-time`, written to `data/meta/test-durations.lino`,
//   which the shard planner reads to put the longest tests first;
// - `--javascript-files`: per suite file under rust/tests/web/, the seconds it
//   takes alone on this machine, written to `data/meta/javascript-test-durations.lino`
//   for the js tier's shard plan.
//
// Usage:
//   node experiments/formal_ai_subagent/ci-durations.mjs [--branch B] [--runs N] [--write]
//   node experiments/formal_ai_subagent/ci-durations.mjs --tests [--run ID ...] [--write]
//   node experiments/formal_ai_subagent/ci-durations.mjs --javascript-files [--write]
//
// Without `--write` it prints what it would write. It only reads from GitHub.
// Cancelled runs and skipped jobs are left out: they say nothing about how
// long the work takes. A job whose display name no current workflow job
// matches is a renamed or removed job, and is left out too.

import { execFileSync, spawnSync } from 'node:child_process';
import { readdirSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { median, renderCiDurations, roundMinutes } from '../../scripts/lib/ci-speed-durations.mjs';
import { jobIdForDisplayName, readWorkflows } from '../../scripts/lib/ci-speed-workflows.mjs';

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), '../..');
const JOBS_OUTPUT = join(ROOT, 'data/meta/ci-durations.lino');
const TESTS_OUTPUT = join(ROOT, 'data/meta/test-durations.lino');
const JAVASCRIPT_OUTPUT = join(ROOT, 'data/meta/javascript-test-durations.lino');
// Tests faster than this are noise for scheduling; their mean is recorded once
// as `default-seconds` instead, so the planner still weighs them.
const TEST_FLOOR_SECONDS = 1;
// Keeps the record under the 1500-line cap `data_files::` holds every file to.
const MAXIMUM_RECORDED_TESTS = 700;

const argv = process.argv.slice(2);
const option = (flag, fallback) => {
  const index = argv.indexOf(flag);
  return index >= 0 ? argv[index + 1] : fallback;
};
const options = (flag) => argv.flatMap((value, index) => (argv[index - 1] === flag ? [value] : []));

function gh(args) {
  return execFileSync('gh', args, { cwd: ROOT, encoding: 'utf8', maxBuffer: 1 << 28 });
}
const api = (path) => JSON.parse(gh(['api', path]));
const minutesBetween = (start, end) => (Date.parse(end) - Date.parse(start)) / 60000;

function currentBranch() {
  return execFileSync('git', ['rev-parse', '--abbrev-ref', 'HEAD'], { cwd: ROOT, encoding: 'utf8' }).trim();
}

/** The latest `count` completed, not cancelled, runs of every workflow on a branch. */
function recentRuns(branch, count) {
  const byPath = new Map();
  for (let page = 1; page <= 10; page += 1) {
    const { workflow_runs: runs } = api(
      `repos/{owner}/{repo}/actions/runs?branch=${encodeURIComponent(branch)}&status=completed&per_page=100&page=${page}`,
    );
    for (const run of runs) {
      if (run.conclusion === 'cancelled' || run.conclusion === 'skipped') continue;
      const list = byPath.get(run.path) ?? [];
      if (list.length < count) list.push(run);
      byPath.set(run.path, list);
    }
    if (runs.length < 100) break;
  }
  return byPath;
}

function runJobs(runId) {
  const jobs = [];
  for (let page = 1; page <= 5; page += 1) {
    const batch = api(`repos/{owner}/{repo}/actions/runs/${runId}/jobs?filter=latest&per_page=100&page=${page}`).jobs;
    jobs.push(...batch);
    if (batch.length < 100) break;
  }
  return jobs;
}

function measureJobs() {
  const branch = option('--branch', currentBranch());
  const count = Number(option('--runs', '10'));
  const workflows = new Map(readWorkflows(ROOT).map((workflow) => [workflow.path, workflow]));
  const measured = [];
  for (const [path, runs] of [...recentRuns(branch, count)].sort(([left], [right]) => left.localeCompare(right))) {
    const workflow = workflows.get(path);
    if (!workflow) continue;
    const samples = new Map();
    const walls = [];
    for (const run of runs) {
      walls.push(minutesBetween(run.run_started_at ?? run.created_at, run.updated_at));
      for (const job of runJobs(run.id)) {
        if (!job.started_at || !job.completed_at) continue;
        if (job.conclusion === 'skipped' || job.conclusion === 'cancelled') continue;
        const key = jobIdForDisplayName(workflow, job.name);
        if (!key) continue;
        const entry = samples.get(job.name) ?? { key, minutes: [] };
        entry.minutes.push(minutesBetween(job.started_at, job.completed_at));
        samples.set(job.name, entry);
      }
    }
    if (samples.size === 0) continue;
    measured.push({
      path,
      name: workflow.name,
      runs: runs.length,
      wallMedian: roundMinutes(median(walls)),
      wallMaximum: roundMinutes(Math.max(...walls)),
      jobs: [...samples]
        .map(([display, { key, minutes }]) => ({
          display,
          key,
          median: roundMinutes(median(minutes)),
          maximum: roundMinutes(Math.max(...minutes)),
          samples: minutes.length,
        }))
        .sort((left, right) => right.maximum - left.maximum || left.display.localeCompare(right.display)),
    });
    process.stderr.write(`measured ${path}: ${runs.length} run(s), ${samples.size} job(s)\n`);
  }
  return renderCiDurations({
    comment: [
      'Measured GitHub Actions job durations, the input of the CI speed gate',
      '(`scripts/check-ci-speed.mjs`, `data/meta/ci-speed.lino`, PR #1188).',
      'Generated; do not edit by hand. Per workflow: the run wall-clock (start to',
      'finish, queueing included) and per job display name the median and maximum',
      'minutes from start to completion over the sampled runs. `key` is the job id',
      'in the workflow file, or the calling job for a reusable workflow.',
    ],
    header: {
      issue: 1188,
      branch,
      'measured-date': new Date().toISOString().slice(0, 10),
      'runs-per-workflow': count,
      unit: 'minutes',
      regenerate: 'node experiments/formal_ai_subagent/ci-durations.mjs --write',
    },
    workflows: measured,
  });
}

/** Seconds per test from libtest `--report-time` lines: `test name ... ok <1.234s>`. */
export function parseReportTime(log) {
  const seconds = new Map();
  for (const match of log.matchAll(/test (\S+) \.\.\. (?:ok|FAILED|ignored) <(\d+(?:\.\d+)?)s>/g)) {
    seconds.set(match[1], Math.max(seconds.get(match[1]) ?? 0, Number(match[2])));
  }
  return seconds;
}

function measureTests() {
  const branch = option('--branch', currentBranch());
  let runIds = options('--run');
  if (runIds.length === 0) {
    const runs = recentRuns(branch, 1).get('.github/workflows/release.yml') ?? [];
    runIds = runs.map((run) => String(run.id));
  }
  const seconds = new Map();
  for (const runId of runIds) {
    for (const job of runJobs(runId)) {
      if (!/^Test \(/.test(job.name) || job.conclusion === 'skipped') continue;
      let log = '';
      try {
        log = gh(['api', `repos/{owner}/{repo}/actions/jobs/${job.id}/logs`]);
      } catch {
        continue;
      }
      for (const [name, value] of parseReportTime(log)) seconds.set(name, Math.max(seconds.get(name) ?? 0, value));
    }
  }
  if (seconds.size === 0) {
    throw new Error(`no --report-time lines in the test jobs of run(s) ${runIds.join(', ')}`);
  }
  const all = [...seconds].sort(([leftName, left], [rightName, right]) => right - left || leftName.localeCompare(rightName));
  const recorded = all.filter(([, value]) => value >= TEST_FLOOR_SECONDS).slice(0, MAXIMUM_RECORDED_TESTS);
  const rest = all.slice(recorded.length);
  const defaultSeconds = rest.length ? rest.reduce((sum, [, value]) => sum + value, 0) / rest.length : 0.1;
  const lines = [
    'test_durations',
    '  issue 1188',
    '  purpose "Assign parallel test shards longest-first, so a long test never lands last on a machine everything else has finished waiting for."',
    `  source_run ${runIds.join(' ')}`,
    `  recorded ${new Date().toISOString().slice(0, 10)}`,
    '  unit seconds',
    `  floor "Only tests at or above ${TEST_FLOOR_SECONDS} second are recorded, at most ${MAXIMUM_RECORDED_TESTS}; the rest weigh default-seconds each."`,
    `  default-seconds ${Math.round(defaultSeconds * 1000) / 1000}`,
    '  regenerate "node experiments/formal_ai_subagent/ci-durations.mjs --tests --write"',
  ];
  for (const [name, value] of recorded) {
    lines.push(`  test "${name}"`, `    seconds ${Math.round(value * 10) / 10}`);
  }
  return `${lines.join('\n')}\n`;
}

/**
 * Seconds per JavaScript suite file, measured here by running each file alone
 * (`node --test <file>`), one at a time. Absolute seconds differ from a CI
 * runner's; the order, which is all the shard plan uses, carries over.
 */
function measureJavaScriptFiles() {
  const directory = 'rust/tests/web';
  const files = readdirSync(join(ROOT, directory)).filter((file) => file.endsWith('.test.mjs')).sort();
  const seconds = [];
  for (const file of files) {
    const path = `${directory}/${file}`;
    const started = performance.now();
    spawnSync(process.execPath, ['--test', '--test-concurrency=1', path], { cwd: ROOT, stdio: 'ignore', timeout: 600_000 });
    seconds.push([path, (performance.now() - started) / 1000]);
    process.stderr.write(`${path}: ${seconds.at(-1)[1].toFixed(1)}s\n`);
  }
  seconds.sort(([leftName, left], [rightName, right]) => right - left || leftName.localeCompare(rightName));
  const lines = [
    '# Seconds per JavaScript suite file under rust/tests/web/, the input of the',
    '# longest-first shard plan of the Layered CI js tier (PR #1188). Generated;',
    '# measured by running each file alone, so the order carries over to CI even',
    '# where the absolute seconds do not.',
    'javascript-test-durations',
    '  issue 1188',
    `  recorded ${new Date().toISOString().slice(0, 10)}`,
    '  unit seconds',
    '  regenerate "node experiments/formal_ai_subagent/ci-durations.mjs --javascript-files --write"',
  ];
  for (const [path, value] of seconds) lines.push(`  test "${path}"`, `    seconds ${Math.round(value * 10) / 10}`);
  return `${lines.join('\n')}\n`;
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  const mode = argv.includes('--tests') ? 'tests' : argv.includes('--javascript-files') ? 'javascript' : 'jobs';
  const output = { tests: TESTS_OUTPUT, javascript: JAVASCRIPT_OUTPUT, jobs: JOBS_OUTPUT }[mode];
  const text = { tests: measureTests, javascript: measureJavaScriptFiles, jobs: measureJobs }[mode]();
  if (argv.includes('--write')) {
    writeFileSync(output, text);
    process.stderr.write(`wrote ${output}\n`);
  } else {
    process.stdout.write(text);
  }
}
