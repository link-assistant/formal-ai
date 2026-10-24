// The default token uses owned branches; original public task content stays verifiable.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {requireReadyPullRequest} from './hive-delivery-evidence.mjs';

const hash = value => createHash('sha256').update(value).digest('hex');
export const isolatedTaskSchema = 'formal-ai-isolated-hive-task/v1';
export const isolatedFixturePrefix = 'Formal AI isolated fixture: ';
const tools = ['claude', 'agent', 'codex'];
const languages = {claude: 'Kotlin', agent: 'Scala', codex: 'Rust'};

export function isolatedTaskBase({run, attempt, tool}) {
  assert(Number.isSafeInteger(run) && run > 0);
  assert([1, 2].includes(attempt) && tools.includes(tool));
  return `e2e/hello-world/${run}-${attempt}/${tool}-formal-ai`;
}

/** Undo only Hive's documented workflow staging instruction, not task requirements. */
export function originalWorkflowBody(body) {
  const directory = '- The GitHub Actions workflow should be in `ci/` directory';
  const staging = '- Save the workflow as `ci/test-hello-world.yml`, not under `.github/workflows/`: '
    + "this repository's token cannot push workflow files, so the test harness copies it there and runs it";
  assert.equal(body.split(directory).length, 2, 'one staged workflow directory required');
  assert.equal(body.split('\n' + staging).length, 2, 'one explicit staging instruction required');
  return body.replace(directory, '- The GitHub Actions workflow should be in `.github/workflows/` directory')
    .replace('\n' + staging, '');
}

export function requireIsolatedTask(fixture, {repository, run, attempt, source, protocolHead, inputHash, tool, task}) {
  assert.equal(fixture.schema, isolatedTaskSchema);
  for (const [key, value] of Object.entries({repository, run, attempt, source, protocolHead, tool})) {
    assert.equal(fixture[key], value, 'owned isolated task ' + key);
  }
  assert.equal(fixture.inputSha256, inputHash);
  assert.equal(fixture.originalBodySha256, task.bodySha256);
  assert.equal(fixture.workflowDirectory, 'ci/');
  assert.equal(fixture.resource.repository, repository);
  assert.equal(fixture.resource.baseBranch, isolatedTaskBase({run, attempt, tool}));
  assert(Number.isSafeInteger(fixture.resource.issueNumber) && fixture.resource.issueNumber > 0);
  assert.equal(fixture.resource.issueUrl, `https://github.com/${repository}/issues/${fixture.resource.issueNumber}`);
  assert.match(fixture.protocolHead, /^[a-f0-9]{40}$/u);
  return fixture;
}

export function requireIsolatedIssue(issue, fixture) {
  assert.equal(issue.html_url, fixture.resource.issueUrl);
  assert.equal(issue.number, fixture.resource.issueNumber);
  assert.equal(hash(issue.body), fixture.bodySha256, 'isolated issue changed');
  assert.equal(hash(originalWorkflowBody(issue.body)), fixture.originalBodySha256,
    'the complete original task must survive workflow staging');
  assert.equal(issue.title, 'Implement Hello World in ' + languages[fixture.tool]);
  assert.equal(issue.user.login, 'github-actions[bot]');
  const labels = issue.labels.map(value => typeof value === 'string' ? value : value.name);
  for (const label of ['e2e-task', 'no-formal-ai-draft']) assert(labels.includes(label));
}

export function requireIsolatedPullRequest(value, fixture, {closed = false} = {}) {
  const resource = fixture.resource;
  assert.equal(value.base.ref, resource.baseBranch);
  assert.equal(value.head.repo.full_name, fixture.repository);
  assert.match(value.head.ref, new RegExp('^issue-' + resource.issueNumber + '-[A-Za-z0-9-]+$'));
  if (resource.pullRequestNumber) assert.equal(value.number, resource.pullRequestNumber);
  if (closed) {
    assert.equal(value.state, 'closed');
    assert.equal(value.merged, false);
    assert.equal(value.draft, false);
    assert.equal(value.head.sha, fixture.ready.head);
    assert.equal(value.head.ref, fixture.ready.branch);
    assert.equal(value.base.repo.full_name, fixture.repository);
    assert.equal(fixture.ready.state, 'open');
    assert.equal(fixture.ready.draft, false);
    return value.head.sha;
  }
  return requireReadyPullRequest(value, fixture.repository, value.number);
}

