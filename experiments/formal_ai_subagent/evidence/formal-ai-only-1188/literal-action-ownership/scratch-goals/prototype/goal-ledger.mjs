import {gapAnswer} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/task_obligations.mjs';
import {stableId} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/engine_stable_identifier.mjs';
import {FinalDisposition,resolvedFinalAnswer,canDeliverFinal} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/plan.mjs';
import {quotedSegmentSpans,quoteFault} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/crate/normal_markov.mjs';
import {sentences} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/shell_command_policy.mjs';
import {composeEditClauses} from 'file:///Users/konard/Code/Archive/link-assistant/formal-ai/js/agentic/write_request.mjs';
import {literalWriteOwnership,instructionView} from './write-contract.mjs';
// Each clause keeps its raw source span. A source edit's bytes stay unknown until read.
export function goalLedger(request) {
  const contract=literalWriteOwnership(request);
  // A closed literal is authoritative even when the legacy multiline parse greedily extends to later clauses.
  const firstLiteral=contract===null?undefined:quotedSegmentSpans(request).find(span=>span.start>=contract.payload.start&&/^[\s:]*$/u.test(request.slice(contract.payload.start,span.start)));
  let view=firstLiteral===undefined?instructionView(request,contract):request;
  for(const span of quotedSegmentSpans(request))view=view.slice(0,span.start)+' '.repeat(span.end-span.start)+view.slice(span.end);
  const clauses=sentences(view).map(sentence=>({clause:request.slice(sentence.span.start,sentence.span.end).trim(),span:sentence.span}));
  if(clauses.length===0)return null;
  const goals=clauses.map((item,index)=>{
    const literal=literalWriteOwnership(item.clause);const edit=literal===null?composeEditClauses(item.clause):null;
    const delimiter=literal===null?undefined:quotedSegmentSpans(item.clause).find(span=>span.start>=literal.payload.start&&/^[\s:]*$/u.test(item.clause.slice(literal.payload.start,span.start)));
    const payloadEnd=delimiter?.end??literal?.payload.end;
    const completeLiteral=literal!==null&&/^[\s.!?。！？।;；]*$/u.test(item.clause.slice(payloadEnd));
    const kind=completeLiteral?'literal_file':edit!==null&&edit.spans!==null?'source_edit':'unsupported';
    return {...item,id:index,sourceUnit:'utf16',byteSpan:[new TextEncoder().encode(request.slice(0,item.span.start)).length,new TextEncoder().encode(request.slice(0,item.span.end)).length],kind,target:literal?.target??edit?.edit[0]??null,need:{kind,discharged:false},expected:literal?.content??null};
  });
  if(goals.length===1&&goals[0].kind!=='unsupported')return null;
  return goals.some(goal=>literalWriteOwnership(goal.clause)!==null)?goals:null;
}
export async function planGoalLedger(request,messages,toolNames,planFor) {
  const goals=goalLedger(request);if(goals===null)return null;
  if(quoteFault(request)!==null)return null;
  const missing=goals.find(goal=>goal.kind==='unsupported');
  // Existing obligation planning owns literal-only and underivable goal scheduling; do not replace its partial-delivery contract.
  if(missing){
    if(literalWriteOwnership(missing.clause)!==null)return goalGap(missing);
    if(goals[0].kind==='unsupported')return null;
    return replayOwnedGoals(goals,messages,toolNames,async(own,names,goal)=>goal.kind==='unsupported'
      ?goalGap(goal)
      :planFor(own,names));
  }
  if(!goals.some(goal=>goal.kind==='source_edit'))return null;
  return replayOwnedGoals(goals,messages,toolNames,planFor);
}
/** The conversation with its latest user turn asking `part` alone. */
function withRequest(messages, part) {
  let latest = -1;
  messages.forEach((message, index) => {
    if (String(message.role).toLowerCase() === 'user') latest = index;
  });
  return messages.map((message, index) => (index === latest ? { ...message, content: part } : message));
}

/**
 * Mirrors `fn plan_request_sequence_step`: the next step of the first
 * sentence not yet answered, planned by `planFor` as that sentence alone, or
 * the sentences' answers in order once each is answered.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 * @param {(messages: Array<object>, toolNames: Array<string>) => Promise<object|null>} planFor
 */
export async function replayOwnedGoals(parts, messages, toolNames, planFor) {
  const { base, exchanges } = turnExchanges(messages);
  let taken = 0;
  const answers = [];
  for (const goal of parts) {
    const part=typeof goal==='string'?goal:goal.clause;
    // A step is planned over the tool calls made for it alone: replayed one
    // exchange at a time until it answers or asks for its next call.
    const own = [];
    for (;;) {
      const plan = await planFor(withRequest([...base, ...own], part), toolNames, goal);
      if (plan === null) return null;
      if (plan.kind === 'final') {
        if (!canDeliverFinal(plan)) return plan;
        answers.push(plan.answer);
        break;
      }
      if (taken === exchanges.length) return plan;
      own.push(...exchanges[taken]);
      taken += 1;
    }
  }
  return resolvedFinalAnswer(answers.join('\n\n'), FinalDisposition.Finding, 'request_sequence_verified');
}

/**
 * Mirrors `fn turn_exchanges`: the conversation up to its latest user turn,
 * and the tool exchanges after it, each an assistant message with the
 * messages that answer it.
 * @param {Array<object>} messages
 */
function turnExchanges(messages) {
  let latest = -1;
  messages.forEach((message, index) => {
    if (String(message.role).toLowerCase() === 'user') latest = index;
  });
  const exchanges = [];
  for (const message of messages.slice(latest + 1)) {
    if (String(message.role).toLowerCase() === 'assistant' || exchanges.length === 0) exchanges.push([message]);
    else exchanges.at(-1).push(message);
  }
  return { base: messages.slice(0, latest + 1), exchanges };
}

function goalGap(goal){return resolvedFinalAnswer(gapAnswer(stableId('obligation',goal.clause),goal.clause,goal.byteSpan,'no_artifact_in_clause'),FinalDisposition.Gap,'owned-goal-missing-contract');}
