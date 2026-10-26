import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {createIsolatedTask, cleanupIsolatedTask, requireIsolatedTask, requireIsolatedIssue,
  requireIsolatedPullRequest, originalWorkflowBody, isolatedTaskTransport,
  verifyIsolatedCleanup, requireIsolatedOwner, selectIsolatedFixture,
  isolatedFixturePrefix} from '../../../scripts/lib/hive-isolated-task.mjs';
import {generatedIssue} from '../../../scripts/lib/generated-task-custody.mjs';
import {finalizeGeneratedRun} from '../../../scripts/finalize-generated-test-issues.mjs';

const captured = JSON.parse(readFileSync(new URL('../fixtures/hive-default-token-sources.json', import.meta.url)));
const manifestBytes = readFileSync(new URL('../../../data/benchmarks/issue-1364-client-replay-inputs.json', import.meta.url));
const manifest = JSON.parse(manifestBytes);
const sha256 = value => createHash('sha256').update(value).digest('hex');

// Only module IO bindings change; the pinned Hive fixture functions execute unchanged.
function sourceModule(module, exports) {
  const bytes = Buffer.from(module.source);
  assert.equal(createHash('sha1').update('blob ' + bytes.length + '\0').update(bytes).digest('hex'), module.blob);
  const source = module.source.replace(/^import [^\n]+;\n/gmu, '').replace(/^export /gmu, '');
  const unexpected = () => {throw new Error('Unmocked fixture IO');};
  return new Function('appendFileSync', 'ghApi', 'gh', 'isBranchDeletionRuleViolation', 'process',
    source + '\nreturn {' + exports.join(',') + '};')(
      unexpected, unexpected, unexpected, text => text.includes('repository rule'), {env: {}});
}
const generator = sourceModule(captured.modules.generator, ['helloWorldIssue', 'helloWorldReadme']);
const hive = sourceModule(captured.modules.fixture, ['createBranchFixture', 'cleanupBranchFixture']);
const context = {repository: 'host/project', run: 700, attempt: 1, tool: 'codex',
  source: 'a'.repeat(40), protocolHead: 'b'.repeat(40)};
const inputHash = sha256(manifestBytes);

function github(repository = context.repository) {
  const state = {calls: [], refs: new Set(), comments: [], issue: null, pull: null, fail: null};
  const root = 'repos/' + repository + '/';
  const api = async (endpoint, options = {}) => {
    const method = options.method ?? 'GET';
    assert(endpoint.startsWith(root), 'no operation may touch another repository');
    const route = endpoint.slice(root.length);
    state.calls.push({method, route, body: options.body});
    if (state.fail?.(method, route)) throw new Error('simulated transport failure');
    if (method === 'POST' && ['git/blobs', 'git/trees', 'git/commits'].includes(route)) {
      if (route === 'git/commits') assert.deepEqual(options.body.parents, [], 'fixture must be orphaned');
      return {sha: 'c'.repeat(40)};
    }
    if (method === 'POST' && route === 'git/refs') {
      state.refs.add(options.body.ref); return {ref: options.body.ref};
    }
    if (method === 'POST' && route === 'issues') {
      state.issue = {number: 17, html_url: `https://github.com/${repository}/issues/17`,
        ...options.body, user: {login: 'github-actions[bot]'}, state: 'open',
        comments: 0, created_at: '2026-10-24T00:01:00Z'};
      return structuredClone(state.issue);
    }
    if (method === 'POST' && route === 'issues/17/comments') {
      const comment = {id: 500, ...options.body, user: {login: 'github-actions[bot]'}};
      state.comments.push(comment); state.issue.comments++; return comment;
    }
    if (method === 'PATCH' && route === 'pulls/18') {state.pull.state = options.body.state; return state.pull;}
    if (method === 'PATCH' && route === 'issues/17') {state.issue.state = options.body.state; return state.issue;}
    if (method === 'DELETE' && route.startsWith('git/refs/heads/')) {
      state.refs.delete('refs/heads/' + route.slice('git/refs/heads/'.length)); return null;
    }
    if (route === 'issues/17') return structuredClone(state.issue);
    if (route === 'pulls/18') return structuredClone(state.pull);
    if (route.startsWith('pulls?')) {
      if(!state.pull)return [];
      const listed=structuredClone(state.pull);delete listed.merged;
      return [listed];
    }
    if (route === 'issues/17/comments?per_page=100') return structuredClone(state.comments);
    if (route.startsWith('git/matching-refs/heads/')) {
      const prefix = 'refs/heads/' + route.slice('git/matching-refs/heads/'.length);
      return [...state.refs].filter(ref => ref.startsWith(prefix)).map(ref => ({ref}));
    }
    throw new Error('Unmocked API request: ' + method + ' ' + route);
  };
  return {state, api};
}

