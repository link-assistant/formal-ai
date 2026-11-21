import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
const root=path.resolve(import.meta.dirname,'../../..');
const YAML=createRequire(import.meta.url)('yaml');
const action=YAML.parse(fs.readFileSync(path.join(root,'.github/actions/setup-sccache/action.yml'),'utf8'));
const step=name=>action.runs.steps.find(row=>row.name===name);
const cache=`#!/bin/sh
case "$1" in
  --start-server)
    echo 'cache diagnostic https://cache.example.invalid/blob?sig=fixture-secret'
    case "$CACHE_SCENARIO" in unavailable|already-running) exit 2;; esac
    exit 0;;
  --show-stats)
    [ "$CACHE_SCENARIO" != telemetry-failure ] || exit 2
    echo 'Compile requests 1'; exit 0;;
  *)
    if [ "$CACHE_SCENARIO" = unavailable ]; then
      echo 'ServerBusy: https://cache.example.invalid/blob?sig=fixture-secret' >&2
      exit 2
    fi
    exec "$@";;
esac
`;
const compiler=`#!/bin/sh
if [ "$1" = -vV ]; then
  echo 'fixture compiler metadata'; exit 0
fi
echo 'fixture compiler rejected the source' >&2
exit 23
`;
function runSetup(t,scenario){
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'sccache-probe-'));
  t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  const bin=path.join(directory,'tool directory');fs.mkdirSync(bin);
  for(const[name,source]of[['sccache',cache],['rustc',compiler]]){
    fs.writeFileSync(path.join(bin,name),source,{mode:0o700});
  }
  const environmentFile=path.join(directory,'environment');fs.writeFileSync(environmentFile,'');
  const enable=step('Enable sccache for Rust');
  let environment={...process.env,...enable.env,PATH:bin+path.delimiter+process.env.PATH,
    CACHE_SCENARIO:scenario,GITHUB_ENV:environmentFile};
  const applyEnvironment=()=>{
    for(const line of fs.readFileSync(environmentFile,'utf8').split('\n').filter(Boolean)){
      const split=line.indexOf('=');assert(split>0);
      environment[line.slice(0,split)]=line.slice(split+1);
    }
  };
  const execute=source=>spawnSync('bash',['-e','-c',source],{
    cwd:root,env:environment,encoding:'utf8',timeout:10000,maxBuffer:131072,
  });
  const enabled=execute(enable.run);assert.equal(enabled.status,0,enabled.stderr);applyEnvironment();
  const probe=execute(step('Start the sccache server').run);
  assert.equal(probe.status,0,probe.stderr);applyEnvironment();
  assert(!probe.stdout.includes('fixture-secret'));
  assert(!probe.stderr.includes('fixture-secret'));
  assert(probe.stdout.includes('<cache-url-redacted>'));
  const executable=environment.RUSTC_WRAPPER||'rustc';
  const args=environment.RUSTC_WRAPPER?['rustc','--compile-fixture']:['--compile-fixture'];
  const compile=spawnSync(executable,args,{env:environment,encoding:'utf8',timeout:10000});
  assert.equal(compile.status,23,'actual compiler failure must remain fatal');
  assert.match(compile.stderr,/fixture compiler rejected the source/u);
  return{environment,stdout:probe.stdout};
}
for(const scenario of ['healthy','already-running','telemetry-failure']){
  test(`real setup retains a usable compiler cache: ${scenario}`,t=>{
    const observed=runSetup(t,scenario);
    assert.equal(observed.environment.RUSTC_WRAPPER,'sccache');
    assert.equal(observed.environment.SCCACHE_IGNORE_SERVER_IO_ERROR,'1');
    assert(!observed.stdout.includes('subsequent Cargo steps use the real compiler'));
  });
}
test('real setup falls back to the real compiler after storage startup failure',t=>{
  const observed=runSetup(t,'unavailable');
  assert.equal(observed.environment.RUSTC_WRAPPER,'');
  assert.match(observed.stdout,/subsequent Cargo steps use the real compiler without cache/u);
});
test('Desktop reuses the shared setup after its original pinned installer',()=>{
  const workflow=YAML.parse(fs.readFileSync(path.join(root,'.github/workflows/desktop-release.yml'),'utf8'));
  const steps=workflow.jobs.native.steps;
  const install=steps.find(row=>row.name==='Cache Rust compiler outputs');
  const enable=steps.find(row=>row.name==='Enable Rust compiler cache');
  assert.equal(install.uses,'mozilla-actions/sccache-action@v0.0.11');
  assert.equal(install.with.version,'v0.16.0');
  assert.equal(enable.if,install.if);
  assert.equal(enable.if,"runner.os != 'Windows' && matrix.target != 'x86_64-apple-darwin'");
  assert.equal(enable.uses,'./.github/actions/setup-sccache');
  assert.deepEqual(enable.with,{install:'false'});
  assert.equal(action.inputs.install.default,'true');
  assert.equal(step('Install sccache').if,"inputs.install != 'false'");
  assert.equal(step('Install sccache').with.version,'v0.16.0');
  assert(steps.indexOf(install)<steps.indexOf(enable));
});

