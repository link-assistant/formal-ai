// Exact-field and multi-file branches of the local file-read recipe
// (rust/src/agentic_coding/file_read/exact.rs).

import { detect } from '../crate/language.mjs';
import { toAsciiLowercase, trim } from '../crate/rust_str.mjs';
import { localizedResponse } from '../crate/seed.mjs';
import { agenticMessage } from '../messages.mjs';
import { FinalDisposition, FinalPayloadRole, resolvedFinalAnswer, finalAnswer, jsonText, plannedCall, toolCalls } from '../plan.mjs';
import { sentences } from '../shell_command_policy.mjs';
import { stripTransportEnvelope } from '../tool_result.mjs';
import { sourceReadAnswer, readObservation } from './source.mjs';
import {
  failedStepAnswer, fileAnalysisPattern, grepArguments, grepResultForPath, readArguments, readCommandFor,
  readResultForPath, runRecordForCommand,
} from '../file_read.mjs';
import { fileReadFinalAnswer } from './audit.mjs';

/**
 * Mirrors `fn plan_direct_file_reads` in rust/src/agentic_coding/file_read/exact.rs:
 * read every named input before composing the observation.
 */
export function planDirectFileReads(paths, mode, readTool, runTool, grepTool, records, request, progress) {
  if (mode.kind === 'audit' && grepTool !== null) {
    const pattern = fileAnalysisPattern();
    const contents = [];
    for (const path of paths) {
      const raw = grepResultForPath(records, path, pattern);
      if (raw === null) continue;
      const failure = failedStepAnswer(path, raw, request);
      if (failure !== null) return finalAnswer(failure);
      contents.push([path, raw]);
    }
    if (contents.length === paths.length) return resolvedFinalAnswer(fileReadFinalAnswer(mode, contents, request),
      FinalDisposition.Finding, 'file_read_observed', FinalPayloadRole.AuditReport);
    return toolCalls(paths
      .filter((path) => grepResultForPath(records, path, pattern) === null)
      .map((path) => plannedCall(grepTool, grepArguments(path, pattern))));
  }

  const exactRun = exactLineKey(request) !== null && runTool !== null;
  if (!exactRun) {
    const answer = sourceReadAnswer(paths, mode, records, progress, request);
    if (answer !== null) return answer;
  }
  let complete = true;
  const contents = [];
  for (const path of paths) {
    const command = readCommandFor(path, mode);
    if (exactRun) {
      const raw = runRecordForCommand(records, command);
      if (raw !== null) {
        const failure = failedStepAnswer(command, raw, request);
        if (failure !== null) return finalAnswer(failure);
        contents.push([path, stripTransportEnvelope(raw)]);
        continue;
      }
    }
    if (!exactRun) {
      const read = readObservation(records, progress, path);
      if (read !== null) {
        contents.push([path, read.source ?? '']);
        complete &&= read.complete;
        continue;
      }
      const ran = runRecordForCommand(records, command);
      if (ran !== null) {
        const failure = failedStepAnswer(command, ran, request);
        if (failure !== null) return finalAnswer(failure);
        contents.push([path, stripTransportEnvelope(ran)]);
      }
    }
  }
  if (contents.length === paths.length) return resolvedFinalAnswer(fileReadFinalAnswer(mode, contents, request),
    complete ? FinalDisposition.Finding : FinalDisposition.Unknown, 'file_read_observed');

  const runCalls = (tool) => toolCalls(paths
    .filter((path) => runRecordForCommand(records, readCommandFor(path, mode)) === null)
    .map((path) => plannedCall(tool, jsonText({ command: readCommandFor(path, mode) }))));
  if (exactRun && runTool !== null) return runCalls(runTool);
  if (readTool !== null) {
    return toolCalls(paths
      .filter((path) => readResultForPath(records, path) === null)
      .map((path) => plannedCall(readTool, readArguments(path, mode))));
  }
  if (runTool !== null) return runCalls(runTool);
  return finalAnswer(localizedResponse('file_read_many_unavailable', detect(request)) ?? '');
}

/**
 * Mirrors `fn exact_line_key` in rust/src/agentic_coding/file_read/exact.rs:
 * the field prefix a sentence says to take from one exact line, or null.
 * @param {string} prompt
 */
export function exactLineKey(prompt) {
  for (const sentence of sentences(prompt)) {
    const lower = toAsciiLowercase(sentence.text);
    const identifiesLine = lower.includes(agenticMessage('file_read_cue_line_beginning_exactly'))
      || lower.includes(agenticMessage('file_read_cue_line_that_begins_exactly'))
      || lower.includes(agenticMessage('file_read_cue_line_starting_exactly'))
      || lower.includes(agenticMessage('file_read_cue_line_that_starts_exactly'));
    if (!identifiesLine) continue;
    const quoted = sentence.text.split('`').filter((_, index) => index % 2 === 1).map(trim);
    for (const candidate of quoted) {
      if (!candidate.endsWith('=')) continue;
      const key = candidate.slice(0, -1);
      if (key !== '' && /^[0-9A-Za-z_]+$/.test(key)) return key;
    }
  }
  return null;
}
