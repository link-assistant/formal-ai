import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { parseCiDurations, renderCiDurations } from '../../../scripts/lib/ci-speed-durations.mjs';

const root = fileURLToPath(new URL('../../..', import.meta.url));
const plain = (name, samples = 3) => `    job "${name}"\n      key build\n      median 1\n      maximum 2\n      samples ${samples}\n`;
const shared = (name, head, job) => `    cohort ${name}\n      samples 1\n      source-head ${head}\n      run 123\n      job "${job}"\n        key cli\n        median 1.4\n        maximum 1.4\n        job-identifier 789\n        runner-start 2026-10-09T10:41:11Z\n        job-completed 2026-10-09T10:42:30Z\n`;

test('explicit nested cohort inherits only its own jobs and preserves adjacent historical jobs', () => {
  const text = 'ci-durations\n  workflow "a.yml"\n    name "A"\n' + plain('Before')
    + shared('fresh', 'a'.repeat(40), 'Fresh') + plain('After', 8)
    + '  workflow "b.yml"\n    name "B"\n' + plain('Other', 7);
  const parsed = parseCiDurations(text);
  assert.deepEqual(parsed.workflows[0].jobs.map((job) => [job.display, job.samples]), [['Before', 3], ['Fresh', 1], ['After', 8]]);
  assert.equal(parsed.workflows[0].jobs[1].executionCohort, 'fresh');
  for (const job of [parsed.workflows[0].jobs[0], parsed.workflows[0].jobs[2], parsed.workflows[1].jobs[0]]) {
    assert.equal(job.executionCohort, undefined);
    assert.equal(job.sourceHead, undefined);
    assert.equal(job.run, undefined);
  }
  const rendered = renderCiDurations(parsed);
  assert.ok(rendered.indexOf('job "Before"') < rendered.indexOf('cohort "fresh"'));
  assert.ok(rendered.indexOf('cohort "fresh"') < rendered.indexOf('job "After"'));
  assert.deepEqual(parseCiDurations(rendered), parsed);
});

test('distinct cohorts and a local override preserve exact observed sample counts', () => {
  const text = 'ci-durations\n  workflow "a.yml"\n' + shared('one', 'a'.repeat(40), 'One')
    + '        samples 4\n' + shared('two', 'b'.repeat(40), 'Two');
  const parsed = parseCiDurations(text);
  assert.deepEqual(parsed.workflows[0].jobs.map((job) => [job.executionCohort, job.sourceHead, job.samples]),
    [['one', 'a'.repeat(40), 4], ['two', 'b'.repeat(40), 1]]);
  assert.deepEqual(parseCiDurations(renderCiDurations(parsed)), parsed);
});

test('shared values declared after members are inherited without changing explicit observations', () => {
  const parsed = parseCiDurations('ci-durations\n  workflow "a"\n    cohort "c"\n      job "J"\n        maximum 7.8\n        samples 2\n      samples 1\n      source-head ' + '1'.repeat(40) + '\n');
  assert.equal(parsed.workflows[0].jobs[0].samples, 2);
  assert.equal(parsed.workflows[0].jobs[0].maximum, 7.8);
  assert.equal(parsed.workflows[0].jobs[0].sourceHead, '1'.repeat(40));
});

test('generic render preserves job identifiers, runner times, provenance and additional observed fields', () => {
  const parsed = parseCiDurations('ci-durations\n  workflow "a"\n' + shared('c', 'b'.repeat(40), 'J') + '        feature-profile "all-features-release"\n');
  const job = parsed.workflows[0].jobs[0];
  assert.equal(job.jobIdentifier, 789);
  assert.equal(job.runnerStart, '2026-10-09T10:41:11Z');
  assert.equal(job.jobCompleted, '2026-10-09T10:42:30Z');
  assert.equal(job.featureProfile, 'all-features-release');
  assert.deepEqual(parseCiDurations(renderCiDurations(parsed)), parsed);
});

