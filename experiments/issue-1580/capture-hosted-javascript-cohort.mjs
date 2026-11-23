// Capture every hosted file event with its original job and log identity.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {readFileSync, mkdirSync, writeFileSync, copyFileSync} from 'node:fs';
import {join} from 'node:path';

const [input, output, mode] = process.argv.slice(2);
assert(mode === undefined || mode === '--retain-partial');
const retainPartial = mode === '--retain-partial';
assert(input && output, 'provide the retained log directory and a new fixture directory');
const read = name => readFileSync(join(input, name), 'utf8');
const run = JSON.parse(read('run.json'));
const jobs = JSON.parse(read('jobs.json')).flatMap(page => page.jobs)
  .filter(job => /^js tier .*suite group [0-9]+[)]$/.test(job.name));
assert.equal(jobs.length, 9);
if (retainPartial) {
  assert.equal(run.conclusion, 'failure');
  assert(jobs.every(job => ['success', 'failure'].includes(job.conclusion)));
} else {
  assert(jobs.every(job => job.conclusion === 'success'));
  assert.equal(run.conclusion, 'success');
}
const raw = read('complete-layered.log');
const rows = [];
const queued = new Map(jobs.map(job => [job.name, []]));
const warnings = [];
const testedCommits = new Set();
const lines = raw.split('\n');
for (let index = 0; index < lines.length; index++) {
  const [name, step, ...rest] = lines[index].split('\t');
  const job = jobs.find(job => job.name === name);
  if (!job) continue;
  const text = rest.join('\t');
  if (step === 'Run actions/checkout@v7' && text.includes('git log -1 --format=%H')) {
    const commit = lines[index + 1].split('\t').at(-1).split(' ').at(-1);
    assert.match(commit, /^[a-f0-9]{40}$/);
    testedCommits.add(commit);
  }
  if (step !== 'Run this shard of the JavaScript suites') continue;
  const event = text.match(/\[test-file\] (queued|completed|failed) (\S+)(?: (\{.*\}))?$/);
  if (event?.[1] === 'queued') queued.get(name).push(event[2]);
  else if (event) {
    const milliseconds = JSON.parse(event[3]).duration_ms;
    assert(Number.isFinite(milliseconds) && milliseconds >= 0);
    rows.push({job: name, file: event[2], seconds: milliseconds / 1000,
      complete: true, result: event[1], completionSource: 'node-file-summary', rawLogLine: index + 1});
  }
  if (text.includes('##[warning]')) {
    assert(text.includes('still running after 270s'));
    warnings.push({shard: Number(name.match(/group ([0-9]+)/)[1]), warningSeconds: 270,
      rawLogLine: index + 1, text});
  }
}
assert.equal(testedCommits.size, 1);
assert.equal(new Set(rows.map(row => row.file)).size, rows.length);
for (const job of jobs) {
  const observed = rows.filter(row => row.job === job.name).map(row => row.file).sort();
  assert.deepEqual(observed, queued.get(job.name).slice().sort());
}
const sourceFiles = execFileSync('git', ['ls-tree', '-r', '--name-only', run.head_sha, 'rust/tests/web/'],
  {encoding: 'utf8'}).trim().split('\n').filter(file => /^rust\/tests\/web\/[^/]+[.]test[.]mjs$/.test(file));
const observedFiles = new Set(rows.map(row => row.file));
assert(rows.every(row => sourceFiles.includes(row.file)));
const unobservedFiles = sourceFiles.filter(file => !observedFiles.has(file)).sort();
if (retainPartial) assert(unobservedFiles.length > 0, 'partial capture must retain its missing coverage');
else assert.deepEqual(rows.map(row => row.file).sort(), sourceFiles.sort());
const digest = text => createHash('sha256').update(text).digest('hex');
const durations = rows.map(row => JSON.stringify(row)).join('\n') + '\n';
const prior = execFileSync('git', ['show', run.head_sha + ':data/meta/javascript-test-durations.lino'], {encoding: 'utf8'});
const identity = {
  sourceCommit: run.head_sha, testedMergeCommit: [...testedCommits][0], run: run.id,
  runUrl: run.html_url, runConclusion: run.conclusion, completeObservations: rows.length,
  modeledWorkers: 4, warningSeconds: 270, budgetSeconds: 360, runnerOverheadAllowanceSeconds: 30,
  unobservedFallbackSeconds: 0.4, rawLogSha256: digest(raw), durationsSha256: digest(durations),
  priorSchedulingFloorSha256: digest(prior), warnings,
  jobs: jobs.map(job => ({job: job.name, id: job.id, conclusion: job.conclusion})),
  executionProof: false, autonomousAuthorshipCredit: 0
};
if (retainPartial) Object.assign(identity, {
  cohortComplete: false,
  unobservedFiles,
  coverageBoundary: 'Full-suite jobs stopped by focused-test failures are unobserved, never treated as completed.',
});
mkdirSync(output);
writeFileSync(join(output, 'durations.jsonl'), durations);
writeFileSync(join(output, 'identity.json'), JSON.stringify(identity, null, 2) + '\n');
writeFileSync(join(output, 'prior-scheduling-floor.lino'), prior);
copyFileSync(join(input, 'run.json'), join(output, 'run-observation.json'));
console.log(JSON.stringify({observations: rows.length, warnings, testedMergeCommit: identity.testedMergeCommit}));
