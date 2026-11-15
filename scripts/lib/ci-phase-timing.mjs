// Issue #1486: split a Hive draft or delivery run into the phases its feedback
// time is spent in, with the bytes it moved and how many solver attempts were
// useful, then judge those numbers against numeric service-level objectives.
//
// The Actions API records a job's queue (created_at to started_at) and every
// step's start and end, so a run can be measured after the fact, with no clock
// inside the workflow. Steps the runner skipped are never execution evidence:
// they contribute no seconds and no attempts.

export const RECORD_SCHEMA = 'formal-ai-ci-phase-timing/v1';
export const SLO_SCHEMA = 'formal-ai-ci-phase-slo/v1';
export const PHASES = ['queue', 'runner', 'checkout', 'install', 'compile', 'image-prep', 'image-transfer', 'image-load', 'solver',
  'verification', 'publication', 'transfer', 'other'];

// Ordered: the first matching rule names the phase. Unnamed `uses:` steps
// appear in the API as `Run <action>@<ref>`.
const RULES = [
  ['runner', /^(Set up job|Complete job|Post )/u],
  ['image-transfer', /same-run task image delta/u],
  ['image-load', /^(Load and reverify|Release transferred image archive)/u],
  ['image-prep', /^(Build and verify actual same-source task image|Free hosted image space|Free disk for the Hive Mind image)/u],
  ['solver', /^(Attempt through shared Hive Mind|Run actual Hive solve)/u],
  ['checkout', /^(Run actions\/checkout@|Checkout )/u],
  ['install', /^(Run (actions\/setup-node|oven-sh\/setup-bun|coursier\/setup-action|dtolnay\/rust-toolchain)|Install |Restore shared Cargo|Resolve Rust compiler)/u],
  ['compile', /^(Build current main binary|Extract exact shipped binary|Compute the effective Formal AI build identity|Claim the build identity|Reuse or await the single producer|Download the authenticated shared build|Verify the shared build|Run \.\/\.github\/actions\/formal-ai-binary)/u],
  ['verification', /^(Verify |Independently |Authenticate |Read-only |Measure released|Admit one attempt|Decide before building|Claim one bounded client effect)/u],
  ['publication', /^(Publish |Upload |Persist |Stage |Retain |Dispatch |Link |Record |Seal |Describe |Transfer only|Run actions\/upload-artifact@)/u],
  ['transfer', /^(Run actions\/download-artifact@|Download |Restore source)/u],
];

const IMAGE_ARTIFACT = 'hive-same-source-task-image';
const BINARY_ARTIFACT = /^(formal-ai-build-[0-9a-f]{64}|hive-mind-main-binary-|hive-same-source-formal-ai-binary)/u;

export function classifyStep(name) {
  for (const [phase, pattern] of RULES) if (pattern.test(name ?? '')) return phase;
  return 'other';
}

const seconds = (from, to) => (from && to ? Math.max(0, (Date.parse(to) - Date.parse(from)) / 1000) : 0);
const executed = step => step.conclusion && step.conclusion !== 'skipped' && step.started_at && step.completed_at;
const zeroPhases = () => Object.fromEntries(PHASES.map(phase => [phase, 0]));

/**
 * Measure one completed run from its run, jobs and artifacts records.
 * Before the image steps were named (issue #1486), the leg's image download
 * was an unnamed `download-artifact` step; it is recognised as the step that
 * directly precedes the image load.
 */
export function measureRun({ run, jobs, artifacts = [] }) {
  const totals = zeroPhases();
  const maxJob = zeroPhases();
  const attempts = { total: 0, useful: 0, skipped: 0 };
  const measuredJobs = [];
  let compileMode = null;
  let imageLoads = 0;
  for (const job of jobs) {
    const phases = zeroPhases();
    const queue = seconds(job.created_at, job.started_at);
    phases.queue = queue;
    const steps = [...(job.steps ?? [])].sort((left, right) => left.number - right.number);
    steps.forEach((step, index) => {
      let phase = classifyStep(step.name);
      if (phase === 'transfer' && /^Run actions\/download-artifact@/u.test(step.name) && classifyStep(steps[index + 1]?.name) === 'image-load'
        && /^Load and reverify/u.test(steps[index + 1].name)) phase = 'image-transfer';
      if (phase === 'solver') {
        if (executed(step)) {
          attempts.total += 1;
          if (step.conclusion === 'success') attempts.useful += 1;
        } else if (step.conclusion === 'skipped') attempts.skipped += 1;
      }
      if (!executed(step)) return;
      if (/^Load and reverify/u.test(step.name)) imageLoads += 1;
      if (/^Build current main binary/u.test(step.name)) compileMode = 'build';
      else if (/^Download the authenticated shared build/u.test(step.name) && compileMode === null) compileMode = 'reuse';
      phases[phase] += seconds(step.started_at, step.completed_at);
    });
    for (const phase of PHASES) {
      totals[phase] += phases[phase];
      maxJob[phase] = Math.max(maxJob[phase], phases[phase]);
    }
    measuredJobs.push({ name: job.name, conclusion: job.conclusion, queueSeconds: queue, seconds: seconds(job.started_at, job.completed_at), phases });
  }
  const live = artifacts.filter(artifact => !artifact.expired);
  const image = artifacts.find(artifact => artifact.name === IMAGE_ARTIFACT);
  const binary = artifacts.find(artifact => BINARY_ARTIFACT.test(artifact.name));
  const imageBundle = image && !image.expired ? image.size_in_bytes : null;
  return {
    schema: RECORD_SCHEMA,
    run: {
      id: run.id, workflow: run.name, event: run.event, branch: run.head_branch, sha: run.head_sha, attempt: run.run_attempt,
      conclusion: run.conclusion, createdAt: run.created_at,
    },
    wallSeconds: seconds(run.run_started_at ?? run.created_at, run.updated_at),
    queue: { maxSeconds: Math.max(0, ...measuredJobs.map(job => job.queueSeconds)) },
    phase: Object.fromEntries(PHASES.map(phase => [phase, { seconds: totals[phase], maxJobSeconds: maxJob[phase] }])),
    compile: { mode: compileMode },
    attempts,
    bytes: {
      artifacts: live.reduce((sum, artifact) => sum + artifact.size_in_bytes, 0),
      imageBundle,
      imageTransfer: imageBundle === null ? null : imageBundle * imageLoads,
      binary: binary && !binary.expired ? binary.size_in_bytes : null,
      expiredArtifacts: artifacts.length - live.length,
    },
    jobs: measuredJobs,
  };
}

