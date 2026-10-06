// Answering a question about a task's own structure from Formal AI's
// decomposition of it (issue #1066): rust/src/agentic_coding/task_structure.rs.

import { announcesAListItDoesNotMake, defersToTheOpenWeb, isInconclusive } from './crate/engine_answer.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { trim } from './crate/rust_str.mjs';
import { looksLikeTaskDecomposition } from './crate/solver_handlers_task_decomposition.mjs';
import { solve } from './host.mjs';
import { composedDocumentSpecificationSpan } from './note_composition.mjs';
import { finalAnswer } from './plan.mjs';
import { traceRoute } from './planner/continuation.mjs';
import { hasLatestTurnResult } from './tool_result.mjs';

/**
 * Mirrors `fn plan_task_structure_step` in rust/src/agentic_coding/task_structure.rs
 * (async: the answer is the engine's).
 * @param {Array<object>} messages
 * @param {string} task
 * @returns {Promise<object|null>}
 */
export async function planTaskStructureStep(messages, task) {
  if (!nothingHasBeenObservedYet(messages)) return null;
  if (!looksLikeTaskDecomposition(normalizePrompt(task))) return null;
  if (composedDocumentSpecificationSpan(task) !== null) return null;
  const answer = await solve(task, []);
  if (isInconclusive(answer) || defersToTheOpenWeb(answer) || announcesAListItDoesNotMake(answer)) return null;
  const text = trim(answer.answer);
  if (text === '') return null;
  traceRoute('task_structure', answer.intent);
  return finalAnswer(text);
}

/** Mirrors `fn nothing_has_been_observed_yet`. */
function nothingHasBeenObservedYet(messages) {
  return !hasLatestTurnResult(messages);
}
