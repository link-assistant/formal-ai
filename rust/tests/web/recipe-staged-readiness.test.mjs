import test,{before} from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {execFileSync} from 'node:child_process';
import {installNodeHost} from '../../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../../js/server/worker-host.mjs';
import {boundedCommand} from '../../../js/server/bounded-command.mjs';
import {planRecipeCiFallback,recipeCiResult,recipeStagedReadyCommand,recipeStagedReadyResult} from '../../../js/agentic/recipe_resume.mjs';
before(async()=>installNodeHost(new WorkerHost()));
const reference='https://github.com/fixture/project/pull/2',workflow='Source verification';
const source='name: '+workflow+'\non: [push, pull_request]\njobs: {}\n';
function fixture(t){
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'recipe-staged-ready-'));t.after(()=>fs.rmSync(root,{recursive:true,force:true}));
 const cwd=path.join(root,'work'),bin=path.join(root,'bin'),state=path.join(root,'state.json');fs.mkdirSync(cwd);fs.mkdirSync(bin);
 const git=(...args)=>execFileSync('git',args,{cwd,encoding:'utf8',stdio:['ignore','pipe','pipe']}).trim();
 git('init','-b','owned-task');git('config','user.name','Fixture');git('config','user.email','fixture@example.invalid');
 fs.mkdirSync(path.join(cwd,'ci'));fs.writeFileSync(path.join(cwd,'ci/test.yml'),source);fs.writeFileSync(path.join(cwd,'owned.py'),'print(17)\n');
 git('add','ci/test.yml','owned.py');git('commit','-m','owned task source');const head=git('rev-parse','HEAD');
 fs.writeFileSync(state,JSON.stringify({head,isDraft:true,readyWrites:0,checks:0}));
 fs.writeFileSync(path.join(bin,'gh'),`#!${process.execPath}
const fs=require('node:fs'),file=process.env.TEST_STAGED_STATE,s=JSON.parse(fs.readFileSync(file)),a=process.argv.slice(2);
if(a[0]!=='pr')throw Error('only PR operations admitted');
if(a[1]==='view')console.log(JSON.stringify({headRefOid:s.head,headRefName:'owned-task',isDraft:s.isDraft,state:'OPEN',url:${JSON.stringify(reference)}}));
else if(a[1]==='ready'){s.isDraft=false;s.readyWrites++;}
else if(a[1]==='checks'){s.checks++;console.error('no checks reported on staged branch');process.exitCode=1;}
else throw Error('unreviewed operation');
fs.writeFileSync(file,JSON.stringify(s));
`,{mode:0o755});
 const run=command=>boundedCommand('/bin/sh',['-c',command],{cwd,
  env:{...process.env,PATH:bin+path.delimiter+process.env.PATH,GH_TOKEN:undefined,GITHUB_TOKEN:undefined,TEST_STAGED_STATE:state},
  timeout:15000,maxBuffer:1048576,encoding:'utf8'});
 return {cwd,state,head,run,git,read:()=>JSON.parse(fs.readFileSync(state))};
}
async function drive(f,recipe){
 const messages=[];
 for(let step=0;step<12;step++){
  const plan=planRecipeCiFallback(messages,['Bash'],recipe,reference,workflow,[],true,'https://github.com/fixture/project/issues/1');
  if(plan.kind==='final')return plan;
  assert.equal(plan.calls.length,1);const call=plan.calls[0],args=JSON.parse(call.arguments);
  const result=await f.run(args.command);assert.equal(result.cleanup_complete,true);
  const id='owned-'+step;messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:call.tool,arguments:call.arguments}}]},
   {role:'tool',name:call.tool,tool_call_id:id,is_error:result.status!==0,content:result.status===0?result.stdout:JSON.stringify({exit_code:result.status,stderr:result.stderr})});
 }
 throw Error('bounded readiness driver exhausted');
}
const recipe={path:'owned.py',source:'print(17)\n',supporting_files:[{path:'ci/test.yml',source}],commands:[]};
test('real shared staged program makes one ready transition, preserves Git and never claims hosted CI',async t=>{
 const f=fixture(t),result=await drive(f,recipe);
 assert.match(result.answer,/pull request is ready/iu);assert.match(result.answer,/Hosted CI is not verified/u);
 assert.equal(f.read().readyWrites,1);assert.equal(f.read().checks,0);assert.equal(f.git('rev-parse','HEAD'),f.head);
 assert.equal(f.git('status','--porcelain'),'');
 await drive(f,recipe);assert.equal(f.read().readyWrites,1,'fresh repeat observes already-ready state without a write');
 const command=recipeStagedReadyCommand(reference,workflow,recipe.supporting_files[0],true),observed=await f.run(command);
 const receipt=recipeStagedReadyResult(observed.stdout,reference,workflow,'ci/test.yml',true);
 assert.equal(receipt.ciVerified,false);assert.equal(receipt.checks,0);
 assert.equal(recipeCiResult(observed.stdout,reference,workflow,true),null,'readiness cannot be consumed as a green CI receipt');
});
test('source drift, dirty workspace, foreign PR head and normal workflow paths refuse readiness before writes',async t=>{
 for(const mode of ['source','dirty','head','normal']){
  const f=fixture(t);let file=recipe.supporting_files[0];
  if(mode==='source')file={...file,source:source+'# different\n'};
  if(mode==='dirty')fs.writeFileSync(path.join(f.cwd,'unowned.txt'),'preserve');
  if(mode==='head')fs.writeFileSync(f.state,JSON.stringify({...f.read(),head:'f'.repeat(40)}));
  if(mode==='normal')file={...file,path:'.github/workflows/test.yml'};
  const result=await f.run(recipeStagedReadyCommand(reference,workflow,file,true));
  assert.equal(result.status,1,mode);assert.equal(f.read().readyWrites,0,mode);
 }
});
test('normal hosted workflows retain mandatory checks and cannot use the staged exception',async t=>{
 const f=fixture(t),normal={...recipe,supporting_files:[{path:'.github/workflows/test.yml',source}]};
 fs.mkdirSync(path.join(f.cwd,'.github/workflows'),{recursive:true});fs.writeFileSync(path.join(f.cwd,'.github/workflows/test.yml'),source);
 f.git('add','.github/workflows/test.yml');f.git('commit','-m','normal hosted workflow');
 fs.writeFileSync(f.state,JSON.stringify({...f.read(),head:f.git('rev-parse','HEAD')}));
 const result=await drive(f,normal);assert.doesNotMatch(result.answer,/pull request is ready/iu);
 assert.equal(f.read().readyWrites,0);assert.equal(f.read().checks,1);
});
