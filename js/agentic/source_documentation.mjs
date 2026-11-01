import {Progress} from './progress.mjs';
import {Capability} from './capability.mjs';
import {toolFor} from './capability_router.mjs';
import {readText,parseLino} from './host.mjs';
import {planOne,jsonText,resolvedFinalAnswer,FinalDisposition} from './plan.mjs';
import {workItemEvidencePaths} from './work_item_evidence.mjs';
import {sha256Hex} from './crate/source_fetch.mjs';
import {clausesWithSpans} from './crate/obligation_ledger.mjs';
import {sentences} from './shell_command_policy.mjs';
import {quotedSegmentSpans,quoteFault} from './crate/normal_markov.mjs';
import {readPolicyBlocksPlan} from './file_read/ownership.mjs';
import {replaceAllLiteral} from './crate/rust_str.mjs';

export function conceptWords(text) {
  return text.replace(/([a-z0-9])([A-Z])/gu,'$1 $2').toLowerCase().split(/[^\p{L}\p{N}]+/u).filter(Boolean);
}
function grammar() {
  try {
    const parsed=parseLino(readText('data/meta/source-documentation.lino'));
    const root=parsed.name==='source-documentation'?parsed:parsed.children?.find(node=>node.name==='source-documentation');
    const children=root?.children??[];
    const values=name=>children.filter(node=>node.name===name).map(node=>node.value??node.id);
    return {actions:values('action'),preserve:values('preserve'),negations:values('negation'),prefixes:values('prefix'),template:values('template')[0]};
  } catch {return null;}
}
function documentationAuthority(task,guidePath,policy) {
  if(quoteFault(task)!==null)return 'DocumentationActionUnowned';
  let masked=task;
  for(const span of [...quotedSegmentSpans(task)].reverse())masked=masked.slice(0,span.start)+masked.slice(span.start,span.end).replace(/[^\r\n]/g,' ')+masked.slice(span.end);
  if(!policy.preserve.some(cue=>conceptWords(masked).includes(cue)))return null;
  let mentioned=false;
  for(const clause of sentences(masked)){
    const original=task.slice(clause.span.start,clause.span.end);
    if(!original.includes(guidePath))continue;
    const words=conceptWords(clause.text);
    for(const action of policy.actions){
      const index=words.indexOf(action);if(index<0)continue;mentioned=true;
      const before=words.slice(0,index).join(' ');
      if(policy.negations.some(negation=>(' '+before+' ').includes(' '+conceptWords(negation).join(' ')+' ')))return 'NegatedDocumentationAction';
      if(index===0||policy.prefixes.includes(before))return 'owned';
    }
  }
  const raw=conceptWords(task);
  return mentioned||policy.actions.some(action=>raw.includes(action))?'DocumentationActionUnowned':null;
}
export function deriveSourceDocumentation(task,sourcePath,source,guide,template) {
  let object;
  try {object=JSON.parse(source);}catch{return {gap:'InvalidSourceJson'};}
  if(!object||typeof object!=='object'||Array.isArray(object))return {gap:'SourceObjectRequired'};
  const requested=new Set(conceptWords(task));
  const bindings=Object.entries(object).filter(([key,value])=>{
    const words=conceptWords(key);
    return words.length>0&&words.every(word=>requested.has(word))
      && ['string','boolean'].includes(typeof value);
  });
  if(bindings.length!==1)return {gap:bindings.length?'AmbiguousPropertyBinding':'MissingPropertyBinding'};
  const [key,value]=bindings[0];
  const text=String(value);
  if(/[\r\n`]/u.test(text)||/[\r\n`]/u.test(sourcePath)||typeof template!=='string')return {gap:'UnsupportedDocumentationValue'};
  const sourceBound=replaceAllLiteral(template,'{source}',sourcePath);
  const conceptBound=replaceAllLiteral(sourceBound,'{concept}',conceptWords(key).join(' '));
  const paragraph=replaceAllLiteral(conceptBound,'{value}',text);
  const suffix='\n\n'+paragraph+'\n';
  return {key,value,paragraph,sourceIdentity:sha256Hex(source),content:guide.includes(paragraph)?guide:guide.replace(/\n?$/u,'')+suffix};
}
const MAXIMUM_SOURCE_BYTES=262144;
function argumentsOf(attempt){try{return JSON.parse(attempt.arguments);}catch{return null;}}
function alias(argumentsValue,names){
  if(!argumentsValue||typeof argumentsValue!=='object'||Array.isArray(argumentsValue))return null;
  const values=names.filter(name=>Object.hasOwn(argumentsValue,name)).map(name=>argumentsValue[name]);
  return values.length>0&&values.every(value=>typeof value==='string'&&value===values[0])?values[0]:null;
}
function pathOf(attempt){return alias(argumentsOf(attempt),['filePath','file_path','path']);}
function complete(attempt){const value=attempt?.source_read;return attempt?.succeeded===true&&value?.error===null&&value.complete===true&&typeof value.source==='string'&&new TextEncoder().encode(value.source).length<=MAXIMUM_SOURCE_BYTES?value.source:null;}
export function planSourceDocumentationStep(task,messages,tools,workItem=null) {
  const policy=grammar();if(!policy)return null;
  const paths=workItemEvidencePaths(task),sources=paths.filter(path=>path.endsWith('.json')),guides=paths.filter(path=>path.endsWith('.md'));
  if(paths.length!==2||sources.length!==1||guides.length!==1)return null;
  const [sourcePath]=sources,[guidePath]=guides;
  const authority=documentationAuthority(task,guidePath,policy);if(authority===null)return null;
  const progress=Progress.scan(messages),attempts=progress.attempts;
  const requirements=clausesWithSpans(task).map(([text,span])=>({text,span,kind:'source-documentation',status:'unverified'}));
  if(!requirements.length)requirements.push({text:task,span:[0,new TextEncoder().encode(task).length],kind:'source-documentation',status:'unverified'});
  const state={schema:'source-documentation-controller/v1',task,taskIdentity:sha256Hex(task),workItem,requirements,source:sourcePath,target:guidePath,observations:[],candidateEdits:[],checks:[],effects:[],remainingWork:['independent-requirement-acceptance'],authored:false,verified:false,complete:false};
  const finish=(reason,failure=false)=>resolvedFinalAnswer(jsonText({...state,reason}),failure?FinalDisposition.Failure:FinalDisposition.Gap,'source-documentation-controller');
  if(authority!=='owned')return finish(authority);
  const read=path=>{const tool=toolFor(tools,Capability.Read);if(!tool)return finish('MissingReadTool');const step=planOne(tool,jsonText({filePath:path}));return readPolicyBlocksPlan(task,step)?finish('ReadPolicyRefused'):step;};
  if(attempts.some(attempt=>attempt.capability===Capability.Read&&pathOf(attempt)===null))return finish('AmbiguousReadPathAttempt',true);
  const mutations=attempts.map((attempt,index)=>({attempt,index})).filter(({attempt})=>attempt.capability===Capability.Edit||attempt.capability===Capability.Write);
  if(mutations.some(({attempt})=>attempt.capability!==Capability.Edit||pathOf(attempt)!==guidePath))return finish('UnrelatedOrAmbiguousMutationAttempt',true);
  if(mutations.length>1)return finish('MultipleEditAttempts',true);
  const readsFor=(path,before=attempts.length,after=-1)=>attempts.map((attempt,index)=>({attempt,index})).filter(item=>item.index<before&&item.index>after&&item.attempt.capability===Capability.Read&&pathOf(item.attempt)===path);
  for(const path of [sourcePath,guidePath]){
    const observed=readsFor(path).at(-1);
    if(!observed)return read(path);
    const content=complete(observed.attempt);
    if(content===null)return finish('IncompleteOrOversizedSourceObservation');
    state.observations.push({path,sourceIdentity:sha256Hex(content),complete:true});
  }
  const latest=mutations[0];
  if(latest){
    const originalSourceRead=readsFor(sourcePath,latest.index).find(item=>complete(item.attempt)!==null);
    const originalGuideRead=readsFor(guidePath,latest.index).at(-1);
    const originalSource=complete(originalSourceRead?.attempt),originalGuide=complete(originalGuideRead?.attempt);
    if(originalSource===null||originalGuide===null)return finish('EditPreconditionObservationMissing',true);
    const expected=deriveSourceDocumentation(task,sourcePath,originalSource,originalGuide,policy.template);
    if(expected.gap)return finish(expected.gap,true);
    const argumentsValue=argumentsOf(latest.attempt);
    const old=alias(argumentsValue,['oldString','old_string','old_str','old']),next=alias(argumentsValue,['newString','new_string','new_str','new']);
    state.binding={key:expected.key,value:expected.value,sourceIdentity:expected.sourceIdentity};
    state.candidateEdits.push({path:guidePath,preimageIdentity:sha256Hex(originalGuide),postimageIdentity:sha256Hex(expected.content)});
    if(old!==originalGuide||next!==expected.content||next===old)return finish('UnrelatedEditCandidate',true);
    if(!latest.attempt.succeeded)return finish('CandidateEditFailed',true);
    const sourceRead=readsFor(sourcePath,attempts.length,latest.index).at(-1);
    if(!sourceRead)return read(sourcePath);
    const observedSource=complete(sourceRead.attempt);
    if(observedSource===null||sha256Hex(observedSource)!==expected.sourceIdentity)return finish('SourceFreshnessMismatch',true);
    const guideRead=readsFor(guidePath,attempts.length,sourceRead.index).at(-1);
    if(!guideRead)return read(guidePath);
    const guide=complete(guideRead.attempt);
    if(guide!==expected.content)return finish('ReadbackMismatch',true);
    state.checks.push({kind:'source-freshness',passed:true},{kind:'exact-readback',passed:true});
    state.effects.push({path:guidePath,sourceIdentity:sha256Hex(guide)});
    return finish('UsefulCandidateReadbackVerified');
  }
  const source=complete(readsFor(sourcePath).at(-1).attempt),guide=complete(readsFor(guidePath).at(-1).attempt);
  const derived=deriveSourceDocumentation(task,sourcePath,source,guide,policy.template);
  if(derived.gap)return finish(derived.gap);
  state.binding={key:derived.key,value:derived.value,sourceIdentity:derived.sourceIdentity};
  if(derived.content===guide){state.checks.push({kind:'source-bound-noop',passed:true});return finish('VerifiedNoOp');}
  const edit=toolFor(tools,Capability.Edit);
  return edit?planOne(edit,jsonText({filePath:guidePath,oldString:guide,newString:derived.content})):finish('MissingEditTool');
}
