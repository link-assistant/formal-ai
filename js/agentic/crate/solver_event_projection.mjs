// The native solver's event log for one server turn (R1013, server parity):
// the worker's own log (js/worker/formal_ai_worker_solver_events.js: the
// impulse / language prelude, the handler's events and the finalize tail)
// completed with the records rust/src/solver.rs
// `solve_with_history_probability_store_and_intent_cache` appends between
// them, in its order:
//
// 1. on the intent-cache miss every protocol solve meets (`solve_with_history`
//    opens a fresh cache per turn): `record_formalization_selection` and
//    `record_formalization` over `formalize_prompt_candidates`
//    (js/agentic/crate/solver_formalization.mjs);
// 2. `record_intent_formalization` (js/agentic/crate/intent_formalization.mjs),
//    which replaces the worker's bare `intent_formalization:route` event;
// 3. unless the meta reasoner answered (`try_meta_answer` returns first):
//    the supplied server callback adds `meta_core::record_meta_core`; the
//    browser omits this callback while its full meta-core port is pending, then
//    `search:local`, then the `selected_rule` event `try_construct_unknown_rule`
//    logs for a route `select_rule_for_intent` maps to `SelectedRule::Unknown`;
// 4. the worker's handler events and finalize tail;
// 5. the `method` event `meta_method_dispatch::record_method_answer` appends
//    when a registry method of the handler table answered.
//
// Not mirrored: the `try_meta_discovery` answers of the unknown branch (the
// worker marks them like `try_meta_answer` ones, so they get no meta core),
// `requirement_contradiction`, and the decomposition / rule-synthesis events
// of prompts that construct a rule.

import { realm } from '../host.mjs';
import { formalizeIntentRecord, recordIntentFormalization } from './intent_formalization.mjs';
import { methodForRoute, methodRegistry } from './method_registry.mjs';
import {
  recordFormalization,
  recordFormalizationSelection,
  selectFormalizationCandidate,
  selectedCandidate,
} from './solver_formalization.mjs';
import { formalizePromptCandidates } from './translation_formalization.mjs';

const IMPULSE = 'impulse';
const LANGUAGE = 'language';
const ROUTE = 'intent_formalization:route';
const CANDIDATE = 'candidate';
const META_RESPONSE = 'response:meta_reasoner';
const DISPATCH_STEP = 'dispatch_handler';
const UNKNOWN_RULE_SELECTION = 'initial unknown reason no_seed_route next try_rule_synthesis';

/** The routes `select_rule_for_intent` (rust/src/intent_formalization.rs) maps to a rule other than `Unknown`. */
const SEEDED_RULE_ROUTES = new Set([
  'greeting', 'wellbeing', 'farewell', 'test_status', 'courtesy_response',
  'assistant_free_time', 'assistant_name', 'identity', 'write_program',
]);

/**
 * The registry method a worker answer came from: the handler-table row the
 * dispatch step names, as the seed slug the native registry lists, or null.
 */
function dispatchedMethod(result) {
  const steps = Array.isArray(result?.steps) ? result.steps : [];
  const step = steps.find((entry) => entry?.step === DISPATCH_STEP);
  if (!step || typeof realm().workerHandlerRegistryDefinition !== 'function') return null;
  const detail = String(step.detail ?? '');
  const handlers = realm().workerHandlerRegistryDefinition().workerHandlers || {};
  const slug = Object.hasOwn(handlers, detail) ? detail : Object.keys(handlers).find((key) => handlers[key] === detail);
  if (!slug) return null;
  const method = methodForRoute(methodRegistry(), slug);
  return method && method.name === slug ? slug : null;
}

/**
 * Mirrors the order rust/src/solver.rs logs a turn in: the worker's events
 * (`result.solverEvents`) with the formalization, intent-formalization and
 * optional meta-core records spliced in after the prelude. The server supplies
 * recordCore; the browser shares formalization and evidence selection without
 * the full meta-core import closure. Events without a prelude stay unchanged.
 * @param {object} result the worker's `solve` value
 * @param {Function|null} recordCore optional native-order meta-core recorder
 * @returns {Array<{kind: string, payload: string}>}
 */
export function nativeSolverLog(result, recordCore = null) {
  const raw = result?.rawSolverEvents ?? result?.solverEvents;
  const events = Array.isArray(raw) ? raw.map((event) => ({ kind: String(event.kind), payload: String(event.payload ?? '') })) : [];
  if (events.length < 2 || events[0].kind !== IMPULSE || events[1].kind !== LANGUAGE) return events;
  const prompt = events[0].payload;
  const language = events[1].payload;
  const log = events.slice(0, 2);
  const selection = selectFormalizationCandidate(formalizePromptCandidates(prompt, language), prompt);
  recordFormalizationSelection(log, selection);
  const candidate = selectedCandidate(selection);
  if (candidate) recordFormalization(log, candidate);
  const record = formalizeIntentRecord(prompt, language, candidate);
  recordIntentFormalization(log, record);
  const rest = events.slice(2).filter((event) => event.kind !== ROUTE);
  const metaAnswer = rest.some((event) => event.kind === 'response' && event.payload === META_RESPONSE);
  if (!metaAnswer) {
    if (recordCore) recordCore(log, record);
    log.push({ kind: 'search:local', payload: prompt });
    if (!SEEDED_RULE_ROUTES.has(record.route)) log.push({ kind: 'selected_rule', payload: UNKNOWN_RULE_SELECTION });
  }
  // `finalize_simple` logs its `candidate` only when none is logged yet: the
  // worker logs one in its tail, which the formalization candidate replaces.
  const logged = log.some((event) => event.kind === CANDIDATE);
  log.push(...(logged ? rest.filter((event) => event.kind !== CANDIDATE) : rest));
  const method = metaAnswer ? null : dispatchedMethod(result);
  if (method) log.push({ kind: 'method', payload: method });
  return log;
}
