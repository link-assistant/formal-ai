import { ownedReadPaths } from './file_read/ownership.mjs';
// Which functions a module exports (PR #1188 T99, gap G26): "Which functions
// does src/m.mjs export?" read the module and dumped it. A question that
// names a module, the seeded export relation (`module_export_question`) and
// functions (`coding_declaration_noun`) is answered from the module's own
// declarations: a line that opens with a seeded export marker
// (`module_export_marker`: export, pub) and declares a function within its
// next words (`function_declaration_keyword`) exports that function.
// rust/src/agentic_coding/module_exports.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { detect } from './crate/language.mjs';
import { splitWhitespace } from './crate/rust_str.mjs';
import { renderResponse } from './crate/seed.mjs';
import { mentionsRole, wordsForRole } from './crate/seed_meanings.mjs';
import { readSource } from './module_function.mjs';
import { FinalDisposition, planOne, resolvedFinalAnswer } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { readArguments } from './workspace_change.mjs';

const MARKER_REACH = 3;
const CHANGE_ROLES = ['file_write_action_cue', 'file_edit_action_cue', 'coding_text_remove_action'];
const isIdentifierChar = (character) => /^[A-Za-z0-9_$]$/u.test(character);

/**
 * Mirrors `fn export_question` in rust/src/agentic_coding/module_exports.rs:
 * the module an export question names, or null.
 * @param {string} task
 */
export function exportQuestion(task) {
  const normalized = normalizePrompt(task);
  if (!mentionsRole('module_export_question', normalized) || !mentionsRole('coding_declaration_noun', normalized)
    // A request that changes the module (add an exported function) is not a question about it.
    || CHANGE_ROLES.some((role) => mentionsRole(role, normalized))) {
    return null;
  }
  return ownedReadPaths(task, 'module_export_question')[0] ?? null;
}

/**
 * Mirrors `fn exported_functions`: the names of the functions `source`
 * declares behind a seeded export marker, each once, in order.
 * @param {string} source
 */
export function exportedFunctions(source) {
  const markers = wordsForRole('module_export_marker');
  const keywords = wordsForRole('function_declaration_keyword');
  const names = [];
  for (const line of source.split('\n')) {
    const words = splitWhitespace(line);
    if (!words.length || !markers.includes(words[0])) continue;
    const at = words.findIndex((word, index) => index > 0 && index <= MARKER_REACH && keywords.includes(word));
    if (at < 0 || at + 1 >= words.length) continue;
    const chars = Array.from(words[at + 1]);
    const end = chars.findIndex((character) => !isIdentifierChar(character));
    const name = chars.slice(0, end < 0 ? chars.length : end).join('');
    if (name !== '' && !names.includes(name)) names.push(name);
  }
  return names;
}

/**
 * Mirrors `fn plan_module_exports_step`.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planModuleExportsStep(task, messages, toolNames) {
  const path = exportQuestion(task);
  if (path === null) return null;
  const source = readSource(messages.slice(evidenceWindowStart(messages)), path);
  if (source === null) {
    const read = toolFor(toolNames, Capability.Read);
    return read === null ? null : planOne(read, readArguments(path));
  }
  const names = exportedFunctions(source);
  const language = detect(task);
  const values = [['path', path], ['count', String(names.length)],
    ['names', names.map((name) => `\`${name}\``).join(', ')]];
  const intent = names.length ? 'module_exports_listed' : 'module_exports_none';
  return resolvedFinalAnswer(renderResponse(intent, language, values) ?? renderResponse(intent, 'en', values),
    FinalDisposition.Finding, intent);
}
