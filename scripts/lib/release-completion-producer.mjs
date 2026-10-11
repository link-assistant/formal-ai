import assert from 'node:assert/strict';
import { readFileSync, lstatSync, realpathSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { execFileSync, spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import YAML from 'yaml';
const parse = YAML.parse;
const profiles = new WeakSet(),
  proofs = new WeakSet();
const maximum = 1024 * 1024;
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
const commit = value => assert.match(value, /^[a-f0-9]{40}$/u);
const positive = value => assert.ok(Number.isSafeInteger(value) && value > 0);
const git = (root, args) => execFileSync('git', args, {
  cwd: root,
  timeout: 10000,
  maxBuffer: maximum
});
const source = (root, revision, path) => git(root, ['show', revision + ':' + path]);
const text = bytes => new TextDecoder('utf-8', {
  fatal: true
}).decode(bytes);
function ownSourceBytes() {
  const sourcePath = fileURLToPath(import.meta.url);
  const stat = lstatSync(sourcePath);
  assert.ok(stat.isFile() && !stat.isSymbolicLink());
  assert.equal(realpathSync(sourcePath), sourcePath);
  assert.ok(stat.size > 0 && stat.size <= maximum, 'bounded qualifier source');
  const bytes = readFileSync(sourcePath);
  assert.equal(bytes.length, stat.size, 'qualifier changed during source read');
  return bytes;
}
const loadedQualifierSha256 = hash(ownSourceBytes());
/** This profile derives step obligations from committed source, not caller JSON. */
export function productionProducerProfile(root, protocolCommit, workflowPath, jobId) {
  assert.equal(typeof root, 'string');
  root = realpathSync(root);
  commit(protocolCommit);
  assert.match(workflowPath, /^\.github\/workflows\/[A-Za-z0-9_-]+\.yml$/u);
  assert.match(jobId, /^[A-Za-z0-9_-]+$/u);
  const remote = text(git(root, ['remote', 'get-url', 'origin'])).trim();
  const repository = /^(?:https:\/\/github\.com\/|git@github\.com:)([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+?)(?:\.git)?$/u.exec(remote)?.[1];
  assert.ok(repository, 'exact credential-free GitHub origin is required');
  const selfPath = 'scripts/lib/release-completion-producer.mjs';
  const selfBytes = ownSourceBytes();
  assert.equal(hash(selfBytes), loadedQualifierSha256, 'loaded qualifier source drift');
  assert.deepEqual(source(root, protocolCommit, selfPath), selfBytes, 'qualifier must be the committed protocol source');
  const bytes = source(root, protocolCommit, workflowPath);
  assert.ok(bytes.length > 0 && bytes.length <= maximum);
  const workflow = parse(text(bytes)),
    job = workflow.jobs?.[jobId];
  assert.ok(job && typeof job === 'object', 'unregistered source job');
  assert.equal(job.strategy?.matrix, undefined, 'MissingContract: matrix producer identity needs a source-qualified expansion');
  const jobName = job.name ?? jobId;
  assert.ok(typeof jobName === 'string' && jobName.length > 0 && jobName.length <= 512 && !jobName.includes('${{'), 'MissingContract: dynamic producer name');
  assert.ok(Array.isArray(job.steps) && job.steps.length > 0 && job.steps.length <= 128);
  const requiredSteps = job.steps.filter(step => Object.hasOwn(step, 'run')).map(step => {
    assert.ok(typeof step.run === 'string' && step.run.length > 0);
    assert.ok(typeof step.name === 'string' && step.name.length > 0 && step.name.length <= 512 && !step.name.includes('${{'), 'MissingContract: run step needs a stable source name');
    return Object.freeze({
      name: step.name,
      sourceSha256: hash(Buffer.from(JSON.stringify(step)))
    });
  });
  assert.ok(requiredSteps.length > 0, 'MissingContract: no source run steps');
  assert.equal(new Set(requiredSteps.map(step => step.name)).size, requiredSteps.length, 'ambiguous source step names');
  const profile = Object.freeze({
    root,
    repository,
    protocolCommit,
    workflowPath,
    workflowSha256: hash(bytes),
    qualifierSha256: hash(selfBytes),
    jobId,
    jobName,
    requiredSteps: Object.freeze(requiredSteps),
    sourceTree: text(git(root, ['rev-parse', protocolCommit + '^{tree}'])).trim()
  });
  profiles.add(profile);
  return profile;
}
function profileOnly(profile) {
  assert.ok(profile && profiles.has(profile), 'unknown or copied source profile');
}
/** Pure fixture checking cannot mint authenticated proofs or delivery authority. */
export function validateProducerObservation(profile, {
  run,
  attempt,
  workflow,
  job,
  defaultBranch
}) {
  profileOnly(profile);
  positive(run.id);
  positive(run.run_attempt);
  positive(workflow.id);
  assert.equal(attempt.id, run.id);
  assert.equal(attempt.run_attempt, run.run_attempt);
  for (const observed of [run, attempt]) {
    assert.equal(observed.status, 'completed');
    assert.equal(observed.repository?.full_name, profile.repository);
    assert.equal(observed.head_repository?.full_name, profile.repository);
    assert.equal(observed.head_sha, profile.protocolCommit);
    assert.equal(observed.workflow_id, workflow.id);
    assert.equal(observed.path?.split('@')[0], profile.workflowPath);
    assert.ok(['push', 'workflow_dispatch', 'workflow_run', 'release'].includes(observed.event), 'nonproduction event');
    if (observed.event !== 'release') assert.equal(observed.head_branch, defaultBranch, 'production must use actual default branch');else assert.match(observed.head_branch, /^v(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)$/u);
  }
  assert.equal(attempt.event, run.event);
  assert.equal(attempt.head_branch, run.head_branch);
  assert.equal(workflow.path, profile.workflowPath);
  positive(job.id);
  assert.equal(job.run_id, run.id);
  assert.equal(job.head_sha, profile.protocolCommit);
  assert.equal(job.name, profile.jobName);
  assert.equal(job.status, 'completed');
  assert.equal(job.conclusion, 'success');
  assert.ok(Array.isArray(job.steps) && job.steps.length > 0 && job.steps.length <= 256);
  for (const required of profile.requiredSteps) {
    const matches = job.steps.filter(step => step.name === required.name);
    assert.equal(matches.length, 1, 'missing or ambiguous source-registered step');
    assert.equal(matches[0].status, 'completed');
    assert.equal(matches[0].conclusion, 'success', 'skipped step requires separate authenticated existing-artifact evidence');
  }
  return Object.freeze({
    state: 'ProducerStepsOnly',
    authenticated: false,
    publicationComplete: false,
    registryEvidence: 'Missing',
    runtimeReceiptEvidence: 'Missing',
    selectedProductSourceEvidence: 'Missing',
    authorityScope: 'ProducerMetadataAndStepStatusOnly',
    configuredPublicationEvidence: 'Missing',
    producerExecutionSourceReceipt: 'Missing',
    signerAuthority: false,
    writerLeaseAuthority: false
  });
}
/** Fixed authenticated API observer; no caller transport, booleans, or supplied observations. */
export function observeProductionProducer(profile, runId, runAttempt) {
  profileOnly(profile);
  positive(runId);
  positive(runAttempt);
  const started = Date.now(),
    observations = [];
  let metadataBytes = 0;
  const boundedGit = args => {
    const remaining = 90000 - (Date.now() - started);
    assert.ok(remaining > 0, 'shared source observation deadline');
    return execFileSync('git', args, {
      cwd: profile.root,
      timeout: Math.min(10000, remaining),
      maxBuffer: maximum
    });
  };
  const api = endpoint => {
    const remaining = 90000 - (Date.now() - started);
    assert.ok(remaining > 0, 'bounded metadata observation deadline');
    const result = spawnSync('gh', ['api', '--hostname', 'github.com', endpoint], {
      cwd: profile.root,
      encoding: null,
      timeout: Math.min(10000, remaining),
      maxBuffer: maximum
    });
    const stdout = result.stdout ?? Buffer.alloc(0),
      stderr = result.stderr ?? Buffer.alloc(0);
    observations.push({
      endpoint,
      status: result.status,
      signal: result.signal,
      stdoutBytes: stdout.length,
      stdoutSha256: hash(stdout),
      stderrBytes: stderr.length,
      stderrSha256: hash(stderr),
      stdout: text(stdout),
      stderr: text(stderr)
    });
    metadataBytes += stdout.length + stderr.length;
    assert.ok(metadataBytes <= 6 * maximum, 'cumulative metadata byte budget');
    assert.equal(result.error, undefined, 'metadata transport unavailable');
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, 'authenticated API unavailable; not product absence');
    return JSON.parse(text(stdout));
  };
  try {
    const prefix = 'repos/' + profile.repository;
    const repository = api(prefix);
    assert.equal(repository.full_name, profile.repository);
    assert.ok(typeof repository.default_branch === 'string' && repository.default_branch.length > 0);
    const run = api(prefix + '/actions/runs/' + runId);
    assert.equal(run.id, runId);
    assert.equal(run.run_attempt, runAttempt, 'stale attempt refusal');
    const attempt = api(prefix + '/actions/runs/' + runId + '/attempts/' + runAttempt);
    const workflow = api(prefix + '/actions/workflows/' + profile.workflowPath.split('/').at(-1));
    const defaultCommit = api(prefix + '/commits/' + encodeURIComponent(repository.default_branch)).sha;
    commit(defaultCommit);
    boundedGit(['merge-base', '--is-ancestor', profile.protocolCommit, defaultCommit]);
    if (run.event === 'release') {
      const release = api(prefix + '/releases/tags/' + encodeURIComponent(run.head_branch));
      assert.equal(release.tag_name, run.head_branch);
      assert.equal(release.draft, false);
      assert.equal(release.prerelease, false);
      assert.ok(typeof release.published_at === 'string' && Number.isFinite(Date.parse(release.published_at)));
      const tagCommit = api(prefix + '/commits/' + encodeURIComponent(run.head_branch)).sha;
      assert.equal(tagCommit, profile.protocolCommit, 'published tag source differs');
      assert.equal(text(boundedGit(['rev-parse', 'refs/tags/' + run.head_branch + '^{commit}'])).trim(), tagCommit, 'cached tag differs from authenticated API');
    }
    const jobs = [];
    let total = null;
    for (let page = 1; page <= 10; page++) {
      const data = api(prefix + '/actions/runs/' + runId + '/attempts/' + runAttempt + '/jobs?per_page=100&page=' + page);
      assert.ok(Number.isSafeInteger(data.total_count) && data.total_count > 0 && data.total_count <= 1000);
      if (total === null) total = data.total_count;else assert.equal(data.total_count, total, 'job enumeration changed');
      assert.ok(Array.isArray(data.jobs) && data.jobs.length <= 100);
      jobs.push(...data.jobs);
      assert.ok(jobs.length <= total, 'duplicate/truncated job enumeration');
      if (jobs.length === total) break;
      assert.equal(data.jobs.length, 100, 'truncated job page');
    }
    assert.equal(jobs.length, total, 'incomplete job inventory');
    assert.equal(new Set(jobs.map(job => job.id)).size, jobs.length, 'duplicate authenticated jobs');
    const matches = jobs.filter(job => job.name === profile.jobName);
    assert.equal(matches.length, 1, 'source producer job ambiguous or missing');
    const structural = validateProducerObservation(profile, {
      run,
      attempt,
      workflow,
      job: matches[0],
      defaultBranch: repository.default_branch
    });
    assert.equal(hash(boundedGit(['show', profile.protocolCommit + ':' + profile.workflowPath])), profile.workflowSha256, 'protocol source drift');
    assert.equal(hash(ownSourceBytes()), profile.qualifierSha256, 'qualifier source drift');
    assert.ok(Date.now() - started <= 90000, 'shared observer deadline exceeded');
    const proof = Object.freeze({
      ...structural,
      authenticated: true,
      repository: profile.repository,
      protocolCommit: profile.protocolCommit,
      sourceTree: profile.sourceTree,
      workflowPath: profile.workflowPath,
      workflowSha256: profile.workflowSha256,
      jobId: profile.jobId,
      jobName: profile.jobName,
      observedJobId: matches[0].id,
      runId,
      runAttempt,
      defaultCommit,
      observations: Object.freeze(observations.map(Object.freeze))
    });
    proofs.add(proof);
    return proof;
  } catch (error) {
    return Object.freeze({
      state: 'Unavailable',
      authenticated: false,
      publicationComplete: false,
      signerAuthority: false,
      writerLeaseAuthority: false,
      error: String(error.message),
      observations: Object.freeze(observations.map(Object.freeze))
    });
  }
}
/** A source-required producer join never substitutes for product delivery evidence. */
export function joinProductionProducerEvidence(requiredProfiles, observedProofs) {
  assert.ok(Array.isArray(requiredProfiles) && requiredProfiles.length > 0 && requiredProfiles.length <= 128);
  assert.ok(Array.isArray(observedProofs) && observedProofs.length === requiredProfiles.length);
  requiredProfiles.forEach(profileOnly);
  observedProofs.forEach(proof => assert.ok(proof && proofs.has(proof), 'unknown, structural-only or copied producer proof'));
  const keys = requiredProfiles.map(profile => profile.workflowPath + '::' + profile.jobId);
  assert.equal(new Set(keys).size, keys.length, 'duplicate required producer');
  const first = requiredProfiles[0];
  for (const profile of requiredProfiles) {
    assert.equal(profile.repository, first.repository);
    assert.equal(profile.protocolCommit, first.protocolCommit);
  }
  for (const profile of requiredProfiles) {
    const matches = observedProofs.filter(proof => proof.workflowPath === profile.workflowPath && proof.jobId === profile.jobId);
    assert.equal(matches.length, 1, 'missing required producer');
    assert.equal(matches[0].repository, profile.repository);
    assert.equal(matches[0].protocolCommit, profile.protocolCommit);
    assert.equal(matches[0].workflowSha256, profile.workflowSha256);
  }
  return Object.freeze({
    state: 'RequiredProducerStepsQualified',
    publicationComplete: false,
    advertisedTargetCoverage: 'NotEstablished',
    registryEvidence: 'Missing',
    runtimeReceiptEvidence: 'Missing',
    selectedProductSourceEvidence: 'Missing',
    authorityScope: 'ProducerMetadataAndStepStatusOnly',
    configuredPublicationEvidence: 'Missing',
    producerExecutionSourceReceipt: 'Missing',
    signerAuthority: false,
    writerLeaseAuthority: false,
    recoveryDecision: 'PendingAuthenticatedProductEvidence'
  });
}

// Additive static matrix profiles preserve the scalar API and its original refusals.
const matrixProfiles = new WeakMap(), matrixProofs = new WeakSet();
function matrixAtom(value) {
  assert.ok(typeof value === 'boolean' || typeof value === 'number' && Number.isSafeInteger(value)
    || typeof value === 'string' && value.length > 0 && value.length <= 128 && !value.includes('$' + '{{'),
  'MissingContract: static primitive matrix value required');
  return value;
}
function staticCombinations(matrix) {
  assert.ok(matrix && Object.getPrototypeOf(matrix) === Object.prototype, 'static source matrix required');
  const axes = Object.keys(matrix).filter(key => !['include', 'exclude'].includes(key));
  assert.ok(axes.length <= 8);
  const original = [];
  let combinations = [{}];
  for (const axis of axes) {
    assert.match(axis, /^[A-Za-z_][A-Za-z0-9_]*$/u);
    const values = matrix[axis];
    assert.ok(Array.isArray(values) && values.length > 0 && values.length <= 16);
    values.forEach(matrixAtom);
    assert.equal(new Set(values.map(value => JSON.stringify(value))).size, values.length, 'duplicate axis value');
    assert.ok(combinations.length * values.length <= 128, 'bounded static matrix expansion');
    combinations = combinations.flatMap(row => values.map(value => ({...row, [axis]: value})));
  }
  const entries = kind => {
    const values = matrix[kind] ?? [];
    assert.ok(Array.isArray(values) && values.length <= 128);
    for (const value of values) {
      assert.ok(value && Object.getPrototypeOf(value) === Object.prototype);
      const keys = Object.keys(value); assert.ok(keys.length > 0 && keys.length <= 16);
      for (const key of keys) { assert.match(key, /^[A-Za-z_][A-Za-z0-9_]*$/u); matrixAtom(value[key]); }
    }
    return values;
  };
  const exclusions = entries('exclude');
  for (const exclusion of exclusions) assert.ok(Object.keys(exclusion).every(key => axes.includes(key)), 'unknown exclude axis');
  if (axes.length) combinations = combinations.filter(row => !exclusions.some(exclusion =>
    Object.entries(exclusion).every(([key, value]) => row[key] === value)));
  else { assert.equal(exclusions.length, 0); combinations = []; }
  original.push(...combinations.map(row => ({...row})));
  const expanded = combinations.map(row => ({...row}));
  for (const include of entries('include')) {
    let merged = false;
    original.forEach((row, index) => {
      if (Object.keys(include).every(key => !Object.hasOwn(row, key) || row[key] === include[key])) {
        Object.assign(expanded[index], include); merged = true;
      }
    });
    if (!merged) expanded.push({...include});
    assert.ok(expanded.length <= 128, 'bounded include expansion');
  }
  assert.ok(expanded.length > 0, 'empty required matrix');
  const identities = expanded.map(row => JSON.stringify(Object.entries(row).sort(([left], [right]) => left.localeCompare(right))));
  assert.equal(new Set(identities).size, expanded.length, 'duplicate expanded identity');
  return expanded;
}
function matrixName(template, row) {
  assert.ok(typeof template === 'string' && template.length > 0 && template.length <= 512,
    'MissingContract: explicit source matrix name required');
  const name = template.replace(/\$\{\{\s*matrix\.([A-Za-z_][A-Za-z0-9_]*)\s*\}\}/gu, (_, key) => {
    assert.ok(Object.hasOwn(row, key), 'unbound matrix name field'); return String(row[key]);
  });
  assert.ok(!name.includes('$' + '{{') && name.length > 0 && name.length <= 512,
    'MissingContract: unsupported dynamic name expression');
  return name;
}
/** Committed regular source and private registration are necessary for matrix identity. */
export function productionMatrixProducerProfile(root, protocolCommit, workflowPath, jobId) {
  root = realpathSync(root); commit(protocolCommit);
  assert.match(workflowPath, /^\.github\/workflows\/[A-Za-z0-9_-]+\.yml$/u);
  assert.match(jobId, /^[A-Za-z0-9_-]+$/u);
  const repository = /^(?:https:\/\/github\.com\/|git@github\.com:)([A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+?)(?:\.git)?$/u
    .exec(text(git(root, ['remote', 'get-url', 'origin'])).trim())?.[1];
  assert.ok(repository, 'exact credential-free GitHub origin required');
  const ownBytes = ownSourceBytes(); assert.equal(hash(ownBytes), loadedQualifierSha256);
  assert.deepEqual(source(root, protocolCommit, 'scripts/lib/release-completion-producer.mjs'), ownBytes);
  const entry = text(git(root, ['ls-tree', '-z', protocolCommit, '--', workflowPath]));
  assert.match(entry, /^100(?:644|755) blob [a-f0-9]{40}\t[^\0]+\0$/u, 'ordinary committed workflow required');
  assert.equal(entry.slice(entry.indexOf('\t') + 1, -1), workflowPath);
  const workflowBytes = source(root, protocolCommit, workflowPath);
  assert.ok(workflowBytes.length > 0 && workflowBytes.length <= maximum);
  const workflow = parse(text(workflowBytes));
  assert.ok(workflow.jobs && Object.keys(workflow.jobs).length <= 128, 'bounded source job namespace required');
  const sourceTree = text(git(root, ['rev-parse', protocolCommit + '^{tree}'])).trim();
  const job = workflow.jobs?.[jobId];
  assert.ok(job && typeof job === 'object' && !Object.hasOwn(job, 'uses'), 'ordinary source matrix job required');
  assert.equal(job.if, undefined, 'MissingContract: conditional matrix producer identity');
  const combinations = staticCombinations(job.strategy?.matrix);
  assert.ok(Array.isArray(job.steps) && job.steps.length > 0 && job.steps.length <= 128);
  const children = combinations.map(row => {
    const requiredSteps = job.steps.filter(step => Object.hasOwn(step, 'run')).map(step => {
      assert.ok(typeof step.run === 'string' && step.run.length > 0);
      return Object.freeze({name: matrixName(step.name, row), sourceSha256: hash(Buffer.from(JSON.stringify(step)))});
    });
    assert.ok(requiredSteps.length > 0);
    assert.equal(new Set(requiredSteps.map(step => step.name)).size, requiredSteps.length, 'ambiguous source run step');
    const child = Object.freeze({root, repository, protocolCommit, workflowPath, jobId,
      jobName: matrixName(job.name, row), requiredSteps: Object.freeze(requiredSteps),
      workflowSha256: hash(workflowBytes), qualifierSha256: hash(ownBytes),
      sourceTree});
    profiles.add(child); return child;
  });
  assert.equal(new Set(children.map(child => child.jobName)).size, children.length, 'ambiguous expanded job names');
  const names = new Set(children.map(child => child.jobName));
  for (const [otherId, otherJob] of Object.entries(workflow.jobs)) {
    if (otherId === jobId) continue;
    assert.ok(otherJob && typeof otherJob === 'object' && !Object.hasOwn(otherJob, 'uses'),
      'MissingContract: reusable descendant name namespace');
    assert.ok(!otherJob.strategy?.matrix || typeof otherJob.name === 'string',
      'MissingContract: foreign generated matrix display-name namespace');
    const rows = otherJob.strategy?.matrix ? staticCombinations(otherJob.strategy.matrix) : [{}];
    for (const row of rows) assert.ok(!names.has(matrixName(otherJob.name ?? otherId, row)),
      'foreign source job collides with required matrix identity');
  }
  const profile = Object.freeze({repository, protocolCommit, workflowPath, jobId,
    jobNames: Object.freeze(children.map(child => child.jobName)), matrixSize: children.length,
    sourceTree: children[0].sourceTree, workflowSha256: children[0].workflowSha256,
    qualifierSha256: children[0].qualifierSha256,
    authorityScope: 'SourceDeclaredMatrixIdentityOnly', publicationComplete: false});
  matrixProfiles.set(profile, Object.freeze(children)); return profile;
}
function matrixChildren(profile) {
  const children = matrixProfiles.get(profile); assert.ok(children, 'unknown or copied matrix profile'); return children;
}
/** Caller observations can establish structural checks only, never authentication. */
export function validateMatrixProducerObservation(profile, observed) {
  const children = matrixChildren(profile);
  assert.ok(Array.isArray(observed.jobs) && observed.jobs.length > 0 && observed.jobs.length <= 1000);
  assert.equal(new Set(observed.jobs.map(job => job.id)).size, observed.jobs.length, 'duplicate job inventory');
  const selected = children.map(child => {
    const matches = observed.jobs.filter(job => job.name === child.jobName);
    assert.equal(matches.length, 1, 'missing or ambiguous expanded source job');
    return validateProducerObservation(child, {...observed, job: matches[0]});
  });
  return Object.freeze({...selected[0], state: 'MatrixProducerStepsOnly', matrixSize: children.length,
    authenticated: false, publicationComplete: false});
}
/** Reuse the unchanged bounded authenticated transport; its proof supplies all API bytes. */
export function observeProductionMatrixProducer(profile, runId, runAttempt) {
  const children = matrixChildren(profile);
  const started = Date.now();
  const firstProof = observeProductionProducer(children[0], runId, runAttempt);
  if (!proofs.has(firstProof)) return firstProof;
  try {
    const prefix = 'repos/' + profile.repository;
    const one = endpoint => {
      const matches = firstProof.observations.filter(record => record.endpoint === endpoint);
      assert.equal(matches.length, 1); return JSON.parse(matches[0].stdout);
    };
    const jobs = firstProof.observations.filter(record => record.endpoint.startsWith(prefix + '/actions/runs/'
      + runId + '/attempts/' + runAttempt + '/jobs?')).flatMap(record => JSON.parse(record.stdout).jobs);
    const structural = validateMatrixProducerObservation(profile, {
      run: one(prefix + '/actions/runs/' + runId),
      attempt: one(prefix + '/actions/runs/' + runId + '/attempts/' + runAttempt),
      workflow: one(prefix + '/actions/workflows/' + profile.workflowPath.split('/').at(-1)),
      defaultBranch: one(prefix).default_branch, jobs
    });
    assert.ok(Date.now() - started <= 90000, 'shared matrix observer deadline exceeded');
    assert.equal(hash(ownSourceBytes()), profile.qualifierSha256, 'matrix qualifier source drift');
    const proof = Object.freeze({...structural, authenticated: true, repository: profile.repository,
      protocolCommit: profile.protocolCommit, workflowPath: profile.workflowPath,
      jobId: profile.jobId, runId, runAttempt, sourceTree: profile.sourceTree,
      workflowSha256: profile.workflowSha256, qualifierSha256: profile.qualifierSha256,
      observations: firstProof.observations});
    matrixProofs.add(proof); return proof;
  } catch (error) {
    return Object.freeze({state: 'Unavailable', authenticated: false, publicationComplete: false,
      signerAuthority: false, writerLeaseAuthority: false, error: String(error.message),
      observations: firstProof.observations});
  }
}
export function joinProductionMatrixProducerEvidence(profile, proof) {
  matrixChildren(profile); assert.ok(matrixProofs.has(proof), 'unknown or copied authenticated matrix proof');
  for (const key of ['repository', 'protocolCommit', 'workflowPath', 'jobId', 'sourceTree',
    'workflowSha256', 'qualifierSha256']) assert.equal(proof[key], profile[key]);
  assert.equal(proof.matrixSize, profile.matrixSize);
  return Object.freeze({...proof, observations: undefined, state: 'MatrixProducerStepsQualified',
    publicationComplete: false, advertisedTargetCoverage: 'NotEstablished', registryEvidence: 'Missing',
    runtimeReceiptEvidence: 'Missing', selectedProductSourceEvidence: 'Missing', signerAuthority: false,
    writerLeaseAuthority: false, recoveryDecision: 'PendingAuthenticatedProductEvidence'});
}
