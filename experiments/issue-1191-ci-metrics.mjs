#!/usr/bin/env node
// Issue #1191, requirement 12 and acceptance "a comparable before/after
// sample": delivery metrics of one pull request, read from the Actions API.
//
//   node experiments/issue-1191-ci-metrics.mjs 1188 [more PR numbers...] > metrics.json
//
// For each pull request it reports:
//   pushes          distinct head SHAs that started at least one workflow run
//   runs            every workflow run on the branch (pull_request and push events), by conclusion
//   heavy           the same for the pipelines that compile Rust (CI/CD Pipeline, Layered CI, Coverage)
//   jobMinutes      runner minutes of the heavy pipelines (sum of job durations)
//   firstFailureMin median minutes from a push's first run to its first failed run
//   firstGreenHours hours from PR creation to the first head after the first push whose every run is green or skipped
//   mergeHours      hours from PR creation to merge
// Calls are bounded: one paginated run listing per day plus one job listing per heavy run (at most 400).
import {execFile} from 'node:child_process';
import fs from 'node:fs';
import {promisify} from 'node:util';

const REPO = process.env.REPO || 'link-assistant/formal-ai';
const HEAVY = new Set(['CI/CD Pipeline', 'Layered CI', 'Coverage']);
const GREEN = new Set(['success', 'skipped', 'neutral']);
const MAX_JOB_LISTINGS = 400;
const CONCURRENCY = 8;

const run = promisify(execFile);
const gh = async (endpoint, jq) => (await run('gh', ['api', '--paginate', endpoint, '--jq', jq],
  {encoding: 'utf8', timeout: 300_000, maxBuffer: 64 * 1024 * 1024})).stdout.split('\n').filter(Boolean).map(line => JSON.parse(line));
/** Runs `task` over `items` with at most CONCURRENCY calls in flight. */
async function pool(items, task) {
  const results = new Array(items.length);
  let next = 0;
  await Promise.all(Array.from({length: Math.min(CONCURRENCY, items.length)}, async () => {
    while (next < items.length) { const index = next++; results[index] = await task(items[index]); }
  }));
  return results;
}
const minutes = (from, to) => (Date.parse(to) - Date.parse(from)) / 60_000;
const median = values => {
  if (values.length === 0) return null;
  const sorted = [...values].sort((a, b) => a - b), middle = Math.floor(sorted.length / 2);
  return sorted.length % 2 ? sorted[middle] : (sorted[middle - 1] + sorted[middle]) / 2;
};
const tally = runs => runs.reduce((counts, run) => {
  const key = run.status === 'completed' ? run.conclusion : run.status;
  counts[key] = (counts[key] ?? 0) + 1;
  return counts;
}, {total: runs.length});
const round = value => value === null ? null : Math.round(value * 10) / 10;

