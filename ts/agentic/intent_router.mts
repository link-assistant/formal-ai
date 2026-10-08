// Issue #680: the general, intent-based capability router: the JavaScript twin
// of rust/src/agentic_coding/intent_router.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { renderSeededChange } from './code_task.mjs';
import { finalAnswer, jsonText, planOne } from './plan.mjs';
import { Progress } from './progress.mjs';
import { requestBlocks } from './stated_request.mjs';
import { latestTurnAnswer, latestTurnQuietSuccess, render, renderFailure } from './tool_result.mjs';
import { statedWebSearchQueryForBlock } from './web_research.mjs';
import { composeEditRequest } from './write_request.mjs';
import { eqIgnoreAsciiCase } from './crate/rust_str.mjs';

/**
 * Mirrors `fn plan_web_search_step` in rust/src/agentic_coding/intent_router.rs.
 * @param {string} task
 * @param {object[]} messages
 * @param {string[]} toolNames
 */
export function planWebSearchStep(task, messages, toolNames) {
  let query = null;
  for (const block of requestBlocks(task)) {
    query = statedWebSearchQueryForBlock(block);
    if (query !== null) break;
  }
  if (query === null) return null;
  const tool = toolFor(toolNames, Capability.Search);
  if (tool === null) {
    const discovery = toolNames.find((name) => eqIgnoreAsciiCase(name, 'tool_search'));
    if (discovery === undefined || toolResultExists(messages, discovery)) return null;
    return planOne(discovery, jsonText({ query: 'web search', max_results: 5 }));
  }
  const progress = Progress.scan(messages);
  const failure = progress.latestFailure();
  if (failure && failure.capability === Capability.Search) {
    return finalAnswer(renderFailure('web_search', failure.detail, task));
  }
  if (progress.done(Capability.Search)) return finalAnswer(render('web_search', progress.searchResult() ?? '', task));
  return planOne(tool, jsonText({ query }));
}

function toolResultExists(messages, toolName) {
  let currentTurn = 0;
  messages.forEach((message, index) => {
    if (eqIgnoreAsciiCase(message.role, 'user')) currentTurn = index + 1;
  });
  for (let index = currentTurn; index < messages.length; index += 1) {
    const message = messages[index];
    if (!eqIgnoreAsciiCase(message.role, 'tool')) continue;
    if (typeof message.name === 'string' && eqIgnoreAsciiCase(message.name, toolName)) return true;
    const callId = message.tool_call_id;
    if (typeof callId !== 'string') continue;
    if (messages.slice(0, index).some((prior) => (prior.tool_calls || [])
      .some((call) => call.id === callId && eqIgnoreAsciiCase(call.function.name, toolName)))) {
      return true;
    }
  }
  return false;
}

/**
 * Mirrors `fn plan_edit_step` in rust/src/agentic_coding/intent_router.rs.
 * @param {string} task
 * @param {object[]} messages
 * @param {string[]} toolNames
 */
export function planEditStep(task, messages, toolNames) {
  const edit = composeEditRequest(task);
  if (!edit) return null;
  const [target, old, replacement] = edit;
  const tool = toolFor(toolNames, Capability.Edit);
  if (tool === null) return null;
  const progress = Progress.scan(messages);
  if (progress.done(Capability.Edit)) {
    const answer = latestTurnAnswer(messages, toolNames, task);
    if (answer === null) return null;
    // An edit tool's quiet reply is its success: the answer names the
    // replacement, not the empty output (PR #1188 T181).
    const stated = latestTurnQuietSuccess(messages)
      ? renderSeededChange('coding_text_replaced_unchecked', task, target, [['{old}', old], ['{new}', replacement]])
      : null;
    return finalAnswer(stated ?? answer);
  }
  const readTool = toolFor(toolNames, Capability.Read);
  if (readTool !== null && !progress.done(Capability.Read)) {
    return planOne(readTool, jsonText({ path: target, filePath: target, file_path: target }));
  }
  return planOne(tool, editArguments(target, old, replacement));
}

/**
 * Mirrors `fn edit_arguments` in rust/src/agentic_coding/intent_router.rs.
 * @param {string} path
 * @param {string} old
 * @param {string} replacement
 */
export function editArguments(path, old, replacement) {
  return jsonText({
    path,
    filePath: path,
    file_path: path,
    oldString: old,
    old_string: old,
    old_str: old,
    old,
    newString: replacement,
    new_string: replacement,
    new_str: replacement,
    new: replacement,
  });
}
