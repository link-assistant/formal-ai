// Request-bound journal authority shared by actual execution and pinned replay.
import {composeGeneralChangePlan,parseWriteContract} from '../agentic/general_planner.mjs';
import {ownsCompleteLiteralRequest,ownedDeclaredCreateFrame,goalLedger} from '../agentic/planner/owned_goals.mjs';
import {collectionSummaryOwns} from '../agentic/planner/collection_summary.mjs';
import {bareSurfaces} from '../agentic/write_request.mjs';
import {normalizePrompt} from '../agentic/crate/engine.mjs';
import {writesWholeFile} from '../agentic/literal_write_guard.mjs';
export function literalJournalAuthority(prompt,exclusiveCreate=false){
 const plan=composeGeneralChangePlan(prompt),frame=parseWriteContract(prompt);
 return !exclusiveCreate&&ownsCompleteLiteralRequest(prompt)&&writesWholeFile(prompt)
  &&plan?.mode==='literal_file'&&frame!==null&&plan.target===frame.target&&plan.content===frame.content?plan:null;
}
export function literalJournalCallsMatch(authority,calls,canonical){
 if(authority===null)return true;
 if(canonical?.kind!=='tool_calls'||!Array.isArray(canonical.calls)||!Array.isArray(calls)
  ||calls.length===0||calls.length>canonical.calls.length)return false;
 const remaining=canonical.calls.slice();
 for(const call of calls){
  if(!call||typeof call.tool!=='string'||typeof call.arguments!=='string')return false;
  const index=remaining.findIndex(owned=>owned?.tool===call.tool&&owned.arguments===call.arguments);
  if(index<0)return false;
  remaining.splice(index,1);
 }
 return true;
}

export function driverRequestAuthority(prompt){
  const ownedCreate = ownedDeclaredCreateFrame(prompt);
  const declaredGoals = goalLedger(prompt);
  const collectionGoals = Array.isArray(declaredGoals) && collectionSummaryOwns(declaredGoals)
    ? declaredGoals.slice(1) : declaredGoals;
  const firstUnsupported = Array.isArray(collectionGoals) ? collectionGoals.findIndex(goal => goal.kind === 'unsupported') : -1;
  const missingClause = firstUnsupported < 0 ? null : normalizePrompt(collectionGoals[firstUnsupported].clause);
  const independentMissing = missingClause !== null && bareSurfaces('enumeration_cue').some(cue => {
    const normalized = normalizePrompt(cue);
    return missingClause.startsWith(normalized) && /[\s,，:：]/u.test(missingClause[normalized.length] ?? '');
  });
  const supportedGoals = Array.isArray(collectionGoals) && firstUnsupported > 0 && independentMissing
    ? collectionGoals.slice(0, firstUnsupported) : collectionGoals;
  const mixedGoals = Array.isArray(supportedGoals) && supportedGoals.some(goal => goal.kind === 'source_edit');
  const collection = Array.isArray(supportedGoals) && supportedGoals.length > 0
    && supportedGoals.every(goal => ['literal_file', 'source_edit'].includes(goal.kind)
      && goal.sourceUnit === 'utf16' && prompt.slice(goal.span.start, goal.span.end) === goal.clause)
    ? supportedGoals.filter(goal => goal.kind === 'literal_file').map(goal => {
      const frame = ownedDeclaredCreateFrame(goal.clause);
      return frame !== null && frame.target === goal.target && frame.content === goal.expected
        ? Object.freeze({ ...frame, request: goal.clause }) : null;
    }) : null;
  const createContracts = ownedCreate !== null
    ? [Object.freeze({ ...ownedCreate, request: prompt })]
    : collection !== null && collection.every(frame => frame !== null)
      && new Set(collection.map(frame => frame.target)).size === collection.length ? collection : [];
  const createContract = createContracts[0] ?? null;
  // A whole-file literal journal has its own custody; it never grants exclusive creation.
  const literalJournal = literalJournalAuthority(prompt,createContract !== null);
  return {declaredGoals,mixedGoals,createContracts,createContract,literalJournal};
}
