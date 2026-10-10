import { ownedReadPaths } from './file_read/ownership.mjs';
// Summarizing a named file (PR #1188 T100, gap G27): "Summarize README.md in
// one sentence." answered "Read 1 file(s): README.md: # Demo". A request with
// the seeded summarization action (`text_summarization_action`, every
// registered language) that names one file reads it and hands the request,
// the file's text in place of its name, to the shared solver's summarization
// handlers. rust/src/agentic_coding/file_summary.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { splitWhitespace } from './crate/rust_str.mjs';
import { mentionsRole } from './crate/seed_meanings.mjs';
import { solve } from './host.mjs';
import { readSource } from './module_function.mjs';
import { FinalDisposition, planOne, resolvedFinalAnswer } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import { readArguments } from './workspace_change.mjs';

const SUMMARY_ROLE = 'text_summarization_action';
const SUMMARY_INTENT_PREFIX = 'summarization';
const CLAUSE_END = /[.!?。！？।]+$/u;

/**
 * Mirrors `fn summarized_file` in rust/src/agentic_coding/file_summary.rs:
 * the one file a summarization request names, or null.
 * @param {string} task
 */
export function summarizedFile(task) {
  if (!mentionsRole(SUMMARY_ROLE, normalizePrompt(task))) return null;
  const paths = ownedReadPaths(task, SUMMARY_ROLE);
  return paths.length === 1 ? paths[0] : null;
}

/** Mirrors `fn summary_request`: the request with the file's text in place of its name. */
export function summaryRequest(task, path, text) {
  const words = splitWhitespace(task).filter((word) => !word.includes(path));
  const request = words.join(' ').replace(CLAUSE_END, '');
  return `${request}: ${text.trim()}`;
}

/**
 * Mirrors `fn plan_file_summary_step`.
 * @param {string} task
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export async function planFileSummaryStep(task, messages, toolNames) {
  const path = summarizedFile(task);
  if (path === null) return null;
  const source = readSource(messages.slice(evidenceWindowStart(messages)), path);
  if (source === null) {
    const read = toolFor(toolNames, Capability.Read);
    return read === null ? null : planOne(read, readArguments(path));
  }
  if (source.trim() === '') return null;
  const answer = await solve(summaryRequest(task, path, source), []);
  const intent = String(answer?.intent ?? '');
  return intent.startsWith(SUMMARY_INTENT_PREFIX) && answer.answer
    ? resolvedFinalAnswer(answer.answer, FinalDisposition.Finding, intent) : null;
}
