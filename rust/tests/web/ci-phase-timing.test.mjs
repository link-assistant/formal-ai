// Issue #1486: Hive draft and delivery runs are split into phases from the
// Actions API records, with bytes and useful-attempt counts, and judged
// against numeric objectives. Everything here replays recorded API JSON; no
// network, Docker or Rust build is involved.
import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import YAML from 'yaml';
import { classifyStep, evaluateSlo, measureRun, metric, SLO_SCHEMA } from '../../../scripts/lib/ci-phase-timing.mjs';
import { main } from '../../../scripts/ci-phase-timing.mjs';

const root = new URL('../../../', import.meta.url);
const readJson = path => JSON.parse(readFileSync(new URL(path, root), 'utf8'));
const workflow = name => YAML.parse(readFileSync(new URL('.github/workflows/' + name, root), 'utf8'));
const baseline = id => {
  const prefix = 'data/benchmarks/ci-phase-timing/baseline-runs/' + id;
  return { run: readJson(prefix + '-run.json'), jobs: readJson(prefix + '-jobs.json').jobs, artifacts: readJson(prefix + '-artifacts.json').artifacts };
};
const at = second => new Date(Date.UTC(2026, 10, 1, 0, 0, second)).toISOString();
const step = (number, name, start, end, conclusion = 'success') => ({ number, name, conclusion, started_at: at(start), completed_at: at(end) });
const run = (id, name, created = 0) => ({ id, name, event: 'issues', head_branch: 'main', head_sha: 'a'.repeat(40), run_attempt: 1, conclusion: 'success', created_at: at(created), run_started_at: at(created), updated_at: at(created + 100) });

test('steps classify into the phases the issue names; unnamed actions and post steps are not execution phases', () => {
  const expected = {
    'Set up job': 'runner', 'Post Run actions/checkout@v7': 'runner', 'Run actions/checkout@v7': 'checkout', 'Checkout exact main source': 'checkout',
    'Run actions/setup-node@v7': 'install', 'Restore shared Cargo registry without an extra cache writer': 'install',
    'Build current main binary in CI': 'compile', 'Download the authenticated shared build': 'compile', 'Reuse or await the single producer of this build identity': 'compile',
    'Extract exact shipped binary from immutable published runtime': 'compile', 'Build and verify actual same-source task image': 'image-prep',
    'Free hosted image space': 'image-prep', 'Publish the same-run task image delta': 'image-transfer', 'Download the same-run task image delta': 'image-transfer',
    'Load and reverify same-run source-bound task image': 'image-load', 'Attempt through shared Hive Mind scripts (including image pull retry)': 'solver',
    'Run actual Hive solve and original assertions': 'solver', "Verify this attempt's exact pull request": 'verification',
    'Independently compile delivered source and require exactly one output line': 'verification',
    'Upload complete session evidence on success and failure': 'publication', 'Run actions/upload-artifact@v7': 'publication',
    'Run actions/download-artifact@v8': 'transfer', 'Resolve current main commit': 'other',
  };
  for (const [name, phase] of Object.entries(expected)) assert.equal(classifyStep(name), phase, name);
});

test('recorded delivery baseline: the per-leg multi-GB image download dominates, three useful attempts', () => {
  const record = measureRun(baseline('40475832633'));
  assert.equal(record.run.workflow, 'Original Hive Mind three-client delivery');
  // The pre-#1486 unnamed download directly before the load is the image transfer.
  assert.equal(record.phase['image-transfer'].maxJobSeconds, 1882);
  assert.equal(record.phase['image-transfer'].seconds, 1882 + 1187 + 571);
  assert.equal(record.phase['image-load'].maxJobSeconds, 407);
  assert.equal(record.phase.compile.seconds, 373);
  assert.deepEqual(record.attempts, { total: 3, useful: 3, skipped: 0 });
  assert.equal(record.bytes.imageBundle, null, 'the one-day image artifact had expired: unknown, never zero');
  assert.equal(record.wallSeconds, 4734);
});

test('recorded draft baseline: a cold compile and one failed attempt; skipped steps add nothing', () => {
  const record = measureRun(baseline('41191429608'));
  assert.equal(record.compile.mode, 'build');
  assert.equal(record.phase.compile.maxJobSeconds, 353);
  assert.equal(record.queue.maxSeconds, 87);
  assert.deepEqual(record.attempts, { total: 1, useful: 0, skipped: 0 });
  const skipped = baseline('41191429608').jobs.flatMap(job => job.steps).filter(item => item.conclusion === 'skipped');
  assert.ok(skipped.length > 0);
});

