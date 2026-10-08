// The symbolic answer the protocol adapters project: the worker's `solve`
// result recast as the Rust `SymbolicAnswer` (rust/src/engine_answer.rs), plus
// the deterministic helpers every adapter shares (`stable_id`,
// `estimate_tokens`) and the tool-policy refusals of
// rust/src/protocol_policy.rs.

import { finalizeServerAnswer } from './derivation-store.mjs';
import { beginTurn, endTurn, gateTurn } from './debug-session.mjs';
import { ensureNodeHost } from './node-host-install.mjs';
import { EventLog, buildEvidenceLinks } from './evidence-links.mjs';
import { f32 } from './json.mjs';
import { estimateTokens, stableId } from './ids.mjs';
import { serverMessage } from './messages.mjs';
import { noteLearned } from './meta-learned.mjs';
import { nativeProgramAnswer } from './program-report.mjs';
import { answerFromMemoryIfRequested } from './memory-answer.mjs';
import { nativeSolverLog } from './solver-log.mjs';
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

const PRIOR_TURN_USER = 'prior_turn:user';
const PRIOR_TURN_ASSISTANT = 'prior_turn:assistant';
const ASSISTANT_ROLE = 'assistant';
const IMPULSE_KIND = 'impulse';
const RESPONSE_KIND = 'response';
const META_RESPONSE_LINK = 'response:meta_reasoner';
const BROWSER_TRACE_PREFIX = 'trace:';

/**
 * The worker's own evidence without the browser trace `finalize` appends
 * after it (`trace:<event>` per dispatch event): the handler's evidence.
 */
function handlerEvidence(evidence) {
  let end = evidence.length;
  while (end > 0 && evidence[end - 1].startsWith(BROWSER_TRACE_PREFIX)) end -= 1;
  return evidence.slice(0, end);
}

/**
 * Mirrors rust/src/event_log.rs `build_evidence_links` over the native log
 * the worker recorded (`solverEvents`), after the `prior_turn:*` events
 * `solve_with_history` opens the log with; a meta-reasoner answer extends the
 * links with its own evidence (rust/src/meta_reasoner/integration.rs
 * `project`). Null when the worker recorded no native log.
 */
export function solverEvidenceLinks(result, history = []) {
  const events = Array.isArray(result?.solverEvents) ? result.solverEvents : null;
  if (!events) return null;
  const impulse = events.find((event) => event.kind === IMPULSE_KIND);
  if (!impulse) return null;
  const log = new EventLog();
  for (const turn of history || []) {
    log.append(turn?.role === ASSISTANT_ROLE ? PRIOR_TURN_ASSISTANT : PRIOR_TURN_USER, String(turn?.content ?? ''));
  }
  for (const event of events) log.append(String(event.kind), String(event.payload ?? ''));
  const response = log.lastOf(RESPONSE_KIND);
  const responseLink = response ? response.payload : `${RESPONSE_KIND}:${String(result?.intent ?? 'unknown')}`;
  const links = buildEvidenceLinks(String(impulse.payload), log, responseLink);
  if (responseLink === META_RESPONSE_LINK && Array.isArray(result?.evidence)) {
    links.push(...handlerEvidence(result.evidence.map(String)));
  }
  return links;
}

/**
 * Recast a worker answer as a `SymbolicAnswer`.
 * @param {object} result the worker's `solve` value
 * @param {Array<{role: string, content: string}>} [history] the turns it was solved after
 * @param {(intent: string) => string|null} [seedReport] the seed's English report text
 * @returns {SymbolicAnswer}
 */
export function symbolicFromWorker(result, history = [], seedReport = undefined) {
  const answer = nativeProgramAnswer(String(result?.content ?? ''), result?.programExecution, seedReport);
  noteLearned(result?.memoryOperation);
  return {
    intent: String(result?.intent ?? 'unknown'),
    answer,
    confidence: Number(result?.confidence ?? 0),
    evidence_links: solverEvidenceLinks(result, history)
      ?? (Array.isArray(result?.evidence) ? result.evidence.map(String) : []),
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
    // Off the wire: a function the browser synthesized, which the agentic
    // reroute turns into an execution recipe (`attachExecutionRecipe`).
    synthesized_program: result?.synthesizedProgram ?? undefined,
  };
}

/** The seed's English report text (`seed::report_text`), through the worker's readers. */
async function seedReportReader(ctx) {
  const seed = await ctx.worker.seedReaders();
  return (intent) => seed.responseFor(intent, 'en');
}

/**
 * Solve `prompt` after `history` with the booted worker.
 * @param {{worker: import('./worker-host.mjs').WorkerHost}} ctx
 * @param {string} prompt
 * @param {Array<{role: string, content: string}>} history
 * @returns {Promise<SymbolicAnswer>}
 */
export async function solveSymbolic(ctx, prompt, history) {
  // Issue #667 (R383): under a debug session the solve waits at its first
  // stage (`impulse`) before the worker runs; `gateTurn` continues the turn.
  await beginTurn(ctx, prompt);
  try {
    return await solveBegun(ctx, prompt, history);
  } finally {
    endTurn(ctx);
  }
}

async function solveBegun(ctx, prompt, history) {
  const result = await ctx.worker.solve(prompt, history);
  // R1013: the formalization, intent-formalization and meta-core records the
  // native solver logs between the worker's prelude and its handler events.
  if (Array.isArray(result?.solverEvents)) {
    await ensureNodeHost(ctx);
    result.solverEvents = nativeSolverLog(result);
  }
  // The native solver answers a summarize request (not a returning-user recap)
  // with the `Conversation summary: … User turns:` envelope; the browser
  // worker's report is statistics only. A client that compacts its session
  // keeps that answer as the summary, so the server answers like native
  // (PR #1188 dogfooding: the Agent CLI lost the task after a compaction).
  if (result?.intent === 'summarize_conversation' && !(result.evidence || []).includes('summarization:format:plain')) {
    await ensureNodeHost(ctx);
    const { conversationSummaryEnvelope } = await import('../agentic/crate/conversation_summary.mjs');
    const envelope = conversationSummaryEnvelope(prompt, history);
    if (envelope !== null) result.content = envelope;
  }
  // Issue #1184 R1184-9: every native solve ends in `finalize_answer`, which
  // links, appends and persists the answer's derivation record.
  const symbolic = symbolicFromWorker(result, history, await seedReportReader(ctx));
  // Issue #667 (R383): a held turn is neither persisted nor answered until it is released.
  if (ctx.debugSession) await ensureNodeHost(ctx);
  await gateTurn(ctx, symbolic.thinking_steps);
  const answer = await finalizeServerAnswer(ctx, symbolic, result);
  return answer;
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
