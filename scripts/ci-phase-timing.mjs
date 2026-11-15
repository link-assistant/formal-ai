#!/usr/bin/env node
// Issue #1486: measure Hive draft and delivery runs per phase and alert on a
// sustained regression against data/benchmarks/ci-phase-timing/slo.json.
//
//   measure --run ID [--repo OWNER/NAME] [--out FILE]
//   measure --fixture PREFIX [--out FILE]      offline: PREFIX-{run,jobs,artifacts}.json
//   check   [--runs N] [--slo FILE] [--out FILE] [--fixtures DIR]
//           measures the latest N (default 30) succeeded or failed default-branch runs of every
//           workflow the objectives name; exits 1 on a sustained breach
//
// Writes a Markdown table to GITHUB_STEP_SUMMARY when it is set.
import { appendFileSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { evaluateSlo, measureRun, summaryMarkdown } from './lib/ci-phase-timing.mjs';

const DEFAULT_SLO = fileURLToPath(new URL('../data/benchmarks/ci-phase-timing/slo.json', import.meta.url));
const option = (argv, name, fallback) => {
  const index = argv.indexOf(name);
  return index >= 0 ? argv[index + 1] : fallback;
};
const readJson = path => JSON.parse(readFileSync(path, 'utf8'));

export function githubApi(env) {
  const base = env.GITHUB_API_URL ?? 'https://api.github.com';
  const token = env.GH_TOKEN ?? env.GITHUB_TOKEN;
  return async path => {
    const response = await fetch(base + path, {
      headers: { accept: 'application/vnd.github+json', authorization: 'Bearer ' + token, 'x-github-api-version': '2022-11-28' },
      signal: AbortSignal.timeout(30000),
    });
    if (!response.ok) throw Error('GitHub API ' + response.status + ' for ' + path);
    return response.json();
  };
}

export async function fetchRun(api, repository, id) {
  const run = await api('/repos/' + repository + '/actions/runs/' + id);
  const jobs = [];
  for (let page = 1; ; page += 1) {
    const batch = await api('/repos/' + repository + '/actions/runs/' + id + '/jobs?filter=latest&per_page=100&page=' + page);
    jobs.push(...batch.jobs);
    if (batch.jobs.length < 100) break;
  }
  const artifacts = (await api('/repos/' + repository + '/actions/runs/' + id + '/artifacts?per_page=100')).artifacts;
  return { run, jobs, artifacts };
}

const fixture = prefix => ({ run: readJson(prefix + '-run.json'), jobs: readJson(prefix + '-jobs.json').jobs, artifacts: readJson(prefix + '-artifacts.json').artifacts });

function emit(env, argv, value, markdown) {
  const out = option(argv, '--out');
  if (out) writeFileSync(out, JSON.stringify(value, null, 2) + '\n');
  if (env.GITHUB_STEP_SUMMARY) appendFileSync(env.GITHUB_STEP_SUMMARY, markdown);
  process.stdout.write(markdown);
}

export async function main(argv = process.argv.slice(2), env = process.env, api = githubApi(env)) {
  const repository = option(argv, '--repo', env.GITHUB_REPOSITORY);
  if (argv[0] === 'measure') {
    const prefix = option(argv, '--fixture');
    const record = measureRun(prefix ? fixture(prefix) : await fetchRun(api, repository, option(argv, '--run')));
    emit(env, argv, record, summaryMarkdown([record]));
    return 0;
  }
  if (argv[0] === 'check') {
    const slo = readJson(option(argv, '--slo', DEFAULT_SLO));
    const limit = Number(option(argv, '--runs', '30'));
    if (!(Number.isInteger(limit) && limit > 0 && limit <= 100)) throw Error('CiPhaseRuns: ' + limit);
    const records = [];
    const fixtures = option(argv, '--fixtures');
    if (fixtures) {
      for (const name of readdirSync(fixtures).filter(file => file.endsWith('-run.json'))) records.push(measureRun(fixture(join(fixtures, name.slice(0, -'-run.json'.length)))));
    } else {
      for (const file of [...new Set(slo.objectives.map(objective => objective.workflowFile))]) {
        // Most delivery runs skip every job; only runs that executed are sampled.
        const runs = [];
        for (const status of ['success', 'failure']) runs.push(...(await api('/repos/' + repository + '/actions/workflows/' + file + '/runs?branch=main&status=' + status + '&per_page=' + limit)).workflow_runs);
        runs.sort((left, right) => Date.parse(right.created_at) - Date.parse(left.created_at));
        for (const run of runs.slice(0, limit)) records.push(measureRun(await fetchRun(api, repository, run.id)));
      }
    }
    const verdicts = evaluateSlo(records, slo);
    emit(env, argv, { records, verdicts }, summaryMarkdown(records, verdicts));
    let sustained = 0;
    for (const verdict of verdicts) {
      const values = verdict.samples.map(sample => sample.run + '=' + sample.value).join(', ');
      if (verdict.sustained) {
        sustained += 1;
        console.log('::error title=Sustained CI phase regression::' + verdict.id + ' exceeded ' + verdict.max + ' in all of the latest ' + slo.window + ' runs (' + values + ')');
      } else if (verdict.breaches) console.log('::warning title=CI phase objective breached::' + verdict.id + ' exceeded ' + verdict.max + ' in ' + verdict.breaches + ' of ' + verdict.samples.length + ' runs (' + values + ')');
    }
    return sustained ? 1 : 0;
  }
  console.error('usage: ci-phase-timing.mjs measure|check');
  return 2;
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main().then(code => {
    process.exitCode = code;
  }, error => {
    console.error(error.message);
    process.exitCode = 1;
  });
}
