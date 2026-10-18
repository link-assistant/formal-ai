import {observedProvenancePhase} from './phase-trace.mjs';
// Compose actual Git execution custody, a clean tracked checkout, and caller-pinned fresh replay.
// Private source/tool/environment archives stay outside the repository and are never uploaded here.
import fileSystem from 'node:fs';
import path from 'node:path';
import {createHash}from 'node:crypto';
import {repositoryGit,repositoryText}from './git-evidence.mjs';
import {verifyRepositoryEvidence}from './repository-verifier.mjs';
import {verifyPinnedFreshReplay}from './fresh-verifier.mjs';
import {verifyArchivedFreshReplay}from './replay-archived.mjs';
import {regularEvidenceBytes,retainedEvidenceBytes}from './verify-drive.mjs';
import {decodeTyped}from '../logging/typed-codec.mjs';
function verifyExecutionBinding(workspace,directory,records,commitRecord){
 const executions=records.filter(record=>record.kind==='commit_execution'&&record.payload.commitRecord===commitRecord.hash);
 if(executions.length!==1)throw Error('FullProofCommitExecutionRequired');
 const execution=executions[0],payload=execution.payload;
 const process=records.find(record=>record.hash===payload.process),start=records.find(record=>record.hash===payload.toolStart);
 const transition=records.find(record=>record.hash===payload.transition);
 if(!process||process.kind!=='tool_process'||!start||start.kind!=='tool_start'
  ||!transition||transition.kind!=='repository_transition')throw Error('FullProofExecutionLinks');
 const observed=process.payload;
 const anchor=records.find(record=>record.hash===commitRecord.payload.precommitAnchor.hash);
 if(!anchor||anchor.index<start.index||anchor.index>=process.index
  ||start.payload.beforeHead!==payload.before)throw Error('FullProofPreExecutionAnchorRequired');
 if(start.index>=process.index||process.index>=commitRecord.index||commitRecord.index>=execution.index
  ||observed.toolStart!==start.hash||observed.messageOut!==commitRecord.payload.messageOut
  ||start.payload.messageOut!==observed.messageOut||transition.payload.toolStart!==start.hash
  ||transition.payload.process!==process.hash||payload.commit!==commitRecord.payload.commit
  ||JSON.stringify(payload.parents)!==JSON.stringify(commitRecord.payload.parents))throw Error('FullProofExecutionOrder');
 if(observed.scope!=='actual-shell-process-completion-and-snapshotted-streams'
  ||!Number.isSafeInteger(observed.pid)||observed.pid<=0||observed.status!==0||observed.signal!==null||observed.error!==null
  ||fileSystem.realpathSync(observed.cwd)!==workspace)throw Error('FullProofProcessNotSuccessful');
 const call=JSON.parse(retainedEvidenceBytes(directory,start.payload.call).toString('utf8'));
 const argumentsValue=JSON.parse(call.arguments);
 if(call.tool!=='bash'||argumentsValue.command!==observed.command)throw Error('FullProofCommandBinding');
 for(const field of ['stdout','stderr']){
  const stream=observed[field],bytes=retainedEvidenceBytes(directory,stream.evidence);
  if(bytes.length!==stream.bytes||createHash('sha256').update(bytes).digest('hex')!==stream.sha256)throw Error('FullProofStreamBinding');
 }
 const actual=repositoryText(repositoryGit(workspace,['rev-list','--reverse','--parents',payload.before+'..'+payload.after])).trimEnd().split('\n');
 if(payload.before!==transition.payload.before||payload.after!==transition.payload.after
  ||JSON.stringify(actual)!==JSON.stringify(transition.payload.topology)
  ||!actual.includes([payload.commit,...payload.parents].join(' ')))throw Error('FullProofCommitRangeBinding');
 const results=records.filter(record=>record.kind==='tool_result'&&record.payload.toolStart===start.hash);
 if(results.length!==1||results[0].payload.process!==process.hash||results[0].index<=process.index)throw Error('FullProofToolResultRequired');
 const receipt=JSON.parse(retainedEvidenceBytes(directory,results[0].payload.evidence).toString('utf8'));
 const nextInput=records.find(record=>record.kind==='message_in'&&record.index>results[0].index);
 if(!nextInput)throw Error('FullProofObservedToolInputRequired');
 const input=JSON.parse(retainedEvidenceBytes(directory,nextInput.payload.evidence).toString('utf8'));
 if(!input.messages.some(message=>message.role==='tool'&&message.name==='bash'&&message.content===receipt.content))throw Error('FullProofToolInputBinding');
 return {commit:payload.commit,process:process.hash,toolStart:start.hash,execution:execution.hash,status:observed.status};
}
export function verifyFullRepositoryProof(options){
 if(!options||typeof options!=='object'||Array.isArray(options))throw Error('FullProofRequestRequired');
 const {directory,workspace,expectedAnchor,trustedPublicKey,base,head,minShare=1}=options;
 const sourceMode=options.sourceMode??'installed';
 if(!['installed','archived'].includes(sourceMode))throw Error('FullProofSourceModeUnsupported');
 if(typeof directory!=='string'||typeof workspace!=='string'||typeof base!=='string'||typeof head!=='string'
  ||!(/^[a-f0-9]{40}$/u.test(base)&&/^[a-f0-9]{40}$/u.test(head)))throw Error('FullProofImmutableRangeRequired');
 const working=fileSystem.realpathSync(workspace),evidence=fileSystem.realpathSync(directory);
 if(evidence===working||evidence.startsWith(working+path.sep))throw Error('FullProofEvidenceInsideWorkspace');
 const text=args=>repositoryText(repositoryGit(working,args)).trimEnd();
 if(fileSystem.realpathSync(text(['rev-parse','--show-toplevel']))!==working||text(['rev-parse','--verify','HEAD'])!==head)throw Error('FullProofCheckoutIdentity');
 if(repositoryGit(working,['status','--porcelain=v1','-z','--untracked-files=all','--ignore-submodules=none']).length!==0)throw Error('FullProofCheckoutNotClean');
 const lines=regularEvidenceBytes(evidence,'chain.lino').toString('utf8').trimEnd().split('\n'),records=lines.map(decodeTyped);
 if(records.at(-1)?.kind!=='terminal'||records.at(-1).payload.reason!=='final')throw Error('FullProofSessionNotFinal');
 const seal=decodeTyped(regularEvidenceBytes(evidence,'seal.lino').toString('utf8').trimEnd());
 const repository=observedProvenancePhase('repository-custody',()=>verifyRepositoryEvidence(working,{lines,expectedAnchor,trustedPublicKey,seal,base,head,minShare}));
 if(!repository.meetsMinimum||!repository.allBlobsAttributed||repository.commits.length===0)throw Error('FullProofRepositoryCoverageIncomplete');
 const execution=[];
 for(const observed of repository.commits){
  const matches=records.filter(record=>record.kind==='commit'&&record.payload.commit===observed.commit);
  if(matches.length!==1)throw Error('FullProofCommitObservationAmbiguous');
  execution.push(verifyExecutionBinding(working,evidence,records,matches[0]));
 }
 const replayOptions={...options,directory:evidence,workspace:working};
 const replay=observedProvenancePhase('full-replay-'+sourceMode,()=>sourceMode==='archived'?verifyArchivedFreshReplay(replayOptions):verifyPinnedFreshReplay(replayOptions));
 if(!replay.fullReplay.verified)throw Error('FullProofFreshReplayIncomplete');
 if(text(['rev-parse','--verify','HEAD'])!==head
  ||repositoryGit(working,['status','--porcelain=v1','-z','--untracked-files=all','--ignore-submodules=none']).length!==0)throw Error('FullProofCheckoutChanged');
 return {schema:'formal-ai-full-repository-proof-v1',verification:'Verified',trustedVerification:true,
  taskURL:replay.taskURL,sourceIdentity:replay.sourceIdentity,runtime:replay.runtime,fullReplay:replay.fullReplay,
  recomputation:replay.recomputation,repository:{...repository,workingTree:'tracked-and-nonignored-untracked-clean-at-head'},
  originalTerminalReason:records.at(-1).payload.reason,taskCorrectness:'requires-independent-task-oracle',
  execution,externalEffects:'observed-original-execution-not-reexecuted-during-replay',autonomousAuthorship:'not-established'};
}
