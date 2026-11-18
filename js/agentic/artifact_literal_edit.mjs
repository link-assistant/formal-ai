// A contextual single-line substitution never serializes a Read presentation as source.
import {Progress} from './progress.mjs';
import {Capability} from './capability.mjs';
import {toolFor} from './capability_router.mjs';
import {editArguments} from './intent_router.mjs';
import {readArguments} from './workspace_change.mjs';
import {agenticMessage} from './messages.mjs';
import {finalAnswer,planOne} from './plan.mjs';
function targets(attempt,path){try{const arguments_=JSON.parse(attempt.arguments);return ['path','filePath','file_path'].some(key=>arguments_[key]===path);}catch{return false;}}
/** Mirrors `fn plan` in rust/src/agentic_coding/code_artifact/literal_edit.rs. */
export function planLiteralArtifactEdit(artifact,rewrite,messages,tools){
 const rule=rewrite.program.rules.length===1?rewrite.program.rules[0]:null;
 const edit=toolFor(tools,Capability.Edit),read=toolFor(tools,Capability.Read);
 if(!edit||!read||!rule||!rule.pattern||/[\r\n]/u.test(rule.pattern+rule.replacement))return null;
 const attempts=Progress.scan(messages).attempts.filter(attempt=>targets(attempt,artifact.path));
 const fail=()=>finalAnswer(agenticMessage('code-artifact-literal-unverified',{path:artifact.path}));
 const edited=attempts.findLastIndex(attempt=>attempt.capability===Capability.Edit);
 if(edited>=0){
  if(!attempts[edited].succeeded)return fail();
  const before=attempts.slice(0,edited).findLast(attempt=>attempt.capability===Capability.Read)?.source_read;
  const observed=attempts.slice(edited+1).findLast(attempt=>attempt.capability===Capability.Read);
  if(!observed)return planOne(read,readArguments(artifact.path));
  const after=observed.source_read;
  if(!observed.succeeded||before?.source==null||after?.source==null||before.error||after.error
    ||before.source.replace(rule.pattern,()=>rule.replacement)!==after.source)return fail();
  return finalAnswer(agenticMessage('code-artifact-literal-verified',{path:artifact.path}));
 }
 const observed=attempts.findLast(attempt=>attempt.capability===Capability.Read);
 if(!observed)return planOne(read,readArguments(artifact.path));
 const source=observed.source_read;
 if(!observed.succeeded||source?.source==null||source.error||source.source.split(rule.pattern).length!==2)return fail();
 return planOne(edit,editArguments(artifact.path,rule.pattern,rule.replacement));
}
