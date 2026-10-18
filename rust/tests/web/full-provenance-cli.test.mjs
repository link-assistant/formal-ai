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
test('full CLI verifies a real fresh planner write and Git execution with independent pins, then refuses dirty, foreign and truncated proof',()=>{
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'full-provenance-cli-'));
 try{
  const workspace=path.join(directory,'work');fileSystem.mkdirSync(workspace);
  git(workspace,['init']);git(workspace,['config','user.name','Fixture']);git(workspace,['config','user.email','fixture@example.invalid']);
  fileSystem.writeFileSync(path.join(workspace,'base.txt'),'base\n');git(workspace,['add','base.txt']);git(workspace,['commit','-m','base']);
  const base=git(workspace,['rev-parse','HEAD']),runtime=observeRuntimeExecutable(),keys=path.join(directory,'keys');
  const command='git add -- retained.mjs '+PLAN_PATH+' && git commit -m observed --trailer "Formal-AI-Chain: $FORMAL_AI_CHAIN_PREFIX"';
  const prompt='Create file retained.mjs containing «export const sample = 97;».';
  const captured=captureFreshPlannerRun({directory:path.join(directory,'capture'),workspace,prompt,steps:12,atomicRecordAppend:true,
   requests:[{prompt,steps:12},{prompt:'Run '+command,steps:6}],
   memoryFile:path.join(directory,'memory.lino'),taskURL:'https://example.invalid/tasks/full-proof-97',signing:{directory:keys}});
  assert.equal(captured.result.stop,'final',JSON.stringify(captured.result));
  const delivered=path.join(workspace,'retained.mjs');assert.equal(fileSystem.readFileSync(delivered,'utf8'),'export const sample = 97;');
  const capturedRecords=fileSystem.readFileSync(path.join(captured.result.provenance.directory,'chain.lino'),'utf8').trimEnd().split('\n').map(decodeTyped);
  assert.deepEqual(capturedRecords.filter(record=>record.kind==='edit').map(record=>record.payload.path).sort(),[PLAN_PATH,'retained.mjs'].sort());
  assert.match(fileSystem.readFileSync(path.join(workspace,PLAN_PATH),'utf8'),/target "retained\.mjs"/u);
  const callerRecords=capturedRecords.filter(record=>record.kind==='caller_request');assert.equal(callerRecords.length,2);
  for(const [index,record]of callerRecords.entries()){
   const observed=JSON.parse(retainedEvidenceBytes(captured.result.provenance.directory,record.payload.evidence));
   const request={prompt:observed.prompt,steps:observed.steps};
   assert.equal(verifyCallerRequestObservation(observed,index,request,null),observed);
   const {sourceMaintenanceOwnership,...missing}=observed;assert.equal(sourceMaintenanceOwnership,null);
   assert.throws(()=>verifyCallerRequestObservation(missing,index,request,null),/FreshReplayCallerRequestMismatch/u);
   assert.throws(()=>verifyCallerRequestObservation({...observed,sourceMaintenanceOwnership:{target:'forged.mjs'}},index,request,null),/FreshReplayCallerRequestMismatch/u);
  }
  const head=git(workspace,['rev-parse','HEAD']);assert.notEqual(head,base);
  const request={directory:captured.result.provenance.directory,workspace,expectedAnchor:captured.result.provenance.anchor,
   trustedPublicKey:trustedInstallationKey(keys),expectedRuntimeSha256:runtime.sha256,expectedSourceIdentity:captured.sourceIdentity,
   expectedTaskURL:'https://example.invalid/tasks/full-proof-97',base,head,minShare:1};
  const requestFile=path.join(directory,'independent-request.json');fileSystem.writeFileSync(requestFile,JSON.stringify(request),{mode:0o600});
  const cli=fileURLToPath(new URL('../../../js/provenance/cli.mjs',import.meta.url));
  const invoke=filename=>spawnSync(process.execPath,[cli,'verify-provenance','--request',filename],{
   encoding:'utf8',timeout:180000,maxBuffer:2*1024*1024,env:{...process.env,NODE_TEST_CONTEXT:undefined,NODE_OPTIONS:''}});
  const before=fileSystem.statSync(delivered,{bigint:true}),positive=invoke(requestFile);
  assert.equal(positive.status,0,positive.stdout+'\n'+positive.stderr);
  const report=decodeTyped(positive.stdout.trimEnd());assert.equal(report.verification,'Verified');
  assert.equal(report.trustedVerification,true);assert.equal(report.fullReplay.verified,true);
  assert.equal(report.repository.allBlobsAttributed,true);assert.equal(report.repository.share,1);
  assert.equal(report.execution.length,1);assert.equal(report.autonomousAuthorship,'not-established');
  const archivedRequest=path.join(directory,'independent-archived-request.json');
  fileSystem.writeFileSync(archivedRequest,JSON.stringify({...request,sourceMode:'archived'}),{mode:0o600});
  const archived=invoke(archivedRequest);assert.equal(archived.status,0,archived.stdout+'\n'+archived.stderr);
  const archivedReport=decodeTyped(archived.stdout.trimEnd());assert.equal(archivedReport.verification,'Verified');
  assert.equal(archivedReport.trustedVerification,true);assert.equal(archivedReport.fullReplay.verified,true);
  assert.equal(archivedReport.fullReplay.scope,'authenticated-retained-js-engine-logical-memory-and-recorded-tool-inputs');
  assert(archivedReport.fullReplay.consumedInputs.modules>100);assert(archivedReport.fullReplay.consumedInputs.reads>100);
  assert.equal(archivedReport.repository.share,1);assert.equal(archivedReport.execution.length,1);
  assert.throws(()=>verifyFullRepositoryProof({...request,sourceMode:'untrusted-source'}),/FullProofSourceModeUnsupported/u);
  assert.equal(fileSystem.statSync(delivered,{bigint:true}).ctimeNs,before.ctimeNs);assert.equal(git(workspace,['rev-parse','HEAD']),head);
  assert.throws(()=>verifyFullRepositoryProof({...request,head:base}),/FullProofCheckoutIdentity/u);
  assert.throws(()=>verifyFullRepositoryProof({...request,expectedRuntimeSha256:'0'.repeat(64)}),/PinnedReplayRuntimeMismatch/u);
  assert.throws(()=>verifyFullRepositoryProof({...request,expectedSourceIdentity:'0'.repeat(64)}),/PinnedReplaySourceIdentityMismatch/u);
  assert.throws(()=>verifyFullRepositoryProof({...request,expectedTaskURL:'https://example.invalid/wrong'}),/PinnedReplayTaskMismatch/u);
  const dirty=path.join(workspace,'untracked.txt');fileSystem.writeFileSync(dirty,'not committed');
  assert.throws(()=>verifyFullRepositoryProof(request),/FullProofCheckoutNotClean/u);fileSystem.unlinkSync(dirty);
  const internal=path.join(request.directory,'not-independent.json');fileSystem.writeFileSync(internal,JSON.stringify(request),{mode:0o600});
  const self=invoke(internal);assert.notEqual(self.status,0);assert.match(self.stderr,/FullProofIndependentRequestRequired/u);fileSystem.unlinkSync(internal);
  const chainFile=path.join(request.directory,'chain.lino'),chain=fileSystem.readFileSync(chainFile);
  const rows=chain.toString('utf8').trimEnd().split('\n');fileSystem.writeFileSync(chainFile,rows.slice(0,-1).join('\n')+'\n');
  try{const truncated=invoke(requestFile);assert.equal(truncated.status,2);const unknown=decodeTyped(truncated.stdout.trimEnd());
   assert.equal(unknown.verification,'Unknown');assert.equal(unknown.trustedVerification,false);assert.equal(unknown.fullReplay.verified,false);
  }finally{fileSystem.writeFileSync(chainFile,chain);}
  fileSystem.appendFileSync(delivered,'\n// actual foreign delta\n');git(workspace,['add','retained.mjs']);git(workspace,['commit','-m','foreign']);
  assert.throws(()=>verifyFullRepositoryProof({...request,head:git(workspace,['rev-parse','HEAD'])}),/FullProofRepositoryCoverageIncomplete/u);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});

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
test('declared atomic append cannot substitute an unowned path, content or request identity in a create operation',async()=>{
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
   const result=await drive(malicious,workspace,'Create file guarded.mjs containing export const guarded = 29;',
    {steps:6,atomicRecordAppend:true});
   assert.equal(result.stop,'unbound-create-operation',JSON.stringify(result));
   assert.equal(fileSystem.existsSync(path.join(workspace,'guarded.mjs')),false);
   assert.equal(fileSystem.existsSync(path.join(workspace,'unowned.lino')),false);
   assert.equal(fileSystem.existsSync(path.join(workspace,PLAN_PATH)),false);
  }
  assert(originalAppend!==null);
  const workspace=path.join(directory,'undeclared');fileSystem.mkdirSync(workspace);
  const refused=await drive(async()=>({kind:'tool_calls',calls:[originalAppend]}),workspace,
   'Create file guarded.mjs containing export const guarded = 29;',{steps:1,atomicRecordAppend:false});
  assert.equal(refused.stop,'undeclared-append-contract');assert.equal(fileSystem.existsSync(path.join(workspace,PLAN_PATH)),false);
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});}
});
