// Failure identities are diagnostics only; no source bytes or private paths escape.
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const names=new Set(['Error','TypeError','ReferenceError','SyntaxError','RangeError','EvalError','URIError','AggregateError','AssertionError']);
const codes=new Set(['UnclassifiedRuntimeFailure','RuntimeChildFailure','ERR_ASSERTION','ERR_MODULE_NOT_FOUND',
 'ERR_INVALID_ARG_TYPE','ERR_INVALID_MODULE_SPECIFIER','ERR_UNKNOWN_FILE_EXTENSION',
 'FreshReplayLiteralJournalAuthorityMismatch','FreshReplayCallerRequestMismatch','FreshReplayPlannerMismatch',
 'FreshReplayRawPlanMismatch','FreshReplayDispatchedPlanMismatch','FreshReplayGenesisMismatch',
 'FreshReplayMemoryMismatch','FreshReplayEnvironmentMismatch','FreshReplayFinalMemoryMismatch',
 'FreshRuntimeProcessFailed','FreshRuntimeExecutableBinding','FreshRuntimeBootstrapChanged','PinnedReplayLoadedSourceMismatch']);
const modules=new Set(['js/provenance/fresh-recompute.mjs','js/provenance/fresh-planner-engine.mjs',
 'js/provenance/fresh-runtime.mjs','js/provenance/fresh-verifier.mjs','js/provenance/literal-journal-authority.mjs',
 'js/provenance/pinned-source-comparison.mjs','js/provenance/runtime-failure-identity.mjs',
 'js/agentic/planner.mjs','js/agentic/planner/owned_goals.mjs','js/agentic/general_execution.mjs',
 'js/agentic/general_planner.mjs','js/agentic/tool_result.mjs','js/agentic/progress.mjs','js/agentic/host.mjs',
 'js/agentic/node-host.mjs','js/server/node-source-session-host.mjs','experiments/js_dogfood/drive.mjs']);
const knownModule=value=>modules.has(value);
export function checkedRuntimeFailureIdentity(value){
 if(!value||typeof value!=='object'||Array.isArray(value)||!names.has(value.name)||!codes.has(value.code)
  ||Object.keys(value).some(key=>!['name','code','module','line','column'].includes(key)))return null;
 const result={name:value.name,code:value.code};
 if(value.module!==undefined||value.line!==undefined||value.column!==undefined){
  if(!knownModule(value.module)||!Number.isSafeInteger(value.line)||value.line<1||value.line>10000000
   ||!Number.isSafeInteger(value.column)||value.column<1||value.column>10000000)return null;
  Object.assign(result,{module:value.module,line:value.line,column:value.column});
 }
 return result;
}
export function runtimeFailureIdentity(error,repository){
 const result={name:names.has(error?.name)?error.name:'Error',
  code:codes.has(error?.code)?error.code:codes.has(error?.message)?error.message:'UnclassifiedRuntimeFailure'};
 const stack=typeof error?.stack==='string'?error.stack.slice(0,16384):'';
 for(const match of stack.matchAll(/(file:\/\/[^\s()]+):(\d+):(\d+)/gu)){
  try{
   const filename=fileURLToPath(match[1]),relative=path.relative(repository,filename).split(path.sep).join('/');
   const candidate={...result,module:relative,line:Number(match[2]),column:Number(match[3])};
   const checked=checkedRuntimeFailureIdentity(candidate);if(checked)return checked;
  }catch{}
 }
 return result;
}
