// Actual native provider observations reuse the installed creation proof; no new task grammar.
import {Progress} from '../../js/agentic/progress.mjs';
import {composeGeneralChangePlan} from '../../js/agentic/general_planner.mjs';
import {ownedCreationObservation} from '../../js/agentic/general_execution.mjs';
import {hasHost} from '../../js/agentic/host.mjs';
import {installNodeHost} from '../../js/agentic/node-host.mjs';
import {WorkerHost} from '../../js/server/worker-host.mjs';
export async function installNativeCaptureObservationHost(){
 if(hasHost())return;
 const forbidden=()=>{throw Error('NativeCaptureJavaScriptOnly');};
 await installNodeHost(new WorkerHost({WebAssembly:{compile:forbidden,instantiate:forbidden,
  instantiateStreaming:forbidden}}),{sourceUsageRustGrammar:false});
}
export function nativeLiteralCaptureObservation(session,legacy,verification={}){
 if(session.task!==legacy.task||!Array.isArray(session.steps)||!Array.isArray(session.tools_advertised))return null;
 const plan=composeGeneralChangePlan(legacy.task),target=legacy.steps.filter(step=>step.tool==='write_file').at(-1);
 const targetPath=target?.arguments.path??target?.arguments.filePath??target?.arguments.file_path;
 if(plan?.mode!=='literal_file'||plan.target!==targetPath||plan.content!==target?.arguments.content
  ||verification.command!==plan.verification_command||verification.stdout!==plan.content)return null;
 const messages=[{role:'user',content:legacy.task}];
 for(const [index,step]of session.steps.entries()){
  if(typeof step.tool!=='string'||typeof step.result!=='string'||!session.tools_advertised.includes(step.tool)
   ||!step.arguments||typeof step.arguments!=='object'||Array.isArray(step.arguments))return null;
  const id='native-capture-'+index;
  messages.push({role:'assistant',content:'',tool_calls:[{id,type:'function',function:{name:step.tool,arguments:JSON.stringify(step.arguments)}}]});
  messages.push({role:'tool',tool_call_id:id,name:step.tool,content:step.result,
   ...(step.source_read?{source_read:step.source_read}:{}),...(step.is_error?{is_error:true}:{})});
 }
 return ownedCreationObservation(plan,Progress.scan(messages));
}
