// Reading this turn's tool results back out as execution records
// (rust/src/agentic_coding/transcript_evidence.rs).

import { plainText } from './content.mjs';
import { EvidenceSource, evidenceFromToolResult } from './crate/execution_evidence.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';

/** Mirrors `const NAMING_KEYS`. */
const NAMING_KEYS = ['path', 'filePath', 'file_path', 'file', 'command', 'cmd', 'query', 'url'];

/**
 * Mirrors `fn records` in rust/src/agentic_coding/transcript_evidence.rs: every
 * tool result of the current evidence window, as an `Evidence` record.
 * @param {Array<object>} messages
 */
export function records(messages) {
  const currentTurn = evidenceWindowStart(messages);
  const out = [];
  for (let index = currentTurn; index < messages.length; index += 1) {
    const message = messages[index];
    if (message.role.toLowerCase() !== 'tool') continue;
    const raw = plainText(message.content);
    const command = commandFor(messages, index, message);
    if (command === null) continue;
    out.push(evidenceFromToolResult(command, raw, EvidenceSource.Harness));
  }
  return out;
}

/** Mirrors `fn command_for` in rust/src/agentic_coding/transcript_evidence.rs. */
function commandFor(messages, index, result) {
  const call = precedingCall(messages, index, result);
  const tool = result.name ?? (call ? call[0] : null);
  if (tool === null || tool === undefined) return null;
  let rendered = tool;
  if (call) {
    let value;
    try {
      value = JSON.parse(call[1]);
    } catch {
      value = undefined;
    }
    if (value !== undefined) {
      for (const key of NAMING_KEYS) {
        const named = value && typeof value === 'object' && !Array.isArray(value) ? value[key] : undefined;
        if (typeof named === 'string') rendered += ` ${named}`;
      }
    }
  }
  return rendered;
}

/** Mirrors `fn preceding_call`: `[name, arguments]` or null. */
function precedingCall(messages, index, result) {
  for (let position = index - 1; position >= 0; position -= 1) {
    const calls = messages[position].tool_calls || [];
    if (!calls.length) continue;
    const id = result.tool_call_id ?? null;
    const call = calls.find((candidate) => id === null || candidate.id === id);
    if (call) return [call.function.name, call.function.arguments];
  }
  return null;
}
