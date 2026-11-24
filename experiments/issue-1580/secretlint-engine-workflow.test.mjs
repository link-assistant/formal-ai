import test from 'node:test';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync,spawnSync} from 'node:child_process';
import {copyFileSync,existsSync,mkdirSync,mkdtempSync,readFileSync,readdirSync,rmSync,writeFileSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {dirname,join,resolve} from 'node:path';
import YAML from 'yaml';

const workflow=()=>YAML.parse(readFileSync('.github/workflows/security.yml','utf8'));
const job=()=>{
 const value=workflow().jobs['secretlint-engine'];
 assert(value,'the locked engine controls must execute on the hosted Node 22 lane');
 return value;
};
const required=[
 '.secretlintrc.json','.github/secretlint/package.json','.github/secretlint/package-lock.json',
 'scripts/check-secrets.sh','scripts/check-secrets-files.mjs','scripts/lib/compressed-secrets.mjs',
 'experiments/issue-1580/secretlint-locked-live.py',
];
const temporary=()=>mkdtempSync(join(tmpdir(),'secretlint-engine-workflow-'));

test('the actual locked engine runs on Node 22 without credentials or failure suppression',()=>{
 const value=job(),steps=value.steps;
 assert.equal(value['runs-on'],'ubuntu-24.04');
 assert(value['timeout-minutes']>0&&value['timeout-minutes']<=10);
 assert.deepEqual(value.permissions,{contents:'read'});
 assert.equal(value.if,undefined,'ordinary PRs, scheduled audits and main must retain these controls');
 assert.equal(value['continue-on-error'],undefined);
 const checkout=steps.find(step=>step.uses?.startsWith('actions/checkout@'));
 assert.equal(checkout.with['persist-credentials'],false);
 assert.equal(checkout.with.ref,'${{ github.event.pull_request.head.sha || github.sha }}');
 assert.equal(checkout.with['fetch-depth'],1);
 const node=steps.find(step=>step.uses?.startsWith('actions/setup-node@'));
 assert.equal(String(node.with['node-version']),'22');
 const run=steps.find(step=>step.name==='Exercise the locked scanner');
 assert(run['timeout-minutes']>0&&run['timeout-minutes']<=5);
 assert.equal(run['continue-on-error'],undefined);
 assert.equal(run.run.trim(),'python3 experiments/issue-1580/secretlint-locked-live.py "$PROBE_OUTPUT" --node-major 22');
 assert.equal(run.env.PROBE_OUTPUT,'${{ runner.temp }}/secretlint-locked-engine');
 assert(steps.indexOf(node)<steps.indexOf(run));
 const artifact=steps.find(step=>step.uses?.startsWith('actions/upload-artifact@'));
 assert.equal(artifact.if,'${{ always() && !cancelled() }}','a failed probe must retain partial reports');
 assert.equal(artifact.with.path,'${{ runner.temp }}/secretlint-locked-engine/');
 assert.equal(artifact.with['if-no-files-found'],'error');
 assert(artifact.with.name.includes('${{ github.run_id }}'));
 assert(artifact.with.name.includes('${{ github.run_attempt }}'));
 assert.equal(artifact.with['retention-days'],7);
 assert(steps.indexOf(artifact)>steps.indexOf(run));
});

test('the maintained job propagates the actual probe failure and preserves its literal output argument',()=>{
 const run=job().steps.find(step=>step.name==='Exercise the locked scanner');
 const directory=temporary();
 try{
  const bin=join(directory,'bin');mkdirSync(bin);
  const calls=join(directory,'calls.json');
  writeFileSync(join(bin,'python3'),'#!'+process.execPath+'\n'+
   'require("node:fs").writeFileSync(process.env.PROBE_CALLS,JSON.stringify(process.argv.slice(2)));process.exit(17);\n',{mode:0o755});
  const output=join(directory,'output with spaces');
  const result=spawnSync('bash',['--noprofile','--norc','-eo','pipefail','-c',run.run],{
   encoding:'utf8',timeout:5000,env:{...process.env,PATH:bin+':'+process.env.PATH,
    BASH_ENV:'/dev/null',PROBE_CALLS:calls,PROBE_OUTPUT:output},
  });
  assert.equal(result.error,undefined);assert.equal(result.signal,null);
  assert.equal(result.status,17,result.stdout+result.stderr);
  assert.deepEqual(JSON.parse(readFileSync(calls,'utf8')),[
   'experiments/issue-1580/secretlint-locked-live.py',output,'--node-major','22',
  ]);
 }finally{rmSync(directory,{recursive:true,force:true});}
});

test('a real sparse checkout retains the actual scanner closure without the application tree',()=>{
 const checkout=job().steps.find(step=>step.uses?.startsWith('actions/checkout@'));
 assert.equal(checkout.with['sparse-checkout-cone-mode'],false);
 const directory=temporary();
 try{
  for(const name of [...required,'rust/application-only.txt']){
   const target=join(directory,name);mkdirSync(dirname(target),{recursive:true});
   if(required.includes(name))copyFileSync(resolve(name),target);else writeFileSync(target,'excluded application source\n');
  }
  const git=(args,input)=>execFileSync('git',args,{cwd:directory,input,encoding:'utf8',timeout:5000});
  git(['init','--quiet']);git(['add','.']);
  git(['-c','user.name=Fixture','-c','user.email=fixture@example.invalid','commit','--quiet','-m','scanner closure']);
  git(['sparse-checkout','set','--no-cone','--stdin'],checkout.with['sparse-checkout']);
  for(const name of required)assert.equal(existsSync(join(directory,name)),true,name);
  assert.equal(existsSync(join(directory,'rust/application-only.txt')),false);
 }finally{rmSync(directory,{recursive:true,force:true});}
});

test('the real probe rejects a wrong Node major before an installation or fixture is created',()=>{
 const script='experiments/issue-1580/secretlint-locked-live.py';
 assert(readFileSync(script,'utf8').includes('--node-major'),'the probe must check its actual runtime');
 const directory=temporary(),before=readdirSync('experiments/issue-1580').sort();
 try{
  const actualMajor=Number(process.versions.node.split('.')[0]),expected=actualMajor+1;
  const result=spawnSync('python3',[script,directory,'--node-major',String(expected)],{
   encoding:'utf8',timeout:15000,env:{...process.env,PYTHONDONTWRITEBYTECODE:'1'},
  });
  assert.equal(result.error,undefined);assert.equal(result.signal,null);assert.notEqual(result.status,0);
  assert.match(result.stderr,/Node major mismatch/);
  const runtime=JSON.parse(readFileSync(join(directory,'runtime.json'),'utf8'));
  assert.equal(runtime.expectedNodeMajor,expected);
  assert.equal(runtime.node.stdout.trim().split('.')[0],'v'+actualMajor);
  assert.equal(runtime.runtimeAccepted,false);
  assert.equal(runtime.source.stdout.trim(),execFileSync('git',['rev-parse','HEAD'],{encoding:'utf8'}).trim());
  for(const name of required){
   assert.equal(runtime.scannerSources[name],createHash('sha256').update(readFileSync(name)).digest('hex'),name);
  }
  assert.equal(existsSync(join(directory,'actual-result.json')),false);
  assert.deepEqual(readdirSync('experiments/issue-1580').sort(),before);
 }finally{rmSync(directory,{recursive:true,force:true});}
});
