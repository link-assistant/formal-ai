// The issue-#558 rebuild-and-reattach recipe
// (rust/src/agentic_coding/rebuild_plan.rs), over the plan of
// js/agentic/crate/rebuild_plan.mjs.

import { cached, host } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { rebuildPlanFor, rebuildPlanLinksNotation } from './crate/rebuild_plan.mjs';
import { ownedContentId } from './crate/change_request.mjs';
import { repairData, repairIntent } from './repair_input.mjs';
import { trimEnd } from './crate/rust_str.mjs';

/** Mirrors `const REBUILD_PATH`. */
export const REBUILD_PATH = 'rebuild-and-reattach.lino';

/** Mirrors `const REBUILD_KEYWORDS` (a `|`-separated list in agentic-messages.lino). */
const rebuildKeywords = () => cached('rebuild-keywords', () => agenticMessage('rebuild_plan_keywords').split('|'));

/**
 * Mirrors `fn is_rebuild_task` in rust/src/agentic_coding/rebuild_plan.rs.
 * @param {string} prompt
 */
export function isRebuildTask(prompt) {
  const lower = repairIntent(prompt).toLowerCase();
  return rebuildKeywords().some((keyword) => lower.includes(keyword));
}

/** Join a request to the operator's accepted change receipt. */
export function planFromTask(task) {
  const changeIdentifier = repairData(task)?.change_id;
  if (typeof changeIdentifier !== 'string' || !/^change_request_[0-9a-f]{16}$/u.test(changeIdentifier)) return null;
  try {
    const records = JSON.parse(host().readReviewDocument?.('accepted_changes') ?? '[]');
    if (!Array.isArray(records)) return null;
    const matches = records.filter(record => record?.change_id === changeIdentifier);
    const [record] = matches;
    if (matches.length !== 1 || !Number.isSafeInteger(record.benchmark_passed) || record.benchmark_passed < 1) return null;
    const keys = ['request', 'requirement', 'test', 'benchmark_suite', 'reviewer', 'target_module'];
    if (keys.some(key => typeof record[key] !== 'string' || !record[key].trim()) || !ownedContentId(record.target_module)) return null;
    return rebuildPlanFor(record);
  } catch {
    return null;
  }
}

export function renderDocument(plan) {
  return `${trimEnd(rebuildPlanLinksNotation(plan))}\n`;
}

export function finalAnswer(rebuild, document) {
  return agenticMessage('rebuild_plan_final_answer', {
    change: rebuild.change_id,
    reviewer: rebuild.reviewer,
    steps: rebuild.steps.length,
    artifacts: rebuild.artifacts.length,
    path: REBUILD_PATH,
    document: trimEnd(document),
  });
}
