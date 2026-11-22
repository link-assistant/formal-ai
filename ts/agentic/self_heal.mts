import { cached, host } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';
import { readRepairTrace, repairData, repairIntent } from './repair_input.mjs';
import { quote, ownedContentId } from './crate/change_request.mjs';
import { stableId } from './crate/engine_stable_identifier.mjs';

export const SELF_HEAL_PATH = 'self-healing-case.lino';

const SELF_HEAL_KEYWORDS = ['self-healing', 'self-heal', 'auto-learning', 'auto learning', 'repair case', 'repair loop'];

/** Mirrors `const SELF_REPAIR_PHRASES` (a `|`-separated list in agentic-messages.lino). */
const selfRepairPhrases = () => cached('self-heal-repair-phrases', () => agenticMessage('self_heal_repair_phrases').split('|'));

/**
 * Mirrors `fn is_self_heal_task` in rust/src/agentic_coding/self_heal.rs.
 * @param {string} prompt
 */
export function isSelfHealTask(prompt) {
  const lower = repairIntent(prompt).toLowerCase();
  return SELF_HEAL_KEYWORDS.some((keyword) => lower.includes(keyword))
    || selfRepairPhrases().some((phrase) => lower.includes(phrase))
    || ((lower.includes('cannot answer') || lower.includes("can't answer") || lower.includes('failure'))
      && (lower.includes('heal') || lower.includes('learn a')));
}

/** Bind the repair proposal to the caller's trace and independently verified source. */
export async function caseFromTask(task) {
  const input = repairData(task);
  const trace = readRepairTrace(input?.trace);
  const module = input?.source_module;
  if (!trace || typeof module !== 'string' || !ownedContentId(module)) return null;
  const runtime = host();
  if (!runtime.sourceDocument || !runtime.reviewRepairTrace) throw Error('SourceModelEngineUnavailable');
  const source = await runtime.sourceDocument(module);
  if (!source?.trim()) throw Error('SourceModelEngineUnavailable');
  const { run, traceLinks, learningLinks } = runtime.reviewRepairTrace(trace);
  const outcome = run.proposals.length ? 'blocked_by_benchmark' : 'no_candidate';
  const identifier = stableId('repair_case', `${trace.id}:${module}:${run.id}`);
  return { id: identifier, module, trace, outcome, source, traceLinks, learningLinks };
}

export function renderDocument(repair) {
  const field = (key, value, indent = '  ') => `${indent}${key} "${quote(value)}"\n`;
  const block = (key, value) => `  ${key}\n` + trimEnd(value).split('\n').map(line => line ? `    ${line}\n` : '\n').join('');
  return 'repair_case\n' + field('id', repair.id) + field('failure_prompt', repair.trace.prompt)
    + field('outcome', repair.outcome) + field('human_gated', 'true')
    + '  source_round_trip\n' + field('module_path', repair.module, '    ') + field('faithful', 'true', '    ')
    + block('source_content', repair.source) + block('failure_trace', repair.traceLinks) + block('learning_run', repair.learningLinks);
}

export function finalAnswer(repair, document) {
  return agenticMessage('self_heal_final_answer', {
    module: repair.module,
    outcome: repair.outcome,
    path: SELF_HEAL_PATH,
    document: trimEnd(document),
  });
}
