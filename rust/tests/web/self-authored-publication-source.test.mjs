import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {publicationContract, resolvePublishedProducer} from '../../../scripts/self-authored-publication-source.mjs';
const cwd = fileURLToPath(new URL('../../../', import.meta.url));
const head = 'a'.repeat(40);
const repository = 'fixture/repository';
const base = 'repos/' + repository + '/';
const environment = {EVENT: 'push', REPOSITORY: repository, PRODUCER_RUN_ID: '41', PRODUCER_RUN_ATTEMPT: '2', CURRENT_RUN_ID: '41', CURRENT_RUN_ATTEMPT: '2', REFERENCE: 'refs/heads/main', CURRENT_WORKFLOW_REF: repository + '/.github/workflows/release.yml@refs/heads/main'};
function fixture() {
  const contract = publicationContract(cwd);
  const route = contract.routes.find(item => item.stepIdentifier === 'create-release' && item.event === 'push');
  const run = {id: 41, run_attempt: 2, repository: {full_name: repository}, head_repository: {full_name: repository}, head_branch: 'main', head_sha: head, event: 'push', status: 'in_progress', conclusion: null, workflow_id: 19, path: '.github/workflows/release.yml'};
  const publisher = {id: 73, run_id: 41, run_attempt: 2, name: route.descendantName, status: 'completed', conclusion: 'success', started_at: '2026-10-11T00:01:00Z', completed_at: '2026-10-11T00:02:00Z'};
  const packet = JSON.parse(readFileSync(new URL('../../../experiments/formal_ai_subagent/evidence/specification-delivery-1188/dormant-staged-release/stage-source-coverage.json', import.meta.url), 'utf8'));
  const operation = packet.callers.find(item => item.caller === route.caller).steps.find(item => item.ordinal === route.ordinal);
  publisher.steps = [{name: operation.name, status: 'completed', conclusion: 'success', started_at: '2026-10-11T00:01:10Z', completed_at: '2026-10-11T00:01:50Z'}];
  const entries = new Map([
    [base + 'actions/runs/41', run],
    [base + 'actions/workflows/19', {id: 19, path: run.path}],
    [base + 'actions/runs/41/attempts/2/jobs?per_page=100&page=1', {total_count: 1, jobs: [publisher]}],
    [base + 'releases?per_page=30', [{tag_name: 'v1.2.3', draft: false, prerelease: false}]],
    [base + 'commits/v1.2.3', {sha: 'b'.repeat(40), parents: [{sha: head}]}],
  ]);
  for (const source of contract.sources) {
    const bytes = readFileSync(cwd + source.path);
    const blob = createHash('sha1').update(Buffer.concat([Buffer.from('blob ' + bytes.length + '\0'), bytes])).digest('hex');
    entries.set(base + 'contents/' + source.path + '?ref=' + head, {type: 'file', path: source.path, encoding: 'base64', size: bytes.length, sha: blob, content: bytes.toString('base64')});
  }
  const calls = [];
  const read = endpoint => {calls.push(endpoint); assert.ok(entries.has(endpoint), endpoint); return structuredClone(entries.get(endpoint));};
  return {contract, run, publisher, entries, calls, read};
}
test('completed registered publisher permits observation while caller is still running', () => {
  const f = fixture(), result = resolvePublishedProducer(cwd, environment, f.read);
  assert.equal(result.publish, true); assert.equal(result.publisherJob, 73);
  assert.equal(result.effectAuthority, 'Unknown'); assert.equal(result.pipelineGreen, 'Pending');
  assert.equal(f.contract.routes.length, 52); assert.equal(f.contract.bindings, 104);
  assert.ok(f.calls.some(endpoint => endpoint.includes('/contents/')));
});
test('all foreign run, attempt, source, workflow and unfinished publisher witnesses refuse', () => {
  for (const change of [
    f => {f.run.head_repository.full_name = 'foreign/repository';},
    f => {f.run.repository.full_name = 'foreign/repository';},
    f => {f.run.head_branch = 'feature';},
    f => {f.run.run_attempt = 3;},
    f => {f.run.event = 'pull_request';},
    f => {f.run.conclusion = 'cancelled';},
    f => {f.entries.get(base + 'actions/workflows/19').path = '.github/workflows/foreign.yml';},
    f => {f.publisher.status = 'in_progress'; f.publisher.conclusion = null;},
    f => {f.publisher.conclusion = 'failure';},
    f => {f.publisher.name = 'prefix ' + f.publisher.name;},
    f => {f.publisher.run_attempt = 1;},
    f => {const response = f.entries.get(base + 'actions/runs/41/attempts/2/jobs?per_page=100&page=1'); response.total_count = 2; response.jobs.push({...f.publisher, id: 74});},
    f => {f.entries.get(base + 'actions/runs/41/attempts/2/jobs?per_page=100&page=1').total_count = 2;},
    f => {const source = f.contract.sources[0]; f.entries.get(base + 'contents/' + source.path + '?ref=' + head).sha = 'c'.repeat(40);},
    f => {f.entries.get(base + 'actions/runs/41/attempts/2/jobs?per_page=100&page=1').total_count = 1001;},
  ]) {const f = fixture(); change(f); assert.throws(() => resolvePublishedProducer(cwd, environment, f.read));}
});
test('controller inputs cannot substitute another run, attempt, main source or event', () => {
  for (const delta of [{PRODUCER_RUN_ID: '42'}, {PRODUCER_RUN_ATTEMPT: '3'}, {REFERENCE: 'refs/heads/feature'}, {CURRENT_WORKFLOW_REF: 'foreign/repository/.github/workflows/release.yml@refs/heads/main'}, {EVENT: 'pull_request'}]) {
    const f = fixture(); assert.throws(() => resolvePublishedProducer(cwd, {...environment, ...delta}, f.read));
    assert.equal(f.calls.length, 0);
  }
});
test('no matching independently published tag stays inactive and deadlines refuse', () => {
  const f = fixture(); f.entries.get(base + 'commits/v1.2.3').parents = [];
  assert.equal(resolvePublishedProducer(cwd, environment, f.read).publish, false);
  let tick = 0; assert.throws(() => resolvePublishedProducer(cwd, environment, fixture().read, () => tick++ * 90001));
});

