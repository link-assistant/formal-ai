// Semantic diagnostics consume actual planner origins and provider receipts.
// They neither infer task delivery from prose nor grant execution authority.
import {observedFinalResult} from '../agentic/final_result.mjs';
import {stepOutcome,StepOutcome} from '../agentic/tool_result.mjs';
export function observedPlanTerminal(plan){
 if(plan?.kind!=='final')return null;
 const result=observedFinalResult(plan),origin=result?.origin??null;
 if(origin==='repeated_call_guard')return {reason:'repeated_call_guard',origin};
 if(origin==='planned_not_executed'||result?.disposition==='gap')return {reason:'planned_not_executed',origin};
 if(result?.disposition==='failure')return {reason:'tool_failure',origin};
 if(result?.disposition==='finding'&&typeof origin==='string'&&origin.length>0)return {reason:'success',origin};
 return {reason:'Unknown',origin};
}
export function observedExceptionTerminal(error){
 if(error?.name==='AbortError'||error?.code==='ABORT_ERR')return {reason:'cancellation'};
 if(error?.name==='TimeoutError'||error?.code==='ETIMEDOUT')return {reason:'timeout'};
 return {reason:'Unknown'};
}
export function observedToolTerminal(receipt){
 const content=receipt?.content??receipt?.result;
 let envelope=null;try{envelope=typeof content==='string'?JSON.parse(content):null;}catch{}
 if(envelope?.timeout_ms!==null&&envelope?.timeout_ms!==undefined||envelope?.timed_out===true)return {reason:'timeout'};
 if(envelope?.aborted===true||envelope?.signal||typeof content==='string'&&/\nSignal: [A-Z0-9]+$/u.test(content))return {reason:'cancellation'};
 if(receipt?.is_error===true||typeof content==='string'&&stepOutcome(content)===StepOutcome.Failed)return {reason:'tool_failure'};
 return null;
}
const refusalStops=new Set(['invalid-tools','invalid-append-policy','invalid-command-policy','command-policy-denied','undeclared-tool','undeclared-append-contract','missing-source-reference-effect-authority','unbound-read-operation','unbound-create-operation','unbound-adapter-operation','changed-create-workspace','unsupported-create-workspace','unowned-create-request','source-reference-consumer-refused','stale-source-reference-prerequisite','unbound-source-reference-prerequisite']);
export function createDriveEventSink(logger,{dialogId=null}={}){
 let lastPlan=null,lastTool=null,ended=false;
 const emit=(level,kind,fields)=>logger?.log(level,kind,{component:'agent',dialog_id:dialogId,...fields});
 return event=>{
  if(ended)return;
  if(event.kind==='plan'){
   lastPlan=observedPlanTerminal(event.plan);
   emit('debug','agent.plan',{plan_kind:event.plan?.kind??'none',...(lastPlan??{reason:'awaiting_tool_result'})});
  }else if(event.kind==='tool'){
   const outcome=observedToolTerminal(event.receipt);if(outcome)lastTool=outcome;
   emit(outcome?'warn':'debug','agent.tool_result',outcome??{reason:'observed',task_delivery:'Unknown'});
  }else if(event.kind==='terminal'){
   ended=true;const result=event.result;
   const outcome=refusalStops.has(result?.stop)?{reason:'refusal'}
    :result?.stop==='steps'?{reason:'planned_not_executed'}
    :result?.stop==='final'?(lastPlan?.reason==='Unknown'?(lastTool??lastPlan):lastPlan)??lastTool??{reason:'Unknown'}
    :lastTool??{reason:'Unknown'};
   emit(outcome.reason==='success'?'info':'warn','agent.terminal',{...outcome,stop:result?.stop??null,task_delivery:'Unknown',tool_calls:Array.isArray(result?.transcript)?result.transcript.length:0});
  }else if(event.kind==='exception'){
   ended=true;emit('error','agent.terminal',{...observedExceptionTerminal(event.error),stop:'exception',task_delivery:'Unknown'});
  }
 };
}