test('actual eleven consumer records retain every ID, runner time and source-bound measurement', () => {
  const parsed = parseCiDurations(readFileSync(root + '/data/meta/ci-durations.lino', 'utf8'));
  const workflow = parsed.workflows.find((entry) => entry.path.endsWith('desktop-release.yml'));
  const cohort = workflow.cohorts[0];
  const expected = [
  {
    "display": "Build macos-x64",
    "key": "build",
    "median": 13.4,
    "maximum": 13.4,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:33:41Z",
    "jobCompleted": "2026-10-09T10:47:04Z",
    "jobIdentifier": 113779538906
  },
  {
    "display": "Build windows-arm64",
    "key": "build",
    "median": 21.1,
    "maximum": 21.1,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:32:31Z",
    "jobCompleted": "2026-10-09T10:53:34Z",
    "jobIdentifier": 113779538936
  },
  {
    "display": "Build CLI cli-macos-x64",
    "key": "cli",
    "median": 2.1,
    "maximum": 2.1,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:35:02Z",
    "jobCompleted": "2026-10-09T10:37:04Z",
    "jobIdentifier": 113779538786
  },
  {
    "display": "Build windows-x64",
    "key": "build",
    "median": 15,
    "maximum": 15,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:42:33Z",
    "jobCompleted": "2026-10-09T10:57:29Z",
    "jobIdentifier": 113779538881
  },
  {
    "display": "Build CLI cli-windows-x64",
    "key": "cli",
    "median": 1.7,
    "maximum": 1.7,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:34:46Z",
    "jobCompleted": "2026-10-09T10:36:26Z",
    "jobIdentifier": 113779538787
  },
  {
    "display": "Build macos-arm64",
    "key": "build",
    "median": 4.2,
    "maximum": 4.2,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:43:41Z",
    "jobCompleted": "2026-10-09T10:47:52Z",
    "jobIdentifier": 113779538867
  },
  {
    "display": "Build linux-arm64",
    "key": "build",
    "median": 10,
    "maximum": 10,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:35:04Z",
    "jobCompleted": "2026-10-09T10:45:00Z",
    "jobIdentifier": 113779538792
  },
  {
    "display": "Build linux-x64",
    "key": "build",
    "median": 12.2,
    "maximum": 12.2,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T11:21:58Z",
    "jobCompleted": "2026-10-09T11:34:07Z",
    "jobIdentifier": 113779539062
  },
  {
    "display": "Build CLI cli-macos-arm64",
    "key": "cli",
    "median": 1.4,
    "maximum": 1.4,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:41:11Z",
    "jobCompleted": "2026-10-09T10:42:30Z",
    "jobIdentifier": 113779538823
  },
  {
    "display": "Build CLI cli-linux-x64",
    "key": "cli",
    "median": 0.9,
    "maximum": 0.9,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:50:52Z",
    "jobCompleted": "2026-10-09T10:51:46Z",
    "jobIdentifier": 113779538712
  },
  {
    "display": "Build CLI cli-linux-arm64",
    "key": "cli",
    "median": 1,
    "maximum": 1,
    "samples": 1,
    "executionCohort": "same-run-native-reuse",
    "sourceHead": "d7437d4bcfc6d94823ebeba634de016f25561b41",
    "run": 37914168811,
    "runnerStart": "2026-10-09T10:40:31Z",
    "jobCompleted": "2026-10-09T10:41:29Z",
    "jobIdentifier": 113779538853
  }
];
  assert.equal(cohort.name, 'same-run-native-reuse');
  assert.deepEqual(cohort.shared, { executionCohort: 'same-run-native-reuse', samples: 1,
    sourceHead: 'd7437d4bcfc6d94823ebeba634de016f25561b41', run: 37914168811, key: 'build' });
  assert.deepEqual(cohort.jobs, expected);
  assert.deepEqual(workflow.jobs.filter((job) => job.executionCohort === cohort.name), expected);
  assert.deepEqual(parseCiDurations(renderCiDurations(parsed)), parsed);
});

test('legacy flat rendering still preserves existing contract and custom job observation fields', () => {
  const input = { header: { branch: 'b' }, workflows: [{ path: 'x.yml', name: 'X', runs: 3, wallMedian: 12.5,
    wallMaximum: 20, jobs: [{ display: 'Test', key: 'test', median: 9.5, maximum: 11, samples: 3, runnerStart: '2026-10-09T00:00:00Z' }] }] };
  const result = parseCiDurations(renderCiDurations(input));
  assert.deepEqual(result, input);
});
