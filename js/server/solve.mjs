// The symbolic answer the protocol adapters project: the worker's `solve`
// result recast as the Rust `SymbolicAnswer` (rust/src/engine_answer.rs), plus
// the deterministic helpers every adapter shares (`stable_id`,
// `estimate_tokens`) and the tool-policy refusals of
// rust/src/protocol_policy.rs.

import { f32 } from './json.mjs';
import { estimateTokens, stableId } from './ids.mjs';
import { serverMessage } from './messages.mjs';
import { thinkingStep, thinkingStepsFromWorker } from './thinking.mjs';

export { estimateTokens, stableId };

/**
 * @typedef {{
 *   intent: string, answer: string, confidence: number,
 *   evidence_links: Array<string>, thinking_steps: Array<object>,
 *   links_notation: string
 * }} SymbolicAnswer
 */

/**
 * Recast a worker answer as a `SymbolicAnswer`.
 * @param {object} result the worker's `solve` value
 * @returns {SymbolicAnswer}
 */
export function symbolicFromWorker(result) {
  const answer = String(result?.content ?? '');
  return {
    intent: String(result?.intent ?? 'unknown'),
    answer,
    confidence: Number(result?.confidence ?? 0),
    evidence_links: Array.isArray(result?.evidence) ? result.evidence.map(String) : [],
    thinking_steps: thinkingStepsFromWorker(result?.steps || []),
    links_notation: String(result?.derivation ?? ''),
  };
}

/**
 * Solve `prompt` after `history` with the booted worker.
 * @param {{worker: import('./worker-host.mjs').WorkerHost}} ctx
 * @param {string} prompt
 * @param {Array<{role: string, content: string}>} history
 * @returns {Promise<SymbolicAnswer>}
 */
export async function solveSymbolic(ctx, prompt, history) {
  return symbolicFromWorker(await ctx.worker.solve(prompt, history));
}

function policyThinkingSteps(detail) {
  return [thinkingStep(0, 'policy_refusal', detail, 'high', 'policy')];
}

/** `tool_call_refusal_answer`. @returns {SymbolicAnswer} */
export function toolCallRefusalAnswer() {
  const policy = 'agent_mode_required_for_tools';
  return {
    intent: 'tool_call_refused',
    answer: serverMessage('tool_call_refused'),
    confidence: 1,
    evidence_links: [`policy:${policy}`],
    thinking_steps: policyThinkingSteps(policy),
    links_notation: `tool_call_refusal\n  policy "${policy}"\n  thinking_step "policy_refusal ${policy}"\n`,
  };
}

/** The `confidence` field as Rust prints an `f32`. @param {number} value */
export function confidenceValue(value) {
  return f32(value);
}