function cacheWriteReport(t, stats, options = {}) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'sccache-write-report-'));
  t.after(() => fs.rmSync(directory, { recursive: true, force: true }));
  const summary = path.join(directory, 'summary.md');
  const backendLog = path.join(directory, 'cache server.log');
  if (options.log !== undefined) fs.writeFileSync(backendLog, options.log);
  const result = spawnSync('bash', ['scripts/check-sccache-write-health.sh', '-'], {
    cwd: root,
    input: typeof stats === 'string' ? stats : JSON.stringify(stats),
    encoding: 'utf8',
    timeout: 5000,
    maxBuffer: 131072,
    env: {
      ...process.env,
      SCCACHE_WRITE_ERROR_WARN_PERCENT: '25',
      SCCACHE_WRITE_MIN_ATTEMPTS: '20',
      SCCACHE_HEALTH_LABEL: 'fixture',
      FORMAL_AI_CI_VERBOSE: options.verbose ? 'true' : 'false',
      SCCACHE_ERROR_LOG: backendLog,
      GITHUB_STEP_SUMMARY: summary,
    },
  });
  assert.equal(result.status, 0, result.stderr);
  return { ...result, summary: fs.existsSync(summary) ? fs.readFileSync(summary, 'utf8') : '' };
}

test('cache write errors report measured counts without inventing future cache misses', t => {
  const observed = cacheWriteReport(t, { cache_writes: 1, cache_write_errors: 87 });
  assert.match(observed.stderr, /::warning title=fixture: Compiler cache reported errors for 98%/u);
  assert.match(observed.stderr, /88 cache writes: 87 recorded errors and 1 were stored/u);
  assert.match(observed.stderr, /another writer may already have stored the same key/u);
  assert.doesNotMatch(observed.stderr, /Every refused write is a crate the next run compiles again/u);
  assert.equal(observed.summary, '- Compiler cache writes (fixture): 1 stored, 87 write errors, 98% errors.\n');
});

test('pretty JSON statistics retain the same write-error warning', t => {
  const observed = cacheWriteReport(t, JSON.stringify({ cache_writes: 60, cache_write_errors: 20 }, null, 2));
  assert.match(observed.stderr, /reported errors for 25%/u);
  assert.match(observed.summary, /60 stored, 20 write errors/u);
});

test('healthy, undersampled and unrelated statistics do not create cache warnings', t => {
  for (const stats of [
    { cache_writes: 100, cache_write_errors: 0 },
    { cache_writes: 0, cache_write_errors: 19 },
    { cache_writes_total: 0, cache_write_errors: 100 },
  ]) {
    const observed = cacheWriteReport(t, stats);
    assert.equal(observed.stderr, '');
    assert.equal(observed.summary, '');
  }
});

