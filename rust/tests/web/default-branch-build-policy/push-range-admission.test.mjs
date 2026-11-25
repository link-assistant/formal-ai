// Issue #1637: a push onto a pull request head that already passed is judged by
// the pushed range alone, so a `.gitkeep` revert or a docs-only push no longer
// re-runs every check the pull request ever selected. Everything uncertain
// widens back to the full pull request diff.
import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import YAML from 'yaml';
import {admission, detectOutputs, isInertPath, pushRangeCandidate, previousHeadPassed, selectChangedPaths}
  from '../../../../scripts/ci-pull-request-admission.mjs';

const BASE = 'a'.repeat(40), BEFORE = 'b'.repeat(40), AFTER = 'c'.repeat(40), MERGE = 'd'.repeat(40);
const ENV = Object.freeze({
  GITHUB_EVENT_NAME: 'pull_request', GITHUB_EVENT_ACTION: 'synchronize',
  GITHUB_EVENT_BEFORE: BEFORE, GITHUB_EVENT_AFTER: AFTER,
  GITHUB_WORKFLOW_REF: 'link-assistant/formal-ai/.github/workflows/release.yml@refs/pull/1638/merge',
  GITHUB_REPOSITORY: 'link-assistant/formal-ai', GITHUB_TOKEN: 'token',
});
const FULL = ['rust/src/lib.rs', 'scripts/ci-pull-request-admission.mjs', '.gitkeep'];
const PUSHED = ['.gitkeep'];

function repository({parents = [MERGE, BASE, AFTER], ancestor = true} = {}) {
  const calls = [];
  const invoke = args => {
    calls.push(args.join(' '));
    if (args[0] === 'rev-list') return parents.join(' ') + '\n';
    if (args[0] === 'merge-base') { if (!ancestor) throw new Error('not an ancestor'); return ''; }
    assert.equal(args[0], 'diff');
    const [from, to] = args.slice(-2);
    if (from === BASE) return FULL.join('\0') + '\0';
    if (from === BEFORE && to === AFTER) return PUSHED.join('\0') + '\0';
    throw new Error('unexpected diff ' + from + '..' + to);
  };
  return {invoke, calls};
}
function api(body, {status = 200} = {}) {
  const requests = [];
  const fetcher = async (url, init) => {
    requests.push({url, init});
    const bytes = new TextEncoder().encode(JSON.stringify(body));
    return {status, redirected: false, body: new ReadableStream({start(c) { c.enqueue(bytes); c.close(); }})};
  };
  return {fetcher, requests};
}
const passed = {total_count: 1, workflow_runs: [{head_sha: BEFORE, conclusion: 'success'}]};
const quiet = () => {};

test('.gitkeep placeholders are inert wherever they live', () => {
  for (const path of ['.gitkeep', 'rust/src/.gitkeep', 'docs/x/.gitkeep']) assert.equal(isInertPath(path), true, path);
  for (const path of ['.gitkeeper', 'gitkeep', 'src/.gitkeep.rs', '.gitignore']) assert.equal(isInertPath(path), false, path);
});

test('a .gitkeep-only change selects no check, not even the native type check', () => {
  for (const verifiedPushRange of [false, true]) {
    const result = admission(['.gitkeep'], {verifiedPushRange});
    assert.deepEqual(Object.entries(result).filter(([, value]) => value), [], JSON.stringify(result));
  }
});

test('a docs-only change still reports docs without selecting code checks', () => {
  const result = admission(['docs/guide.md', 'changelog.d/20261125_note.md'], {verifiedPushRange: true});
  assert.equal(result['docs-changed'], true);
  for (const key of ['any-code-changed', 'rs-changed', 'js-changed', 'rust-changed', 'native-type-check'])
    assert.equal(result[key], false, key);
});

test('an empty range is unknown unless it is a verified push range', () => {
  assert.equal(admission([])['native-type-check'], true);
  assert.equal(admission([], {verifiedPushRange: true})['native-type-check'], false);
});

test('only an appended synchronize push of this workflow is a push-range candidate', () => {
  assert.deepEqual(pushRangeCandidate(ENV),
    {before: BEFORE, after: AFTER, workflow: 'release.yml', repository: 'link-assistant/formal-ai'});
  for (const change of [{GITHUB_EVENT_NAME: 'push'}, {GITHUB_EVENT_ACTION: 'opened'},
    {GITHUB_EVENT_ACTION: 'reopened'}, {GITHUB_EVENT_BEFORE: '0'.repeat(39)}, {GITHUB_EVENT_BEFORE: AFTER},
    {GITHUB_EVENT_AFTER: ''}, {GITHUB_WORKFLOW_REF: 'link-assistant/formal-ai/.github/workflows/../x.yml'},
    {GITHUB_WORKFLOW_REF: undefined}, {GITHUB_REPOSITORY: 'a/b/c'}])
    assert.equal(pushRangeCandidate({...ENV, ...change}), null, JSON.stringify(change));
});

