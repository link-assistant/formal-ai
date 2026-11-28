import { AsyncLocalStorage } from 'node:async_hooks';
import { isInconclusive } from '../agentic/crate/engine_answer.mjs';
import { childrenNamed, parseLino, readRepoFile } from './lino.mjs';

const retrieval = new AsyncLocalStorage();
let capabilityRefusalKinds;
const refusedIntents = new Set([
  'tool_call_refused', 'browser_code_execution_refused', 'policy_inappropriate_content',
  'policy_bounded_autonomy', 'policy_add_only_history', 'policy_cache_flush_requires_confirmation',
  'policy_destructive_action_requires_confirmation', 'policy_agent_time_budget',
]);

export function answerMetadata(status = 'answered') {
  return { status, provenance: ['local_rules'] };
}

export function hasCapabilityRefusal(evidence, events) {
  capabilityRefusalKinds ??= childrenNamed(parseLino(readRepoFile('data/seed/capability-routing.lino')), 'claim')
    .flatMap(row => childrenNamed(row, 'refusal_event').map(event => event.value));
  return capabilityRefusalKinds.some(kind => events?.some(event => event.kind === kind)
    || evidence?.some(link => link === kind || String(link).startsWith(`${kind}:`)));
}

export function symbolicAnswerMetadata(answer) {
  if (refusedIntents.has(answer.intent)) {
    return answerMetadata('refused');
  }
  const unanswered = answer.intent === 'capability_gap' || isInconclusive(answer) || answer.capability_refusal;
  return answerMetadata(unanswered ? 'unanswered' : 'answered');
}

/** Record a live HTTP attempt after the caller's network policy allows it. */
export function observeNetworkRetrieval() {
  const turn = retrieval.getStore();
  if (turn) turn.used = true;
}

export async function withNetworkObservation(run, record) {
  const turn = { used: false };
  const response = await retrieval.run(turn, run);
  if (turn.used) record(response);
  return response;
}