export function metric(record, path) {
  let value = record;
  for (const key of path.split('.')) value = value?.[key];
  return value ?? null;
}

const matches = (value, expected) => (expected !== null && typeof expected === 'object' && 'min' in expected
  ? typeof value === 'number' && value >= expected.min : value === expected);

/**
 * Judge records against objectives. An objective is sustained-breached only
 * when every one of the latest `window` applicable measurements exceeds it;
 * a single slow run is reported, not alerted. An objective may narrow its
 * window (a byte size is deterministic, so one breach is already sustained)
 * and its runs: `when` maps a metric to a value or to `{min}`.
 */
export function evaluateSlo(records, slo) {
  if (slo.schema !== SLO_SCHEMA) throw Error('CiPhaseSloSchema: ' + slo.schema);
  const window = slo.window;
  if (!(Number.isInteger(window) && window > 0)) throw Error('CiPhaseSloWindow: ' + window);
  const newestFirst = [...records].sort((left, right) => Date.parse(right.run.createdAt) - Date.parse(left.run.createdAt));
  return slo.objectives.map(objective => {
    if (!(typeof objective.max === 'number' && objective.max >= 0)) throw Error('CiPhaseSloMax: ' + objective.id);
    const size = objective.window ?? window;
    if (!(Number.isInteger(size) && size > 0)) throw Error('CiPhaseSloWindow: ' + objective.id);
    const samples = newestFirst
      .filter(record => record.run.workflow === objective.workflow)
      .filter(record => Object.entries(objective.when ?? {}).every(([path, expected]) => matches(metric(record, path), expected)))
      .map(record => ({ run: record.run.id, value: metric(record, objective.metric) }))
      .filter(sample => typeof sample.value === 'number')
      .slice(0, size);
    const breaches = samples.filter(sample => sample.value > objective.max).length;
    return { ...objective, window: size, samples, breaches, sustained: samples.length === size && breaches === size };
  });
}

const formatSeconds = value => (value === null ? 'n/a' : Math.round(value) + ' s');
const formatBytes = value => (value === null ? 'n/a' : (value / 1024 ** 2).toFixed(1) + ' MiB');

export function summaryMarkdown(records, verdicts = []) {
  const lines = ['## Hive CI phase timing (issue #1486)', ''];
  const shown = PHASES.filter(phase => records.some(record => record.phase[phase].seconds > 0));
  lines.push('| Run | Workflow | Wall | ' + shown.join(' | ') + ' | Image bundle | Image transfer | Attempts (useful/total) |');
  lines.push('|' + ' --- |'.repeat(shown.length + 6));
  for (const record of records) {
    lines.push('| ' + record.run.id + ' | ' + record.run.workflow + ' | ' + formatSeconds(record.wallSeconds) + ' | '
      + shown.map(phase => formatSeconds(record.phase[phase].seconds)).join(' | ') + ' | ' + formatBytes(record.bytes.imageBundle) + ' | '
      + formatBytes(record.bytes.imageTransfer) + ' | ' + record.attempts.useful + '/' + record.attempts.total + ' |');
  }
  if (verdicts.length) {
    lines.push('', '| Objective | Max | Latest samples | Breaches | Sustained |', '| --- | --- | --- | --- | --- |');
    for (const verdict of verdicts) {
      lines.push('| ' + verdict.id + ' | ' + verdict.max + ' | ' + verdict.samples.map(sample => sample.value).join(', ') + ' | '
        + verdict.breaches + '/' + verdict.samples.length + ' | ' + (verdict.sustained ? '**yes**' : 'no') + ' |');
    }
  }
  return lines.join('\n') + '\n';
}
