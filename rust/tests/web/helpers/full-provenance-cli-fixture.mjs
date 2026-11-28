import assert from 'node:assert/strict';
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath}from 'node:url';
import {spawnSync}from 'node:child_process';
import {captureFreshPlannerRun,observeRuntimeExecutable}from '../../../../js/provenance/fresh-runtime.mjs';
import {repositoryGit,repositoryText}from '../../../../js/provenance/git-evidence.mjs';
import {trustedInstallationKey}from '../../../../js/provenance/installation-key.mjs';
import {PLAN_PATH}from '../../../../js/agentic/general_planner.mjs';
import {decodeTyped}from '../../../../js/logging/typed-codec.mjs';
import {verifyCallerRequestObservation}from '../../../../js/provenance/fresh-recompute.mjs';
import {retainedEvidenceBytes}from '../../../../js/provenance/verify-drive.mjs';
const git=(workspace,args)=>repositoryText(repositoryGit(workspace,args)).trimEnd();

// Each verifier scenario has independent evidence, workspace, key and request files.
export function withFullCliFixture(check){
 const directory=fileSystem.mkdtempSync(path.join(os.tmpdir(),'full-provenance-cli-'));
 const savedTrace=process.env.FORMAL_AI_PROVENANCE_TRACE;process.env.FORMAL_AI_PROVENANCE_TRACE='1';
 try{
  const workspace=path.join(directory,'work');fileSystem.mkdirSync(workspace);
  git(workspace,['init']);git(workspace,['config','user.name','Fixture']);git(workspace,['config','user.email','fixture@example.invalid']);
  fileSystem.writeFileSync(path.join(workspace,'base.txt'),'base\n');git(workspace,['add','base.txt']);git(workspace,['commit','-m','base']);
  const base=git(workspace,['rev-parse','HEAD']),runtime=observeRuntimeExecutable(),keys=path.join(directory,'keys');
  const command='git add -- retained.mjs '+PLAN_PATH+' && git commit -m observed --trailer "Formal-AI-Chain: $FORMAL_AI_CHAIN_PREFIX"';
  const prompt='Set the contents of retained.mjs to exactly this content: «export const sample = 97;».';
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
  const cli=fileURLToPath(new URL('../../../../js/provenance/cli.mjs',import.meta.url));
  const invoke=filename=>spawnSync(process.execPath,[cli,'verify-provenance','--request',filename],{
   encoding:'utf8',timeout:180000,maxBuffer:2*1024*1024,env:{...process.env,NODE_TEST_CONTEXT:undefined,NODE_OPTIONS:'',FORMAL_AI_PROVENANCE_TRACE:'1'}});
  const before=fileSystem.statSync(delivered,{bigint:true});
  return check({directory,workspace,captured,request,requestFile,invoke,delivered,head,base,before});
 }finally{fileSystem.rmSync(directory,{recursive:true,force:true});
  if(savedTrace===undefined)delete process.env.FORMAL_AI_PROVENANCE_TRACE;else process.env.FORMAL_AI_PROVENANCE_TRACE=savedTrace;
 }
}
