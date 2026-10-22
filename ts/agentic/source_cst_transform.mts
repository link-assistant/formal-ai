import {quoteShellWord as quote} from './portable/string_contracts.mjs';
// JavaScript host route for the declared complete-CST parameter rule.
// Native planner parity is not implemented by this host-specific module.
import { agenticMessage } from './messages.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { Capability } from './capability.mjs';
import { classifyTool, toolFor } from './capability_router.mjs';
import { sourceFromReadResult } from './code_artifact.mjs';
import { planOne, jsonText, resolvedFinalAnswer, FinalDisposition } from './plan.mjs';
import { sourceCstTransformCommand, currentDirectory } from './host.mjs';
import { plainText } from './content.mjs';
import { stepOutcome, StepOutcome } from './tool_result.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';

/** A declaration-qualified request; ambiguity remains a refusal in the CST rule. */
export function sourceCstTransformRequest(task) {
 if(typeof task!=='string'||task.length>4096)return null;
 const match=/^Rename\s+(?:the\s+)?parameter\s+`?([A-Za-z_$][A-Za-z0-9_$]*)`?\s+(?:of|in)\s+(?:the\s+)?function\s+`?([A-Za-z_$][A-Za-z0-9_$]*)`?\s+to\s+`?([A-Za-z_$][A-Za-z0-9_$]*)`?\s+in\s+`?([A-Za-z0-9_./-]+\.(?:mjs|js|rs|mts|ts))`?\.?$/iu.exec(task.trim());
 if(!match)return null;
 const [,binding,declaration,target,owner]=match;
 if(owner.startsWith('/')||owner.split('/').some(part=>!part||part==='.'||part==='..'))return null;
 return {owner,declaration,binding,target};
}
/** Only a successful read of this exact requested owner supplies its preimage. */
function observedRead(messages,owner){
 for(let index=messages.length-1;index>=0;index-=1){
  const message=messages[index];if(message.role.toLowerCase()!=='tool'||!message.tool_call_id)continue;
  let call=null;
  for(let prior=index-1;prior>=0;prior-=1){call=(messages[prior].tool_calls??[]).find(candidate=>candidate.id===message.tool_call_id)??null;if(call)break;}
  if(!call||classifyTool(call.function.name)!==Capability.Read)continue;
  let argumentsValue;try{argumentsValue=JSON.parse(call.function.arguments);}catch{return {refusal:agenticMessage('source-cst-read-arguments')};}
  const paths=['path','filePath','file_path'].filter(name=>Object.hasOwn(argumentsValue,name)).map(name=>argumentsValue[name]);
  const absolute=(currentDirectory()??'').replace(/\/$/u,'')+'/'+owner;
  if(!paths.length||paths.some(value=>value!==owner&&value!==absolute))return {refusal:agenticMessage('source-cst-read-owner')};
  const raw=plainText(message.content);
  if(message.is_error===true||message.isError===true||stepOutcome(raw)===StepOutcome.Failed)return {refusal:agenticMessage('source-cst-read-failed')};
  return {source:sourceFromReadResult(raw)};
 }
 return null;
}
const refused=message=>resolvedFinalAnswer(message,FinalDisposition.Gap,'source-cst-transform-refused');
/** Source is observed by Read, never passed through command arguments or logs. */
export function planSourceCstTransformStep(task,messages,toolNames,planShellStep){
 const request=sourceCstTransformRequest(task);if(request===null)return null;
 const installed=sourceCstTransformCommand();
 if(installed===null)return refused(agenticMessage('source-cst-host-missing'));
 if(toolFor(toolNames,Capability.Run)===null)return refused(agenticMessage('source-cst-run-missing'));
 const current=messages.slice(evidenceWindowStart(messages));
 const read=observedRead(current,request.owner);
 if(read===null){const tool=toolFor(toolNames,Capability.Read);return tool?planOne(tool,jsonText({path:request.owner,filePath:request.owner,file_path:request.owner})):refused(agenticMessage('source-cst-read-missing'));}
 if(read.refusal)return refused(read.refusal);
 const expected=sha256Hex(read.source);
 const command=agenticMessage('installed-source-operation-command',{node:quote(installed[0]),script:quote(installed[1]),task:quote(task),digest:expected});
 return planShellStep(current,toolNames,command);
}
