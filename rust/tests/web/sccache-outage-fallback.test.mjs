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
