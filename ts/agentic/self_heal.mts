// The issue-#558 self-healing recipe (rust/src/agentic_coding/self_heal.rs).
//
// The Rust document is `crate::self_healing::canonical_case().links_notation()`,
// which parses a real module through the CST/AST engine; the committed
// data/meta/self-healing-case.lino is asserted byte-for-byte equal to it
// (rust/tests/unit/agentic-coding/issue_558_self_healing.rs), so the port reads that artifact.

import { cached, childValue, childrenNamed, parseLino, readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';

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
export function renderDocument() {
  return cached('self-heal-document', () => `${trimEnd(readText(COMMITTED_CASE))}\n`);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/self_heal.rs. */
export function finalAnswer(document) {
  const repairCase = topNode(parseLino(renderDocument()), 'repair_case');
  const roundTrip = childrenNamed(repairCase, 'source_round_trip')[0];
  return agenticMessage('self_heal_final_answer', {
    module: childValue(roundTrip, 'module_path'),
    outcome: childValue(repairCase, 'outcome'),
    path: SELF_HEAL_PATH,
    document: trimEnd(document),
  });
}
