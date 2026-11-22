import { cached } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { readRepairTrace, repairData, repairIntent } from './repair_input.mjs';
import { quote } from './crate/change_request.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';
import { trimEnd } from './crate/rust_str.mjs';

export const REPAIR_STRATEGY_PATH = 'repair-strategies.lino';

/** Mirrors `const REPAIR_STRATEGY_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const repairStrategyKeywords = () => cached('repair-strategy-keywords', () => agenticMessage('repair_strategy_keywords').split('|'));

/**
 * Mirrors `fn is_repair_strategy_task` in rust/src/agentic_coding/repair_strategy.rs.
 * @param {string} prompt
 */
export function isRepairStrategyTask(prompt) {
  const lower = repairIntent(prompt).toLowerCase();
  return repairStrategyKeywords().some((keyword) => lower.includes(keyword));
}

const classes = ['solver_method', 'data_record', 'test'];

function classMessage(target, field, values) {
  return agenticMessage(`repair_class_${target}_${field}`, values);
}

function classify(trace) {
  const haystack = [trace.prompt, ...trace.events.map(event => `${event.kind} ${event.payload}`)].join('\n').toLowerCase();
  for (const target of classes) {
    const signal = classMessage(target, 'cues').split('|').find(cue => cue && haystack.includes(cue));
    if (signal) return { target, rationale: classMessage(target, 'rationale', { signal }) };
  }
  return { target: classes[0], rationale: agenticMessage('repair_strategy_default_rationale') };
}

export function strategiesFromTask(task) {
  const traces = repairData(task)?.traces;
  if (!Array.isArray(traces) || !traces.length || traces.length > 32) return null;
  const observations = traces.map(readRepairTrace);
  if (observations.some(trace => !trace)) return null;
  return observations.map(trace => {
    const { target, rationale } = classify(trace);
    return {
      id: stableId('repair_strategy', `${trace.id}:${target}:${trace.prompt}`),
      failure_prompt: trace.prompt,
      trace_id: trace.id,
      target,
      human_gated: 'true',
      rationale,
      proposed_change: classMessage(target, 'proposal', { prompt: trace.prompt }),
      verification: classMessage(target, 'verification'),
    };
  });
}

const targets = strategies => [...new Set(strategies.map(strategy => strategy.target))];

export function renderDocument(strategies) {
  let document = `repair_strategies\n  covers "${targets(strategies).join(',')}"\n  strategy_count "${strategies.length}"\n`;
  for (const strategy of strategies) {
    document += '  repair_strategy\n';
    for (const [key, value] of Object.entries(strategy)) document += `    ${key} "${quote(value)}"\n`;
  }
  return document;
}

export function finalAnswer(strategies, document) {
  return agenticMessage('repair_strategy_final_answer', {
    count: strategies.length,
    targets: targets(strategies).join(', '),
    path: REPAIR_STRATEGY_PATH,
    document: trimEnd(document),
  });
}