async function prepared(repository = context.repository) {
  const localContext = {...context, repository};
  const {state, api} = github(repository);
  const saved = [];
  const fixture = await createIsolatedTask({context: localContext, task: manifest.clients.codex, inputHash,
    readme: generator.helloWorldReadme, issue: generator.helloWorldIssue,
    create: hive.createBranchFixture, api, save: value => saved.push(structuredClone(value)),
    runGh: async args => {
      assert.equal(args[0], 'label'); assert.equal(args[1], 'create');
      assert.equal(args[args.indexOf('--repo') + 1], repository);
    }});
  state.pull = {number: 18, state: 'open', draft: false, merged: false, merged_at: null,
    html_url: `https://github.com/${repository}/pull/18`,
    head: {ref: 'issue-17-fixture', sha: 'd'.repeat(40), repo: {full_name: repository}},
    base: {ref: fixture.resource.baseBranch, repo: {full_name: repository}}};
  state.refs.add('refs/heads/' + state.pull.head.ref);
  fixture.ready = {head: state.pull.head.sha, branch: state.pull.head.ref, state: 'open', draft: false};
  return {fixture, state, api, saved};
}

test('pinned Hive tasks preserve every original requirement while staging workflows for the default token', () => {
  for (const [tool, language] of Object.entries({claude: 'Kotlin', agent: 'Scala', codex: 'Rust'})) {
    const staged = generator.helloWorldIssue(language, {workflowDir: 'ci/'}).body;
    assert.equal(sha256(originalWorkflowBody(staged)), manifest.clients[tool].bodySha256);
    assert.throws(() => originalWorkflowBody(staged.replace('ci/', 'other/')));
    assert.notEqual(sha256(originalWorkflowBody(staged.replace('standard library', 'any library'))),
      manifest.clients[tool].bodySha256);
  }
});

test('actual pinned Hive fixture creation uses an orphan tree in the current repository', async () => {
  const {fixture, state, saved} = await prepared();
  requireIsolatedTask(fixture, {...context, inputHash, task: manifest.clients.codex});
  requireIsolatedIssue(state.issue, fixture);
  requireIsolatedPullRequest(state.pull, fixture);
  assert.equal(fixture.resource.baseBranch, 'e2e/hello-world/700-1/codex-formal-ai');
  assert(saved.some(value => value.resource.issueNumber === undefined), 'partial branch custody retained');
  assert.equal(state.calls.filter(call => call.method === 'POST' && call.route === 'issues').length, 1);
  for (const key of ['repository', 'run', 'attempt', 'source', 'protocolHead']) {
    const value = structuredClone(fixture); value[key] = 'another-owner';
    assert.throws(() => requireIsolatedTask(value, {...context, inputHash, task: manifest.clients.codex}));
  }
});

