import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,realpathSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

function run({present=false,failures=0,stall=false,probe=0}={}){
 const directory=realpathSync(mkdtempSync(join(tmpdir(),'formal-ai-search-install-')));
 try{
  const bin=join(directory,'bin'),calls=join(directory,'calls'),mirror=join(directory,'apt-mirrors.txt');mkdirSync(bin);
  writeFileSync(mirror,'http://azure.archive.ubuntu.com/ubuntu\n');
  writeFileSync(join(directory,'environment'),'command(){ if [[ "$1" == -v && "${2:-}" == rg ]]; then [[ "$FAKE_RG_PRESENT" == 1 ]]; else builtin command "$@"; fi; }\n');
  const program=(name,body)=>writeFileSync(join(bin,name),'#!'+process.execPath+'\n'+body,{mode:0o755});
  program('apt-get',[
   "const fs=require('node:fs'),args=process.argv.slice(2),p=process.env.FAKE_CALLS;",
   "fs.appendFileSync(p,JSON.stringify({kind:'apt',args})+'\\n');",
   "const updates=fs.readFileSync(p,'utf8').trim().split('\\n').map(JSON.parse).filter(r=>r.kind==='apt'&&r.args.includes('update')).length;",
   "if(args.includes('update')&&updates<=Number(process.env.FAKE_FAILURES)){",
   " if(process.env.FAKE_STALL==='1')setInterval(()=>{},1000);else process.exitCode=100;",
   "}",
  ].join('\n'));
  program('rg',"require('node:fs').appendFileSync(process.env.FAKE_CALLS,JSON.stringify({kind:'probe',args:process.argv.slice(2)})+'\\n');process.exitCode=Number(process.env.FAKE_PROBE);");
  writeFileSync(join(bin,'sudo'),'#!/bin/bash\nexec "$@"\n',{mode:0o755});
  const started=Date.now(),result=spawnSync('bash',[resolve('scripts/install-agentic-search-tools.sh')],{
   encoding:'utf8',timeout:8000,env:{...process.env,PATH:bin+':'+process.env.PATH,
    BASH_ENV:join(directory,'environment'),FORMAL_AI_APT_GET:join(bin,'apt-get'),FORMAL_AI_APT_PRIVILEGE:'',
    FORMAL_AI_APT_ATTEMPTS:'2',FORMAL_AI_APT_ATTEMPT_SECONDS:'1',FORMAL_AI_APT_RETRY_DELAY_SECONDS:'0',
    FORMAL_AI_APT_MIRROR_LIST:mirror,FORMAL_AI_DEADLINE_GRACE_SECONDS:'0',TEST_BUDGET_SECONDS:'5',
    FAKE_CALLS:calls,FAKE_RG_PRESENT:present?'1':'0',FAKE_FAILURES:String(failures),FAKE_STALL:stall?'1':'0',FAKE_PROBE:String(probe)}});
  assert.equal(result.error,undefined,String(result.error));assert.equal(result.signal,null,result.stderr);
  return {...result,elapsed:Date.now()-started,records:readFileSync(calls,'utf8').trim().split('\n').map(JSON.parse),mirror:readFileSync(mirror,'utf8')};
 }finally{rmSync(directory,{recursive:true,force:true});}
}

test('existing search adapter is probed without package-manager changes',()=>{
 const r=run({present:true});assert.equal(r.status,0,r.stderr);
 assert.deepEqual(r.records,[{kind:'probe',args:['--version']}]);
});
test('missing search adapter installs the real package and probes it',()=>{
 const r=run();assert.equal(r.status,0,r.stderr);
 assert.deepEqual(r.records.filter(x=>x.kind==='apt').map(x=>x.args),[
  ['-o','DPkg::Lock::Timeout=60','update','-q'],
  ['-o','DPkg::Lock::Timeout=60','install','-y','-q','--no-install-recommends','ripgrep']]);
 assert.equal(r.records.at(-1).kind,'probe');
});
test('transient index failure is retried within a finite attempt limit',()=>{
 const r=run({failures:1});assert.equal(r.status,0,r.stderr);
 assert.equal(r.records.filter(x=>x.kind==='apt'&&x.args.includes('update')).length,2);
});
test('stalled coverage setup is terminated and recovers through the official mirror',()=>{
 const r=run({failures:1,stall:true});assert.equal(r.status,0,r.stderr);assert(r.elapsed<7000);
 assert.match(r.stderr,/exited 124/);assert.match(r.stdout,/official Ubuntu archive HTTPS alternate/);
 assert.equal(r.mirror,'https://archive.ubuntu.com/ubuntu\n');
 assert.equal(r.records.filter(x=>x.kind==='apt'&&x.args.includes('update')).length,2);
 assert.equal(r.records.at(-1).kind,'probe');
});
test('permanent setup failures stop after two attempts and prevent the probe',()=>{
 const r=run({failures:99});assert.equal(r.status,100);assert.equal(r.records.length,2);
 assert(r.records.every(x=>x.kind==='apt'&&x.args.includes('update')));assert.match(r.stderr,/failed every attempt/);
});
test('a failed installed-adapter probe remains a failure',()=>{
 const r=run({present:true,probe:42});assert.equal(r.status,42);assert.equal(r.records.length,1);
});
