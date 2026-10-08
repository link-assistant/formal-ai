// The issue-#558 learning-ledger recipe (rust/src/agentic_coding/ledger.rs).
//
// The Rust document is `crate::learning_ledger::canonical_ledger().links_notation()`;
// the committed data/meta/learning-ledger.lino is asserted byte-for-byte equal
// to it (rust/tests/unit/agentic-coding/issue_558_learning_ledger.rs), so the port reads it.

import { cached, childrenNamed, parseLino, readText } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** The top-level `name` node (the shared parser returns a lone top node itself). */
const topNode = (parsed, name) => (parsed?.name === name ? parsed : childrenNamed(parsed, name)[0]);

/** Mirrors `const LEDGER_PATH`. */
export const LEDGER_PATH = 'learning-ledger.lino';

const COMMITTED_LEDGER = 'data/meta/learning-ledger.lino';

/** Mirrors `const LEDGER_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const ledgerKeywords = () => cached('ledger-keywords', () => agenticMessage('ledger_keywords').split('|'));

/**
 * Mirrors `fn is_ledger_task` in rust/src/agentic_coding/ledger.rs.
 * @param {string} prompt
 */
export function isLedgerTask(prompt) {
  const lower = prompt.toLowerCase();
  return ledgerKeywords().some((keyword) => lower.includes(keyword))
    || (lower.includes('promote') && (lower.includes('lesson') || lower.includes('learning')) && lower.includes('ledger'));
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/ledger.rs. */
export function renderDocument() {
  return cached('ledger-document', () => `${trimEnd(readText(COMMITTED_LEDGER))}\n`);
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/ledger.rs (`LearningLedger::len`). */
export function finalAnswer(document) {
  const ledger = topNode(parseLino(renderDocument()), 'learning_ledger');
  return agenticMessage('ledger_final_answer', {
    count: childrenNamed(ledger, 'lesson').length,
    path: LEDGER_PATH,
    document: trimEnd(document),
  });
}