test('successful fixtures close the issue and PR, delete both branches, and are independently corroborated', async () => {
  const {fixture, state, api} = await prepared();
  await cleanupIsolatedTask(fixture, {api, save() {}, outcome: 'success'});
  assert.equal(state.issue.state, 'closed'); assert.equal(state.pull.state, 'closed');
  assert.equal(state.refs.size, 0); assert.equal(fixture.cleanup.complete, true);
  const transport = isolatedTaskTransport(fixture);
  assert.equal(transport.historicalExternalDeliveryVerified, false);
  assert.equal(transport.workflowExecution, 'act');
  assert.equal(transport.cleanup, 'after-owner-run');
  assert.equal(requireIsolatedPullRequest(state.pull, fixture, {closed: true}), fixture.ready.head);
  const before = state.calls.length;
  const result = await verifyIsolatedCleanup(fixture, {repository: context.repository,
    run: {id: 700, run_attempt: 1, head_sha: context.protocolHead},
    job: {name: 'Actual Hive codex original delivery', started_at: '2026-10-24T00:00:00Z',
      completed_at: '2026-10-24T00:10:00Z'}, client: {json: api}, recognizeIssue: generatedIssue});
  assert.equal(result.complete, true);
  assert(state.calls.slice(before).every(call => call.method === 'GET'), 'cleanup acknowledgment is read-only');
});

test('fresh PR detail is mandatory before cleanup and must match the list head', async () => {
  for(const mutation of [
    pull=>{delete pull.merged;},
    pull=>{pull.merged=true;},
    pull=>{pull.merged_at='2026-10-25T20:14:56Z';},
    pull=>{pull.head.sha='f'.repeat(40);},
  ]) {
    const {fixture,state,api}=await prepared();
    const before=state.calls.length;
    const observed=async(endpoint,options)=>{
      const result=await api(endpoint,options);
      if(endpoint.endsWith('/pulls/18')&&(options?.method??'GET')==='GET')mutation(result);
      return result;
    };
    await assert.rejects(cleanupIsolatedTask(fixture,{api:observed,save(){},outcome:'failure'}));
    assert(state.calls.slice(before).every(call=>call.method==='GET'));
    assert.equal(state.issue.state,'open');assert.equal(state.pull.state,'open');
    assert.equal(state.refs.size,2);
  }
});

test('task body edits and a changed task head stop cleanup before any close or deletion', async () => {
  for (const mutate of [state => {state.issue.body += '\nHuman edit';},
    state => {state.pull.head.sha = 'f'.repeat(40);}]) {
    const {fixture, state, api} = await prepared(); mutate(state);
    const before = state.calls.length;
    await assert.rejects(cleanupIsolatedTask(fixture, {api, cleanup: hive.cleanupBranchFixture,
      save() {}, outcome: 'failure'}));
    assert(state.calls.slice(before).every(call => call.method === 'GET'));
    assert.equal(state.issue.state, 'open'); assert.equal(state.refs.size, 2);
  }
});

test('failed cleanup is not called complete and uncertain branch deletions are not retried', async () => {
  const {fixture, state, api} = await prepared();
  state.fail = (method, route) => method === 'DELETE' && route.includes('issue-17-fixture');
  await assert.rejects(cleanupIsolatedTask(fixture, {api,
    save() {}, outcome: 'failure'}), /cleanup failed/u);
  assert.equal(fixture.cleanup, null);
  assert.equal(state.calls.filter(call => call.method === 'DELETE' && call.route.includes('issue-17-fixture')).length, 1);
  assert.equal(isolatedTaskTransport(fixture).cleanup, 'after-owner-run');
});

test('a stale owner, forged cleanup comment, or retained branch cannot establish terminal cleanup', async () => {
  for (const mutate of [
    fixture => {fixture.protocolHead = 'f'.repeat(40);},
    (fixture, state) => {state.comments[0].user.login = 'human';},
    (fixture, state) => {state.refs.add('refs/heads/' + fixture.resource.baseBranch);},
  ]) {
    const {fixture, state, api} = await prepared();
    await cleanupIsolatedTask(fixture, {api, cleanup: hive.cleanupBranchFixture, save() {}, outcome: 'failure'});
    mutate(fixture, state);
    await assert.rejects(verifyIsolatedCleanup(fixture, {repository: context.repository,
      run: {id: 700, run_attempt: 1, head_sha: context.protocolHead},
      job: {name: 'Actual Hive codex original delivery', started_at: '2026-10-24T00:00:00Z',
        completed_at: '2026-10-24T00:10:00Z'}, client: {json: api}, recognizeIssue: generatedIssue}));
  }
});