test('backend response is retained only in opt-in diagnostics, even without statistics', t => {
  const backend='::error::untrusted backend annotation\nbackend reserve rejected HTTP 429 https://cache.example.invalid/key?sig=fixture-secret\n';
  const quiet=cacheWriteReport(t,{cache_writes:100,cache_write_errors:0},{log:backend});
  assert.equal(quiet.stdout,'');
  assert.equal(quiet.stderr,'');
  const verbose=cacheWriteReport(t,'',{log:backend,verbose:true});
  assert.match(verbose.stderr,/\[sccache\] backend reserve rejected HTTP 429 <cache-url-redacted>/u);
  assert.doesNotMatch(verbose.stderr,/fixture-secret|https:\/\//u);
  assert.doesNotMatch(verbose.stderr,/^::error::/mu);
  assert.match(verbose.stderr,/^\[sccache\] ::error::untrusted backend annotation$/mu);
  const token=verbose.stderr.match(/^::stop-commands::([0-9a-f]{32})$/mu)?.[1];
  assert.ok(token,'fresh workflow-command suspension token');
  assert.ok(verbose.stderr.endsWith(`::${token}::\n`),'workflow commands resume after the diagnostic');
  const again=cacheWriteReport(t,'',{log:backend,verbose:true});
  assert.notEqual(again.stderr.match(/^::stop-commands::([0-9a-f]{32})$/mu)?.[1],token);

  assert.equal(verbose.summary,'');
});

test('opt-in diagnostics retain a bounded log tail and preserve warning counts', t => {
  const backend='old prefix that must be excluded\n'+'x'.repeat(100000)+'\nmost recent backend failure\n';
  const observed=cacheWriteReport(t,{cache_writes:1,cache_write_errors:87},{log:backend,verbose:true});
  assert.match(observed.stderr,/88 cache writes: 87 recorded errors and 1 were stored/u);
  assert.match(observed.stderr,/most recent backend failure/u);
  assert.doesNotMatch(observed.stderr,/old prefix that must be excluded/u);
  assert.ok(Buffer.byteLength(observed.stderr)<68000,'diagnostic reads at most 64 KiB, plus annotations/prefixes');
  assert.equal(observed.summary,'- Compiler cache writes (fixture): 1 stored, 87 write errors, 98% errors.\n');
  const manyLines=cacheWriteReport(t,'',{log:'noise\n'.repeat(20000)+'last backend response\n',verbose:true});
  assert.match(manyLines.stderr,/last backend response/u);
  assert.ok(manyLines.stderr.trim().split('\n').length<=83,'at most 80 log lines plus heading and command-suspension markers');
});

test('missing opt-in backend logs remain a nonfatal diagnostic', t => {
  const observed=cacheWriteReport(t,'',{verbose:true});
  assert.equal(observed.stdout,'');
  assert.equal(observed.stderr,'');
  assert.equal(observed.summary,'');
});

test('actual budget wrapper retains backend evidence without replacing command outcomes', {timeout:10000}, t => {
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'sccache-budget-report-'));
  t.after(()=>fs.rmSync(directory,{recursive:true,force:true}));
  const backendLog=path.join(directory,'server.log');
  fs.writeFileSync(backendLog,'last backend error after command\n');
  const unavailable=path.join(directory,'sccache');
  fs.writeFileSync(unavailable,'#!/bin/sh\nexit 2\n',{mode:0o700});
  for(const status of [0,23]){
    const observed=spawnSync('bash',['scripts/run-with-budget-warning.sh','10','cache fixture','bash','-c',`exit ${status}`],{
      cwd:root,encoding:'utf8',timeout:5000,maxBuffer:131072,
      env:{...process.env,RUSTC_WRAPPER:'sccache',SCCACHE_PATH:unavailable,
        SCCACHE_ERROR_LOG:backendLog,FORMAL_AI_CI_VERBOSE:'true'},
    });
    assert.equal(observed.status,status,observed.stderr);
    assert.match(observed.stderr,/\[sccache\] last backend error after command/u);
    assert.match(observed.stdout,/cache fixture took/u);
  }
});
