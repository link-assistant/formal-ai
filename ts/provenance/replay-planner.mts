// Recompute real planner decisions; never claim external effects or unknown engine genesis were replayed.
import {planChatStep} from '../agentic/planner.mjs';
import {verifyDriveEvidence,regularEvidenceBytes,retainedEvidenceBytes} from './verify-drive.mjs';
import {decodeTyped} from '../logging/typed-codec.mjs';
import {withEvidenceContext} from './capture-context.mjs';
import {blake3Hex} from '../crypto/blake3.mjs';
const digest=value=>blake3Hex(Buffer.from(JSON.stringify(value),'utf8'));
export async function recomputePlannerEvidence(directory,workspace,options={}){
 verifyDriveEvidence(directory,workspace,options.expectedAnchor,{trustedPublicKey:options.trustedPublicKey});
 const rows=regularEvidenceBytes(directory,'chain.lino').toString('utf8').trimEnd().split('\n').map(decodeTyped);
 const comparisons=[];
 for(const row of rows){
  if(row.kind!=='planner_thinking')continue;
  const observed=JSON.parse(retainedEvidenceBytes(directory,row.payload.evidence).toString('utf8'));
  if(observed.schema!=='formal-ai-planner-observation-v1'){
    comparisons.push({record:row.hash,status:'not-recomputed',reason:'RecordedPlannerFailure'});continue;
  }
  let actualObservation=null;
  const context={active:true,planningObservation:value=>{actualObservation=value;},engineObservation:()=>{}};
  const actual=await withEvidenceContext(context,()=>planChatStep(observed.messages,observed.tools));
  const outputMatches=digest(actual)===digest(observed.projected);
  const traceMatches=actualObservation!==null&&digest(actualObservation.eventLog)===digest(observed.eventLog)
    &&actualObservation.derivation===observed.derivation;
  comparisons.push({record:row.hash,status:outputMatches&&traceMatches?'matched':'mismatch',outputMatches,traceMatches});
 }
 return {schema:'formal-ai-planner-recomputation-v1',comparisons,
  plannerComputation:comparisons.length>0&&comparisons.every(row=>row.status==='matched')?'matched':'not-matched',
  fullReplay:{verified:false,reason:'AuthoritativeEngineGenesisNotYetBound'},
  externalEffects:'not-reexecuted',authorship:'not-established'};
}
