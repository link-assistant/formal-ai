import test from 'node:test';
import assert from 'node:assert/strict';
import { requireRegisteredTreeEntry, createGovernedGithubCommandProvider,
  runGovernedGithubRead } from '../../../scripts/governed-github-command-provider.mjs';
test('registered immutable tree source and bounded provider reject ambiguous authority', () => {
  const tree = {
    sha: 'a'.repeat(40),
    truncated: false,
    tree: [{
      path: 'release.yml',
      type: 'blob',
      mode: '100644',
      sha: 'b'.repeat(40)
    }]
  };
  assert.equal(requireRegisteredTreeEntry(tree, 'a'.repeat(40), 'release.yml', 'blob', '100644').sha, 'b'.repeat(40));
  let count = 0;
  for (const mutate of [t => t.sha = 'c'.repeat(40), t => t.truncated = true, t => t.tree.push({
    ...t.tree[0]
  }), t => t.tree[0].mode = '120000', t => t.tree[0].mode = '160000', t => t.tree[0].type = 'commit', t => t.tree[0].sha = 'unknown', t => t.tree[0].path = 'nested/release.yml']) {
    const changed = structuredClone(tree);
    mutate(changed);
    assert.throws(() => requireRegisteredTreeEntry(changed, 'a'.repeat(40), 'release.yml', 'blob', '100644'));
    count++;
  }
  for (const name of ['nested/release.yml', '..', '.']) {
    assert.throws(() => requireRegisteredTreeEntry(tree, 'a'.repeat(40), name, 'blob', '100644'));
    count++;
  }
  const expected = {
    repository: 'owner/repo',
    head: 'a'.repeat(40),
    run: '123',
    attempt: '1',
    workflowPath: '.github/workflows/release.yml'
  };
  for (const paths of [['.github/workflows/release.yml', '.github/workflows/release.yml'], ['.github/workflows/release.yml', '.github/workflows/nested/release-staged.yml'], ['.github/workflows/release.yml', '../release-staged.yml']]) {
    assert.throws(() => createGovernedGithubCommandProvider({
      expected,
      sourcePaths: paths,
      writerGroup: 'declared-writer'
    }));
    count++;
  }
  assert.equal(count, 14);
});

test('hosted governed provider refuses absent or blank explicit credential before transport',async()=>{
 const {spawnSync}=await import('node:child_process');
 const {fileURLToPath}=await import('node:url');const {dirname,resolve}=await import('node:path');
 const root=resolve(dirname(fileURLToPath(import.meta.url)),'../../..');
 const source="import {createGovernedGithubCommandProvider} from './scripts/governed-github-command-provider.mjs';createGovernedGithubCommandProvider({expected:{repository:'owner/repo',head:'a'.repeat(40),run:'1',attempt:'1',workflowPath:'.github/workflows/release.yml'},sourcePaths:['.github/workflows/release.yml','.github/workflows/release-staged.yml'],writerGroup:'writer'});";
 for(const credential of [undefined,'','   ']) {
  const environment={...process.env,GITHUB_ACTIONS:'true'};if(credential===undefined)delete environment.GH_TOKEN;else environment.GH_TOKEN=credential;
  const result=spawnSync(process.execPath,['--input-type=module','-e',source],{cwd:root,env:environment,encoding:'utf8',timeout:5000});
  assert.notEqual(result.status,0);assert.match(result.stderr,/explicit hosted GitHub read credential required/);assert.equal(result.stdout,'');
 }
});

function wire(status, body, headers = '') {
  return {status: status === 200 ? 0 : 1, signal: null,
    stdout: Buffer.from(`HTTP/2.0 ${status} Response\r\n${headers}\r\n${JSON.stringify(body)}`),
    stderr: Buffer.alloc(0)};
}
const timeout = () => ({status: null, signal: 'SIGTERM', error: {code: 'ETIMEDOUT'},
  stdout: Buffer.alloc(0), stderr: Buffer.from('private stderr sentinel')});