test('a push onto a green head is judged by the pushed range alone', async () => {
  const {invoke} = repository(), {fetcher, requests} = api(passed);
  const selected = await selectChangedPaths({env: ENV, invoke, fetcher, log: quiet});
  assert.deepEqual(selected, {range: 'push', paths: PUSHED});
  assert.equal(requests.length, 1);
  assert.equal(requests[0].url, 'https://api.github.com/repos/link-assistant/formal-ai/actions/workflows/release.yml/runs?head_sha='
    + BEFORE + '&event=pull_request&status=success&per_page=1');
  assert.equal(requests[0].init.headers.Authorization, 'Bearer token');
  const result = admission(selected.paths, {verifiedPushRange: true});
  assert.equal(Object.values(result).some(Boolean), false);
});

test('every uncertainty widens back to the full pull request diff', async () => {
  const cases = {
    'not a synchronize push': [{env: {...ENV, GITHUB_EVENT_ACTION: 'opened'}}, 0],
    'merge commit built from another head': [{repo: {parents: [MERGE, BASE, BEFORE]}}, 0],
    'force push or rebase': [{repo: {ancestor: false}}, 0],
    'previous head never passed': [{body: {total_count: 0, workflow_runs: []}}, 1],
    'previous head run for another sha': [{body: {total_count: 1, workflow_runs: [{head_sha: BASE, conclusion: 'success'}]}}, 1],
    'previous head run not successful': [{body: {total_count: 1, workflow_runs: [{head_sha: BEFORE, conclusion: 'failure'}]}}, 1],
    'malformed API body': [{body: {workflow_runs: []}}, 1],
    'API denied': [{status: 403}, 1],
    'no token': [{env: {...ENV, GITHUB_TOKEN: '', GH_TOKEN: undefined}}, 0],
  };
  for (const [name, [{env = ENV, repo, body = passed, status}, expectedRequests]] of Object.entries(cases)) {
    const {invoke} = repository(repo), {fetcher, requests} = api(body, {status});
    const logs = [];
    const selected = await selectChangedPaths({env, invoke, fetcher, log: message => logs.push(message)});
    assert.deepEqual(selected, {range: 'full', paths: FULL}, name);
    assert.equal(requests.length, expectedRequests, name);
    assert.equal(admission(selected.paths)['any-code-changed'], true, name);
  }
});

test('a lookup that outlives its deadline is not trusted', async () => {
  const fetcher = (url, {signal}) => new Promise((resolve, reject) =>
    signal.addEventListener('abort', () => reject(new Error('aborted'))));
  assert.equal(await previousHeadPassed({repository: 'o/r', workflow: 'release.yml', sha: BEFORE, token: 't',
    fetcher, timeoutMs: 20}), null);
});

test('every pull request admission job can read its own previous runs', () => {
  for (const [file, job] of [['release.yml', 'detect-changes'], ['coverage.yml', 'detect-changes'], ['layered-ci.yml', 'changes']]) {
    const spec = YAML.parse(readFileSync('.github/workflows/' + file, 'utf8')).jobs[job];
    assert.deepEqual(spec.permissions, {contents: 'read', actions: 'read'}, file);
    assert.equal(spec.steps.find(step => step.uses?.startsWith('actions/checkout@')).with['fetch-depth'], 0, file);
    const detect = spec.steps.find(step => step.run?.includes('node scripts/ci-pull-request-admission.mjs detect'));
    assert.equal(detect.env.GITHUB_EVENT_ACTION, '${{ github.event.action }}', file);
    assert.equal(detect.env.GITHUB_EVENT_AFTER, '${{ github.event.after }}', file);
    assert.equal(detect.env.GITHUB_TOKEN, '${{ github.token }}', file);
  }
});

test('detect outputs keep whole-pull-request facts beside the narrowed range', async () => {
  const narrowed = await detectOutputs({env: ENV, invoke: repository().invoke, fetcher: api(passed).fetcher, log: quiet});
  assert.equal(narrowed['admission-range'], 'push');
  assert.equal(narrowed['any-code-changed'], false);
  assert.equal(narrowed['js-changed'], false);
  assert.equal(narrowed['pull-request-code-changed'], true, 'the changelog fragment is still judged on the whole pull request');
  const full = await detectOutputs({env: {...ENV, GITHUB_EVENT_ACTION: 'opened'}, invoke: repository().invoke,
    fetcher: api(passed).fetcher, log: quiet});
  assert.equal(full['admission-range'], 'full');
  assert.equal(full['any-code-changed'], true);
  assert.equal(full['pull-request-code-changed'], true);
});

test('narrowed pushes gate the JavaScript tier and never the changelog fragment', () => {
  const layered = YAML.parse(readFileSync('.github/workflows/layered-ci.yml', 'utf8')).jobs;
  assert.equal(layered.changes.outputs['admission-range'], '${{ steps.changes.outputs.admission-range }}');
  assert.match(layered.js.if, /needs\.changes\.outputs\.js-changed == 'true' \|\| \(github\.event_name == 'pull_request' && needs\.changes\.outputs\.admission-range != 'push'\)/u);
  const release = YAML.parse(readFileSync('.github/workflows/release.yml', 'utf8')).jobs;
  assert.equal(release['detect-changes'].outputs['pull-request-code-changed'], '${{ steps.changes.outputs.pull-request-code-changed }}');
  assert.match(release.changelog.if, /pull-request-code-changed == 'true'/u);
  assert.doesNotMatch(release.changelog.if, /any-code-changed/u);
});
