// The solver event log projected into thinking steps: rust/src/event_log.rs
// `EventLog::thinking_steps_for_answer`, `curate_thinking_event` and
// `CalculationCluster`.
//
// The worker records, beside its own browser trace, the events the native
// solver appends for the same turn (`solverEvents`: `[{kind, payload}]`, see
// js/worker/formal_ai_worker_solver_events.js). This module curates them
// exactly as the Rust log does, so both servers narrate one trace.

import { localizeThinkingSteps, thinkingStep } from './thinking.mjs';

const ROLE_NORMAL = 'normal';
const ROLE_PARENT = 'parent';
const ROLE_CHILD = 'child';
const STEP_KEY_SEPARATOR = '\u001f';

function planned(step, detail, level, source, role = ROLE_NORMAL) {
  return { step, detail: String(detail), level, source, role };
}

/** Mirrors `collapse_thinking_whitespace`. */
function collapseWhitespace(value) {
  return String(value).split(/\s+/).filter(Boolean).join(' ');
}

/** Mirrors `CalculationCluster` (absorb / has_data / drain_into). */
class CalculationCluster {
  constructor() {
    this.reset();
  }

  reset() {
    this.request = null;
    this.engine = null;
    this.expression = null;
    this.steps = null;
    this.result = null;
  }

  absorb(kind, payload) {
    const value = String(payload).trim();
    if (kind === 'calculation') this.result = value;
    else if (kind === 'calculation:request') this.request = value;
    else if (kind === 'calculation:engine') this.engine = value;
    else if (kind === 'calculation:lino') this.expression = value;
    else if (kind === 'calculation:steps') this.steps = value;
  }

  hasData() {
    return [this.result, this.request, this.engine, this.expression, this.steps].some((value) => value !== null);
  }

  drainInto(plan) {
    plan.push(planned('compute', this.result ?? this.request ?? '', 'high', 'calculation', ROLE_PARENT));
    if (this.engine !== null) plan.push(planned('compute_engine', this.engine, 'detailed', 'calculation:engine', ROLE_CHILD));
    if (this.expression !== null) plan.push(planned('compute_expression', this.expression, 'detailed', 'calculation:lino', ROLE_CHILD));
    if (this.steps !== null) plan.push(planned('compute_steps', this.steps, 'detailed', 'calculation:steps', ROLE_CHILD));
    this.reset();
  }
}

/** `kind` -> `[step, level, keepsDetail]` for the exact-kind rows of `curate_thinking_event`. */
const CURATED_KINDS = new Map([
  ['impulse', ['impulse', 'high', true]],
  ['language', ['detect_language', 'high', true]],
  ['language_from', ['detect_language', 'high', true]],
  ['language_to', ['resolve_response_language', 'high', true]],
  ['intent_formalization:route', ['formalize', 'high', true]],
  ['intent', ['dispatch_handler', 'high', true]],
  ['legacy_intent', ['dispatch_handler', 'high', true]],
  ['program_plan', ['program_plan', 'high', true]],
  ['program_parameters', ['program_plan', 'high', true]],
  ['concept_lookup:request', ['scan_memory', 'detailed', true]],
  ['procedural_how_to:request', ['scan_memory', 'detailed', true]],
  ['concept_lookup:hit', ['lookup_fact', 'detailed', true]],
  ['fact_query:relation', ['lookup_fact', 'detailed', true]],
  ['fact_query:subject', ['lookup_fact', 'detailed', true]],
  ['fact_lookup:hit', ['lookup_fact', 'detailed', true]],
  ['release_timeline:hit', ['lookup_fact', 'detailed', true]],
  ['web_search:request', ['http_chat', 'detailed', true]],
  ['http_fetch:request', ['http_chat', 'detailed', true]],
  ['url_navigate:request', ['http_chat', 'detailed', true]],
  ['tool_call', ['invoke_tool', 'detailed', true]],
  ['tool_result', ['invoke_tool', 'detailed', true]],
  ['coreference', ['coreference_binding', 'detailed', false]],
  ['program_coreference', ['coreference_binding', 'detailed', false]],
  ['modifier_detection', ['modifier_detection', 'detailed', false]],
  ['program_modifiers', ['modifier_detection', 'detailed', false]],
  ['rule_construction', ['rule_construction', 'detailed', false]],
  ['validation', ['rule_verification', 'detailed', false]],
]);

/** Mirrors `curate_thinking_event`: the planned step for one event, or null. */
export function curateThinkingEvent(kind, payload, finalAnswer) {
  const detail = String(payload ?? '').trim();
  if (kind === 'response') return planned('deformalize', finalAnswer || detail, 'high', kind);
  const row = CURATED_KINDS.get(kind);
  if (row) return planned(row[0], row[2] ? detail : '', row[1], kind);
  if (kind.startsWith('tool_')) return planned('invoke_tool', detail, 'detailed', kind);
  if (kind.startsWith('agent_mode') || kind === 'action_log') return planned('agent_plan', detail, 'detailed', kind);
  return null;
}

/**
 * Mirrors `EventLog::thinking_steps_for_answer`: curate, fold the calculator
 * trace into one composite step, drop consecutive repeats, number, parent,
 * and narrate the finished trace in its own language.
 * @param {Array<{kind: string, payload: string}>} events
 * @param {string} finalAnswer
 */
export function thinkingStepsFromEvents(events, finalAnswer = '') {
  const answer = collapseWhitespace(finalAnswer);
  const plan = [];
  const calculation = new CalculationCluster();
  for (const event of events) {
    const kind = String(event.kind);
    if (kind === 'calculation' || kind.startsWith('calculation:')) {
      calculation.absorb(kind, event.payload ?? '');
      continue;
    }
    if (calculation.hasData()) calculation.drainInto(plan);
    const step = curateThinkingEvent(kind, event.payload, answer);
    if (step) plan.push(step);
  }
  if (calculation.hasData()) calculation.drainInto(plan);

  const steps = [];
  let previousKey = null;
  let currentParent = null;
  for (const entry of plan) {
    const key = `${entry.step}${STEP_KEY_SEPARATOR}${entry.detail}`;
    const isChild = entry.role === ROLE_CHILD;
    if (!isChild && previousKey === key) continue;
    previousKey = key;
    const parent = isChild ? currentParent : null;
    const step = thinkingStep(steps.length, entry.step, entry.detail, entry.level, entry.source, parent);
    if (entry.role === ROLE_PARENT) currentParent = step.id;
    else if (entry.role === ROLE_NORMAL) currentParent = null;
    steps.push(step);
  }
  return localizeThinkingSteps(steps);
}
