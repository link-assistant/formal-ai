import {requestTargetBinding, validWorkflowTarget} from './git_commit.mjs';
// The workflow a work item asks for beside the program
// (rust/src/agentic_coding/ci_workflow.rs, issue #1133).
//
// An `ExecutionRecipe` is `{language, source, path, supporting_files:
// [{path, source}], commands}`.

import { fill } from './work_item_steps.mjs';
import { runtimeSteps } from './crate/coding_program_contract.mjs';
import { ObligationKind, requestDemands } from './crate/intent_formalization_obligations.mjs';
import { fillWorkflowVersions, forGeneration, provenanceNote } from './crate/version_resolution.mjs';
import { isAsciiAlphanumeric } from './write_str.mjs';

/** Mirrors `fn workflow_path`. */
export function workflowPath() {
  return fill('workflow_path', []);
}

/** Read the canonical obligation graph after preserving independent work. */
export function requestedIn(objective) {
  return requestDemands(objective, ObligationKind.CiWorkflow);
}

/** `Path::file_stem` of a relative path, or null. */
function fileStem(path) {
  const name = path.split('/').filter((part) => part !== '' && part !== '.').pop();
  if (name === undefined || name === '..') return null;
  const dot = name.lastIndexOf('.');
  return dot <= 0 ? name : name.slice(0, dot);
}

/** Mirrors `fn attach`: add the workflow to `recipe` (mutated) once. */
export function attach(recipe, objective = '') {
  const name = Array.from(fileStem(recipe.path) ?? 'run')
    .map((value) => (isAsciiAlphanumeric(value) || value === '-' || value === '_' ? value : '-'))
    .join('');
  const binding = requestTargetBinding(objective, 'workflow-binding-patterns', validWorkflowTarget);
  if (binding.kind === 'refused') { recipe.commands.unshift(fill('target_binding_refused_command', [])); return; }
  const path = binding.kind === 'bound' ? binding.value : fill('workflow_path_named', [['{name}', name]]);
  if (recipe.supporting_files.some((file) => file.path === path)) return;
  recipe.supporting_files.push({ path, source: render(recipe) });
}

/** Mirrors `fn render`: the workflow over the generation-time version set. */
export function render(recipe) {
  return renderWith(recipe, forGeneration());
}

/**
 * Mirrors `fn render_with`: checkout, then the recipe's commands in order,
 * every pin filled from the injected `versions` (R1168-8), so a parity lane
 * renders the same workflow in both runtimes from one offline version set.
 */
export function renderWith(recipe, versions) {
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
