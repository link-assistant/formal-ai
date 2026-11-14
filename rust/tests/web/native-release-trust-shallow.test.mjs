import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { existsSync, mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const helper = fileURLToPath(new URL('../../../scripts/native-release-trust.mjs', import.meta.url));
const gitEnvironment = () => ({
  ...process.env, GIT_CONFIG_COUNT: '1',
  GIT_CONFIG_KEY_0: 'core.hooksPath', GIT_CONFIG_VALUE_0: '/dev/null'
});
const git = (cwd, ...args) => execFileSync('git', args, {
  env: gitEnvironment(),
  cwd, encoding: 'utf8', stdio: 'pipe', timeout: 5000, maxBuffer: 262144,
}).trim();
function fixture(t, { shallow = true, unrelated = false, detached = false, advance = false } = {}) {
  const directory = mkdtempSync(join(tmpdir(), 'trust-shallow-history-'));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const origin = join(directory, 'tiny-origin');
  mkdirSync(origin);
  git(origin, 'init', '--initial-branch=main');
  git(origin, 'config', 'user.name', 'Fixture');
  git(origin, 'config', 'user.email', 'fixture@example.invalid');
  writeFileSync(join(origin, 'source'), 'stable source\n');
  git(origin, 'add', '.');git(origin, 'commit', '-m', 'stable');
  const stable = git(origin, 'rev-parse', 'HEAD');
  git(origin, 'tag', 'v1.2.3');
  writeFileSync(join(origin, 'source'), 'current default source\n');
  git(origin, 'commit', '-am', 'default advances');
  let current = git(origin, 'rev-parse', 'HEAD');
  let release = stable;
  if (unrelated) {
    git(origin, 'checkout', '--orphan', 'unrelated');
    git(origin, 'rm', '-rf', '.');
    writeFileSync(join(origin, 'unrelated'), 'unrelated published source\n');
    git(origin, 'add', '.');git(origin, 'commit', '-m', 'unrelated root');
    release = git(origin, 'rev-parse', 'HEAD');
    git(origin, 'tag', '-f', 'v1.2.3');git(origin, 'checkout', 'main');
  }
  const checkout = join(directory, 'tiny-checkout');
  git(directory, 'clone', '--no-tags', '--branch', 'main', ...(shallow ? ['--depth', '1'] : []),
    pathToFileURL(origin).href, checkout);
  assert.equal(git(checkout, 'rev-parse', '--is-shallow-repository'), String(shallow));
  const selected = current;
  if (detached) {
    git(checkout, 'checkout', '--detach', selected);
    for (const ref of git(checkout, 'for-each-ref', '--format=%(refname)').split('\n').filter(Boolean))
      git(checkout, 'update-ref', '-d', ref);
  }
  if (advance) {
    writeFileSync(join(origin, 'source'), 'new independently observed default source\n');
    git(origin, 'commit', '-am', 'default advances after source selection');
    current = git(origin, 'rev-parse', 'HEAD');
  }
  const prefix = 'repos/fixture/repository';
  const endpoints = {
    [prefix + '/releases/tags/v1.2.3']: {
      tag_name: 'v1.2.3', draft: false, prerelease: false, published_at: '2026-10-01T00:00:00Z',
    },
    [prefix]: { default_branch: 'main' },
    [prefix + '/commits/main']: { sha: current },
    [prefix + '/commits/v1.2.3']: { sha: release },
  };
  const preload = join(directory, 'api-only.mjs');
  const observed = join(directory, 'fetch.jsonl'), packets = join(directory, 'packets.log');
  const metadataObserved = join(directory, 'api-requests.jsonl');
  const pausesObserved = join(directory, 'api-pauses.jsonl');
  writeFileSync(preload, `
    import childProcess from 'node:child_process';
    import {appendFileSync} from 'node:fs';
    import { syncBuiltinESMExports } from 'node:module';
    const original = childProcess.execFileSync;
    const originalSpawn = childProcess.spawnSync;
    const endpoints = JSON.parse(process.env.FORMAL_AI_SHALLOW_API);
    const attempts = new Map();
    let totalRequests = 0;
    if (process.env.FORMAL_AI_SHALLOW_API_MODE === 'expired') {
      let clockReads = 0;
      Object.defineProperty(globalThis, 'performance', {
        configurable: true,
        value: {now: () => clockReads++ === 0 ? 0 : 90000}
      });
    }
    if (['recover', 'exhausted'].includes(process.env.FORMAL_AI_SHALLOW_API_MODE)) {
      Atomics.wait = (array, index, value, milliseconds) => {
        appendFileSync(process.env.FORMAL_AI_API_PAUSES,
          JSON.stringify(milliseconds) + String.fromCharCode(10));
        return 'timed-out';
      };
    }
    function metadata(args, options, transport) {
      const normalized = args.length === 4 && args[1] === '--hostname' &&
        args[2] === 'github.com' ? [args[0], args[3]] : args;
      if (normalized.length !== 2 || normalized[0] !== 'api' ||
          !Object.hasOwn(endpoints, normalized[1])) {
        throw Error('unexpected metadata endpoint or mutating gh arguments');
      }
      const endpoint = normalized[1];
      const attempt = (attempts.get(endpoint) ?? 0) + 1;
      if (attempt > 3) throw Error('unbounded metadata retry');
      attempts.set(endpoint, attempt);
      totalRequests++;
      appendFileSync(process.env.FORMAL_AI_API_OBSERVED, JSON.stringify({
        endpoint, args, attempt, transport,
        timeout: options.timeout, maxBuffer: options.maxBuffer
      }) + String.fromCharCode(10));
      const mode = process.env.FORMAL_AI_SHALLOW_API_MODE;
      const timeout = mode === 'exhausted' || mode === 'recover' && totalRequests === 1;
      const bytes = Buffer.from(JSON.stringify(endpoints[endpoint]));
      return {timeout, output: options.encoding ? bytes.toString(options.encoding) : bytes};
    }
    const timedOut = () => Object.assign(Error('captured fixture gh timeout'), {
      code: 'ETIMEDOUT', status: null, signal: 'SIGTERM',
      stdout: Buffer.alloc(0), stderr: Buffer.from('fixture timeout')
    });
    childProcess.execFileSync = (command, args, options) => {
      if (command === 'git' && args[0] === 'fetch' && process.env.FORMAL_AI_OMIT_NEGOTIATION_TIP === 'true')
        args = args.filter(arg => arg !== '--negotiation-tip=HEAD');
      if (command === 'git' && args[0] === 'fetch')
        appendFileSync(process.env.FORMAL_AI_FETCH_OBSERVED, JSON.stringify({args,timeout:options.timeout})+String.fromCharCode(10));
      if (command !== 'gh') return original(command, args, options);
      const result = metadata(args, options, 'execFileSync');
      if (result.timeout) throw timedOut();
      return result.output;
    };
    childProcess.spawnSync = (command, args, options) => {
      if (command !== 'gh') return originalSpawn(command, args, options);
      const result = metadata(args, options, 'spawnSync');
      return result.timeout
        ? {status: null, signal: 'SIGTERM', error: timedOut(),
          stdout: Buffer.alloc(0), stderr: Buffer.from('fixture timeout')}
        : {status: 0, signal: null, stdout: result.output, stderr: Buffer.alloc(0)};
    };
    syncBuiltinESMExports();
  `);
  const output = join(directory, 'github-env');
  return { checkout, endpoints, prefix, output, release, current, selected, observed, packets, metadataObserved, pausesObserved,
    invoke: ({omitTip=false, apiMode='normal'} = {}) => spawnSync(process.execPath, ['--import', preload, helper], {
      cwd: checkout, encoding: 'utf8', timeout: 10000, maxBuffer: 262144,
      env: { ...gitEnvironment(), NATIVE_RELEASE_TAG: 'v1.2.3', NATIVE_REPOSITORY: 'fixture/repository',
        GITHUB_ENV: output, FORMAL_AI_SHALLOW_API: JSON.stringify(endpoints),
        FORMAL_AI_FETCH_OBSERVED: observed, GIT_TRACE_PACKET: packets,
        FORMAL_AI_API_OBSERVED: metadataObserved, FORMAL_AI_SHALLOW_API_MODE: apiMode,
        FORMAL_AI_API_PAUSES: pausesObserved,
        FORMAL_AI_OMIT_NEGOTIATION_TIP: String(omitTip) },
    }),
  };
}
test('actual authorization retries one captured metadata timeout before fresh ancestry checks', t => {
  const f = fixture(t);
  const result = f.invoke({apiMode: 'recover'});
  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.signal, null);
  const requests = readFileSync(f.metadataObserved, 'utf8').trim().split('\n').map(JSON.parse);
  const endpoints = Object.keys(f.endpoints);
  assert.deepEqual(requests.map(request => request.endpoint), [endpoints[0], ...endpoints]);
  assert.deepEqual(requests.slice(0, 2).map(request => request.attempt), [1, 2]);
  assert.deepEqual(
    readFileSync(f.pausesObserved, 'utf8').trim().split('\n').map(JSON.parse),
    [2000]
  );
  assert.ok(requests.every(request => ['execFileSync', 'spawnSync'].includes(request.transport)));
  assert.ok(requests.every(request => request.timeout > 0 && request.timeout <= 20000));
  assert.ok(requests.every(request => request.maxBuffer === 1048576));
  assert.match(result.stderr, /ETIMEDOUT/u);
  assert.match(result.stderr, /SIGTERM/u);
  assert.equal(git(f.checkout, 'merge-base', '--is-ancestor', f.release, f.current), '');
  assert.deepEqual(JSON.parse(result.stdout), {
    tag: 'v1.2.3', releaseCommit: f.release, defaultCommit: f.current
  });
  assert.match(readFileSync(f.output, 'utf8'), new RegExp(f.release));
});

