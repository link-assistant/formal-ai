// The decomposition-question recognizer of
// rust/src/solver_handlers/task_decomposition.rs (`classify`,
// `looks_like_task_decomposition`) and its pure seeded answer renderer.
// The shared tree preserves the existing native atomicity contract.

import { mentionsRole, mentionsRoleRaw, mentionsRoleSeparated } from './seed_meanings.mjs';
import {normalizePrompt} from './engine.mjs';
import {detect} from './language.mjs';
import {localizedResponse} from './seed.mjs';
import {DEFAULT_SPLIT_DEPTH_BOUND} from './task_decomposition.mjs';
import {decomposeTask, leaves} from './task_decomposition_tree.mjs';
import {statedTask} from './task_decomposition_stated_task.mjs';

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

/** Mirrors native Decomposition::numbered_lines without rebuilding its tree. */
export function numberedDecompositionLines(decomposition, marker) {
  const rows = [];
  function visit(node) {
    if (node.depth > 0) {
      const suffix = node.reason === 'depth_bound' ? ' ' + marker : '';
      rows.push(node.path + '. ' + node.text + ' [' + node.completion_criterion + ']' + suffix);
    }
    for (const child of node.children) visit(child);
  }
  visit(decomposition.root);
  return rows;
}

function seededDecompositionResponse(intent, language) {
  return localizedResponse(intent, language) ?? '';
}

function withSubTasks(lead, decomposition, language) {
  const marker = seededDecompositionResponse('task_decomposition_depth_marker', language);
  const lines = [lead, ...numberedDecompositionLines(decomposition, marker)];
  if (leaves(decomposition).some(leaf => leaf.reason === 'depth_bound')) {
    lines.push(seededDecompositionResponse('task_decomposition_depth_bound', language));
  }
  return lines.join('\n');
}

/**
 * Pure counterpart of try_task_decomposition_with_depth.
 * This renders an inspected plan; it never claims that its leaves were executed.
 */
export function answerTaskDecomposition(prompt, maxDepth = DEFAULT_SPLIT_DEPTH_BOUND) {
  const question = classifyDecompositionQuestion(normalizePrompt(prompt));
  if (question === null) return null;
  const task = statedTask(prompt,
    sentence => looksLikeTaskDecomposition(normalizePrompt(sentence)));
  if (task === '') return null;
  const decomposition = decomposeTask(task, maxDepth);
  const language = detect(prompt);
  const response = intent => seededDecompositionResponse(intent, language);
  const unenumerable = !decomposition.root.atomic && decomposition.root.children.length === 0
    ? decomposition.root.reason : null;
  let intent;
  let answer;
  if (question === 'first_step') {
    intent = 'task_first_step';
    answer = response('task_first_step_lead') + '\n'
      + (leaves(decomposition)[0]?.text ?? decomposition.task);
  } else if (question === 'atomicity') {
    intent = 'task_atomicity';
    answer = decomposition.root.atomic ? response('task_atomicity_yes')
      : unenumerable !== null
        ? response(unenumerable === 'depth_bound'
          ? 'task_decomposition_unsplit_depth_bound' : 'task_decomposition_single_need')
        : withSubTasks(response('task_atomicity_no'), decomposition, language);
  } else {
    intent = 'task_decomposition';
    answer = decomposition.root.atomic ? response('task_decomposition_atomic')
      : unenumerable !== null
        ? response(unenumerable === 'depth_bound'
          ? 'task_decomposition_unsplit_depth_bound' : 'task_decomposition_single_need')
        : withSubTasks(response('task_decomposition_lead'), decomposition, language);
  }
  return {intent, answer, confidence: 0.86, question, decomposition};
}