test('fixture custody stays bounded and monotonic and never authorizes unrelated branch deletion', async () => {
  const {fixture, saved, api, state} = await prepared();
  assert.deepEqual(selectIsolatedFixture([...saved, fixture]), fixture);
  assert.throws(() => selectIsolatedFixture(Array(17).fill(fixture)));
  const changed = structuredClone(fixture); changed.resource.issueNumber = 99;
  assert.throws(() => selectIsolatedFixture([changed, fixture]));
  fixture.resource.branches.push('main');
  const before = state.calls.length;
  await assert.rejects(cleanupIsolatedTask(fixture, {api, save() {}, outcome: 'failure'}), /owned task branches/u);
  assert.equal(state.calls.length, before);
});

test('only a terminal owner without human feedback authorizes fixture cleanup', async () => {
  const {fixture, state, api} = await prepared();
  const owner = {repository: context.repository, client: {json: api}, recognizeIssue: generatedIssue,
    run: {id: 700, run_attempt: 1, head_sha: context.protocolHead, status: 'completed'},
    job: {name: 'Actual Hive codex original delivery', status: 'completed',
      started_at: '2026-10-24T00:00:00Z', completed_at: '2026-10-24T00:10:00Z'}};
  await requireIsolatedOwner(fixture, owner);
  await assert.rejects(requireIsolatedOwner(fixture, {...owner, run: {...owner.run, status: 'in_progress'}}));
  state.comments.push({user: {login: 'human'}, body: 'Keep this task'}); state.issue.comments++;
  await assert.rejects(requireIsolatedOwner(fixture, owner), /human feedback/u);
});

test('terminal success and cancellation finalize only the current repository fixture after full verification', async () => {
  for (const conclusion of ['success', 'cancelled']) {
    const repository = 'link-assistant/formal-ai';
    const {fixture, api, state} = await prepared(repository);
    const run = {id: 700, run_attempt: 1, head_sha: context.protocolHead, status: 'completed', conclusion,
      repository: {full_name: repository}, path: '.github/workflows/hive-original-delivery.yml'};
    const job = {id: 90, name: 'Actual Hive codex original delivery', status: 'completed', conclusion,
      started_at: '2026-10-24T00:00:00Z', completed_at: '2026-10-24T00:10:00Z'};
    const client = {
      transferredBytes: () => 0,
      async json(endpoint) {
        if (endpoint.endsWith('/actions/runs/700')) return run;
        if (endpoint.includes('/actions/runs/700/jobs?')) return {total_count: 1, jobs: [job]};
        return api(endpoint);
      },
      async request(endpoint, options) {
        if (options.log) return {ok: true, streamComplete: true, tailComplete: true,
          text: isolatedFixturePrefix + JSON.stringify(fixture)};
        const mutation = options.mutation, body = {};
        for (let index = 2; index < mutation.length; index += 2) {
          const value = mutation[index + 1], split = value.indexOf('=');
          body[value.slice(0, split)] = value.slice(split + 1);
        }
        const value = await api(endpoint, {method: mutation[1], body});
        return {ok: true, text: value ? JSON.stringify(value) : ''};
      },
    };
    const dry = await finalizeGeneratedRun({runId: 700, client});
    assert.equal(dry.discovery[0].status, 'would-close-owned-fixture');
    assert.equal(state.issue.state, 'open'); assert.equal(state.pull.state, 'open');
    const result = await finalizeGeneratedRun({runId: 700, client, apply: true});
    assert.equal(result.cleanupReceipt.complete, true);
    assert.equal(result.discovery[0].status, 'closed-owned-fixture');
    assert.equal(state.issue.state, 'closed'); assert.equal(state.pull.state, 'closed');
    assert.equal(state.refs.size, 0);
  }
});
