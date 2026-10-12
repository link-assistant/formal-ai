import {test} from 'node:test';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdtempSync, writeFileSync, rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {resolveSelfAuthoredRelease} from '../../../scripts/resolve-self-authored-release.mjs';

function fixture(t) {
  const cwd = mkdtempSync(join(tmpdir(), 'author-release-'));
  t.after(() => rmSync(cwd, {recursive: true, force: true}));
  const git = (...arguments_) => execFileSync('git', arguments_, {cwd, encoding: 'utf8', stdio: 'pipe'}).trim();
  git('init', '--initial-branch=main');
  git('config', 'user.name', 'Fixture');
  git('config', 'user.email', 'fixture@example.invalid');
  writeFileSync(join(cwd, 'source'), 'released');
  git('add', '.'); git('commit', '-m', 'release'); git('tag', 'v1.2.3');
  const commit = git('rev-parse', 'HEAD');
  const release = {id: 11, tag_name: 'v1.2.3', draft: false, prerelease: false, published_at: '2026-10-01T00:00:00Z'};
  const repository = {full_name: 'fixture/repository', default_branch: 'main'};
  const read = endpoint => {
    if (endpoint === 'repos/fixture/repository/') return repository;
    if (endpoint.includes('/releases/tags/')) return release;
    if (endpoint.includes('/commits/')) return {sha: commit};
    throw Error('unexpected source request ' + endpoint);
  };
  const environment = {EVENT: 'release', RELEASE_ACTION: 'published', REPOSITORY: repository.full_name, RELEASE_TAG: release.tag_name};
  return {cwd, git, commit, release, repository, environment, read};
}

test('published source resolves default branch while retaining acceptance boundaries', t => {
  const f = fixture(t);
  const result = resolveSelfAuthoredRelease(f.cwd, f.environment, f.read);
  assert.equal(result.active, true); assert.equal(result.branch, 'main');
  assert.equal(result.releaseCommit, f.commit); assert.equal(result.releaseId, 11);
  assert.equal(result.effectAuthority, 'Unknown'); assert.equal(result.reviewedContribution, 'Pending');
});

test('unpublished, unstable, wrong repository, identifier or default source refuses', t => {
  const f = fixture(t);
  for (const delta of [{published_at: null}, {id: 0}, {draft: true}, {prerelease: true}]) {
    const read = endpoint => endpoint.includes('/releases/tags/') ? {...f.release, ...delta} : f.read(endpoint);
    assert.throws(() => resolveSelfAuthoredRelease(f.cwd, f.environment, read));
  }
  assert.throws(() => resolveSelfAuthoredRelease(f.cwd, {...f.environment, RELEASE_ACTION: 'created'}, f.read));
  assert.throws(() => resolveSelfAuthoredRelease(f.cwd, {...f.environment, EVENT: 'issues'}, f.read));
  for (const delta of [{full_name: 'foreign/repository'}, {default_branch: 'foreign'}]) {
    const read = endpoint => endpoint.endsWith('/') ? {...f.repository, ...delta} : f.read(endpoint);
    assert.throws(() => resolveSelfAuthoredRelease(f.cwd, f.environment, read));
  }
});

test('release-tag detached checkout and changed default source refuse before authoring', t => {
  const f = fixture(t);
  f.git('checkout', '--detach', f.commit);
  assert.throws(() => resolveSelfAuthoredRelease(f.cwd, f.environment, f.read));
  f.git('checkout', 'main');
  writeFileSync(join(f.cwd, 'source'), 'changed'); f.git('commit', '-am', 'move');
  assert.throws(() => resolveSelfAuthoredRelease(f.cwd, f.environment, f.read));
});

