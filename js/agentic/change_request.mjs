// The issue-#558 self-change recipe (rust/src/agentic_coding/change_request.rs):
// a request to change Formal AI itself becomes a reviewable pull-request
// document (js/agentic/crate/change_request.mjs).

import { cached } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { canonicalChangeRequest, changeRequestLinksNotation } from './crate/change_request.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** Mirrors `const CHANGE_PATH`. */
export const CHANGE_PATH = 'requested-change.lino';

/** Mirrors `fn cached_change_request`. */
function cachedChangeRequest() {
  return cached('canonical-change-request', canonicalChangeRequest);
}

/** Mirrors `const CHANGE_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const changeKeywords = () => cached('change-request-keywords', () => agenticMessage('change_request_keywords').split('|'));

/**
 * Mirrors `fn is_change_request_task` in rust/src/agentic_coding/change_request.rs.
 * @param {string} prompt
 */
export function isChangeRequestTask(prompt) {
  const lower = prompt.toLowerCase();
  if (changeKeywords().some((keyword) => lower.includes(keyword))) return true;
  const targetsTheSystem = agenticMessage('change_request_system_targets').split('|').some((target) => lower.includes(target));
  const asksToChange = (lower.includes('change') || lower.includes('modify') || lower.includes('add a'))
    && (lower.includes('capability') || lower.includes('feature') || lower.includes('support'));
  return targetsTheSystem && asksToChange;
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/change_request.rs. */
export function renderDocument() {
  return `${trimEnd(changeRequestLinksNotation(cachedChangeRequest()))}\n`;
}

/** Mirrors `fn change_request`. */
export function changeRequest() {
  return { ...cachedChangeRequest() };
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/change_request.rs. */
export function finalAnswer(document) {
  const request = cachedChangeRequest();
  return agenticMessage('change_request_final_answer', {
    requirement: request.derived_requirement,
    test: request.proposed_test,
    steps: request.patch_plan.length,
    target: request.target_module,
    path: CHANGE_PATH,
    document: trimEnd(document),
  });
}
