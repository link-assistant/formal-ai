// A successful source-registered publication job is not a green pipeline or effect authority.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {join} from 'node:path';
import {performance} from 'node:perf_hooks';
import YAML from 'yaml';
import {compileContract} from './staged-caller-authority.mjs';

const evidence = 'experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/';
const digest = bytes => createHash('sha256').update(bytes).digest('hex');
const gitBlob = bytes => createHash('sha1').update(Buffer.concat([
  Buffer.from('blob ' + bytes.length + '\0'), bytes,
])).digest('hex');

/** Derive the registered route from the existing lossless release compiler. */
export function publicationContract(cwd) {
  const canonical = readFileSync(join(cwd, evidence, 'original-release-workflow.yml'), 'utf8');
  const packetSource = readFileSync(join(cwd, evidence, 'stage-source-coverage.json'), 'utf8');
  return compileContract({
    canonical, packet: JSON.parse(packetSource), YAML,
    callerDraft: readFileSync(join(cwd, '.github/workflows/release.yml'), 'utf8'),
    staged: readFileSync(join(cwd, '.github/workflows/release-staged.yml'), 'utf8'),
    retainedSourceProof: {originalSource: canonical, packetSource},
  });
}

/** Authenticate the completed publisher independently while its caller may still run. */
export function resolvePublishedProducer(cwd, environment, readGithub, clock = () => performance.now()) {
  const repository = environment.REPOSITORY;
  assert.match(repository, /^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/u);
  for (const field of ['PRODUCER_RUN_ID', 'PRODUCER_RUN_ATTEMPT', 'CURRENT_RUN_ID', 'CURRENT_RUN_ATTEMPT']) {
    assert.match(environment[field], /^[1-9][0-9]*$/u, field);
  }
  assert.equal(environment.PRODUCER_RUN_ID, environment.CURRENT_RUN_ID);
  assert.equal(environment.PRODUCER_RUN_ATTEMPT, environment.CURRENT_RUN_ATTEMPT);
  assert.equal(environment.REFERENCE, 'refs/heads/main');
  assert.equal(environment.CURRENT_WORKFLOW_REF, repository + '/.github/workflows/release.yml@refs/heads/main');
  assert.ok(['push', 'workflow_dispatch'].includes(environment.EVENT));
  const started = clock();
  let requests = 0;
  const get = endpoint => {
    assert.ok(clock() - started <= 90000, 'publication observation deadline exceeded');
    assert.ok(++requests <= 64, 'publication observation request bound exceeded');
    const value = readGithub(endpoint);
    assert.ok(clock() - started <= 90000, 'publication observation deadline exceeded');
    return value;
  };
  const base = 'repos/' + repository + '/';
  const run = get(base + 'actions/runs/' + environment.PRODUCER_RUN_ID);
  assert.equal(String(run.id), environment.PRODUCER_RUN_ID);
  assert.equal(String(run.run_attempt), environment.PRODUCER_RUN_ATTEMPT);
  assert.equal(run.repository?.full_name, repository);
  assert.equal(run.head_repository?.full_name, repository);
  assert.equal(run.head_branch, 'main');
  assert.equal(run.event, environment.EVENT);
  assert.match(run.head_sha, /^[a-f0-9]{40}$/u);
  assert.ok(['in_progress', 'completed'].includes(run.status));
  assert.ok(!['cancelled', 'skipped', 'neutral', 'timed_out', 'action_required'].includes(run.conclusion));
  assert.ok(Number.isSafeInteger(run.workflow_id) && run.workflow_id > 0);
  const workflow = get(base + 'actions/workflows/' + run.workflow_id);
  assert.equal(workflow.id, run.workflow_id);
  assert.equal(workflow.path, '.github/workflows/release.yml');
  assert.equal(run.path, workflow.path);
  const contract = publicationContract(cwd);
  for (const source of contract.sources) {
    const observed = get(base + 'contents/' + source.path + '?ref=' + run.head_sha);
    assert.equal(observed.type, 'file');
    assert.equal(observed.path, source.path);
    assert.equal(observed.encoding, 'base64');
    assert.ok(Number.isSafeInteger(observed.size) && observed.size >= 0 && observed.size <= 1048576);
    const bytes = Buffer.from(observed.content, 'base64');
    assert.equal(bytes.length, observed.size);
    assert.equal(gitBlob(bytes), observed.sha, 'authenticated Git blob identity differs');
    assert.equal(digest(bytes), source.sha256, 'publisher source differs from compiled default-source contract');
  }
  const routes = contract.routes.filter(route => route.stepIdentifier === 'create-release' && route.event === run.event);
  assert.equal(routes.length, 1, 'one registered production publication operation required');
  const route = routes[0];
  const jobs = [], identifiers = new Set();
  let total;
  for (let page = 1; page <= 10; page++) {
    const response = get(base + 'actions/runs/' + run.id + '/attempts/' + run.run_attempt + '/jobs?per_page=100&page=' + page);
    assert.ok(Number.isSafeInteger(response.total_count) && response.total_count >= 0 && response.total_count <= 1000);
    total ??= response.total_count;
    assert.equal(response.total_count, total, 'job inventory changed during observation');
    assert.ok(Array.isArray(response.jobs) && response.jobs.length <= 100);
    for (const job of response.jobs) {
      assert.ok(Number.isSafeInteger(job.id) && job.id > 0 && !identifiers.has(job.id));
      identifiers.add(job.id);
      assert.equal(job.run_id, run.id);
      assert.equal(job.run_attempt, run.run_attempt);
      jobs.push(job);
    }
    assert.ok(jobs.length <= total);
    if (jobs.length === total) break;
    assert.equal(response.jobs.length, 100, 'truncated job inventory refused');
  }
  assert.equal(jobs.length, total, 'complete bounded exact-attempt job inventory required');
  const publishers = jobs.filter(job => job.name === route.descendantName);
  assert.equal(publishers.length, 1, 'one exact registered final publisher required');
  const publisher = publishers[0];
  assert.equal(publisher.status, 'completed');
  assert.equal(publisher.conclusion, 'success');
  const packet = JSON.parse(readFileSync(join(cwd, evidence, 'stage-source-coverage.json'), 'utf8'));
  const declaration = packet.callers.find(item => item.caller === route.caller);
  const operations = declaration.steps.filter(item => item.ordinal === route.ordinal && item.stepId === route.stepIdentifier);
  assert.equal(operations.length, 1, 'one source-registered publication operation required');
  const operation = operations[0];
  assert.equal(digest(operation.raw), route.operationSha256);
  const registeredStep = YAML.parse('steps:\n' + operation.raw).steps[0];
  assert.ok(typeof registeredStep.name === 'string' && registeredStep.name.length > 0);
  assert.ok(Array.isArray(publisher.steps) && publisher.steps.length <= 1000);
  const publicationSteps = publisher.steps.filter(step => step.name === registeredStep.name);
  assert.equal(publicationSteps.length, 1, 'one exact source-registered publication step required');
  const publicationStep = publicationSteps[0];
  assert.equal(publicationStep.status, 'completed');
  if (publicationStep.conclusion === 'skipped') {
    assert.ok(typeof registeredStep.if === 'string' && registeredStep.if.trim().length > 0,
      'only a source-declared conditional operation can be authentically inactive');
    return {publish: false, tag: '', reason: 'registered-publication-skipped', effectAuthority: 'Unknown'};
  }
  assert.equal(publicationStep.conclusion, 'success', 'successful stage with skipped publication is not publication execution');
  assert.ok(Number.isFinite(Date.parse(publicationStep.started_at)) && Number.isFinite(Date.parse(publicationStep.completed_at)));
  assert.ok(Date.parse(publicationStep.started_at) >= Date.parse(publisher.started_at));
  assert.ok(Date.parse(publicationStep.completed_at) >= Date.parse(publicationStep.started_at));
  assert.ok(Date.parse(publicationStep.completed_at) <= Date.parse(publisher.completed_at));
  assert.ok(Number.isFinite(Date.parse(publisher.started_at)) && Number.isFinite(Date.parse(publisher.completed_at)));
  assert.ok(Date.parse(publisher.started_at) <= Date.parse(publisher.completed_at));
  const releases = get(base + 'releases?per_page=30');
  assert.ok(Array.isArray(releases) && releases.length <= 30);
  for (const release of releases) {
    if (release.draft || release.prerelease || !/^v[0-9]+\.[0-9]+\.[0-9]+$/u.test(release.tag_name)) continue;
    const commit = get(base + 'commits/' + encodeURIComponent(release.tag_name));
    if (commit.sha !== run.head_sha && commit.parents?.[0]?.sha !== run.head_sha) continue;
    return {publish: true, tag: release.tag_name, producerRun: run.id, producerAttempt: run.run_attempt,
      publisherJob: publisher.id, sourceHead: run.head_sha, effectAuthority: 'Unknown', pipelineGreen: 'Pending'};
  }
  return {publish: false, tag: '', effectAuthority: 'Unknown'};
}