export async function createIsolatedTask({context, task, inputHash, readme, issue, create, api, runGh, save}) {
  const language = languages[context.tool];
  assert.equal(hash(issue(language).body), task.bodySha256, 'pinned Hive generator must match the original task');
  const staged = issue(language, {workflowDir: 'ci/'});
  assert.equal(hash(originalWorkflowBody(staged.body)), task.bodySha256);
  const fixture = {
    schema: isolatedTaskSchema, ...context, inputSha256: inputHash,
    originalBodySha256: task.bodySha256, bodySha256: hash(staged.body),
    workflowDirectory: 'ci/', resource: null, ready: null, cleanup: null,
  };
  await create({
    repository: context.repository, baseBranch: isolatedTaskBase(context),
    readme: readme({repoName: context.repository.split('/')[1], randomLanguage: language}),
    ...staged, api, runGh,
    onResource(resource) {fixture.resource = structuredClone(resource); save(fixture);},
  });
  requireIsolatedTask(fixture, {...context, inputHash, task});
  requireIsolatedIssue(await api(`repos/${context.repository}/issues/${fixture.resource.issueNumber}`), fixture);
  save(fixture);
  console.log('Final issue URL: ' + fixture.resource.issueUrl);
  return fixture;
}

export function isolatedCleanupMarker(fixture) {
  return `<!-- formal-ai-isolated-cleanup run=${fixture.run} attempt=${fixture.attempt} tool=${fixture.tool} -->`;
}

export function isolatedTaskTransport(fixture) {
  assert(Number.isSafeInteger(fixture.resource.pullRequestNumber));
  return {kind: 'same-repository', repository: fixture.repository,
    issueUrl: fixture.resource.issueUrl,
    pullRequestUrl: `https://github.com/${fixture.repository}/pull/${fixture.resource.pullRequestNumber}`,
    workflowDirectory: 'ci/', workflowExecution: 'act', cleanup: 'after-owner-run',
    historicalExternalDeliveryVerified: false};
}

export function selectIsolatedFixture(records) {
  assert(Array.isArray(records) && records.length > 0 && records.length <= 16);
  const latest = records.at(-1);
  assert(latest && latest.schema === isolatedTaskSchema);
  for (const record of records) {
    assert(record && record.schema === isolatedTaskSchema);
    for (const key of ['repository', 'run', 'attempt', 'tool', 'source', 'protocolHead',
      'inputSha256', 'originalBodySha256', 'bodySha256', 'workflowDirectory']) {
      assert.equal(record[key], latest[key], 'fixture custody changed: ' + key);
    }
    for (const key of ['repository', 'baseBranch', 'issueNumber', 'issueUrl', 'pullRequestNumber']) {
      if (record.resource?.[key] !== undefined) assert.equal(record.resource[key], latest.resource?.[key]);
    }
    assert(Array.isArray(record.resource?.branches) && Array.isArray(latest.resource?.branches));
    assert(record.resource.branches.every(branch => latest.resource.branches.includes(branch)));
  }
  return structuredClone(latest);
}

