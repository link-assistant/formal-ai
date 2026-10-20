import {agenticMessage} from './messages.mjs';
import {Progress,qualifiedToolAttempt} from './progress.mjs';
import {Capability} from './capability.mjs';
import {toolFor} from './capability_router.mjs';
import {clausesWithSpans} from './crate/obligation_ledger.mjs';
import {sha256Hex} from './crate/source_fetch.mjs';
import {clientWorkingDirectory} from './content.mjs';
import {FinalDisposition,resolvedFinalAnswer,planOne,jsonText} from './plan.mjs';
import {incompleteReceipt} from './tool_result.mjs';

/** Lexical source candidates grant no ownership, implementation or acceptance authority. */
export function semanticSourceCandidates(task){
  if(typeof task!=='string'||new TextEncoder().encode(task).length>16384)return [];
  const seen=new Set(),out=[];
  for(const match of task.matchAll(/\b[A-Za-z_$][A-Za-z0-9_$]*\b/gu)){
    const text=match[0];
    if(!(/^[A-Za-z][A-Za-z0-9]*_[A-Za-z0-9_]+$/u.test(text)||/[a-z][A-Z]/u.test(text))||seen.has(text))continue;
    seen.add(text);const start=new TextEncoder().encode(task.slice(0,match.index)).length;
    out.push({text,span:[start,start+new TextEncoder().encode(text).length],authority:'lexical-candidate-only'});
    if(out.length===8)break;
  }
  return out;
}
export function planSemanticSourcePreflight(task,messages,tools){
  const candidates=semanticSourceCandidates(task);if(!candidates.length)return null;
  const progress=Progress.scan(messages),observations=[],paths=[];
  const finish=(reason)=>{
    const discovery={reason,taskIdentity:sha256Hex(task),clauses:clausesWithSpans(task).map(([text,span])=>({text,span,status:'unbound'})),candidates,observations,inferredWork:'mechanical-source-discovery',coverage:'bounded-candidates-and-paths-not-exhaustive',implementation:'Unknown',validation:'Unknown',authored:false,verified:false};
    const plan=resolvedFinalAnswer(agenticMessage('semantic_source_preflight_gap_output',{discovery:jsonText(discovery)}),FinalDisposition.Gap,'semantic-source-preflight-gap');plan.result.discovery=discovery;return plan;
  };
  const incompleteMarkers=agenticMessage('semantic_source_preflight_incomplete_markers').split('|');
  for(const candidate of candidates){
    const searchArguments={path:'.',pattern:'\\b'+candidate.text.replace(/[.*+?^${}()|[\]\\]/gu,'\\$&')+'\\b'};
    const observed=qualifiedToolAttempt(progress,Capability.Grep,searchArguments);
    if(!observed){const tool=toolFor(tools,Capability.Grep);return tool===null?finish('MissingGrepTool'):planOne(tool,jsonText(searchArguments));}
    const complete=observed.succeeded&&new TextEncoder().encode(observed.detail).length<=65536&&!incompleteReceipt(observed.detail)&&!incompleteMarkers.some(marker=>observed.detail.includes(marker));
    observations.push({candidate,arguments:searchArguments,providerSucceeded:observed.succeeded,complete,detail:observed.detail});
    if(!complete)return finish('IncompleteSourceSearch');
    const root=clientWorkingDirectory(messages);
    for(const line of observed.detail.split('\n')){
      if(!line.endsWith(':')||/^\s/u.test(line))continue;
      let path=line.slice(0,-1);if(root&&path.startsWith(root+'/'))path=path.slice(root.length+1);
      if(path.startsWith('./'))path=path.slice(2);
      if(!/^[A-Za-z0-9_.\/-]+$/u.test(path)||path.startsWith('/')||path.split('/').some(x=>x==='..'||x==='.'||!x))continue;
      if(!paths.includes(path)&&paths.length<8)paths.push(path);
    }
  }
  for(const path of paths){
    const observed=progress.sourceReadFor(path);
    if(!observed){const tool=toolFor(tools,Capability.Read);return tool===null?finish('MissingReadTool'):planOne(tool,jsonText({path,filePath:path,file_path:path}));}
    observations.push({path,sourceIdentity:observed.source===null?null:sha256Hex(observed.source??''),complete:observed.complete,status:observed.status,error:observed.error,bytes:observed.source===null?null:new TextEncoder().encode(observed.source??'').length});
    if(observed.error!==null||!observed.complete)return finish('IncompleteCandidateSource');
  }
  return finish('SourceBoundImplementationAndIndependentValidationMissing');
}
