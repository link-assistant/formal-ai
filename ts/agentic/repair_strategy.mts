// The issue-#558 repair-strategy recipe (rust/src/agentic_coding/repair_strategy.rs).
//
// The Rust document is rendered from `crate::repair_strategy::canonical_strategies()`;
// the committed data/meta/repair-strategies.lino is asserted byte-for-byte equal
// to it (rust/tests/unit/issue_558_repair_strategy.rs), so the port reads it.

import { cached, childValue, childrenNamed, parseLino, readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** The top-level `name` node (the shared parser returns a lone top node itself). */
const topNode = (parsed, name) => (parsed?.name === name ? parsed : childrenNamed(parsed, name)[0]);

/** Mirrors `const REPAIR_STRATEGY_PATH`. */
export const REPAIR_STRATEGY_PATH = 'repair-strategies.lino';

const COMMITTED_STRATEGIES = 'data/meta/repair-strategies.lino';

/** Mirrors `const REPAIR_STRATEGY_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const repairStrategyKeywords = () => cached('repair-strategy-keywords', () => agenticMessage('repair_strategy_keywords').split('|'));

/**
 * Mirrors `fn is_repair_strategy_task` in rust/src/agentic_coding/repair_strategy.rs.
 * @param {string} prompt
 */
export function isRepairStrategyTask(prompt) {
  const lower = prompt.toLowerCase();
  return repairStrategyKeywords().some((keyword) => lower.includes(keyword));
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/repair_strategy.rs. */
export function renderDocument() {
  return cached('repair-strategy-document', () => `${trimEnd(readText(COMMITTED_STRATEGIES))}\n`);
}

/** The strategies' `target` slugs, in order (`RepairTarget::slug`). */
function targets() {
  const root = topNode(parseLino(renderDocument()), 'repair_strategies');
  return childrenNamed(root, 'repair_strategy').map((strategy) => childValue(strategy, 'target'));
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/repair_strategy.rs. */
export function finalAnswer(document) {
  const slugs = targets();
  return agenticMessage('repair_strategy_final_answer', {
    count: slugs.length,
    targets: slugs.join(', '),
    path: REPAIR_STRATEGY_PATH,
    document: trimEnd(document),
  });
}
