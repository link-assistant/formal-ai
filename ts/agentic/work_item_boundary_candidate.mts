// Intended owner: js/agentic/work_item_boundary_candidate.mjs.
// This adapter binds a declared finite subset to current observed files. It
// does not own the surrounding task or discharge unsupported obligations.
import {readText,parseLino,host,hasHost} from './host.mjs';
import {sentences} from './shell_command_policy.mjs';
import {sha256Hex} from './crate/source_fetch.mjs';
import {Progress} from './progress.mjs';
import {Capability} from './capability.mjs';
import {toolFor} from './capability_router.mjs';
import {planOne,jsonText,writeArguments,resolvedFinalAnswer,FinalDisposition} from './plan.mjs';
import {clausesWithSpans} from './crate/obligation_ledger.mjs';
import {readPolicyBlocksPlan} from './file_read/ownership.mjs';

const words = value => value.toLowerCase().replace(/[`'"()[\],;:]/gu,' ').trim().replace(/\s+/gu,' ');
function grammar() {
  const parsed=parseLino(readText('data/meta/comparison-boundary-grammar.lino'));
  const root=parsed.name==='comparison-boundary-grammar'?parsed:parsed.children?.find(node=>node.name==='comparison-boundary-grammar');
  if (!root) throw Error('BoundaryGrammarUnavailable');
  return root.children;
}
const values=(rules,key)=>rules.filter(node=>node.name===key).map(node=>node.value??node.id);
const mentions=(text,surfaces)=>surfaces.some(value=>(' '+text+' ').includes(' '+words(value)+' '));
const validOwner=path=>typeof path==='string' && /^[A-Za-z0-9_-][A-Za-z0-9_./-]*$/u.test(path)
  &&path.split('/').every(part=>part!==''&&part!=='.'&&part!=='..');

/** Select one explicitly requested structural leaf. Remaining clauses stay in
 * the controller ledger; their text never becomes permission for this leaf. */
export function boundaryWorkItemContract(task,pathsForClause) {
  if (typeof task!=='string'||new TextEncoder().encode(task).length>32768||typeof pathsForClause!=='function')return null;
  let rules;try{rules=grammar();}catch{return null;}
  const clauses=sentences(task), owned=[];
  for (const clause of clauses) {
    const text=words(clause.text);
    const regression=mentions(text,values(rules,'regression-action'));
    const repair=mentions(text,values(rules,'repair-action'));
    if ((!regression&&!repair)||mentions(text,values(rules,'negated-property'))
        ||mentions(text,values(rules,'negated-action')))continue;
    const upper=mentions(text,values(rules,'upper-bound')),lower=mentions(text,values(rules,'lower-bound'));
    const inclusive=mentions(text,values(rules,'inclusive')),exclusive=mentions(text,values(rules,'exclusive'));
    if (upper===lower||inclusive===exclusive)continue;
    const sourceSuffix=values(rules,'source-suffix'),testSuffix=values(rules,'test-suffix');
    const referenced=pathsForClause(clause.text).filter(validOwner);
    const tests=referenced.filter(path=>testSuffix.some(suffix=>path.endsWith(suffix)));
    const sources=referenced.filter(path=>sourceSuffix.some(suffix=>path.endsWith(suffix))&&!tests.includes(path));
    if (sources.length!==1||tests.length!==(regression?1:0))continue;
    let declaration=null,ambiguous=false;
    // Identifier capture is owned by a seeded declaration marker, never by a
    // source comment or an issue-specific function name.
    const tokens=clause.text.split(/\s+/u).map(value=>value.replace(/^[`'"(]+|[`'"),.;:]+$/gu,''));
    for (let index=0;index<tokens.length-1;index++) {
      if (!values(rules,'declaration-marker').some(marker=>words(marker)===words(tokens[index])))continue;
      const next=tokens[index+1];
      if (!/^[A-Za-z_$][A-Za-z0-9_$]*$/u.test(next)){ambiguous=true;break;}
      if (declaration!==null&&declaration!==next){ambiguous=true;break;}
      declaration=next;
    }
    if (ambiguous)continue;
    const target=regression?tests[0]:sources[0];
    const clauseStart=task.indexOf(clause.text,clause.span.start);
    if (clauseStart<clause.span.start||clauseStart+clause.text.length>clause.span.end)continue;
    const sourceSpan=[new TextEncoder().encode(task.slice(0,clauseStart)).length,
      new TextEncoder().encode(task.slice(0,clauseStart+clause.text.length)).length];
    owned.push({schema:'work-item-comparison-contract/v1',mode:regression?'regression':'repair',
      owner:sources[0],target,declaration,bound:upper?'upper':'lower',inclusive,
      sourceClause:clause.text,sourceSpan,
      evidencePaths:[sources[0]],targetObservationRequired:true,allowMissing:regression?[target]:[],
      independentAcceptanceRequired:true,scope:'finite-number-interval-only'});
  }
  return owned.length===1?owned[0]:null;
}

