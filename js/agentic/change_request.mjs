// The issue-#558 self-change recipe (rust/src/agentic_coding/change_request.rs):
// a request to change Formal AI itself becomes a reviewable pull-request
// document (js/agentic/crate/change_request.mjs).

import { cached } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { changeRequestForModule, changeRequestLinksNotation } from './crate/change_request.mjs';
import { repairData, repairIntent } from './repair_input.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** Mirrors `const CHANGE_PATH`. */
export const CHANGE_PATH = 'requested-change.lino';

/** Mirrors `const CHANGE_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const changeKeywords = () => cached('change-request-keywords', () => agenticMessage('change_request_keywords').split('|'));

/**
 * Mirrors `fn is_change_request_task` in rust/src/agentic_coding/change_request.rs.
 * @param {string} prompt
 */
export function isChangeRequestTask(prompt) {
  const lower = repairIntent(prompt).toLowerCase();
  if (changeKeywords().some((keyword) => lower.includes(keyword))) return true;
  const targetsTheSystem = agenticMessage('change_request_system_targets').split('|').some((target) => lower.includes(target));
  const asksToChange = (lower.includes('change') || lower.includes('modify') || lower.includes('add a'))
    && (lower.includes('capability') || lower.includes('feature') || lower.includes('support'));
  return targetsTheSystem && asksToChange;
}

/** Ground the requested change in the runtime's owned source. */
export function requestFromTask(task) {
  const input = repairData(task);
  if (typeof input?.request !== 'string' || typeof input.target_module !== 'string') return null;
  try {
    return changeRequestForModule(input.request, input.target_module);
  } catch {
    return null;
  }
}

export function renderDocument(request) {
  return `${trimEnd(changeRequestLinksNotation(request))}\n`;
}

export function finalAnswer(request, document) {
  return agenticMessage('change_request_final_answer', {
    requirement: request.derived_requirement,
    test: request.proposed_test,
    steps: request.patch_plan.length,
    target: request.target_module,
    path: CHANGE_PATH,
    document: trimEnd(document),
  });
}
