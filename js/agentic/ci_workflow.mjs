// The workflow a work item asks for beside the program
// (rust/src/agentic_coding/ci_workflow.rs, issue #1133).
//
// An `ExecutionRecipe` is `{language, source, path, supporting_files:
// [{path, source}], commands}`.

import { fill } from './work_item_steps.mjs';
import { runtimeSteps } from './crate/coding_program_contract.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { fillWorkflowVersions, forGeneration, provenanceNote } from './crate/version_resolution.mjs';
import { mentionsRole } from './write_lexicon.mjs';
import { isAsciiAlphanumeric } from './write_str.mjs';

/** Mirrors `fn workflow_path`. */
export function workflowPath() {
  return fill('workflow_path', []);
}

/** Mirrors `fn requested_in`: whether the work item asks for a CI workflow. */
export function requestedIn(objective) {
  return mentionsRole('ci_workflow_request', normalizePrompt(objective));
}

/** `Path::file_stem` of a relative path, or null. */
function fileStem(path) {
  const name = path.split('/').filter((part) => part !== '' && part !== '.').pop();
  if (name === undefined || name === '..') return null;
  const dot = name.lastIndexOf('.');
  return dot <= 0 ? name : name.slice(0, dot);
}

/** Mirrors `fn attach`: add the workflow to `recipe` (mutated) once. */
export function attach(recipe) {
  const name = Array.from(fileStem(recipe.path) ?? 'run')
    .map((value) => (isAsciiAlphanumeric(value) || value === '-' || value === '_' ? value : '-'))
    .join('');
  const path = fill('workflow_path_named', [['{name}', name]]);
  if (recipe.supporting_files.some((file) => file.path === path)) return;
  recipe.supporting_files.push({ path, source: render(recipe) });
}

/** Mirrors `fn render`: checkout, then the recipe's commands in order. */
export function render(recipe) {
  const versions = forGeneration();
  let out = '';
  for (const note of provenanceNote(versions)) out += `# ${note}\n`;
  out += fill('workflow_template', [['{path}', recipe.path]]);
  // Filling versions trims the fragment's final newline; each fragment is a
  // whole line block, so it is restored before the next one is appended.
  out = `${fillWorkflowVersions(out, versions)}\n`;
  const setup = runtimeSteps(recipe.language);
  if (setup !== null) out += `${fillWorkflowVersions(setup, versions)}\n`;
  for (const command of recipe.commands) out += fill('workflow_command_step', [['{command}', command]]);
  return out;
}
