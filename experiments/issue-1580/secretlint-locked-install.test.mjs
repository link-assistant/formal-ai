import './secretlint-engine-workflow.test.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve,delimiter} from 'node:path';
import {spawnSync} from 'node:child_process';

function run(files,{installStatus=0,version=''}={}){
 const directory=mkdtempSync(join(tmpdir(),'secretlint-locked-install-'));
 try{
  const bin=join(directory,'bin'),receipt=join(directory,'calls.jsonl');mkdirSync(bin);
  writeFileSync(receipt,'');
  const program=(name,body)=>writeFileSync(join(bin,name),'#!'+process.execPath+'\n'+body,{mode:0o755});
  const record="const fs=require('node:fs'),args=process.argv.slice(2);const record=value=>fs.appendFileSync(process.env.FAKE_SCANNER_CALLS,JSON.stringify(value)+String.fromCharCode(10));";
  program('npm',record+[
   "const prefix=args[args.indexOf('--prefix')+1];",
   "record({kind:'install',args,prefix,manifest:JSON.parse(fs.readFileSync(prefix+'/package.json','utf8')),lock:JSON.parse(fs.readFileSync(prefix+'/package-lock.json','utf8'))});",
   "process.exitCode=Number(process.env.FAKE_INSTALL_STATUS);",
  ].join('\n'));
  program('node',record+[
   "const files=fs.readFileSync(0,'utf8').split(String.fromCharCode(0)).filter(Boolean);",
   "record({kind:'scan',args,files,version:process.env.SECRETLINT_VERSION});",
   "if(files.includes('fixture-detected-secret'))process.exitCode=1;",
  ].join('\n'));
  program('npx',record+[
   "const files=fs.readFileSync(0,'utf8').split(String.fromCharCode(0)).filter(Boolean);",
   "record({kind:'unlocked',args,files});",
   "process.exitCode=process.env.SECRETLINT_VERSION==='13.0.5'?0:45;",
  ].join('\n'));
  const result=spawnSync('bash',[resolve('scripts/check-secrets.sh'),...files],{
   encoding:'utf8',timeout:8000,env:{...process.env,PATH:bin+delimiter+process.env.PATH,
    BASH_ENV:'/dev/null',TMPDIR:directory,SECRETLINT_VERSION:version,
    FAKE_SCANNER_CALLS:receipt,FAKE_INSTALL_STATUS:String(installStatus)}});
  assert.equal(result.error,undefined,String(result.error));assert.equal(result.signal,null,result.stderr);
  const records=readFileSync(receipt,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse);
  for(const row of records.filter(row=>row.kind==='install'))assert.equal(existsSync(row.prefix),false,'temporary scanner installation must be removed');
  return {...result,records};
 }finally{rmSync(directory,{recursive:true,force:true});}
}

test('default secret scan installs the committed closure before consuming its literal NUL inventory',()=>{
 const files=['plain.txt',"a file with 'quotes'.txt",'line\nbreak.txt'];
 const result=run(files);assert.equal(result.status,0,result.stdout+result.stderr);
 assert.deepEqual(result.records.map(row=>row.kind),['install','scan']);
 const [install,scan]=result.records;
 assert.deepEqual(install.args.slice(0,-2),['ci','--ignore-scripts','--no-audit','--no-fund','--engine-strict']);
 assert.equal(install.args.at(-2),'--prefix');
 assert.deepEqual(install.manifest,JSON.parse(readFileSync('.github/secretlint/package.json','utf8')));
 assert.deepEqual(install.lock,JSON.parse(readFileSync('.github/secretlint/package-lock.json','utf8')));
 assert.deepEqual(scan.files,files);assert.deepEqual(scan.args,['scripts/check-secrets-files.mjs']);
 assert.equal(scan.version,'14.0.0');
});
test('a failed locked install cannot scan or print a clean result and still cleans its temporary directory',()=>{
 const result=run(['plain.txt'],{installStatus:17});assert.equal(result.status,17,result.stdout+result.stderr);
 assert.deepEqual(result.records.map(row=>row.kind),['install']);assert.doesNotMatch(result.stdout,/No secrets found/);
});
test('a finding after the locked install remains a failure and cleans its temporary directory',()=>{
 const result=run(['fixture-detected-secret']);assert.equal(result.status,1,result.stdout+result.stderr);
 assert.deepEqual(result.records.map(row=>row.kind),['install','scan']);assert.doesNotMatch(result.stdout,/No secrets found/);
});
test('an explicit scanner version override retains the existing separately requested package path',()=>{
 const result=run(['plain.txt'],{version:'13.0.5'});assert.equal(result.status,0,result.stdout+result.stderr);
 assert.deepEqual(result.records.map(row=>row.kind),['unlocked']);
 assert(result.records[0].args.includes('secretlint@13.0.5'));assert.deepEqual(result.records[0].files,['plain.txt']);
});
test('the default scanner lock covers every package with exact registry identity and integrity',()=>{
 const manifest=JSON.parse(readFileSync('.github/secretlint/package.json','utf8'));
 const lock=JSON.parse(readFileSync('.github/secretlint/package-lock.json','utf8'));
 assert.equal(manifest.private,true);assert.deepEqual(manifest.dependencies,{secretlint:'14.0.0'});
 assert.deepEqual(lock.packages[''].dependencies,manifest.dependencies);assert.equal(lock.lockfileVersion,3);
 const packages=Object.entries(lock.packages).filter(([name])=>name!=='');assert(packages.length>1);
 for(const [name,value] of packages){
  assert.equal(new URL(value.resolved).origin,'https://registry.npmjs.org',name);
  assert.equal(typeof value.version,'string',name);assert.match(value.integrity,/^sha512-[A-Za-z0-9+/]+=*$/u,name);
  assert.equal(Buffer.from(value.integrity.slice(7),'base64').length,64,name);
 }
});