function transport(results, options = {}) {
  let clock = 0, calls = 0, guards = 0;
  const result = runGovernedGithubRead('/verified/gh', ['api', '--include', '--method', 'GET', '/registered/source'],
    {deadline: 30000, maximumAttempts: 2, ...options}, {now: () => clock,
      beforeAttempt() {guards++;}, spawn(file, args, settings) {
        assert.equal(file, '/verified/gh'); assert(args.includes('--include'));
        assert.equal(args[args.indexOf('--method') + 1], 'GET');
        assert.equal(settings.shell, false); assert(settings.timeout <= 5000);
        assert.equal(settings.killSignal, 'SIGKILL');
        clock += 1000; return results[calls++];
      }});
  return {result, calls, guards};
}
test('an unavailable immutable read gets one bounded retry and retains transport diagnostics', () => {
  const {result, calls, guards} = transport([timeout(), wire(200, {sha: 'a'.repeat(40)})]);
  assert.equal(calls, 2); assert.equal(guards, 2); assert.equal(result.status, 200);
  assert.equal(result.attempts[0].errorCode, 'ETIMEDOUT');
  assert.equal(result.attempts[0].timeoutMilliseconds, 5000);
  assert.equal(result.attempts[1].httpStatus, 200);
  assert(!JSON.stringify(result.attempts).includes('private stderr sentinel'));
});
test('repeated transport failures stop after two attempts and live observations remain single-attempt', () => {
  assert.equal(transport([timeout(), timeout()]).calls, 2);
  const live = transport([timeout()], {maximumAttempts: 1});
  assert.equal(live.calls, 1); assert.equal(live.result.status, 0);
  assert.throws(() => transport([], {maximumAttempts: Infinity}));
});
test('HTTP status comes from response headers, not a claimed success property in an error body', () => {
  const denied = transport([wire(403, {status: 200}, 'x-ratelimit-remaining: 0\r\nx-ratelimit-reset: 123\r\n')]);
  assert.equal(denied.calls, 1); assert.equal(denied.result.status, 403);
  assert.equal(denied.result.attempts[0].rateLimitRemaining, 0);
  assert.equal(denied.result.attempts[0].rateLimitReset, 123);
  const unavailable = transport([{status: 1, signal: null, stdout: Buffer.from('{"status":200}'), stderr: Buffer.alloc(0)}, timeout()]);
  assert.equal(unavailable.result.status, 0);
});
test('transient HTTP service failures retry once; denied, missing and malformed responses cannot establish authority', () => {
  assert.equal(transport([wire(503, {message: 'unavailable'}), wire(200, {id: 1})]).result.status, 200);
  for (const status of [401, 403, 404]) {
    const value = transport([wire(status, {message: 'unavailable'})]);
    assert.equal(value.calls, 1); assert.equal(value.result.status, status);
  }
  const invalid = wire(200, {}); invalid.stdout = Buffer.from('HTTP/2.0 200 OK\r\n\r\n{invalid');
  const value = transport([invalid]); assert.equal(value.calls, 1); assert.equal(value.result.status, 0);
});
test('aggregate deadline includes the executable guard and refuses another attempt after exhaustion', () => {
  let now = 0, calls = 0;
  const result = runGovernedGithubRead('/verified/gh', ['api'], {deadline: 6000, maximumAttempts: 2}, {
    now: () => now, beforeAttempt() {now += 4000;}, spawn(_file, _args, settings) {
      calls++; assert.equal(settings.timeout, 2000); now += settings.timeout; return timeout();
    }});
  assert.equal(calls, 1); assert.equal(result.status, 0);
});
test('the executable is revalidated before retry and a changed binary never executes', () => {
  let guards = 0, calls = 0;
  assert.throws(() => runGovernedGithubRead('/verified/gh', ['api'], {deadline: 30000, maximumAttempts: 2}, {
    now: () => 0, beforeAttempt() {if (++guards === 2) throw new Error('executable changed');},
    spawn() {calls++; return timeout();}}), /executable changed/u);
  assert.equal(calls, 1);
});
