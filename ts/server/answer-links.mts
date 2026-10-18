// rust/src/engine.rs::answer_links_notation: the actual append-ordered log,
// with the record's response-link thinking projection kept distinct from
// the final-answer thinking shown to a client.
import { EventLog } from '../agentic/crate/event_log.mjs';
import { stableId } from '../agentic/crate/engine_stable_identifier.mjs';
import { flattenLinoValue, formatLinoRecord } from '../agentic/crate/links_format.mjs';
import { thinkingStepsFromEvents } from './solver-trace.mjs';

/** The six-field native answer record over events actually observed. */
export function answerLinksNotation(prompt, intent, answer, events, traceId) {
  const steps = events.map((event, index) =>
    `step_${index} ${event.kind} ${flattenLinoValue(String(event.payload ?? ''))}`).join('; ');
  const thinking = thinkingStepsFromEvents(events).map((step) =>
    `step_${step.order} ${flattenLinoValue(step.step)} ${flattenLinoValue(step.level)} `
      + `${flattenLinoValue(step.source_event)} ${flattenLinoValue(step.detail)}`).join('; ');
  return formatLinoRecord(`answer_${stableId('prompt', prompt)}`, [
    ['prompt', prompt], ['intent', intent], ['answer', answer], ['trace', traceId],
    ['steps', steps], ['thinking_steps', thinking],
  ]);
}

/** solve_with_history appends the declared prior turns before the impulse. */
export function solverLogFromWorker(result, history = []) {
  if (!Array.isArray(result?.solverEvents)) return null;
  const priorTurns = history.map((turn) => ({
    kind: turn?.role === 'assistant' ? 'prior_turn:assistant' : 'prior_turn:user',
    payload: String(turn?.content ?? ''),
  }));
  return new EventLog([...priorTurns, ...result.solverEvents]);
}

/** An absent actual log/impulse/trace stays absent instead of inventing it. */
export function answerLinksFromLog(log, intent, answer) {
  const impulse = log?.firstOf('impulse');
  const trace = log?.lastOf('trace');
  return impulse && trace ? answerLinksNotation(impulse.payload, intent, answer, log.events, trace.id) : '';
}
