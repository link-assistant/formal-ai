// The decomposition-question recognizer of
// rust/src/solver_handlers/task_decomposition.rs (`classify`,
// `looks_like_task_decomposition`). The handler's answer itself comes from the
// host solver.

import { mentionsRole, mentionsRoleRaw, mentionsRoleSeparated } from './seed_meanings.mjs';

const ROLE_DECOMPOSABLE_TASK_NOUN = 'decomposable_task_noun';
const ROLE_SUBTASK_UNIT_NOUN = 'subtask_unit_noun';
const ROLE_TASK_ATOMICITY_PREDICATE = 'task_atomicity_predicate';
const ROLE_TASK_FIRST_STEP_CUE = 'task_first_step_cue';
const ROLE_TASK_DECOMPOSITION_ACTION = 'task_decomposition_action';
const ROLE_SUBTASK_ENUMERATION_CUE = 'subtask_enumeration_cue';

/** Mirrors `fn mentions` in rust/src/solver_handlers/task_decomposition.rs. */
function mentions(role, normalized) {
  return mentionsRole(role, normalized) || mentionsRoleRaw(role, normalized)
    || mentionsRoleSeparated(role, normalized);
}

/**
 * Mirrors `fn classify`: 'atomicity' | 'first_step' | 'split' | null.
 * @param {string} normalized
 */
export function classifyDecompositionQuestion(normalized) {
  const taskNoun = mentions(ROLE_DECOMPOSABLE_TASK_NOUN, normalized);
  const unitNoun = mentions(ROLE_SUBTASK_UNIT_NOUN, normalized);
  if (mentions(ROLE_TASK_ATOMICITY_PREDICATE, normalized) && (taskNoun || unitNoun)) return 'atomicity';
  if (mentions(ROLE_TASK_FIRST_STEP_CUE, normalized) && taskNoun) return 'first_step';
  if (mentions(ROLE_TASK_DECOMPOSITION_ACTION, normalized) && (unitNoun || taskNoun)) return 'split';
  if (mentions(ROLE_SUBTASK_ENUMERATION_CUE, normalized) && unitNoun && taskNoun) return 'split';
  return null;
}

/**
 * Mirrors `fn looks_like_task_decomposition` in
 * rust/src/solver_handlers/task_decomposition.rs.
 * @param {string} normalized
 */
export function looksLikeTaskDecomposition(normalized) {
  return classifyDecompositionQuestion(normalized) !== null;
}
