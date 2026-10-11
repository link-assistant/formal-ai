import { plainText } from './content.mjs';
import { failureMessage, renderFailure } from './tool_result.mjs';
import { renderSeededOutcome } from './code_task.mjs';
// Bridge agentic chat messages into the shared conversation-history solver:
// rust/src/agentic_coding/conversation_recall.rs.

import { planSymbolicCommandReroute } from './command_reroute.mjs';
import { sourceTreeRequest, ownedSourceTreeRequest } from './crate/meta_translate.mjs';
import { chatPromptAndHistory } from './crate/protocol_recording.mjs';
import { trim } from './crate/rust_str.mjs';
import { solve, readText } from './host.mjs';
import { finalAnswer, plannedCall, toolCalls } from './plan.mjs';

/**
 * Mirrors `fn plan_shared_solver_step` in rust/src/agentic_coding/conversation_recall.rs
 * (async: it consults the solver). Resolves to `{kind: 'ready', plan}`,
 * `{kind: 'defer'}` or `{kind: 'not_ours'}` (`SharedSolverStep`).
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export async function planSharedSolverStep(messages, toolNames) {
  const [prompt, history] = chatPromptAndHistory(messages);
  if (trim(prompt) === '') return { kind: 'not_ours' };
  const owned = ownedSourceTreeRequest(prompt);
  if (owned === null && sourceTreeRequest(prompt) !== null) return {kind:'not_ours'};
  if (owned !== null && toolNames.includes('translate')) {
    const {request,write} = owned;
    const expected={from:request.from.name,to:request.to.name,path:request.path,write};
    const failure=repeatedOwnedOperationFailure(messages,'translate',expected);
    if(failure!==null)return {kind:'ready',plan:finalAnswer(renderFailure('translate',failure,prompt))};
    return {kind:'ready',plan:toolCalls([plannedCall('translate',JSON.stringify({from:request.from.name,to:request.to.name,path:request.path,write}))])};
  }
  if (owned !== null) {
    try { readText(owned.request.path); }
    catch (error) {
      if (error?.code === 'ENOENT') {
        const gap = renderSeededOutcome('translate_source_missing',prompt,owned.request.path);
        if (gap !== null) return {kind:'ready',plan:finalAnswer(gap)};
      }
      return {kind:'not_ours'};
    }
  }
  const answer = await solve(prompt, history);
  switch (answer.intent) {
    case 'summarize_conversation':
      return { kind: 'ready', plan: finalAnswer(answer.answer) };
    case 'write_program':
    case 'substitution_rule_export': {
      const reroute = await planSymbolicCommandReroute(messages, toolNames, answer);
      return reroute !== null && reroute !== undefined
        ? { kind: 'defer' }
        : { kind: 'ready', plan: finalAnswer(answer.answer) };
    }
    case 'translate_source_tree': {
      const request = toolNames.includes('translate') ? sourceTreeRequest(prompt) : null;
      if (request === null) return { kind: 'ready', plan: finalAnswer(answer.answer) };
      const args = `{"from":"${request.from.name}","to":"${request.to.name}","path":"${request.path}","write":true}`;
      return { kind: 'ready', plan: toolCalls([plannedCall('translate', args)]) };
    }
    default:
      return { kind: 'not_ours' };
  }
}

/** Match unique failed receipts to the complete current-turn operation. */
function repeatedOwnedOperationFailure(messages, tool, expected) {
  let start = 0;
  for (let index=messages.length-1;index>=0;index-=1) if (messages[index].role.toLowerCase()==='user') { start=index+1;break; }
  const seen = new Set(), failures = [];
  for (let index=start;index<messages.length;index+=1) {
    const message=messages[index];
    if (message.role.toLowerCase()!=='tool' || typeof message.tool_call_id!=='string' || seen.has(message.tool_call_id)) continue;
    let call=null;
    for (let prior=index-1;prior>=start && call===null;prior-=1)
      call=(messages[prior].tool_calls??[]).findLast(candidate=>candidate.id===message.tool_call_id)??null;
    if (call?.function?.name!==tool || (message.name!=null && message.name.toLowerCase()!==tool.toLowerCase())) continue;
    let observed;try { observed=JSON.parse(call.function.arguments); } catch { continue; }
    if (observed===null || typeof observed!=='object' || Array.isArray(observed)
      || Object.keys(observed).length!==Object.keys(expected).length
      || Object.keys(expected).some(key=>observed[key]!==expected[key])) continue;
    seen.add(message.tool_call_id);
    const failure=failureMessage(plainText(message.content),Boolean(message.is_error||message.isError),false);
    if (failure===null) failures.length=0; else failures.push(failure);
  }
  const latest=failures.at(-1);
  return latest!==undefined && failures.filter(failure=>failure===latest).length>=2 ? latest : null;
}
