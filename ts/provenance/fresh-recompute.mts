// Recompute the actual pinned planner and persisted logical-memory transitions.
// External tool responses are recorded inputs; this path never executes tool effects.
import path from 'node:path';
import fileSystem from 'node:fs';
import {withEvidenceContext}from './capture-context.mjs';
import {verifyDriveEvidence,regularEvidenceBytes,retainedEvidenceBytes,firstEvidenceRecord}from './verify-drive.mjs';
import {readBoundedRuntimeFile,normalizeRuntimeRequests}from './fresh-runtime.mjs';
import {decodeTyped,encodeTyped}from '../logging/typed-codec.mjs';
import {blake3Hex}from '../crypto/blake3.mjs';
import {logicalMemoryStep}from './memory-evidence.mjs';
import {projectDriverDispatch,projectDriverHistory}from '../../experiments/js_dogfood/drive.mjs';
export function verifyCallerRequestObservation(recorded,index,request,sourceMaintenanceOwnership){
 if(!request||!recorded||!Object.hasOwn(recorded,'sourceMaintenanceOwnership')
  ||encodeTyped(recorded)!==encodeTyped({index,...request,sourceMaintenanceOwnership}))throw Error('FreshReplayCallerRequestMismatch');
 return recorded;
}
export async function recomputeFreshRun(configuration,engine){
 const input=path.resolve(configuration.replayInput);
 const bundle=JSON.parse(readBoundedRuntimeFile(path.dirname(input),path.basename(input),2*1024*1024).toString('utf8'));
 if(!bundle||typeof bundle.directory!=='string'||!bundle.expectedAnchor
  ||fileSystem.realpathSync(bundle.workspace)!==configuration.workspace)throw Error('FreshReplayRequestBinding');
 const report=verifyDriveEvidence(bundle.directory,bundle.workspace,bundle.expectedAnchor,
  {trustedPublicKey:bundle.trustedPublicKey});
 if(!report.logicalMemory||!configuration.memoryFile)throw Error('FreshReplayLogicalMemoryRequired');
 const records=regularEvidenceBytes(bundle.directory,'chain.lino').toString('utf8').trimEnd().split('\n').map(decodeTyped);
 const retained=evidence=>retainedEvidenceBytes(bundle.directory,evidence);
 const contexts=records.filter(record=>record.kind==='runtime_context');
 if(contexts.length!==1)throw Error('FreshReplayRuntimeContextRequired');
 const observed=JSON.parse(retained(contexts[0].payload.evidence).toString('utf8'));
 if(observed.productVersion!==engine.packageVersion()||observed.runtime!==process.version
  ||observed.workingDirectory!==process.cwd()||observed.engineProfile!=='javascript-product-wasm-disabled'
  ||observed.request.prompt!==configuration.prompt)throw Error('FreshReplayRuntimeContextMismatch');
 if(encodeTyped(observed.prerequisiteGrant)!==encodeTyped(engine.host().prerequisiteGrant?.()??null))throw Error('FreshReplayCallerGrantMismatch');
 const transportKeys=new Set(['FORMAL_AI_RUNTIME_CONFIG','FORMAL_AI_RUNTIME_BOOTSTRAP']);
 const keys=new Set([...Object.keys(observed.environment),...Object.keys(process.env)]);
 for(const key of keys)if(!transportKeys.has(key)&&observed.environment[key]!==process.env[key])throw Error('FreshReplayEnvironmentMismatch');
 const memory=engine.openMemoryCustody({file:configuration.memoryFile,workspace:configuration.workspace,
  sessionId:bundle.expectedAnchor.sessionId,taskURL:report.logicalMemory.taskURL});
 let state=report.logicalMemory.initialMemoryHash,cycle=null,latestPlan=null,actualPlanning=null;
 let observedGenesis=null,expectedGenesis=null;const actualSolves=[],pendingSolves=[],comparisons=[];
 let inputCount=0,outputCount=0,planningCount=0;
 const context={active:true,engineGenesis:observe=>{if(observedGenesis===null)observedGenesis=observe();},
  planningObservation:value=>{actualPlanning=JSON.parse(JSON.stringify(value));},
  engineObservation:value=>{actualSolves.push(JSON.parse(JSON.stringify(value)));}};
 function advance(kind,bytes,expected){
  const transition=memory.advance(kind,bytes),step=logicalMemoryStep(state,transition);
  if(transition.memoryHash!==expected.memoryHash||step.state!==expected.state
   ||step.deltaHash!==expected.deltaHash||transition.revision!==expected.revision)throw Error('FreshReplayMemoryMismatch');
  state=step.state;
 }
 try{
  const initial=memory.snapshot();
  if(initial.memoryHash!==report.logicalMemory.initialMemoryHash
   ||initial.revision!==report.logicalMemory.initialRevision)throw Error('FreshReplayInitialMemoryMismatch');
  await engine.worker.run('0');
  const session=engine.host().sourceSession;
  if(!session||typeof session.run!=='function')throw Error('FreshReplaySourceSessionRequired');
  const declared=normalizeRuntimeRequests(observed.request.prompt,observed.request.steps,observed.requests);
  const markers=records.filter(record=>record.kind==='caller_request');
  const groups=markers.length?markers.map((marker,index)=>{
   const request=declared[index],recorded=JSON.parse(retained(marker.payload.evidence).toString('utf8'));
   if(!request||marker.payload.index!==index)throw Error('FreshReplayCallerRequestMismatch');
   verifyCallerRequestObservation(recorded,index,request,engine.ownedSourceMaintenanceRequest(request.prompt));
   const start=records.indexOf(marker)+1,end=index+1<markers.length?records.indexOf(markers[index+1]):records.length;
   return{request,records:records.slice(start,end)};
  }):[{request:declared[0],records}];
  if(markers.length&&markers.length!==declared.length||!markers.length&&declared.length!==1)throw Error('FreshReplayCallerRequestCount');
  for(const group of groups){
   const request=group.request,requestRecords=group.records;
   const firstInput=firstEvidenceRecord(requestRecords,'message_in');
   if(!firstInput)throw Error('FreshReplayCallerInputMissing');
   const tools=JSON.parse(retained(firstInput.payload.evidence).toString('utf8')).tools;
   await session.run({request:request.prompt,workspace:configuration.workspace,tools},async()=>{
    for(let index=0;index<requestRecords.length;index++){
    const record=requestRecords[index];
    if(record.kind==='engine_genesis'){
     const recorded=JSON.parse(retained(record.payload.evidence).toString('utf8'));
     for(const field of ['sources','inputs']){
      const restored=[];
      for(const entry of recorded[field]){
       const {evidence,...identity}=entry;
       restored.push(evidence?{...identity,...JSON.parse(retained(evidence).toString('utf8'))}:identity);
      }
      recorded[field]=restored;
     }
     expectedGenesis=recorded;continue;
    }
    if(record.kind==='message_in'){
     if(cycle!==null)throw Error('FreshReplayMessageOrder');
     cycle={input:JSON.parse(retained(record.payload.evidence).toString('utf8')),raw:null};
     advance('message_in',retained(record.payload.evidence),record.payload);inputCount++;continue;
    }
    if(record.kind==='engine_solve'){pendingSolves.push(JSON.parse(retained(record.payload.evidence).toString('utf8')));continue;}
    if(record.kind==='planner_thinking'){
     const expected=JSON.parse(retained(record.payload.evidence).toString('utf8'));
     if(expected.schema!=='formal-ai-planner-observation-v1')throw Error('FreshReplayUnsupportedPlannerFailure');
     actualPlanning=null;actualSolves.length=0;
     latestPlan=await withEvidenceContext(context,()=>engine.planChatStep(expected.messages,expected.tools));
     if(JSON.stringify(actualPlanning)!==JSON.stringify(expected)
      ||JSON.stringify(actualSolves)!==JSON.stringify(pendingSolves))throw Error('FreshReplayPlannerMismatch');
     pendingSolves.length=0;planningCount++;continue;
    }
    if(record.kind==='planner_response'){
     if(cycle===null||blake3Hex(Buffer.from(JSON.stringify(latestPlan)))!==record.payload.evidence.blake3)throw Error('FreshReplayRawPlanMismatch');
     cycle.raw=latestPlan;continue;
    }
    if(record.kind==='message_out'){
     if(cycle===null)throw Error('FreshReplayOutputOrder');
     const dispatched=projectDriverDispatch(cycle.raw,{prompt:request.prompt,tools:cycle.input.tools,
      scoped:cycle.input.messages,verificationHistory:projectDriverHistory(cycle.input.messages,[]),sourceSession:session});
     const bytes=Buffer.from(JSON.stringify(dispatched));
     if(blake3Hex(bytes)!==record.payload.evidence.blake3)throw Error('FreshReplayDispatchedPlanMismatch');
     const transitionRecord=records.find(item=>item.hash===record.payload.memoryTransition);
     if(!transitionRecord)throw Error('FreshReplayOutputTransitionMissing');
     advance('message_out',bytes,transitionRecord.payload);outputCount++;
     comparisons.push({message:record.hash,output:'matched',state:'matched'});cycle=null;
    }
   }
   });
  };
  if(cycle!==null||pendingSolves.length||inputCount<1||inputCount!==outputCount||planningCount<1)throw Error('FreshReplayIncomplete');
  if(expectedGenesis===null||encodeTyped(observedGenesis)!==encodeTyped(expectedGenesis))throw Error('FreshReplayGenesisMismatch');
  const final=memory.snapshot();
  if(final.memoryHash!==report.logicalMemory.finalMemoryHash||final.revision!==report.logicalMemory.finalRevision
   ||state!==report.logicalMemory.state)throw Error('FreshReplayFinalMemoryMismatch');
  return {stop:'recomputed',recomputation:{schema:'formal-ai-fresh-planner-recomputation-v1',
   inputCount,outputCount,planningCount,comparisons,finalMemoryHash:final.memoryHash,state,
   engineGenesis:'matched',externalEffects:'not-reexecuted-recorded-tool-responses-used-as-inputs'},
   fullReplay:{verified:false,reason:'CallerMustAuthenticatePinnedRuntimeSourcesEnvironmentAndTask'}};
 }finally{memory.close();}
}
