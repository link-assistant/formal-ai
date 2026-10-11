// Harness bookkeeping prompts that quote the task instead of posing it: the
// JavaScript twin of rust/src/agentic_coding/harness_envelope.rs.

import { agenticMessage } from './messages.mjs';
import { rustLines } from './content.mjs';
import { isWhitespace, rsplitOnce, splitOnce, stripPrefix, trim, trimStart } from './crate/rust_str.mjs';

/**
 * Mirrors `fn summarize_request` in rust/src/agentic_coding/harness_envelope.rs.
 * @param {string} received
 * @returns {string|null}
 */
export function summarizeRequest(received) {
  const rest = stripPrefix(trimStart(received), agenticMessage('harness_envelope_summarize_lead'));
  if (rest === null) return null;
  const opened = splitOnce(rest, '<text>');
  if (opened === null) return null;
  const quoted = opened[1];
  const closed = rsplitOnce(quoted, '</text>');
  return summaryOf(trim(closed === null ? quoted : closed[0]));
}

function summaryOf(text) {
  const firstLine = rustLines(text).map(trim).find((line) => line !== '') ?? '';
  let end = firstLine.length;
  for (let index = 0; index < firstLine.length; index += 1) {
    if (firstLine[index] !== '.') continue;
    const next = firstLine.codePointAt(index + 1);
    if (next === undefined || isWhitespace(String.fromCodePoint(next))) {
      end = index + 1;
      break;
    }
  }
  return firstLine.slice(0, end);
}
