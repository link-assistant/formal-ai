// Bridge agentic chat messages into the shared conversation-history solver:
// rust/src/agentic_coding/conversation_recall.rs.

import { planSymbolicCommandReroute } from './command_reroute.mjs';
import { sourceTreeRequest } from './crate/meta_translate.mjs';
import { chatPromptAndHistory } from './crate/protocol_recording.mjs';
import { trim } from './crate/rust_str.mjs';
import { solve } from './host.mjs';
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