export async function requireIsolatedOwner(fixture, {repository, run, job, client, recognizeIssue}) {
  assert.equal(run.status, 'completed', 'owner run must be terminal before cleanup');
  assert.equal(job.status, 'completed', 'owner job must be terminal before cleanup');
  assert.equal(fixture.schema, isolatedTaskSchema);
  assert.equal(fixture.repository, repository);
  assert.equal(fixture.run, run.id);
  assert.equal(fixture.attempt, run.run_attempt);
  assert.equal(fixture.protocolHead, run.head_sha);
  assert.equal(job.name, `Actual Hive ${fixture.tool} original delivery`);
  assert.equal(fixture.resource.baseBranch, isolatedTaskBase(fixture));
  assert.equal(fixture.resource.repository, repository);
  assert(Number.isSafeInteger(fixture.resource.issueNumber) && fixture.resource.issueNumber > 0);
  const root = 'repos/' + repository;
  const issue = await client.json(`${root}/issues/${fixture.resource.issueNumber}`);
  requireIsolatedIssue(issue, fixture);
  assert(recognizeIssue({...issue, state: 'open', comments: 0}), 'known task generator required');
  assert(Date.parse(issue.created_at) >= Date.parse(job.started_at)
    && Date.parse(issue.created_at) <= Date.parse(job.completed_at));
  const comments = await client.json(`${root}/issues/${issue.number}/comments?per_page=100`);
  assert(Array.isArray(comments) && comments.length === issue.comments && comments.length <= 100);
  assert(comments.every(comment => comment.user.login === 'github-actions[bot]'), 'human feedback preserves a fixture');
  return issue;
}

async function closeFixtureResources(resource, {api}) {
  const root = 'repos/' + resource.repository;
  const operations = [];
  if (resource.pullRequestNumber) operations.push([`${root}/pulls/${resource.pullRequestNumber}`, {method: 'PATCH', body: {state: 'closed'}}]);
  if (resource.issueNumber) operations.push([`${root}/issues/${resource.issueNumber}`, {method: 'PATCH', body: {state: 'closed'}}]);
  for (const branch of [...new Set(resource.branches)].reverse()) {
    operations.push([`${root}/git/refs/heads/${branch}`, {method: 'DELETE'}]);
  }
  const errors = [];
  for (const [endpoint, options] of operations) {
    try { await api(endpoint, options); }
    catch (error) { errors.push(String(error.message)); }
  }
  return {errors, retainedBranches: []};
}

/** Shared bounded cleanup follows Hive's issue/PR/branch lifecycle without retrying uncertain writes. */
export async function cleanupIsolatedTask(fixture, {api, cleanup = closeFixtureResources, save, outcome}) {
  const resource = fixture.resource;
  assert(resource && resource.repository === fixture.repository);
  assert.equal(resource.baseBranch, isolatedTaskBase(fixture));
  assert(Array.isArray(resource.branches) && resource.branches.length > 0 && resource.branches.length <= 2);
  assert(resource.branches.every(branch => branch === resource.baseBranch
    || new RegExp('^issue-' + resource.issueNumber + '-[A-Za-z0-9-]+$').test(branch)),
  'only owned task branches may be deleted');
  const root = 'repos/' + fixture.repository;
  if (resource.issueNumber) {
    requireIsolatedIssue(await api(`${root}/issues/${resource.issueNumber}`), fixture);
    const pulls = await api(`${root}/pulls?state=all&base=${encodeURIComponent(resource.baseBranch)}&per_page=100`);
    assert(Array.isArray(pulls) && pulls.length <= 1, 'one owned task PR at most');
    if (pulls.length) {
      const pull = pulls[0];
      assert.equal(pull.base.ref, resource.baseBranch);
      assert.equal(pull.base.repo.full_name, fixture.repository);
      assert.equal(pull.head.repo.full_name, fixture.repository);
      assert.equal(pull.merged, false, 'merged task PR must be preserved');
      assert.match(pull.head.ref, new RegExp('^issue-' + resource.issueNumber + '-[A-Za-z0-9-]+$'));
      resource.pullRequestNumber = pull.number;
      resource.branches = [...new Set([...resource.branches, pull.head.ref])];
      if (fixture.ready) assert.equal(pull.head.sha, fixture.ready.head, 'task changed before cleanup');
    }
    assert(resource.branches.every(branch => branch === resource.baseBranch
      || pulls.some(pull => pull.head.ref === branch)), 'task branch needs live PR corroboration');
    const comment = await api(`${root}/issues/${resource.issueNumber}/comments`, {method: 'POST', body: {
      body: isolatedCleanupMarker(fixture) + `\nOwned test fixture finished (${outcome}). Evidence: `
        + `https://github.com/${fixture.repository}/actions/runs/${fixture.run}`,
    }});
    fixture.cleanupComment = comment.id;
    save(fixture);
  }
  const result = await cleanup(resource, {api, summaryFile: null});
  assert.equal(result.errors.length, 0, 'owned fixture cleanup failed');
  assert.equal(result.retainedBranches.length, 0, 'owned fixture branches must be deleted');
  if (resource.issueNumber) {
    assert.equal((await api(`${root}/issues/${resource.issueNumber}`)).state, 'closed');
  }
  if (resource.pullRequestNumber) {
    assert.equal((await api(`${root}/pulls/${resource.pullRequestNumber}`)).state, 'closed');
  }
  for (const branch of resource.branches) {
    const refs = await api(`${root}/git/matching-refs/heads/${branch}`);
    assert(Array.isArray(refs) && !refs.some(ref => ref.ref === 'refs/heads/' + branch));
  }
  fixture.cleanup = {complete: true, outcome};
  save(fixture);
  return fixture;
}

