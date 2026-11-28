import test from 'node:test';
import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath}from 'node:url';
import {spawnSync}from 'node:child_process';
import {captureFreshPlannerRun,observeRuntimeExecutable,normalizeRuntimeRequests}from '../../../js/provenance/fresh-runtime.mjs';
import {verifyFullRepositoryProof}from '../../../js/provenance/full-verifier.mjs';
import {repositoryGit,repositoryText}from '../../../js/provenance/git-evidence.mjs';
import {trustedInstallationKey}from '../../../js/provenance/installation-key.mjs';
import {WorkerHost}from '../../../js/server/worker-host.mjs';
import {installNodeHost}from '../../../js/agentic/node-host.mjs';
import {planChatStep}from '../../../js/agentic/planner.mjs';
import {drive}from '../../../experiments/js_dogfood/drive.mjs';
import {PLAN_PATH}from '../../../js/agentic/general_planner.mjs';
import {decodeTyped}from '../../../js/logging/typed-codec.mjs';
import {verifyCallerRequestObservation}from '../../../js/provenance/fresh-recompute.mjs';
import {retainedEvidenceBytes}from '../../../js/provenance/verify-drive.mjs';
const git=(workspace,args)=>repositoryText(repositoryGit(workspace,args)).trimEnd();
import {withFullCliFixture} from './helpers/full-provenance-cli-fixture.mjs';

test("full CLI verifies a real fresh planner write and Git execution with independent installed-source pins", {skip:process.env.FAST_PR_PROFILE==='true'},()=>withFullCliFixture(({directory,workspace,captured,request,requestFile,invoke,delivered,head,base,before})=>{
  const positive=invoke(requestFile);
  assert.equal(positive.status,0,positive.stdout+'\n'+positive.stderr);
  const report=decodeTyped(positive.stdout.trimEnd());assert.equal(report.verification,'Verified');
  assert.equal(report.trustedVerification,true);assert.equal(report.fullReplay.verified,true);
  assert.equal(report.repository.allBlobsAttributed,true);assert.equal(report.repository.share,1);
  assert.equal(report.execution.length,1);assert.equal(report.autonomousAuthorship,'not-established');
  assert.equal(fileSystem.statSync(delivered,{bigint:true}).ctimeNs,before.ctimeNs);assert.equal(git(workspace,['rev-parse','HEAD']),head);
}));

test('runtime request sequences retain owned data and refuse malformed, inherited or accessor authority before execution',()=>{
 const requests=[{prompt:'first',steps:2},{prompt:'second',steps:3}],normalized=normalizeRuntimeRequests('first',2,requests);
 assert.deepEqual(normalized,requests);assert.notEqual(normalized,requests);assert.notEqual(normalized[0],requests[0]);
 requests[0].prompt='changed';assert.equal(normalized[0].prompt,'first');
 assert.deepEqual(normalizeRuntimeRequests('',1,undefined),[{prompt:'',steps:1}]);
 for(const value of [[],Array.from({length:33},()=>({prompt:'first',steps:2})),[{prompt:'other',steps:2}],
  [{prompt:'first',steps:3}],[Object.create({prompt:'first',steps:2})],[{prompt:'first',steps:2,prerequisiteGrant:{allowNetwork:true}}]]){
  assert.throws(()=>normalizeRuntimeRequests('first',2,value),/FreshRuntimeRequestSequence/u);
 }
 let reads=0;
 assert.throws(()=>normalizeRuntimeRequests('first',2,[{get prompt(){reads++;return'first';},steps:2}]),/FreshRuntimeRequestSequenceInvalid/u);
 assert.throws(()=>normalizeRuntimeRequests('first',2,[{prompt:'first',get steps(){reads++;return 2;}}]),/FreshRuntimeRequestSequenceInvalid/u);
 assert.equal(reads,0);
});
test('declared atomic append cannot substitute an unowned path, content or request identity in a Set contents operation',async()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'owned-append-negative-'));
 try{
  await installNodeHost(new WorkerHost({WebAssembly:{compile(){throw Error('ProductWasmDisabled');},
   instantiate(){throw Error('ProductWasmDisabled');},instantiateStreaming(){throw Error('ProductWasmDisabled');}}}));
  let originalAppend=null;
  for(const field of ['path','content','append_request_id']){
   const workspace=path.join(directory,field);fileSystem.mkdirSync(workspace);
   const malicious=async(messages,tools)=>{
    const plan=await planChatStep(messages,tools);
    if(plan?.kind!=='tool_calls')return plan;
    return{...plan,calls:plan.calls.map(call=>{
     const args=JSON.parse(call.arguments);if(call.tool!=='write'||args.append_mode!=='atomic_record_append')return call;
     originalAppend={...call};const changed={...args,[field]:field==='path'?'unowned.lino':args[field]+'forged'};
     return{...call,arguments:JSON.stringify(changed)};
    })};
   };
   const result=await drive(malicious,workspace,'Set the contents of guarded.mjs to exactly this content: export const guarded = 29;',
    {steps:6,atomicRecordAppend:true});
   assert.equal(result.stop,'unbound-create-operation',JSON.stringify(result));
   assert.equal(fileSystem.existsSync(path.join(workspace,'guarded.mjs')),false);
   assert.equal(fileSystem.existsSync(path.join(workspace,'unowned.lino')),false);
   assert.equal(fileSystem.existsSync(path.join(workspace,PLAN_PATH)),false);
  }
  assert(originalAppend!==null);
  const workspace=path.join(directory,'undeclared');fileSystem.mkdirSync(workspace);
  const refused=await drive(async()=>({kind:'tool_calls',calls:[originalAppend]}),workspace,
   'Set the contents of guarded.mjs to exactly this content: export const guarded = 29;',{steps:1,atomicRecordAppend:false});
  assert.equal(refused.stop,'undeclared-append-contract');assert.equal(fileSystem.existsSync(path.join(workspace,PLAN_PATH)),false);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});

test('original Create with atomic append available has no journal effect',async()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'original-create-provenance-boundary-'));
 try{
  await installNodeHost(new WorkerHost({WebAssembly:{compile(){throw Error('ProductWasmDisabled');},
   instantiate(){throw Error('ProductWasmDisabled');},instantiateStreaming(){throw Error('ProductWasmDisabled');}}}));
  const result=await drive(planChatStep,directory,'Create file guarded.mjs containing export const guarded = 29;',
   {steps:6,tools:['write','read'],atomicRecordAppend:true});
  assert.equal(result.stop,'final',JSON.stringify(result));
  assert.equal(fileSystem.readFileSync(path.join(directory,'guarded.mjs'),'utf8'),'export const guarded = 29;');
  assert.deepEqual(result.transcript.map(row=>row.tool),['write','read']);
  assert.equal(fileSystem.existsSync(path.join(directory,PLAN_PATH)),false);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
