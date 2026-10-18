import {finalAnswer,planOne,jsonText,writeArguments} from './plan.mjs';
import {agenticMessage} from './messages.mjs';
import {fill} from './work_item_steps.mjs';
import {splitWhitespace} from './write_str.mjs';
import {host,hasHost,readText} from './host.mjs';
import {parseRoot,findChildValue} from './crate/seed_parser.mjs';
import {Capability} from './capability.mjs';
import {toolFor} from './capability_router.mjs';
import {Progress,writeMatches} from './progress.mjs';
import {shellQuote} from './git_commit.mjs';
import {stepOutcome,StepOutcome} from './tool_result.mjs';
const COMMAND_NOT_FOUND_EXIT=127;
const notFoundMarker=()=>agenticMessage('prerequisite_recovery_not_found_marker');
const bootstrapPath='.formal-ai/bootstrap/prerequisite-installer.py';
export function classifyFailure(command, exitCode, stderr) {
  const program = splitWhitespace(command)[0] ?? '';
  if (!program) return null;
  const missing = exitCode === COMMAND_NOT_FOUND_EXIT || stderr.includes(notFoundMarker());
  const denied = exitCode === 126 || stderr.includes('Permission denied');
  if (!missing && !denied) return null;
  return { program, observed: missing ? 'missing' : 'unusable' };
}


function recoveryContext(command,program) {
  const grant=hasHost()&&typeof host().prerequisiteGrant==='function'?host().prerequisiteGrant():null;
  if(!grant || grant.scope!=='workspace' || !Array.isArray(grant.programs) || !grant.programs.includes(program))return null;
  if(grant.bootstrap?.program!=='python3'||jsonText(grant.bootstrap.arguments)!==jsonText([bootstrapPath])||grant.allowNetwork!==true)return null;
  const publisherNode=parseRoot(readText('data/seed/setup-publishers.lino')).children.find(node=>findChildValue(node,'program')===program);
  if(!publisherNode)return null;
  const publisher=Object.fromEntries(['program','source_id','host','documentation'].map(key=>[key,findChildValue(publisherNode,key)]));
  const probe=parseRoot(readText('data/seed/toolchains.lino')).children.find(node=>findChildValue(node,'program')===program);
  const dependencies=(findChildValue(probe,'requires')||'').split(/\s+/u).map(value=>value.replace(/^[()"]+|[()"]+$/gu,'')).filter(Boolean);
  const request=jsonText({program,publisher,dependencies,grant});
  const active='.formal-ai/toolchains/active-'+program+'.bin';
  const recovery='python3 '+shellQuote(bootstrapPath)+' '+shellQuote(request)+' && read -r __formal_ai_prerequisite_bin < '+shellQuote(active)+' && export PATH="$__formal_ai_prerequisite_bin:$PATH" &&\n# __formal_ai_prerequisite_retry\n'+command;
  return {grant,recovery,source:readText('data/meta/prerequisite-installer.py')};
}
/** Exact authored transaction identity, separate from arbitrary marker-bearing text. */
export function recoveryOutput(messages,command) {
  const program=splitWhitespace(command)[0]??'';
  const context=recoveryContext(command,program);
  return context===null?null:Progress.scan(messages).latestRunOutputFor(context.recovery);
}
export function planRecovery(messages,tools,command,exitCode,output) {
  const need=classifyFailure(command,exitCode,output);
  if(!need||need.observed!=='missing')return null;
  const context=recoveryContext(command,need.program);
  if(context===null)return finalAnswer(fill('prerequisite_grant_required_report',[['{program}',need.program],['{command}',command],['{output}',output]]));
  const progress=Progress.scan(messages),observed=progress.latestRunOutputFor(context.recovery);
  if(observed!==null)return stepOutcome(observed)===StepOutcome.Failed?finalAnswer(observed):null;
  const write=toolFor(tools,Capability.Write),run=toolFor(tools,Capability.Run);
  if(write===null||run===null)return finalAnswer(fill('prerequisite_grant_exceeded_report',[]));
  const latestWrite=progress.latestSuccessfulWriteIndex(bootstrapPath);
  const written=latestWrite!==null&&writeMatches(progress.attempts[latestWrite].arguments,bootstrapPath,context.source);
  return written?planOne(run,jsonText({command:context.recovery})):planOne(write,writeArguments(bootstrapPath,context.source));
}