const complete=observation=>observation&&observation.error===null&&observation.complete===true
  &&observation.absent!==true&&typeof observation.source==='string';

/** The transcript itself retains the candidate preimage across planner calls.
 * A read AFTER our effect is readback, never a replacement baseline that could
 * erase first-attempt authorship or make an incomplete write appear a no-op. */
export function boundaryCandidateObservations(contract,messages) {
  const progress=Progress.scan(messages),attempts=progress.attempts;
  const firstEffect=attempts.findIndex(attempt=>[Capability.Write,Capability.Edit].includes(attempt.capability)
    &&ownedPath(argumentsOf(attempt),contract.target));
  const before=firstEffect<0?attempts:attempts.slice(0,firstEffect),result=new Map();
  for (const path of [contract.owner,contract.target]) {
    const observation=[...before].reverse().find(attempt=>attempt.capability===Capability.Read
      &&ownedPath(argumentsOf(attempt),path))?.source_read??null;
    result.set(path,observation);
  }
  return result;
}

/** `operation` is an installed caller-owned capability. Do not deserialize one
 * from fetched data. Observations are current Progress.sourceReadFor receipts.
 * Returned candidate still requires guarded physical effect + fresh readback. */
export function deriveBoundaryWorkItemCandidate(contract,observations,operation) {
  if (!contract||typeof operation!=='function')return {gap:'BoundaryHostCapabilityUnavailable'};
  const get=path=>observations instanceof Map?observations.get(path):observations[path];
  const source=get(contract.owner),target=get(contract.target);
  if (!complete(source))return {gap:'BoundaryCompleteSourceRequired',missingEvidence:[contract.owner]};
  if (new TextEncoder().encode(source.source).length>131072)return {gap:'BoundarySourceInputLimit'};
  if (!target)return {gap:'BoundaryTargetObservationRequired',missingEvidence:[contract.target]};
  if (contract.mode==='repair'&&!complete(target))return {gap:'BoundaryCompleteTargetRequired'};
  if (contract.mode==='regression'&&!complete(target)&&target.absent!==true)return {gap:'BoundaryTargetAvailabilityUnresolved'};
  let result;
  try {result=operation({source:source.source,owner:contract.owner,declaration:contract.declaration,
    bound:contract.bound,inclusive:contract.inclusive,mode:contract.mode,
    ...(contract.mode==='regression'?{testOwner:contract.target}:{})});}
  catch(error){return {gap:String(error?.message??'BoundaryCandidateRefused')};}
  if (!result||result.schema!=='observed-comparison-boundary-operation/v1'
      ||result.candidate?.path!==contract.target||typeof result.candidate.source!=='string'
      ||result.candidate.afterSha256!==sha256Hex(result.candidate.source))return {gap:'BoundaryCandidateIdentityMismatch'};
  if (contract.mode==='repair'&&result.candidate.beforeSha256!==sha256Hex(source.source))return {gap:'BoundaryCandidatePreimageMismatch'};
  if (contract.mode==='regression'&&result.candidate.sourceOwnerSha256!==sha256Hex(source.source))return {gap:'BoundaryCandidateSourceMismatch'};
  const before=complete(target)?target.source:null;
  if (contract.mode==='regression'&&before!==null&&before!==result.candidate.source)return {gap:'BoundaryExistingRegressionPreserved'};
  return {candidate:{kind:'structural-comparison-boundary',mode:contract.mode,path:contract.target,
    before,after:result.candidate.source,requiresAbsent:before===null,
    sourceOwner:contract.owner,sourceIdentity:sha256Hex(source.source),
    beforeIdentity:before===null?null:sha256Hex(before),afterIdentity:result.candidate.afterSha256,
    noOp:before===result.candidate.source,sourceClause:contract.sourceClause,sourceSpan:contract.sourceSpan,
    preconditions:[{kind:'complete-current-source',path:contract.owner,sha256:sha256Hex(source.source)},
      before===null?{kind:'absent-target',path:contract.target}:{kind:'exact-target-preimage',path:contract.target,sha256:sha256Hex(before)}],
    checks:[{kind:'structural-contract',status:'passed',detail:result.verification},
      {kind:'physical-readback',status:'pending'},{kind:'independent-task-acceptance',status:'pending'}],
    rule:result.rule??null,sourceModel:result.sourceModel,
    suppliedMechanism:true,autonomousAuthorshipCredit:0,independentlyAccepted:false}};
}

