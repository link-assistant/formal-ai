import {Capability} from '../../../../js/agentic/capability.mjs';
import {classifyTool} from '../../../../js/agentic/capability_router.mjs';
import {sourceReadPath} from '../../../../js/agentic/progress.mjs';
import {readOnlyCapability} from '../../../../js/agentic/progress_command_readback.mjs';
import {ownedCreationReadbackCommand} from '../../../../js/agentic/general_execution.mjs';
import {sourceReadObservation,observedPayload,observedBytesMatch,reportedExitCode,incompleteReceipt,harnessReportedFailure} from '../../../../js/agentic/tool_result.mjs';

export function exactLiteralEffectProof(out,target,expected){
  const rows=out.transcript;
  const write=rows.findLastIndex(item=>{
    const args=JSON.parse(item.arguments);
    return classifyTool(item.tool)===Capability.Write&&!item.is_error
      &&sourceReadPath(item.arguments)===target&&args.content===expected;
  });
  if(write<0)return false;
  return rows.some((item,index)=>{
    if(index<=write||rows.slice(index+1).some(later=>!readOnlyCapability(classifyTool(later.tool))))return false;
    const capability=classifyTool(item.tool);
    if(capability===Capability.Read&&sourceReadPath(item.arguments)===target){
      const observed=sourceReadObservation(item.result,Boolean(item.is_error),item.source_read,target);
      return observed.error===null&&observed.complete&&observed.source===expected;
    }
    if(capability!==Capability.Run)return false;
    const args=JSON.parse(item.arguments);
    if(Object.keys(args).length!==1)return false;
    const command=args.command??args.cmd??args.script;
    if(item.is_error||reportedExitCode(item.result)!==0||incompleteReceipt(item.result)||harnessReportedFailure(item.result))return false;
    if(command===ownedCreationReadbackCommand(target))return observedPayload(item.result)==='('+expected+')';
    return command==='cat '+target&&observedBytesMatch(item.result,expected);
  });
}

