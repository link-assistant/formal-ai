// The issue-#558 rebuild-and-reattach recipe
// (rust/src/agentic_coding/rebuild_plan.rs), over the plan of
// js/agentic/crate/rebuild_plan.mjs.

import { cached } from './host.mjs';
import { agenticMessage } from './messages.mjs';
import { canonicalRebuildPlan, rebuildPlanLinksNotation } from './crate/rebuild_plan.mjs';
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
  const lower = prompt.toLowerCase();
  return rebuildKeywords().some((keyword) => lower.includes(keyword));
}

/** Mirrors `fn render_document` in rust/src/agentic_coding/rebuild_plan.rs. */
export function renderDocument() {
  return `${trimEnd(rebuildPlanLinksNotation(canonicalRebuildPlan()))}\n`;
}

/** Mirrors `fn plan`. */
export function plan() {
  return { ...canonicalRebuildPlan() };
}

/** Mirrors `fn final_answer` in rust/src/agentic_coding/rebuild_plan.rs. */
export function finalAnswer(document) {
  const rebuild = canonicalRebuildPlan();
  return agenticMessage('rebuild_plan_final_answer', {
    change: rebuild.change_id,
    reviewer: rebuild.reviewer,
    steps: rebuild.steps.length,
    artifacts: rebuild.artifacts.length,
    path: REBUILD_PATH,
    document: trimEnd(document),
  });
}
