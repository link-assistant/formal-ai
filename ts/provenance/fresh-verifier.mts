import {observedProvenancePhase} from './phase-trace.mjs';
// Caller-pinned signature/runtime/source/task checks around actual fresh recomputation.
import fileSystem from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {fileURLToPath}from 'node:url';
import {captureFreshPlannerRun,observeRuntimeExecutable,collectRuntimeObservation}from './fresh-runtime.mjs';
import {readBoundedFileBytes}from './bounded-file.mjs';
import {verifyDriveEvidence,regularEvidenceBytes,retainedEvidenceBytes,firstEvidenceRecord}from './verify-drive.mjs';
import {encodeTyped,decodeTyped}from '../logging/typed-codec.mjs';
import {blake3Hex}from '../crypto/blake3.mjs';
export function verifyPinnedFreshReplay(options){
 if(!options||typeof options!=='object')throw Error('PinnedReplayRequestRequired');
 const {directory,workspace,expectedAnchor,trustedPublicKey,expectedRuntimeSha256,expectedSourceIdentity,expectedTaskURL}=options;
 if(!expectedAnchor||typeof trustedPublicKey!=='string'||!trustedPublicKey
  ||typeof expectedRuntimeSha256!=='string'||!/^[0-9a-f]{64}$/u.test(expectedRuntimeSha256)
  ||typeof expectedSourceIdentity!=='string'||!/^[0-9a-f]{64}$/u.test(expectedSourceIdentity)
  ||typeof expectedTaskURL!=='string')throw Error('PinnedReplayIndependentTrustRequired');
 const report=observedProvenancePhase('pinned-evidence-integrity',()=>verifyDriveEvidence(directory,workspace,expectedAnchor,{trustedPublicKey}));
 const records=regularEvidenceBytes(directory,'chain.lino').toString('utf8').trimEnd().split('\n').map(decodeTyped);
 if(!report.logicalMemory||report.logicalMemory.taskURL!==expectedTaskURL)throw Error('PinnedReplayTaskMismatch');
 if(records[0].payload.sourceId!==expectedSourceIdentity)throw Error('PinnedReplaySourceIdentityMismatch');
 const executable=observeRuntimeExecutable();
 if(executable.sha256!==expectedRuntimeSha256)throw Error('PinnedReplayRuntimeMismatch');
 if(!records.some(record=>record.kind==='caller_request'))throw Error('PinnedReplayCallerRequestObservationRequired');
 const contexts=records.filter(record=>record.kind==='runtime_context');
 if(contexts.length!==1)throw Error('PinnedReplayRuntimeContextRequired');
 const context=JSON.parse(retainedEvidenceBytes(directory,contexts[0].payload.evidence).toString('utf8'));
 const inventories=records.filter(record=>record.kind==='runtime_sources');
 if(inventories.length!==2)throw Error('PinnedReplaySourceInventoryRequired');
 const checked=new Set(),expectedSources=[];let sourceCount=0;
 for(const inventory of inventories){
  const observed=JSON.parse(retainedEvidenceBytes(directory,inventory.payload.evidence).toString('utf8'));
  if(observed.runtimeExecutable?.sha256!==expectedRuntimeSha256)throw Error('PinnedReplayObservedRuntimeMismatch');
  for(const entry of observed.sources){
   expectedSources.push(entry);
   const key=entry.url+'\0'+entry.format+'\0'+entry.blake3;if(checked.has(key))continue;checked.add(key);
   const address=new URL(entry.url);
   if(address.protocol!=='file:'||address.search||address.hash)throw Error('PinnedReplaySourceUnavailable');
   const filename=fileURLToPath(address),body=readBoundedFileBytes(path.dirname(filename),path.basename(filename),4*1024*1024);
   const actual=entry.format==='commonjs'||entry.format==='json'||entry.format==='repository-read-string'
    ?Buffer.from(body.toString('utf8')):body;
   if(blake3Hex(actual)!==entry.blake3)throw Error('PinnedReplaySourceChanged');sourceCount++;
  }
 }
 const owned=fileSystem.mkdtempSync(path.join(os.tmpdir(),'formal-ai-pinned-replay-'));fileSystem.chmodSync(owned,0o700);
 try{
  const genesis=firstEvidenceRecord(records,'logical_memory_genesis');
  const memoryFile=path.join(owned,'memory.lino');
  fileSystem.writeFileSync(memoryFile,retainedEvidenceBytes(directory,genesis.payload.evidence),{flag:'wx',mode:0o600});
  if(genesis.payload.revisionEvidence!==null)fileSystem.writeFileSync(memoryFile+'.provenance-revision.lino',
   retainedEvidenceBytes(directory,genesis.payload.revisionEvidence),{flag:'wx',mode:0o600});
  const first=firstEvidenceRecord(records,'message_in'),input=JSON.parse(retainedEvidenceBytes(directory,first.payload.evidence).toString('utf8'));
  const replayInput=path.join(owned,'input.json');
  fileSystem.writeFileSync(replayInput,JSON.stringify({directory,workspace,expectedAnchor,trustedPublicKey,tools:input.tools}),{flag:'wx',mode:0o600});
  const fresh=observedProvenancePhase('pinned-recompute',()=>captureFreshPlannerRun({directory:path.join(owned,'run'),workspace,prompt:context.request.prompt,
   steps:context.request.steps,atomicRecordAppend:context.driver?.atomicRecordAppend??false,...(context.requests?{requests:context.requests}:{}),memoryFile,taskURL:expectedTaskURL,replayInput}));
  if(fresh.custody.runtimeExecutable.sha256!==expectedRuntimeSha256||fresh.result.stop!=='recomputed'
   ||fresh.result.recomputation.outputCount<1)throw Error('PinnedReplayRecomputationIncomplete');
  if(encodeTyped(context.bootstrapArtifacts)!==encodeTyped(fresh.custody.bootstrap))throw Error('PinnedReplayBootstrapChanged');
  const actualSources=collectRuntimeObservation(path.join(fresh.directory,'transport')).sources;
  const actualKeys=new Set(actualSources.map(entry=>entry.url+'\0'+entry.format+'\0'+entry.blake3));
  const originalRuntime=fileSystem.realpathSync(path.dirname(path.dirname(path.resolve(directory))));
  const freshRuntime=fileSystem.realpathSync(fresh.directory);
  for(const entry of expectedSources){
   const filename=fileURLToPath(entry.url),relative=path.relative(originalRuntime,filename);
   const owned=context.bootstrapArtifacts.some(artifact=>artifact.file===relative);
   const address=owned?new URL(relative,new URL('file://'+freshRuntime+'/')).href:entry.url;
   if(!actualKeys.has(address+'\0'+entry.format+'\0'+entry.blake3))throw Error('PinnedReplayLoadedSourceMismatch');
  }
  return {schema:'formal-ai-pinned-fresh-replay-v1',taskURL:expectedTaskURL,
   runtime:{sha256:expectedRuntimeSha256,version:executable.version},sourceIdentity:expectedSourceIdentity,sourceCount,
   recomputation:fresh.result.recomputation,
   fullReplay:{verified:true,scope:'available-pinned-js-engine-logical-memory-and-recorded-tool-inputs'},
   repository:'not-checked',autonomousAuthorship:'not-established'};
 }finally{fileSystem.rmSync(owned,{recursive:true,force:true});}
}
