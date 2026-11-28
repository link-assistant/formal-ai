// Structured inputs for proposals and operator-reviewed changes.
import { stableId } from "./crate/engine_stable_identifier.mjs";
/** Mirrors `fn intent` in rust/src/agentic_coding/repair_input.rs. */
export function repairIntent(task) {
  const index = task.indexOf('\n```json');
  return index < 0 ? task : task.slice(0, index);
}

/** Mirrors `fn data` in rust/src/agentic_coding/repair_input.rs. */
export function repairData(task) {
  if (typeof task !== 'string') return null;
  const marker = task.indexOf('\n```json');
  if (marker < 0) return null;
  const block = /^\r?\n([\s\S]*)\n```$/.exec(task.slice(marker + '\n```json'.length).trimEnd());
  if (!block || new TextEncoder().encode(block[1]).length > 128 * 1024) return null;
  try {
    return JSON.parse(block[1]);
  } catch {
    return null;
  }
}

const traceEventKinds = new Set([
  'reasoning:unknown', 'selected_rule', 'rule_synthesis_candidate', 'rule_verification',
  'write_program_plan', 'lexicon_lookup', 'regression_check',
  'answer:intent', 'answer:evidence', 'answer:links_notation',
]);

/** Mirrors `TraceInput::into_trace` in rust/src/agentic_coding/repair_input.rs. */
export function readRepairTrace(input) {
  if (typeof input?.prompt !== 'string' || !input.prompt.trim() || !Array.isArray(input.events) || !input.events.length || input.events.length > 128) return null;
  if (input.events.some(event => !traceEventKinds.has(event?.kind) || typeof event.payload !== 'string')) return null;
  const events = input.events.map(({ kind, payload }, index) => ({ kind, payload, id: stableId(kind, `${kind}:${index}:${payload}`) }));
  const fingerprint = events.map(({ kind, payload }) => `${kind}=${payload}`).join('\n');
  return { id: stableId('unknown_trace', `${input.prompt}\n${fingerprint}`), prompt: input.prompt, events };
}
