// The issue-#558 self-explanation recipe (rust/src/agentic_coding/explain.rs).
//
// The task predicate is an exact port. The document is native-only: see
// `renderDocument`.

import { cached } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { rustLines } from './content.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** Mirrors `const EXPLAIN_PATH`. */
export const EXPLAIN_PATH = 'how-formal-ai-works.lino';

/** Mirrors `const EXPLAIN_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const explainKeywords = () => cached('explain-keywords', () => agenticMessage('explain_keywords').split('|'));

/**
 * Mirrors `fn is_explain_task` in rust/src/agentic_coding/explain.rs.
 * @param {string} prompt
 */
export function isExplainTask(prompt) {
  const lower = prompt.toLowerCase();
  const repositoryWorkItem = lower.includes('github.com/')
    && (lower.includes('/issues/') || lower.includes('/pull/'))
    && rustLines(lower).length > 3;
  if (repositoryWorkItem) return false;
  if (explainKeywords().some((keyword) => lower.includes(keyword))) return true;
  const asksHowItWorks = lower.includes('how')
    && (lower.includes('work') || lower.includes('explain'))
    && (lower.includes('formal ai') || lower.includes('the system') || lower.includes('yourself'));
  const grounded = lower.includes('source') && (lower.includes('data') || lower.includes('test'));
  return asksHowItWorks && grounded;
}

/**
 * Mirrors `fn render_document` in rust/src/agentic_coding/explain.rs.
 * native-only: rust/src/self_explanation.rs `canonical_explanation`; every
 * citation is content-addressed through the owned manifest of the whole
 * embedded Rust source tree (build.rs `OWNED_SOURCE_FILES`), which a JS host
 * cannot enumerate. The stub renders only the record head.
 */
export function renderDocument() {
  return 'system_explanation\n';
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/explain.rs.
 * native-only: the section and citation counts of the stubbed explanation are 0.
 */
export function finalAnswer(document) {
  return agenticMessage('explain_final_answer', { sections: 0, citations: 0, path: EXPLAIN_PATH, document: trimEnd(document) });
}
