// The issue-#558 self-explanation recipe (rust/src/agentic_coding/explain.rs).
//
// An exact port: the document is `crate::self_explanation`'s canonical
// explanation (js/agentic/crate/self_explanation.mjs), whose source citations
// are content-addressed through the owned manifest the host enumerates from
// rust/src exactly as build.rs does.

import { cached } from './host.mjs';
import { canonicalExplanation } from './crate/self_explanation.mjs';
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

/** Mirrors `fn cached_explanation` (a `OnceLock`). */
const cachedExplanation = () => cached('self-explanation', canonicalExplanation);

/**
 * Mirrors `fn render_document` in rust/src/agentic_coding/explain.rs.
 * Deterministic and ends with exactly one trailing newline.
 */
export function renderDocument() {
  return `${trimEnd(cachedExplanation().linksNotation())}\n`;
}

/** Mirrors `fn explanation` in rust/src/agentic_coding/explain.rs. */
export function explanation() {
  return cachedExplanation();
}

/**
 * Mirrors `fn final_answer` in rust/src/agentic_coding/explain.rs.
 * @param {string} document
 */
export function finalAnswer(document) {
  const current = cachedExplanation();
  return agenticMessage('explain_final_answer', {
    sections: current.sectionCount(), citations: current.citationCount(), path: EXPLAIN_PATH, document: trimEnd(document),
  });
}
