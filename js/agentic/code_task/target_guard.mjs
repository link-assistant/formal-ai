// Client-observed preimages authorize bounded generated-source replacement.
// Twin: rust/src/agentic_coding/code_task/target_guard.rs.
import {Capability} from '../capability.mjs';
import {toolFor} from '../capability_router.mjs';
import {sourceFromAgentReadResult,sourceFromReadResult} from '../code_artifact.mjs';
import {readArguments,resultForPath} from '../workspace_change.mjs';
import {failureMessage,renderFailure} from '../tool_result.mjs';
import {normalizePrompt} from '../crate/engine.mjs';
import {detect} from '../crate/language.mjs';
import {mentionsRole,mentionsRoleRaw} from '../crate/seed_meanings.mjs';
import {renderResponse} from '../crate/seed.mjs';
import {FinalDisposition,planOne,resolvedFinalAnswer} from '../plan.mjs';
export function guardedSourceStep(task,artifact,messages,tools) {
  const raw=resultForPath(messages,Capability.Read,artifact.path,null);
  const read=toolFor(tools,Capability.Read);
  const response=(intent,values)=>renderResponse(intent,detect(task),values)??renderResponse(intent,'en',values)??'';
  if(raw===null) {
    if(read!==null)return planOne(read,readArguments(artifact.path));
    return resolvedFinalAnswer(response('general_plan_unverified',[['target',artifact.path],['command','read '+artifact.path]]),FinalDisposition.Gap,'generated_source_preimage_unobserved');
  }
  const envelope=sourceFromAgentReadResult(raw);
  const failure=envelope===null?failureMessage(raw,false,true):null;
  if(failure!==null) {
    if(mentionsRoleRaw('filesystem-absent-result',normalizePrompt(failure)))return null;
    return resolvedFinalAnswer(renderFailure(read??'read',raw,task),FinalDisposition.Failure,'generated_source_preimage_failed');
  }
  const source=envelope??sourceFromReadResult(raw);
  if(source===''||source===artifact.content||mentionsRole('file_overwrite_consent',normalizePrompt(task)))return null;
  return resolvedFinalAnswer(response('general_change_existing_file_kept',[['path',artifact.path]]),FinalDisposition.Gap,'generated_source_existing_file_kept');
}