test('foreign or cancelled completed producer yields inactive, never authoring context', t => {
  const f = fixture(t);
  for (const delta of [{RUN_REPOSITORY: 'foreign/repository'}, {RUN_CONCLUSION: 'cancelled'}, {RUN_BRANCH: 'feature'}]) {
    const environment = {...f.environment, EVENT: 'workflow_run', RUN_BRANCH: 'main', RUN_REPOSITORY: f.repository.full_name, ...delta};
    assert.deepEqual(resolveSelfAuthoredRelease(f.cwd, environment, () => {throw Error('must not observe foreign source');}), {active: false});
  }
});

function completedProducer(f) {
  const environment = {...f.environment, EVENT: 'workflow_run', RUN_ID: '21', RUN_ATTEMPT: '2',
    RUN_WORKFLOW_ID: '31', RUN_BRANCH: 'main', RUN_REPOSITORY: f.repository.full_name,
    RUN_CONCLUSION: 'success', RUN_HEAD: f.commit};
  const run = {id: 21, run_attempt: 2, workflow_id: 31, event: 'push', status: 'completed', conclusion: 'success',
    head_sha: f.commit, head_branch: 'main', repository: f.repository, head_repository: f.repository,
    path: '.github/workflows/release.yml@refs/heads/main'};
  const workflow = {id: 31, path: '.github/workflows/release.yml'};
  const read = endpoint => {
    if (endpoint.endsWith('/actions/runs/21')) return run;
    if (endpoint.endsWith('/actions/workflows/release.yml')) return workflow;
    if (endpoint.endsWith('/releases?per_page=30')) return [f.release];
    return f.read(endpoint);
  };
  return {environment, run, workflow, read};
}

test('authenticated completed producer binds run attempt workflow and actual published source', t => {
  const f = fixture(t); const producer = completedProducer(f);
  assert.equal(resolveSelfAuthoredRelease(f.cwd, producer.environment, producer.read).releaseId, 11);
  for (const delta of [{run_attempt: 3}, {workflow_id: 99}, {head_sha: 'f'.repeat(40)},
    {path: '.github/workflows/foreign.yml'}, {head_repository: {full_name: 'foreign/repository'}},
    {status: 'in_progress'}, {conclusion: 'failure'}]) {
    const read = endpoint => endpoint.endsWith('/actions/runs/21') ? {...producer.run, ...delta} : producer.read(endpoint);
    assert.deepEqual(resolveSelfAuthoredRelease(f.cwd, producer.environment, read), {active: false});
  }
  const read = endpoint => endpoint.endsWith('/actions/workflows/release.yml')
    ? {...producer.workflow, path: '.github/workflows/foreign.yml'} : producer.read(endpoint);
  assert.deepEqual(resolveSelfAuthoredRelease(f.cwd, producer.environment, read), {active: false});
});

test('successful completion without independently published stable release stays inactive', t => {
  const f = fixture(t); const producer = completedProducer(f);
  const read = endpoint => endpoint.endsWith('/releases?per_page=30') ? [] : producer.read(endpoint);
  assert.deepEqual(resolveSelfAuthoredRelease(f.cwd, producer.environment, read), {active: false});
});

test('authenticated PR or unknown event refuses before publication and author source observations', t => {
  const f = fixture(t); const producer = completedProducer(f);
  for (const event of ['pull_request', 'pull_request_target', 'schedule', 'unknown', undefined]) {
    const observations = [];
    const read = endpoint => {
      observations.push(endpoint);
      return endpoint.endsWith('/actions/runs/21') ? {...producer.run, event} : producer.read(endpoint);
    };
    assert.throws(() => resolveSelfAuthoredRelease(f.cwd, producer.environment, read), /production release event/);
    assert.deepEqual(observations, ['repos/fixture/repository/actions/runs/21']);
  }
  const read = endpoint => endpoint.endsWith('/actions/runs/21')
    ? {...producer.run, event: 'workflow_dispatch'} : producer.read(endpoint);
  assert.equal(resolveSelfAuthoredRelease(f.cwd, producer.environment, read).releaseId, 11);
});
