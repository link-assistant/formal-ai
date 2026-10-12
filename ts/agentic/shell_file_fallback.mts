// Adapt a bounded append intent for clients that expose file tools but no
// shell: the JavaScript twin of rust/src/agentic_coding/shell_file_fallback.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { plainText } from './content.mjs';
import { finalAnswer, jsonText, planOne, writeArguments } from './plan.mjs';
import { Progress, resultCapability } from './progress.mjs';
import { normalizedPayload, render } from './tool_result.mjs';
import { eqIgnoreAsciiCase, splitOnce, splitWhitespace } from './crate/rust_str.mjs';

/**
 * Mirrors `fn plan_step` in rust/src/agentic_coding/shell_file_fallback.rs.
 * @param {string} task
 * @param {object[]} messages
 * @param {string[]} toolNames
 * @param {string} command
 */
export function planStep(task, messages, toolNames, command) {
  const operation = parseAppend(command);
  if (operation === null) return null;
  if (toolFor(toolNames, Capability.Run) !== null) return null;
  const readTool = toolFor(toolNames, Capability.Read);
  const writeTool = toolFor(toolNames, Capability.Write);
  if (readTool === null || writeTool === null) {
    const discovery = toolNames.find((name) => eqIgnoreAsciiCase(name, 'tool_search'));
    if (discovery === undefined || hasToolResult(messages, discovery)) return null;
    return planOne(discovery, jsonText({ query: 'select:write_file,run_shell_command', max_results: 2 }));
  }
  const progress = Progress.scan(messages);
  if (!progress.done(Capability.Read)) {
    return planOne(readTool, jsonText({ path: operation.path, filePath: operation.path, file_path: operation.path }));
  }
  if (!progress.done(Capability.Write)) {
    const readResult = latestResult(messages, Capability.Read);
    if (readResult === null) return null;
    const existing = normalizedPayload(readResult);
    if (existing === null) return null;
    return planOne(writeTool, writeArguments(operation.path, `${existing}\n${operation.payload}\n`));
  }
  return finalAnswer(render(command, latestResult(messages, Capability.Write) ?? '', task));
}

function parseAppend(command) {
  const split = splitOnce(command, ' >> ');
  if (split === null) return null;
  const [printf, destination] = split;
  if (!printf.startsWith('printf ')) return null;
  const path = splitWhitespace(destination)[0];
  if (path === undefined || !safeRelativePath(path)) return null;
  const payloadEnd = printf.lastIndexOf("'");
  if (payloadEnd < 0) return null;
  const payloadStart = printf.slice(0, payloadEnd).lastIndexOf("'");
  if (payloadStart < 0) return null;
  const payload = printf.slice(payloadStart + 1, payloadEnd);
  return payload === '' ? null : { path, payload };
}

/**
 * `Path::components` all `Component::Normal`: not rooted, no leading `.`
 * component, no `..` (interior `.` and repeated separators are skipped by
 * `Path::components`).
 */
function safeRelativePath(path) {
  if (path === '' || path.startsWith('/')) return false;
  const parts = path.split('/');
  if (parts[0] === '.') return false;
  return parts.every((part) => part !== '..');
}

function latestResult(messages, capability) {
  let lastUser = -1;
  messages.forEach((message, index) => {
    if (eqIgnoreAsciiCase(message.role, 'user')) lastUser = index;
  });
  for (let index = messages.length - 1; index >= lastUser + 1; index -= 1) {
    const message = messages[index];
    if (eqIgnoreAsciiCase(message.role, 'tool') && resultCapability(messages, index) === capability) {
      return plainText(message.content);
    }
  }
  return null;
}

function hasToolResult(messages, tool) {
  return messages.some((message, index) => {
    if (!eqIgnoreAsciiCase(message.role, 'tool')) return false;
    if (typeof message.name === 'string' && eqIgnoreAsciiCase(message.name, tool)) return true;
    const callId = message.tool_call_id;
    if (callId === null || callId === undefined) return false;
    return messages.slice(0, index).some((prior) => (prior.tool_calls || [])
      .some((call) => call.id === callId && eqIgnoreAsciiCase(call.function.name, tool)));
  });
}