/** A transport Write success alone never satisfies this check. A fresh exact
 * target observation must occur AFTER the bound effect (caller enforces order).
 * `noOp` earns idempotence credit but zero first-attempt implementation credit. */
export function verifyBoundaryCandidateReadback(candidate,observation) {
  if (!candidate||!complete(observation))return {verified:false,reason:'BoundaryCompleteReadbackRequired'};
  if (observation.source!==candidate.after||sha256Hex(observation.source)!==candidate.afterIdentity)return {verified:false,reason:'BoundaryPhysicalReadbackMismatch'};
  return {verified:true,noOp:candidate.noOp,implementationChanged:!candidate.noOp,
    sourceIdentity:candidate.afterIdentity,independentTaskAcceptance:false};
}

function argumentsOf(attempt) {try{return JSON.parse(attempt.arguments);}catch{return null;}}
function ownedPath(argumentsValue,path) {
  if (!argumentsValue)return false;
  const paths=['path','filePath','file_path'].filter(key=>Object.hasOwn(argumentsValue,key)).map(key=>argumentsValue[key]);
  return paths.length>0&&paths.every(value=>value===path);
}
const readArguments=path=>jsonText({path,filePath:path,file_path:path});

/** Execute ONLY a persisted candidate. The enclosing controller retains task
 * identity, candidate source hashes and obligation statuses across these calls.
 * This function returns a pending tool step or an observed effect result; it
 * never labels the surrounding task independently accepted or complete. */
export function planBoundaryCandidateEffect(candidate,messages,tools) {
  const progress=Progress.scan(messages),read=toolFor(tools,Capability.Read);
  const effects=progress.attempts.map((attempt,index)=>({attempt,index,args:argumentsOf(attempt)}))
    .filter(({attempt,args})=>[Capability.Write,Capability.Edit].includes(attempt.capability)&&ownedPath(args,candidate.path));
  const effect=effects.at(-1)??null;
  if (effects.length>1)return {gap:'BoundaryEffectAttemptBudgetExceeded',budget:{maximum:1,observed:effects.length}};
  if (effect) {
    const after=effect.args.content??effect.args.newString??effect.args.new_string;
    if (after!==candidate.after)return {gap:'BoundaryUnrelatedTargetEffect'};
    if (!effect.attempt.succeeded)return {failure:'BoundaryEffectFailed',detail:effect.attempt.detail};
    const reads=progress.attempts.slice(effect.index+1).filter(attempt=>attempt.capability===Capability.Read&&ownedPath(argumentsOf(attempt),candidate.path));
    const observed=reads.at(-1)?.source_read??null;
    if (!observed)return read?{step:planOne(read,readArguments(candidate.path))}:{gap:'BoundaryReadbackToolMissing'};
    const checked=verifyBoundaryCandidateReadback(candidate,observed);
    return checked.verified?{effect:{...checked,path:candidate.path,authorshipBoundary:'candidate-output-only',independentlyAccepted:false}}
      :{failure:checked.reason};
  }
  const target=progress.sourceReadFor(candidate.path);
  if (candidate.noOp) {
    const checked=verifyBoundaryCandidateReadback(candidate,target);
    return checked.verified?{effect:{...checked,path:candidate.path,authorshipBoundary:'existing-artifact-no-authorship',independentlyAccepted:false}}
      :{gap:checked.reason};
  }
  if (candidate.requiresAbsent) {
    if (target?.absent!==true)return {gap:'BoundaryTargetPreconditionChanged'};
    const write=toolFor(tools,Capability.Write);
    return write?{step:planOne(write,writeArguments(candidate.path,candidate.after))}:{gap:'BoundaryWriteToolMissing'};
  }
  if (!complete(target)||target.source!==candidate.before||sha256Hex(target.source)!==candidate.beforeIdentity)return {gap:'BoundaryTargetPreconditionChanged'};
  const edit=toolFor(tools,Capability.Edit);
  return edit?{step:planOne(edit,jsonText({filePath:candidate.path,oldString:candidate.before,newString:candidate.after}))}:{gap:'BoundaryEditToolMissing'};
}