async function metrics(number) {
  const [pull] = await gh(`repos/${REPO}/pulls/${number}`, '{number, title, created_at, merged_at, closed_at, head: .head.ref, commits, additions, deletions}');
  const until = pull.merged_at ?? pull.closed_at ?? new Date().toISOString();
  // The runs listing returns at most 1000 results per query, so query one day at
  // a time, and split a day that reaches the cap into smaller windows.
  const list = range => gh(`repos/${REPO}/actions/runs?branch=${encodeURIComponent(pull.head)}&created=${encodeURIComponent(range)}&per_page=100`,
    '.workflow_runs[] | {id, name, event, status, conclusion, head_sha, created_at, updated_at}');
  const window = async (from, hours) => {
    const to = new Date(from + hours * 3_600_000 - 1000).toISOString().slice(0, 19) + 'Z';
    const batch = await list(`${new Date(from).toISOString().slice(0, 19)}Z..${to}`);
    if (batch.length < 1000) return batch;
    if (hours <= 1) { process.stderr.write(`#${number} ${to}: 1000 runs in one hour, the API cap; counts are a lower bound\n`); return batch; }
    const half = hours / 2;
    return [...await window(from, half), ...await window(from + half * 3_600_000, half)];
  };
  const days = [];
  for (let day = Date.parse(pull.created_at.slice(0, 10)); day <= Date.parse(until); day += 86_400_000) days.push(day);
  const runs = (await pool(days, day => window(day, 24))).flat()
    .filter(run => ['pull_request', 'push'].includes(run.event) && run.created_at >= pull.created_at && run.created_at <= until);
  const heads = new Map();
  for (const run of [...runs].sort((a, b) => a.created_at.localeCompare(b.created_at))) {
    if (!heads.has(run.head_sha)) heads.set(run.head_sha, []);
    heads.get(run.head_sha).push(run);
  }
  const heavy = runs.filter(run => HEAVY.has(run.name));
  const listedRuns = heavy.slice(0, MAX_JOB_LISTINGS), listed = listedRuns.length;
  const jobs = (await pool(listedRuns, run => gh(`repos/${REPO}/actions/runs/${run.id}/jobs?filter=all&per_page=100`,
    '.jobs[] | {started_at, completed_at, conclusion}'))).flat();
  let jobMinutes = 0;
  for (const job of jobs) {
    if (job.started_at && job.completed_at && job.conclusion !== 'skipped') jobMinutes += Math.max(0, minutes(job.started_at, job.completed_at));
  }
  const firstFailure = [], firstGreen = [];
  // The first head is the solver's placeholder commit (task details only), green by construction, so it is not counted.
  for (const [sha, group] of [...heads].slice(1)) {
    const start = group[0].created_at;
    const failed = group.filter(run => run.conclusion === 'failure').map(run => minutes(start, run.updated_at));
    if (failed.length) firstFailure.push(Math.min(...failed));
    const completed = group.every(run => run.status === 'completed');
    if (completed && group.some(run => run.name === 'CI/CD Pipeline') && group.every(run => GREEN.has(run.conclusion)))
      firstGreen.push({sha, at: group.reduce((latest, run) => run.updated_at > latest ? run.updated_at : latest, start)});
  }
  const green = firstGreen.sort((a, b) => a.at.localeCompare(b.at))[0];
  return {
    number: pull.number, title: pull.title, createdAt: pull.created_at, mergedAt: pull.merged_at,
    commits: pull.commits, additions: pull.additions, deletions: pull.deletions,
    pushes: heads.size, runs: tally(runs), heavy: tally(heavy),
    pipeline: tally(runs.filter(run => run.name === 'CI/CD Pipeline')),
    jobMinutes: Math.round(jobMinutes), jobListings: `${listed}/${heavy.length}`,
    runsPerPush: round(runs.length / Math.max(1, heads.size)),
    firstFailureMin: round(median(firstFailure)),
    firstGreenHours: green ? round(minutes(pull.created_at, green.at) / 60) : null,
    mergeHours: pull.merged_at ? round(minutes(pull.created_at, pull.merged_at) / 60) : null,
  };
}

const numbers = process.argv.slice(2).map(Number).filter(Number.isSafeInteger);
if (numbers.length === 0) throw new Error('usage: issue-1191-ci-metrics.mjs PR [PR...]');
// METRICS_CACHE=dir keeps one file per pull request, so an interrupted run resumes.
const cache = process.env.METRICS_CACHE;
if (cache) fs.mkdirSync(cache, {recursive: true});
const results = [];
for (const number of numbers) {
  const file = cache && `${cache}/${number}.json`;
  if (file && fs.existsSync(file)) { results.push(JSON.parse(fs.readFileSync(file, 'utf8'))); continue; }
  let result;
  try { result = await metrics(number); } catch (error) { result = {number, error: String(error.message).slice(0, 300)}; }
  if (file && !result.error) fs.writeFileSync(file, JSON.stringify(result) + '\n');
  results.push(result);
  process.stderr.write(`collected #${number}\n`);
}
process.stdout.write(JSON.stringify(results, null, 2) + '\n');
