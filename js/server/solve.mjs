// The symbolic answer the protocol adapters project: the worker's `solve`
// result recast as the Rust `SymbolicAnswer` (rust/src/engine_answer.rs), plus
// the deterministic helpers every adapter shares (`stable_id`,
// `estimate_tokens`) and the tool-policy refusals of
// rust/src/protocol_policy.rs.

import { f32 } from './json.mjs';
import { estimateTokens, stableId } from './ids.mjs';
import { serverMessage } from './messages.mjs';
import { noteLearned } from './meta-learned.mjs';
import { answerFromMemoryIfRequested } from './memory-answer.mjs';
import { thinkingStepsFromEvents } from './solver-trace.mjs';
import { applyRetainedAmendments, solveWithStandingRequirements } from './standing-requirements.mjs';
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
  noteLearned(result?.memoryOperation);
  return {
    intent: String(result?.intent ?? 'unknown'),
    answer,
    confidence: Number(result?.confidence ?? 0),
    evidence_links: Array.isArray(result?.evidence) ? result.evidence.map(String) : [],
    // The native solver's event log when the worker recorded one (it always
    // does on the finalize path); the browser trace otherwise.
    thinking_steps: Array.isArray(result?.solverEvents)
      ? thinkingStepsFromEvents(result.solverEvents, answer)
      : thinkingStepsFromWorker(result?.steps || []),
    links_notation: String(result?.derivation ?? ''),
    // Off the wire: the raw traces `learning_trace_from_symbolic_answer`
    // (js/server/self-improvement.mjs) rebuilds the solver log from.
    worker_steps: Array.isArray(result?.steps) ? result.steps : [],
    solver_events: Array.isArray(result?.solverEvents) ? result.solverEvents : undefined,
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

/** The store's `MemoryEvent`s the protocol surfaces answer with (`store.events()`). */
function memoryEvents(ctx) {
  return ctx.memory && typeof ctx.memory.events === 'function' ? ctx.memory.events() : [];
}

/**
 * The memory answer the chat-completions and responses surfaces give ahead
 * of the solver: rust/src/protocol.rs runs `answer_from_memory_if_requested`
 * over `memoryPrompt`, then `apply_retained_amendments` over `amendPrompt`.
 * @param {{worker: import('./worker-host.mjs').WorkerHost, memory?: object}} ctx
 * @param {string} memoryPrompt
 * @param {Array<{role: string, content: string}>} history
 * @param {string} [amendPrompt]
 * @returns {Promise<SymbolicAnswer|null>}
 */
export async function answerFromMemory(ctx, memoryPrompt, history, amendPrompt = memoryPrompt) {
  const events = memoryEvents(ctx);
  const seed = await ctx.worker.seedReaders();
  const answer = answerFromMemoryIfRequested(seed, memoryPrompt, history, events);
  return answer ? applyRetainedAmendments(amendPrompt, answer, events) : null;
}

/**
 * Mirrors rust/src/dreaming_application.rs `solve_with_standing_requirements`:
 * the solver answer with every matching retained requirement restated as a
 * user turn, else the pre-learned cache when the solver found no route.
 * @param {{worker: import('./worker-host.mjs').WorkerHost, memory?: object}} ctx
 * @param {string} prompt
 * @param {Array<{role: string, content: string}>} history
 * @returns {Promise<SymbolicAnswer>}
 */
export async function solveWithMemory(ctx, prompt, history) {
  const events = memoryEvents(ctx);
  const seed = await ctx.worker.seedReaders();
  return solveWithStandingRequirements(
    (text, turns) => solveSymbolic(ctx, text, turns),
    seed.normalizePrompt,
    prompt,
    history,
    events,
  );
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