test('actual authorization exhausts only bounded captured metadata reads without emitting authority', t => {
  const f = fixture(t);
  const result = f.invoke({apiMode: 'exhausted'});
  assert.notEqual(result.status, 0);
  assert.equal(result.signal, null);
  assert.equal(result.stdout, '');
  assert.equal(existsSync(f.output), false);
  assert.equal(existsSync(f.observed), false);
  const requests = readFileSync(f.metadataObserved, 'utf8').trim().split('\n').map(JSON.parse);
  assert.deepEqual(requests.map(request => request.attempt), [1, 2, 3]);
  assert.deepEqual(
    readFileSync(f.pausesObserved, 'utf8').trim().split('\n').map(JSON.parse),
    [2000, 4000]
  );
  assert.ok(requests.every(request => request.endpoint === Object.keys(f.endpoints)[0]));
});

test('actual authorization refuses an expired shared metadata deadline before any GitHub observation', t => {
  const f = fixture(t);
  const result = f.invoke({apiMode: 'expired'});
  assert.notEqual(result.status, 0);
  assert.equal(result.signal, null);
  assert.equal(result.stdout, '');
  assert.match(result.stderr, /deadline-expired/u);
  assert.equal(existsSync(f.output), false);
  assert.equal(existsSync(f.observed), false);
  assert.equal(existsSync(f.metadataObserved), false);
});