/** Production route wrapper. Call for the actual fetched objective before the
 * generic failure/read fallback, so a requested NEW test's proven ENOENT can be
 * treated as target precondition rather than a work-item transport failure.
 * `pathsForClause` is the existing shared path parser; no circular import.
 * All whole-task outcomes remain Gap pending external independent acceptance. */
export function planWorkItemBoundaryStep(task,messages,tools,workItem,pathsForClause) {
  const contract=boundaryWorkItemContract(task,pathsForClause);
  if (!contract)return null;
  const progress=Progress.scan(messages),observations=[];
  let candidate=null,effect=null;
  const finish=(reason,detail=null)=>{
    const authored=effect?.implementationChanged===true;
    const obligations=clausesWithSpans(task).map(([text,span])=>({text,span,
      status:span[0]<=contract.sourceSpan[0]&&span[1]>=contract.sourceSpan[1]
        ?'independent_check_pending':'unresolved'}));
    const candidateView=candidate?{kind:candidate.kind,mode:candidate.mode,path:candidate.path,
      beforeIdentity:candidate.beforeIdentity,afterIdentity:candidate.afterIdentity,
      sourceIdentity:candidate.sourceIdentity,sourceOwner:candidate.sourceOwner,
      preconditions:candidate.preconditions,noOp:candidate.noOp,
      sourceRepresentation:{kind:'complete-readable-cst',identity:sha256Hex(JSON.stringify(candidate.sourceModel))},
      checks:candidate.checks.map(check=>check.kind==='physical-readback'
        ?{...check,status:effect?.verified===true?'passed':'pending'}:check)}:null;
    return resolvedFinalAnswer(jsonText({schema:'repository-work-item-controller/v1',
      reason,workItem:workItem??null,taskIdentity:sha256Hex(task),requirements:obligations,
      supportedSubset:'finite-number-interval-boundary-and-regression',
      boundContract:contract,observations,candidate:candidateView,
      checks:candidateView?.checks??[],effects:effect?[effect]:[],
      authored,verified:false,independentlyAccepted:false,complete:false,
      physicalReadbackVerified:effect?.verified===true,noOp:effect?.noOp===true,
      usefulChange:{observed:authored,independentlyAccepted:false,paths:authored?[contract.target]:[]},
      authorshipBoundary:{mechanism:'supplied-generic-structural-operation',
        targetOutputObserved:authored,autonomousCredit:0,laterReviewerWork:'not-observed'},
      remainingWork:obligations.map(obligation=>({text:obligation.text,span:obligation.span,
        reason:obligation.status==='independent_check_pending'?'independent-requirement-acceptance-required':'unsupported-or-unbound-obligation'})),
      budget:{maximumEffects:1,maximumEvidenceFiles:2},detail,
    }),FinalDisposition.Gap,'repository_work_item_boundary');
  };
  const read=toolFor(tools,Capability.Read);
  for (const path of [...new Set([contract.owner,contract.target])]) {
    const observed=progress.sourceReadFor(path);
    if (!observed) {
      if (!read)return finish('MissingReadTool');
      const step=planOne(read,readArguments(path));
      return readPolicyBlocksPlan(task,step)?finish('ReadPolicyRefused'):step;
    }
    const isComplete=complete(observed),allowedAbsent=contract.allowMissing.includes(path)&&observed.absent===true;
    observations.push({path,complete:isComplete,absent:observed.absent===true,error:observed.error??null,status:observed.status,
      sourceIdentity:isComplete?sha256Hex(observed.source):null});
    if (!isComplete&&!allowedAbsent)return finish('IncompleteSourceObservation');
  }
  const operation=hasHost()?host().comparisonBoundaryOperation:null;
  const proposed=deriveBoundaryWorkItemCandidate(contract,boundaryCandidateObservations(contract,messages),operation);
  if (!proposed.candidate)return finish(proposed.gap??'BoundaryCandidateUnavailable',proposed);
  candidate=proposed.candidate;
  const next=planBoundaryCandidateEffect(candidate,messages,tools);
  if (next.step)return readPolicyBlocksPlan(task,next.step)?finish('ReadPolicyRefused'):next.step;
  if (next.effect){effect=next.effect;return finish(effect.noOp?'VerifiedNoOpAwaitingIndependentAcceptance':'UsefulCandidateAwaitingIndependentAcceptance');}
  return finish(next.failure??next.gap??'BoundaryExecutionUnresolved',next);
}
