// User-visible explanations emitted immediately before agentic tool calls
// (rust/src/agentic_coding/narration.rs).

import { Capability } from './capability.mjs';
import { classifyTool } from './capability_router.mjs';
import { programExecution, programLanguageByAlias, programTaskByAlias } from './crate/coding_catalog.mjs';
import { detect } from './crate/language.mjs';
import { localizedResponse } from './crate/seed.mjs';
import { shellIntentVocabulary } from './crate/seed_shell_intents.mjs';
import { narrationFor } from './local_search.mjs';
import { reportCommandIntent } from './report_issue.mjs';
import { commandArgument } from './tool_result.mjs';

const TARGET_SLOT = '{target}';
const SUBJECT_SLOT = '{subject}';

/**
 * Mirrors `fn tool_action_narration` in rust/src/agentic_coding/narration.rs.
 * @param {string} prompt
 * @param {Array<{tool: string, arguments: string}>} calls
 * @returns {string}
 */
export function toolActionNarration(prompt, calls) {
  const language = detect(prompt);
  const first = calls[0];
  if (!first) return '';
  const capability = classifyTool(first.tool);
  if (capability === Capability.Run) {
    const command = commandArgument(first.arguments);
    if (command !== null) {
      const report = reportCommandIntent(command);
      if (report !== null) return localizedResponse(report, language) ?? '';
    }
    const found = narrationFor(prompt);
    if (found) {
      const template = localizedResponse(found.intent, language);
      if (template !== null) return template.split(SUBJECT_SLOT).join(found.subject.trim());
    }
    const intent = programCommandIntent(prompt, first.arguments);
    if (intent !== null) return localizedResponse(intent, language) ?? '';
    if (command !== null) {
      const name = shellActionName(command);
      if (name !== null) {
        return (localizedResponse('agentic-action-run-program-name', language) ?? '').replace(TARGET_SLOT, name);
      }
    }
  }
  const intent = capability === null ? 'agentic_action_generic' : capabilityIntent(capability);
  const template = localizedResponse(intent, language) ?? '';
  return template.split(TARGET_SLOT).join(toolActionTarget(first.arguments));
}

/** Show a known subcommand when available and keep operands out of narration. */
function shellActionName(command) {
  const [executable, ...operands] = command.trim().split(/\s+/u, 3);
  const program = executable.split('/').at(-1);
  if (!/^[A-Za-z0-9][A-Za-z0-9_.-]{0,47}$/u.test(program)) return null;
  const words = [program, ...operands];
  const known = shellIntentVocabulary().intents;
  for (let count = words.length; count > 1; count--) {
    const action = words.slice(0, count).join(' ');
    if (known.some(intent => intent.command === action)) return action;
  }
  return program;
}

/** Mirrors `const fn capability_intent` in rust/src/agentic_coding/narration.rs. */
function capabilityIntent(capability) {
  switch (capability) {
    case Capability.Search: return 'agentic_action_search';
    case Capability.Fetch:
    case Capability.Read:
    case Capability.ReadMany: return 'agentic_action_read';
    case Capability.Grep: return 'agentic_action_search_code';
    case Capability.Write:
    case Capability.Edit:
    case Capability.MultiEdit: return 'agentic_action_edit';
    case Capability.Run: return 'agentic_action_run';
    case Capability.AskUser: return 'agentic_action_ask_user';
    default: return 'agentic_action_generic';
  }
}

/** Mirrors `fn program_command_intent` in rust/src/agentic_coding/narration.rs. */
function programCommandIntent(prompt, args) {
  const normalized = prompt.toLowerCase();
  const task = programTaskByAlias(normalized);
  if (!task) return null;
  const language = programLanguageByAlias(normalized);
  if (!language) return null;
  const command = commandArgument(args);
  if (command === null) return null;
  const execution = programExecution(task.slug, language);
  if (execution.check_command === command) return 'agentic_action_compile_program';
  if (execution.run_command === command) return 'agentic_action_run_program';
  return null;
}

const TARGET_FIELDS = ['url', 'query', 'path', 'file_path', 'filePath', 'command', 'prompt', 'pattern', 'target', 'title', 'name'];

function firstText(value) {
  if (typeof value === 'string') return value.trim() ? value : null;
  if (Array.isArray(value)) {
    for (const item of value) {
      const found = firstText(item);
      if (found !== null) return found;
    }
    return null;
  }
  if (value && typeof value === 'object') {
    for (const field of TARGET_FIELDS) {
      if (Object.prototype.hasOwnProperty.call(value, field)) {
        const found = firstText(value[field]);
        if (found !== null) return found;
      }
    }
    // `values()` of a serde_json map walks keys in sorted order.
    for (const key of Object.keys(value).sort()) {
      const found = firstText(value[key]);
      if (found !== null) return found;
    }
  }
  return null;
}

/** Mirrors `fn tool_action_target` in rust/src/agentic_coding/narration.rs. */
export function toolActionTarget(args) {
  let parsed;
  try {
    parsed = JSON.parse(args);
  } catch {
    parsed = undefined;
  }
  const target = ((parsed === undefined ? null : firstText(parsed)) ?? args).trim();
  const chars = Array.from(target);
  return chars.length > 160 ? `${chars.slice(0, 160).join('')}…` : target;
}
