import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,rmSync,realpathSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {spawnSync} from 'node:child_process';

function run({present=false,failures=0,stall=false,probe=0,interruptInstall=false,recoveryStatus=0,recoveryStall=false,recover='',mirrorText='http://azure.archive.ubuntu.com/ubuntu\n'}={}){
 const directory=realpathSync(mkdtempSync(join(tmpdir(),'formal-ai-search-install-')));
 try{
  const bin=join(directory,'bin'),calls=join(directory,'calls'),mirror=join(directory,'apt-mirrors.txt');mkdirSync(bin);
  writeFileSync(mirror,mirrorText);writeFileSync(calls,'');
  writeFileSync(join(directory,'environment'),'command(){ if [[ "$1" == -v && "${2:-}" == rg ]]; then [[ "$FAKE_RG_PRESENT" == 1 ]]; else builtin command "$@"; fi; }\n');
  const program=(name,body)=>writeFileSync(join(bin,name),'#!'+process.execPath+'\n'+body,{mode:0o755});
  program('apt-get',[
   "const fs=require('node:fs'),args=process.argv.slice(2),p=process.env.FAKE_CALLS;",
   "fs.appendFileSync(p,JSON.stringify({kind:'apt',args})+'\\n');",
   "const updates=fs.readFileSync(p,'utf8').trim().split('\\n').map(JSON.parse).filter(r=>r.kind==='apt'&&r.args.includes('update')).length;",
   "const pending=process.env.FAKE_PENDING;",
   "if(fs.existsSync(pending)){console.error('E: dpkg was interrupted; configure pending packages first');process.exit(100);}",
   "if(args.includes('install')&&process.env.FAKE_INTERRUPT_INSTALL==='1'&&!fs.existsSync(pending+'.started')){",
   " fs.writeFileSync(pending+'.started','1');fs.writeFileSync(pending,'1');setInterval(()=>{},1000);",
   "}",
   "if(args.includes('update')&&updates<=Number(process.env.FAKE_FAILURES)){",
   " if(process.env.FAKE_STALL==='1')setInterval(()=>{},1000);else process.exitCode=100;",
   "}",
  ].join('\n'));
  program('dpkg',"const fs=require('node:fs');fs.appendFileSync(process.env.FAKE_CALLS,JSON.stringify({kind:'configure',args:process.argv.slice(2)})+'\\n');const status=Number(process.env.FAKE_RECOVERY_STATUS);if(process.env.FAKE_RECOVERY_STALL==='1')setInterval(()=>{},1000);else{if(status===0)fs.rmSync(process.env.FAKE_PENDING,{force:true});process.exitCode=status;}");
  program('rg',"require('node:fs').appendFileSync(process.env.FAKE_CALLS,JSON.stringify({kind:'probe',args:process.argv.slice(2)})+'\\n');process.exitCode=Number(process.env.FAKE_PROBE);");
  writeFileSync(join(bin,'sudo'),'#!/bin/bash\nexec "$@"\n',{mode:0o755});
  const started=Date.now(),result=spawnSync('bash',[resolve('scripts/install-agentic-search-tools.sh')],{
   encoding:'utf8',timeout:8000,env:{...process.env,PATH:bin+':'+process.env.PATH,
    BASH_ENV:join(directory,'environment'),FORMAL_AI_APT_GET:join(bin,'apt-get'),FORMAL_AI_APT_PRIVILEGE:'',
    FORMAL_AI_APT_ATTEMPTS:'2',FORMAL_AI_APT_ATTEMPT_SECONDS:'1',FORMAL_AI_APT_RETRY_DELAY_SECONDS:'0',
    FORMAL_AI_APT_RECOVER_DPKG_ON_TIMEOUT:recover,FORMAL_AI_APT_MIRROR_LIST:mirror,FORMAL_AI_DEADLINE_GRACE_SECONDS:'0',TEST_BUDGET_SECONDS:'5',
    FAKE_PENDING:join(directory,'pending'),FAKE_INTERRUPT_INSTALL:interruptInstall?'1':'0',FAKE_RECOVERY_STATUS:String(recoveryStatus),FAKE_RECOVERY_STALL:recoveryStall?'1':'0',
    FAKE_CALLS:calls,FAKE_RG_PRESENT:present?'1':'0',FAKE_FAILURES:String(failures),FAKE_STALL:stall?'1':'0',FAKE_PROBE:String(probe)}});
  assert.equal(result.error,undefined,String(result.error));assert.equal(result.signal,null,result.stderr);
  return {...result,elapsed:Date.now()-started,records:readFileSync(calls,'utf8').trim().split('\n').filter(Boolean).map(JSON.parse),mirror:readFileSync(mirror,'utf8')};
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

test('an interrupted package transaction is configured within the existing second attempt',()=>{
 const r=run({interruptInstall:true});assert.equal(r.status,0,r.stdout+r.stderr);assert(r.elapsed<7000);
 assert.match(r.stderr,/Attempt 1 exited 124/);
 assert.deepEqual(r.records.map(x=>x.kind),['apt','apt','configure','apt','apt','probe']);
 assert.deepEqual(r.records.find(x=>x.kind==='configure').args,['--configure','--pending']);
});
test('failed interrupted-package recovery keeps its error and cannot start another index or probe',()=>{
 const r=run({interruptInstall:true,recoveryStatus:73});assert.equal(r.status,73,r.stdout+r.stderr);
 assert.deepEqual(r.records.map(x=>x.kind),['apt','apt','configure']);
});
test('disabling package recovery keeps interrupted installation fatal',()=>{
 const r=run({interruptInstall:true,recover:'0'});assert.equal(r.status,100,r.stdout+r.stderr);
 assert.equal(r.records.filter(x=>x.kind==='configure'||x.kind==='probe').length,0);
});
test('invalid package-recovery configuration refuses before package-manager effects',()=>{
 const r=run({recover:'2'});assert.equal(r.status,2,r.stdout+r.stderr);assert.deepEqual(r.records,[]);
});
test('the observed EC2 mirror is replaced after the first real deadline',()=>{
 const r=run({failures:1,stall:true,mirrorText:'http://us-east-2.ec2.archive.ubuntu.com/ubuntu\n'});
 assert.equal(r.status,0,r.stdout+r.stderr);assert.equal(r.mirror,'https://archive.ubuntu.com/ubuntu\n');
 assert.equal(r.records.filter(x=>x.kind==='apt'&&x.args.includes('update')).length,2);
});

test('stalled interrupted-package recovery is killed by the existing last-attempt deadline',()=>{
 const r=run({interruptInstall:true,recoveryStall:true});assert.equal(r.status,124,r.stdout+r.stderr);assert(r.elapsed<7000);
 assert.deepEqual(r.records.map(x=>x.kind),['apt','apt','configure']);assert.match(r.stderr,/Attempt 2 exited 124/);
});