test('a reused shared build is the warm path; skipped solver steps are not attempts', () => {
  const record = measureRun({
    run: run(1, 'Hive Mind draft'),
    jobs: [{
      name: 'build', conclusion: 'success', created_at: at(0), started_at: at(5), completed_at: at(60), steps: [
        step(1, 'Compute the effective Formal AI build identity', 5, 7),
        step(2, 'Download the authenticated shared build', 7, 20),
        step(3, 'Build current main binary in CI', 0, 0, 'skipped'),
        step(4, 'Attempt through shared Hive Mind scripts (including image pull retry)', 0, 0, 'skipped'),
      ],
    }],
    artifacts: [{ name: 'formal-ai-build-' + 'b'.repeat(64), size_in_bytes: 4096, expired: false }],
  });
  assert.equal(record.compile.mode, 'reuse');
  assert.equal(record.phase.compile.seconds, 15);
  assert.equal(record.phase.solver.seconds, 0);
  assert.deepEqual(record.attempts, { total: 0, useful: 0, skipped: 1 });
  assert.equal(record.bytes.binary, 4096);
  assert.equal(record.queue.maxSeconds, 5);
});

test('image bytes are counted once per verified leg load and only while the artifact exists', () => {
  const leg = n => ({
    name: 'leg ' + n, conclusion: 'success', created_at: at(0), started_at: at(1), completed_at: at(50), steps: [
      step(1, 'Download the same-run task image delta', 1, 11), step(2, 'Load and reverify same-run source-bound task image', 11, 31),
      step(3, 'Run actual Hive solve and original assertions', 31, 40, n === 3 ? 'failure' : 'success'),
    ],
  });
  const input = { run: run(2, 'Original Hive Mind three-client delivery'), jobs: [leg(1), leg(2), leg(3)], artifacts: [{ name: 'hive-same-source-task-image', size_in_bytes: 1000, expired: false }] };
  const record = measureRun(input);
  assert.equal(record.bytes.imageBundle, 1000);
  assert.equal(record.bytes.imageTransfer, 3000);
  assert.deepEqual(record.attempts, { total: 3, useful: 2, skipped: 0 });
  assert.equal(record.phase['image-transfer'].maxJobSeconds, 10);
  input.artifacts[0].expired = true;
  assert.equal(measureRun(input).bytes.imageTransfer, null);
});

const sample = (id, created, values) => ({ run: { id, workflow: 'W', createdAt: at(created) }, ...values });
const slo = objectives => ({ schema: SLO_SCHEMA, window: 3, objectives });

test('only a breach in every one of the latest window runs is sustained; filters and windows narrow the samples', () => {
  const records = [
    sample(1, 1, { wallSeconds: 900, compile: { mode: 'build' }, attempts: { total: 1 } }),
    sample(2, 2, { wallSeconds: 900, compile: { mode: 'reuse' }, attempts: { total: 1 } }),
    sample(3, 3, { wallSeconds: 900, compile: { mode: 'build' }, attempts: { total: 0 } }),
    sample(4, 4, { wallSeconds: 100, compile: { mode: 'build' }, attempts: { total: 1 } }),
  ];
  const [all, built, attempted, latest] = evaluateSlo(records, slo([
    { id: 'all', workflow: 'W', metric: 'wallSeconds', max: 500 },
    { id: 'built', workflow: 'W', metric: 'wallSeconds', max: 500, when: { 'compile.mode': 'build' } },
    { id: 'attempted', workflow: 'W', metric: 'wallSeconds', max: 500, when: { 'attempts.total': { min: 1 } } },
    { id: 'latest', workflow: 'W', metric: 'wallSeconds', max: 50, window: 1 },
  ]));
  assert.deepEqual(all.samples.map(item => item.run), [4, 3, 2]);
  assert.equal(all.breaches, 2);
  assert.equal(all.sustained, false);
  assert.deepEqual(built.samples.map(item => item.run), [4, 3, 1]);
  assert.deepEqual(attempted.samples.map(item => item.run), [4, 2, 1]);
  assert.equal(latest.sustained, true);
  const slow = evaluateSlo(records.slice(0, 3), slo([{ id: 'slow', workflow: 'W', metric: 'wallSeconds', max: 500 }]));
  assert.equal(slow[0].sustained, true);
  const unknown = evaluateSlo([sample(5, 5, { bytes: { imageBundle: null } })], slo([{ id: 'bytes', workflow: 'W', metric: 'bytes.imageBundle', max: 1, window: 1 }]));
  assert.equal(unknown[0].sustained, false, 'an unknown size is not a measurement');
  assert.throws(() => evaluateSlo(records, { ...slo([]), schema: 'other' }), /CiPhaseSloSchema/);
  assert.throws(() => evaluateSlo(records, slo([{ id: 'x', workflow: 'W', metric: 'wallSeconds' }])), /CiPhaseSloMax/);
  assert.equal(metric({ a: { b: 0 } }, 'a.b'), 0);
});