/** A completed owner's log is corroborated against current GitHub state, never used as write authority. */
export async function verifyIsolatedCleanup(fixture, {repository, run, job, client, recognizeIssue}) {
  assert.equal(fixture.schema, isolatedTaskSchema);
  assert.equal(fixture.repository, repository);
  assert.equal(fixture.run, run.id);
  assert.equal(fixture.attempt, run.run_attempt);
  assert.equal(fixture.protocolHead, run.head_sha);
  assert.equal(job.name, `Actual Hive ${fixture.tool} original delivery`);
  assert.equal(fixture.cleanup?.complete, true);
  assert.equal(fixture.resource.baseBranch, isolatedTaskBase(fixture));
  assert.equal(fixture.resource.repository, repository);
  assert(Number.isSafeInteger(fixture.resource.issueNumber) && fixture.resource.issueNumber > 0);
  assert(Number.isSafeInteger(fixture.cleanupComment) && fixture.cleanupComment > 0);
  if (fixture.resource.pullRequestNumber !== undefined) {
    assert(Number.isSafeInteger(fixture.resource.pullRequestNumber) && fixture.resource.pullRequestNumber > 0);
  }
  const root = 'repos/' + repository;
  const issue = await client.json(`${root}/issues/${fixture.resource.issueNumber}`);
  requireIsolatedIssue(issue, fixture);
  assert(recognizeIssue({...issue, state: 'open', comments: 0}), 'known source-bound task generator required');
  assert.equal(issue.state, 'closed');
  assert(Date.parse(issue.created_at) >= Date.parse(job.started_at)
    && Date.parse(issue.created_at) <= Date.parse(job.completed_at));
  const comments = await client.json(`${root}/issues/${issue.number}/comments?per_page=100`);
  assert(Array.isArray(comments) && comments.length === issue.comments && comments.length <= 100);
  const recorded = comments.filter(comment => comment.id === fixture.cleanupComment);
  assert.equal(recorded.length, 1);
  assert.equal(recorded[0].user.login, 'github-actions[bot]');
  assert(recorded[0].body.startsWith(isolatedCleanupMarker(fixture) + '\n'));
  if (fixture.resource.pullRequestNumber) {
    const pull = await client.json(`${root}/pulls/${fixture.resource.pullRequestNumber}`);
    if (fixture.ready) requireIsolatedPullRequest(pull, fixture, {closed: true});
    else {
      assert.equal(pull.state, 'closed');
      assert.equal(pull.base.ref, fixture.resource.baseBranch);
      assert.match(pull.head.ref, new RegExp('^issue-' + issue.number + '-[A-Za-z0-9-]+$'));
    }
  }
  assert(fixture.resource.branches.length > 0 && fixture.resource.branches.length <= 2);
  for (const branch of fixture.resource.branches) {
    assert(branch === fixture.resource.baseBranch || new RegExp('^issue-' + issue.number + '-[A-Za-z0-9-]+$').test(branch));
    const refs = await client.json(`${root}/git/matching-refs/heads/${branch}`);
    assert(Array.isArray(refs) && !refs.some(ref => ref.ref === 'refs/heads/' + branch));
  }
  return {complete: true, number: issue.number};
}
