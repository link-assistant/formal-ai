// The issue-#558 self-healing recipe (rust/src/agentic_coding/self_heal.rs).
//
// The Rust document is `crate::self_healing::canonical_case().links_notation()`,
// The committed case preserves the historical proposal and failure trace only.
// Current source authority comes from the host-owned recursive content engine.

import { cached, childValue, childrenNamed, parseLino, readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';
import { renderDocument as sourceDocument } from './self_ast.mjs';

/** The top-level `name` node (the shared parser returns a lone top node itself). */
const topNode = (parsed, name) => (parsed?.name === name ? parsed : childrenNamed(parsed, name)[0]);

/** Mirrors `const SELF_HEAL_PATH`. */
export const SELF_HEAL_PATH = 'self-healing-case.lino';

/** The committed twin of `canonical_case().links_notation()`. */
const COMMITTED_CASE = 'data/meta/self-healing-case.lino';

const SELF_HEAL_KEYWORDS = ['self-healing', 'self-heal', 'auto-learning', 'auto learning', 'repair case', 'repair loop'];

/** Mirrors `const SELF_REPAIR_PHRASES` (a `|`-separated list in agentic-messages.lino). */
const selfRepairPhrases = () => cached('self-heal-repair-phrases', () => agenticMessage('self_heal_repair_phrases').split('|'));

/**
 * Mirrors `fn is_self_heal_task` in rust/src/agentic_coding/self_heal.rs.
 * @param {string} prompt
 */
export function isSelfHealTask(prompt) {
  const lower = prompt.toLowerCase();
  return SELF_HEAL_KEYWORDS.some((keyword) => lower.includes(keyword))
    || selfRepairPhrases().some((phrase) => lower.includes(phrase))
    || ((lower.includes('cannot answer') || lower.includes("can't answer") || lower.includes('failure'))
      && (lower.includes('heal') || lower.includes('learn a')));
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/self_heal.rs. */
export async function renderDocument() {
  const original = trimEnd(readText(COMMITTED_CASE));
  const start = original.indexOf('  source_round_trip\n');
  const end = original.indexOf('  failure_trace\n', start);
  if (start < 0 || end <= start) throw Error('RepairCaseSourceBoundaryRequired');
  const document = await sourceDocument();
  const content = document.trimEnd().split('\n').map((line) => '    ' + line).join('\n');
  return original.slice(0, start)
    + '  source_round_trip\n    module_path "src/agentic_coding/planner.rs"\n    faithful "true"\n'
    + '  source_content\n' + content + '\n' + original.slice(end) + '\n';
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/self_heal.rs. */
export function finalAnswer(document) {
  const repairCase = topNode(parseLino(document), 'repair_case');
  const roundTrip = childrenNamed(repairCase, 'source_round_trip')[0];
  return agenticMessage('self_heal_final_answer', {
    module: childValue(roundTrip, 'module_path'),
    outcome: childValue(repairCase, 'outcome'),
    path: SELF_HEAL_PATH,
    document: trimEnd(document),
  });
}