test('only completed original publishers call the scoped author while the independent lease remains', async () => {
  const YAML = (await import('yaml')).default;
  const caller = YAML.parse(readFileSync(new URL('../../../.github/workflows/release.yml', import.meta.url), 'utf8'));
  const author = YAML.parse(readFileSync(new URL('../../../.github/workflows/self-authored-pull-request.yml', import.meta.url), 'utf8'));
  const invocation = caller.jobs['self-authored-after-publication'];
  assert.deepEqual(invocation.needs, ['auto-release', 'manual-release']);
  assert.equal(invocation.uses, './.github/workflows/self-authored-pull-request.yml');
  assert.equal(invocation.if, '$' + "{{ !cancelled() && github.ref == 'refs/heads/main' && (needs.auto-release.result == 'success' || needs.manual-release.result == 'success') }}");
  assert.deepEqual(invocation.permissions, author.jobs.author.permissions);
  assert.deepEqual(invocation.with, {'production-run-id': '$' + '{{ github.run_id }}', 'production-run-attempt': '$' + '{{ github.run_attempt }}'});
  assert.deepEqual(Object.keys(invocation.secrets).sort(), Object.keys(author.on.workflow_call.secrets).sort());
  assert.deepEqual(author.permissions, {});
  assert.deepEqual(author.jobs.author.concurrency, {group: 'formal-ai-repository-writes', queue: 'max'});
  assert.equal(author.jobs.author['timeout-minutes'], 30);
  const installation = author.jobs.author.steps.find(step => step.name === 'Install locked source protocol dependencies');
  assert.equal(installation.run, 'bun install --frozen-lockfile --ignore-scripts');
  assert.equal(installation['timeout-minutes'], 5);
  assert.ok(author.jobs.author.steps.indexOf(installation) < author.jobs.author.steps.findIndex(step => step.id === 'release-context'));
});

test('successful stage cannot mask skipped, missing, duplicate or failed publication operation', () => {
  for (const change of [
    f => {f.publisher.steps = [];},
    f => {f.publisher.steps[0].conclusion = 'failure';},
    f => {f.publisher.steps[0].status = 'in_progress';},
    f => {f.publisher.steps[0].name = 'prefix ' + f.publisher.steps[0].name;},
    f => {f.publisher.steps.push({...f.publisher.steps[0]});},
    f => {f.publisher.steps[0].completed_at = '2026-10-11T00:03:00Z';},
  ]) {const f = fixture(); change(f); assert.throws(() => resolvePublishedProducer(cwd, environment, f.read));}
});

test('authentic no-release conditional operation stays inactive without release lookup or authoring', () => {
  const f = fixture();
  f.publisher.steps[0].conclusion = 'skipped';
  f.publisher.steps[0].started_at = null;
  f.publisher.steps[0].completed_at = null;
  const result = resolvePublishedProducer(cwd, environment, f.read);
  assert.equal(result.publish, false);
  assert.equal(result.tag, '');
  assert.equal(result.reason, 'registered-publication-skipped');
  assert.equal(result.effectAuthority, 'Unknown');
  assert.ok(f.calls.every(endpoint => !endpoint.includes('/releases') && !endpoint.includes('/commits/')));
});