test('detached selected history is advertised while fetching a newly advanced default commit', t => {
  const f = fixture(t, {shallow:false,detached:true,advance:true});
  assert.equal(git(f.checkout, 'for-each-ref', '--format=%(refname)'), '');
  assert.notEqual(f.selected, f.current);
  const result = f.invoke();
  assert.equal(result.status, 0, result.stderr);
  const fetches = readFileSync(f.observed, 'utf8').trim().split('\n').map(JSON.parse);
  assert.equal(fetches.length, 1);
  assert(fetches[0].args.includes('--negotiation-tip=HEAD'));
  assert.equal(fetches[0].timeout, 240000);
  assert.match(readFileSync(f.packets, 'utf8'), new RegExp('have '+f.selected));
  assert.equal(git(f.checkout, 'rev-parse', '--is-shallow-repository'), 'false');
  assert.equal(git(f.checkout, 'merge-base', '--is-ancestor', f.release, f.current), '');
  assert.equal(git(f.checkout, 'show', f.selected+':source'), 'current default source');
  assert.equal(git(f.checkout, 'show', f.current+':source'), 'new independently observed default source');
  const control = fixture(t, {shallow:false,detached:true,advance:true});
  const original = control.invoke({omitTip:true});
  assert.equal(original.status, 0, original.stderr);
  assert.doesNotMatch(readFileSync(control.packets, 'utf8'), new RegExp('have '+control.selected),
    'the original detached checkout omitted its existing history from negotiation');
});
for (const shallow of [true, false]) {
  test(`actual helper authorizes a real ancestor with ${shallow ? 'depth-one' : 'full'} file transport history`, t => {
    const f = fixture(t, { shallow });
    const result = f.invoke();
    assert.equal(result.status, 0, result.stderr);
    const diagnostic = 'Completing shallow release history: tag=v1.2.3 release=' + f.release + ' default=' + f.current;
    assert.equal(result.stderr.includes(diagnostic), shallow);
    assert.equal(result.signal, null);
    assert.equal(git(f.checkout, 'rev-parse', '--is-shallow-repository'), 'false');
    assert.equal(git(f.checkout, 'merge-base', '--is-ancestor', f.release, f.current), '');
    assert.deepEqual(JSON.parse(result.stdout), { tag: 'v1.2.3', releaseCommit: f.release, defaultCommit: f.current });
    assert.equal(readFileSync(f.output, 'utf8'), 'NATIVE_AUTHORIZED_RELEASE_COMMIT=' + f.release + '\n'
      + 'NATIVE_AUTHORIZED_DEFAULT_COMMIT=' + f.current + '\n');
  });
}
test('real unrelated stable tag remains refused after shallow history repair', t => {
  const f = fixture(t, { unrelated: true });
  const result = f.invoke();
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /merge-base --is-ancestor/u);
  assert.equal(existsSync(f.output), false);
});
for (const [name, change] of [
  ['draft release', f => { f.endpoints[f.prefix + '/releases/tags/v1.2.3'].draft = true; }],
  ['prerelease', f => { f.endpoints[f.prefix + '/releases/tags/v1.2.3'].prerelease = true; }],
  ['mismatched published tag', f => { f.endpoints[f.prefix + '/releases/tags/v1.2.3'].tag_name = 'v9.9.9'; }],
  ['mismatched published commit', f => { f.endpoints[f.prefix + '/commits/v1.2.3'].sha = f.current; }],
]) {
  test(`actual shallow helper refuses ${name} without emitting authority`, t => {
    const f = fixture(t);change(f);
    const result = f.invoke();
    assert.notEqual(result.status, 0);
    assert.equal(result.signal, null);
    assert.equal(existsSync(f.output), false);
  });
}
