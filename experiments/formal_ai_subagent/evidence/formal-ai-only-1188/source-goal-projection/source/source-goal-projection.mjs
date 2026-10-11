// Reviewed scratch projection: observed source only, no synthesis or evaluation.
import { latestUserRequest } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/content.mjs';
import { Progress } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/progress.mjs';
import { SourceReadStatus } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/tool_result.mjs';
import { FinalDisposition, resolvedFinalAnswer } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/final_result.mjs';
import { observeSourceCallables } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/module_function/callable_catalog.mjs';
import { sha256Hex } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/source_fetch.mjs';
import { Capability } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/capability.mjs';
import { toolFor } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/capability_router.mjs';
import { toolCalls, plannedCall } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/plan.mjs';
import { readArguments } from '/Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/file_read.mjs';
const bytes = source => new TextEncoder().encode(source);
function outcome(disposition, reason, witness = {}) {
 const projection={reason,authored:false,executed:false,...witness};
 const plan=resolvedFinalAnswer(JSON.stringify(projection),disposition,'source-declaration-projection');
 plan.result.projection=projection;
 return plan;
}
// Explicit typed query only. Natural-language goal classification is not implemented.
export function planSourceProjectionGoal(goal,messages,tools) {
 const fields=['kind','request','path','selector','contentId','requiredConstraints'];
 if(!goal||goal.kind!=='source-declaration'||Object.keys(goal).some(key=>!fields.includes(key)))
   return outcome(FinalDisposition.Gap,'UnsupportedTerminalGoal');
 if(goal.request!==latestUserRequest(messages))return outcome(FinalDisposition.Gap,'GoalRequestChanged');
 if(typeof goal.path!=='string'||goal.path.length===0||!goal.selector
   ||!['local','export'].includes(goal.selector.kind)||typeof goal.selector.name!=='string'||goal.selector.name.length===0
   ||Object.keys(goal.selector).some(key=>!['kind','name'].includes(key)))return outcome(FinalDisposition.Gap,'MissingQueryOperand');
 if(goal.requiredConstraints!==undefined&&(!Array.isArray(goal.requiredConstraints)||goal.requiredConstraints.length>0))
   return outcome(FinalDisposition.Gap,'UnsupportedGoalConstraint');
 const read=Progress.scan(messages).sourceReadFor(goal.path);
 if(read===null){const tool=toolFor(tools,Capability.Read);return tool===null
   ?outcome(FinalDisposition.Gap,'SourceProviderUnavailable'):toolCalls([plannedCall(tool,readArguments(goal.path,{kind:'full'}))]);}
 if(read.error!==null)return outcome(FinalDisposition.Failure,'SourceProviderFailed',{path:goal.path,error:read.error});
 const source=read.source??'',identity=sha256Hex(bytes(source));
 const observed={path:goal.path,moduleContentId:identity,utf8Bytes:bytes(source).length,status:read.status,complete:read.complete};
 if(read.status!==SourceReadStatus.Success||!read.complete)return outcome(FinalDisposition.Gap,'CompleteBoundSourceRequired',{observed});
 if(goal.contentId!==undefined&&goal.contentId!==identity)return outcome(FinalDisposition.Gap,'SourceChanged',{observed});
 const catalog=observeSourceCallables(source,goal.path);
 if(catalog.gaps.some(gap=>['LexicalFailure','DuplicateBinding','DuplicateExport'].includes(gap.reason)))
   return outcome(FinalDisposition.Gap,'AmbiguousOrInvalidSource',{observed,gaps:catalog.gaps});
 let local=goal.selector.name;
 if(goal.selector.kind==='export'){const bindings=catalog.exports.filter(binding=>binding.exposed===goal.selector.name);
   if(bindings.length!==1)return outcome(FinalDisposition.Gap,'ExportUnresolved',{observed});local=bindings[0].local;}
 const declarations=catalog.declarations.filter(entry=>entry.name===local);
 if(declarations.length!==1)return outcome(FinalDisposition.Gap,'DeclarationUnobserved',{observed});
 const declaration=declarations[0];
 const selected=new TextDecoder('utf-8',{ignoreBOM:true}).decode(bytes(source).subarray(declaration.span.byteStart,declaration.span.byteEnd));
 if(selected!==declaration.source||source.slice(declaration.span.start,declaration.span.end)!==selected
   ||declaration.identity.moduleContentId!==identity||declaration.identity.declarationContentId!==sha256Hex(bytes(selected)))
   return outcome(FinalDisposition.Gap,'SourceSpanMismatch',{observed});
 return outcome(FinalDisposition.Finding,'ObservedDeclaration',{
   goal:{kind:goal.kind,path:goal.path,selector:goal.selector},observed,
   declaration:{name:local,parameters:declaration.parameters,source:selected,span:declaration.span,identity:declaration.identity,contract:declaration.contract},
   module:{effects:catalog.moduleEffects,syntax:catalog.moduleSyntax,imports:catalog.imports,
     initialization:catalog.initialization.map(region=>({kind:region.kind,effects:region.effects,span:region.span,contentId:region.contentId}))},
   certification:'exact-lexical-source-region-only',semanticImplementation:false});
}