test('the committed objectives name real workflows and metrics; the recorded baselines breach only what issue #1486 targets', () => {
  const objectives = readJson('data/benchmarks/ci-phase-timing/slo.json');
  const records = [measureRun(baseline('40475832633')), measureRun(baseline('41191429608'))];
  for (const objective of objectives.objectives) {
    assert.equal(workflow(objective.workflowFile).name, objective.workflow, objective.id);
    if (objective.baseline?.value !== undefined) {
      const record = records.find(item => item.run.id === objective.baseline.run);
      assert.equal(metric(record, objective.metric), objective.baseline.value, objective.id);
    }
  }
  const breached = evaluateSlo(records, { ...objectives, window: 1 }).filter(verdict => verdict.breaches).map(verdict => verdict.id);
  assert.deepEqual(breached, ['delivery-image-transfer-per-leg', 'delivery-wall']);
});

test('the timing workflow reads both measured workflows, cannot re-admit a delivery and stays read-only', () => {
  const timing = workflow('ci-phase-timing.yml');
  const delivery = workflow('hive-original-delivery.yml');
  assert.deepEqual(timing.on.workflow_run.workflows, [workflow('hive-mind-draft.yml').name, delivery.name]);
  assert.deepEqual(timing.permissions, { actions: 'read', contents: 'read' });
  assert.ok(delivery.on.workflow_run.workflows.includes('!' + timing.name));
  const job = timing.jobs.measure;
  assert.ok(job['timeout-minutes'] <= 30);
  assert.match(job.if, /workflow_run\.conclusion != 'skipped'/u);
  for (const item of job.steps.filter(entry => entry.run)) assert.doesNotMatch(item.run, /\$\{\{/u, 'inputs reach commands only through the environment');
  const named = jobs => Object.values(jobs).flatMap(entry => entry.steps ?? []);
  const imageSteps = named(delivery.jobs).filter(entry => entry.with?.name === 'hive-same-source-task-image');
  assert.equal(imageSteps.length, 2);
  for (const entry of imageSteps) assert.equal(classifyStep(entry.name), 'image-transfer', entry.name);
  const draft = named(workflow('hive-mind-draft.yml').jobs);
  for (const [name, phase] of [['Build current main binary in CI', 'compile'], ['Download the authenticated shared build', 'compile'],
    ['Attempt through shared Hive Mind scripts (including image pull retry)', 'solver']]) {
    assert.ok(draft.some(entry => entry.name === name), name);
    assert.equal(classifyStep(name), phase);
  }
});

test('check alerts with ::error only on a sustained regression and samples only executed runs', async () => {
  const calls = [];
  const runs = [1, 2, 3].map(id => ({ ...run(id, 'Hive Mind draft', id), updated_at: at(id + 2000) }));
  const api = async path => {
    calls.push(path);
    const match = /\/runs\/(\d+)(\/jobs|\/artifacts)?/u.exec(path);
    if (path.includes('/workflows/')) return { workflow_runs: path.includes('status=success') ? runs : [] };
    if (match[2] === '/jobs') return { jobs: [] };
    if (match[2] === '/artifacts') return { artifacts: [] };
    return runs.find(item => item.id === Number(match[1]));
  };
  const objectives = slo([{ id: 'wall', workflow: 'Hive Mind draft', workflowFile: 'hive-mind-draft.yml', metric: 'wallSeconds', max: 1000 }]);
  const file = new URL('ci-phase-timing-slo.json', 'file://' + (process.env.TMPDIR ?? '/tmp') + '/');
  const { writeFileSync, rmSync } = await import('node:fs');
  writeFileSync(file, JSON.stringify(objectives));
  const lines = [];
  const original = console.log;
  console.log = line => lines.push(line);
  const write = process.stdout.write;
  process.stdout.write = () => true;
  try {
    assert.equal(await main(['check', '--runs', '5', '--slo', file.pathname, '--repo', 'o/r'], {}, api), 1);
    assert.ok(lines.some(line => line.startsWith('::error title=Sustained CI phase regression::wall')));
    assert.ok(calls.some(path => path.includes('status=failure')) && !calls.some(path => path.includes('status=completed')));
    await assert.rejects(main(['check', '--runs', '0', '--slo', file.pathname], {}, api), /CiPhaseRuns/);
  } finally {
    console.log = original;
    process.stdout.write = write;
    rmSync(file);
  }
});
