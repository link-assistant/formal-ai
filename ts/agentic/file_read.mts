import { ownedReadPaths } from './file_read/ownership.mjs';
export { ownedReadPaths, readPolicyBlocksPlan } from './file_read/ownership.mjs';
import { tokens } from './write_request.mjs';
// File-reading agentic recipe for local workspace prompts (issue #627):
// rust/src/agentic_coding/file_read.rs.
//
// A `FileReadTask` is `{kind: 'direct', path, mode, prefer_run}`,
// `{kind: 'direct_many', paths, mode}` or
// `{kind: 'list_then_read', directory, selection, mode}`; every task also
// carries a non-enumerable `isAnalysis()` method (`FileReadTask::is_analysis`).
// A `FileReadMode` is `{kind: 'full' | 'first_line' | 'summary' | 'audit'}` or
// `{kind: 'extract_value', key}`; a `FileSelection` is 'first' | 'last' | 'all'.

import { classifyTool } from './capability_router.mjs';
import { Capability } from './capability.mjs';
import { Progress } from './progress.mjs';
import { sourceReadAnswer } from './file_read/source.mjs';
import { toolResultRecords, readResultForPath, grepResultForPath, runRecordForCommand } from './file_read/records.mjs';
export { toolResultRecords, readResultForPath, grepResultForPath, runRecordForCommand, samePath } from './file_read/records.mjs';
import { latestUserRequest, plainText, rustLines } from './content.mjs';
import { isDottedNumber, peelSentencePunctuation } from './file_path_shape.mjs';
import { hasFileWriteIntent } from './general_planner.mjs';
import { agenticMessage } from './messages.mjs';
import { FinalDisposition, finalAnswer, jsonText, plannedCall, resolvedFinalAnswer, toolCalls } from './plan.mjs';
import { sentences } from './shell_command_policy.mjs';
import { explicitPassthroughCommand } from './shell_command.mjs';

/** Mirrors `SHELL_OPERATORS`: shell syntax that chains, pipes or redirects commands. */
const SHELL_OPERATORS = ['&&', '||', ';', '|', '>'];
import { commandArgument, harnessReportedFailure, render, stripTransportEnvelope } from './tool_result.mjs';
import { byteOrder, eqIgnoreAsciiCase, replaceAllLiteral, rsplitOnce, splitWhitespace, toAsciiLowercase,
  trim, trimMatches } from './crate/rust_str.mjs';
import { mentionsRole, wordsForRole } from './crate/seed_meanings.mjs';
import { fileReadFinalAnswer } from './file_read/audit.mjs';
import { lineSlice } from './workspace_line_operation.mjs';
import { exactLineKey, planDirectFileReads } from './file_read/exact.mjs';

export { suppliedFileAnswer } from './file_read/supplied.mjs';

const AUDIT_READ_LINE_LIMIT = 160;
/** A cue phrase the Rust recipe spells inline, read from the messages file. */
const cue = (key) => agenticMessage(`file_read_cue_${key}`);
const AUDIT_READ_COLUMN_LIMIT = 320;
const ROLE_FILE_READ_ACTION_CUE = 'file_read_action_cue';
const ROLE_WORKSPACE_INSPECTION_ACTION = 'workspace_inspection_action';
const ROLE_FILE_ANALYSIS_GAP_MARKER = 'file_analysis_gap_marker';

/** `FileReadMode` constructors. */
export const FileReadMode = Object.freeze({
  Full: Object.freeze({ kind: 'full' }),
  FirstLine: Object.freeze({ kind: 'first_line' }),
  Summary: Object.freeze({ kind: 'summary' }),
  Audit: Object.freeze({ kind: 'audit' }),
  extractValue: (key) => ({ kind: 'extract_value', key }),
  /** A run of lines (`lineSlice` in workspace_line_operation.mjs, PR #1188 G33). */
  lineSlice: (slice) => ({ kind: 'line_slice', ...slice }),
});

/** `FileReadMode` equality (`==`): Rust built-in `#[derive(PartialEq)]` on `FileReadMode`. */
export function sameMode(left, right) {
  return left.kind === right.kind && (left.kind !== 'extract_value' || left.key === right.key)
    && (left.kind !== 'line_slice'
      || (left.from === right.from && left.to === right.to && left.fromEnd === right.fromEnd));
}

const isAudit = (mode) => mode.kind === 'audit';

function withAnalysis(task) {
  Object.defineProperty(task, 'isAnalysis', { value: () => isAnalysis(task), enumerable: false });
  return task;
}

/** Mirrors `FileReadTask::is_analysis` in rust/src/agentic_coding/file_read.rs. */
export function isAnalysis(task) {
  return isAudit(task.mode);
}

/** `FileReadTask::Direct`. */
export const directTask = (path, mode, preferRun = false) =>
  withAnalysis({ kind: 'direct', path, mode, prefer_run: preferRun });
/** `FileReadTask::DirectMany`. */
export const directManyTask = (paths, mode) => withAnalysis({ kind: 'direct_many', paths, mode });
/** `FileReadTask::ListThenRead`. */
export const listThenReadTask = (directory, selection, mode) =>
  withAnalysis({ kind: 'list_then_read', directory, selection, mode });

/**
 * Mirrors `fn plan_file_read_step` in rust/src/agentic_coding/file_read.rs.
 * @param {object} task a `FileReadTask`
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planFileReadStep(task, messages, toolNames) {
  const readTool = toolFor(toolNames, Capability.Read);
  const runTool = toolFor(toolNames, Capability.Run);
  const grepTool = toolFor(toolNames, Capability.Grep);
  const records = toolResultRecords(messages);
  const progress = Progress.scan(messages);
  const request = latestUserRequest(messages) ?? '';
  switch (task.kind) {
    case 'direct':
      return planDirectFileRead(task.path, task.mode, task.prefer_run, readTool, runTool, grepTool, records, request, progress);
    case 'direct_many':
      return planDirectFileReads(task.paths, task.mode, readTool, runTool, grepTool, records, request, progress);
    default:
      return planListThenRead(task.directory, task.selection, task.mode, readTool, runTool, records, request, progress);
  }
}

/** Mirrors `fn failed_step_answer`. */
export function failedStepAnswer(label, raw, request) {
  return harnessReportedFailure(raw) ? render(label, raw, request) : null;
}

/** Mirrors `fn plan_direct_file_read`. */
function planDirectFileRead(path, mode, preferRun, readTool, runTool, grepTool, records, request, progress) {
  if (isAudit(mode)) {
    return planDirectFileReads([path], mode, readTool, runTool, grepTool, records, request, progress);
  }
  const readCommand = readCommandFor(path, mode);
  const exactRun = exactLineKey(request) !== null && runTool !== null;
  const fromRun = () => {
    const raw = runRecordForCommand(records, readCommand);
    return raw === null ? null : [readCommand, raw, stripTransportEnvelope(raw)];
  };
  if (!exactRun) {
    const answer = sourceReadAnswer([path], mode, records, progress, request);
    if (answer !== null) return answer;
  }
  const recorded = fromRun();
  if (recorded !== null) {
    const [label, raw, observed] = recorded;
    const failure = failedStepAnswer(label, raw, request);
    if (failure !== null) return finalAnswer(failure);
    const content = observed;
    return resolvedFinalAnswer(fileReadFinalAnswer(mode, [[path, content]], request), FinalDisposition.Finding, 'file_read_observed');
  }
  if ((preferRun || exactRun) && runTool !== null) {
    return planOne(runTool, jsonText({ command: readCommandFor(path, mode) }));
  }
  if (readTool !== null) return planOne(readTool, readArguments(path, mode));
  if (runTool !== null) return planOne(runTool, jsonText({ command: readCommandFor(path, mode) }));
  return finalAnswer(agenticMessage('file_read_tool_missing', { path }));
}

/** Mirrors `fn plan_list_then_read`. */
function planListThenRead(directory, selection, mode, readTool, runTool, records, request, progress) {
  const listCommand = listFilesCommand(directory);
  const rawListing = runRecordForCommand(records, listCommand);
  if (rawListing === null) {
    if (runTool !== null) return planOne(runTool, jsonText({ command: listCommand }));
    return finalAnswer(agenticMessage('file_read_listing_tool_missing'));
  }
  const listingFailure = failedStepAnswer(listCommand, rawListing, request);
  if (listingFailure !== null) return finalAnswer(listingFailure);
  const listing = stripTransportEnvelope(rawListing);
  const paths = selectedPathsFromListing(directory, listing, selection);
  if (!paths.length) return finalAnswer(agenticMessage('file_read_listing_empty', { directory }));

  const sourceAnswer = sourceReadAnswer(paths, mode, records, progress, request);
  if (sourceAnswer !== null) return sourceAnswer;

  if (selection === 'all') {
    if (readTool !== null) {
      return toolCalls(paths
        .filter((path) => readResultForPath(records, path) === null)
        .map((path) => plannedCall(readTool, readArguments(path, mode))));
    }
  } else if (paths.length && readTool !== null) {
    return planOne(readTool, readArguments(paths[0], mode));
  }

  if (runTool !== null) {
    const command = selection === 'all' ? catManyCommand(paths) : readCommandFor(paths[0], mode);
    const raw = runRecordForCommand(records, command);
    if (raw !== null) {
      const failure = failedStepAnswer(command, raw, request);
      if (failure !== null) return finalAnswer(failure);
      return resolvedFinalAnswer(fileReadFinalAnswer(mode, [[paths.join(', '), stripTransportEnvelope(raw)]], request), FinalDisposition.Finding, 'file_read_observed');
    }
    return planOne(runTool, jsonText({ command }));
  }
  return finalAnswer(agenticMessage('file_read_selected_tool_missing'));
}

/**
 * Mirrors `fn file_read_task_for` in rust/src/agentic_coding/file_read.rs.
 * @param {string} prompt
 * @returns {object|null} a `FileReadTask`
 */
export function fileReadTaskFor(prompt) {
  const lower = prompt.toLowerCase();
  if (hasFileWriteIntent(lower)) return null;
  // A command quoted after a run prefix that chains, pipes or redirects is
  // run as written, not read file by file (PR #1188 G62).
  if (SHELL_OPERATORS.some((operator) => explicitPassthroughCommand(prompt)?.includes(operator))) return null;
  const catPath = leadingCatPath(prompt);
  if (catPath !== null) return directTask(catPath, FileReadMode.Full, false);
  if (asksToReadEveryFile(lower)) return listThenReadTask('.', 'all', FileReadMode.Summary);
  if (asksToListThenRead(lower)) {
    return listThenReadTask(directoryForListRead(prompt) ?? '.', selectionForPrompt(lower), modeForPrompt(prompt));
  }
  if (asksToReadFileInFolder(lower)) {
    return listThenReadTask(directoryForListRead(prompt) ?? '.', 'first', modeForPrompt(prompt));
  }
  const readPaths = readPathsNamedBesideTheirCue(prompt);
  if (readPaths.length > 1) return directManyTask(readPaths, modeForPrompt(prompt));
  if (readPaths.length === 1) return directTask(readPaths[0], modeForPrompt(prompt), false);
  return null;
}

/** Preserve source clause punctuation owned by lexical path spans. */
export function readSentenceTexts(prompt) {
  const protectedCharacters = prompt.split('');
  for (const token of tokens(prompt)) {
    const path = cleanFileToken(token.text);
    if (!looksLikeLocalFilePath(path)) continue;
    const relativeStart = token.text.indexOf(path);
    if (relativeStart < 0) continue;
    const start = token.start + relativeStart;
    for (let index = start; index < start + path.length; index += 1) {
      if (prompt[index] === '.') protectedCharacters[index] = '_';
    }
  }
  return sentences(protectedCharacters.join('')).map(sentence =>
    trim(prompt.slice(sentence.span.start, sentence.span.end)));
}

/** Mirrors `fn read_paths_named_beside_their_cue`. */
function readPathsNamedBesideTheirCue(prompt) {
  return ownedReadPaths(prompt, ROLE_FILE_READ_ACTION_CUE);
}

/** Mirrors `fn asks_to_read_every_file`. */
function asksToReadEveryFile(lower) {
  return (lower.includes(cue('read_every_file')) || lower.includes(cue('read_all_files')))
    && (lower.includes('summarize') || lower.includes('summary') || lower.includes('here'));
}

/** Mirrors `fn asks_to_list_then_read`. */
function asksToListThenRead(lower) {
  const listsFiles = lower.includes(cue('list_the_files')) || lower.includes('list files')
    || lower.includes(cue('ls_the_folder')) || splitWhitespace(lower).includes('ls');
  const readsAfter = lower.includes('read') || lower.includes('contents') || lower.includes('content')
    || lower.includes('show me');
  return listsFiles && readsAfter;
}

/** Mirrors `fn asks_to_read_file_in_folder`. */
function asksToReadFileInFolder(lower) {
  return lower.includes(cue('read_the_file')) && (lower.includes(' folder') || lower.includes(' directory'));
}

/** Mirrors `fn selection_for_prompt`. */
function selectionForPrompt(lower) {
  return lower.includes('last') ? 'last' : 'first';
}

/** Mirrors `fn mode_for_prompt`. */
function modeForPrompt(prompt) {
  const lower = prompt.toLowerCase();
  if (lower.includes('first line')) return FileReadMode.FirstLine;
  const key = extractValueKey(prompt) ?? exactLineKey(prompt);
  if (key !== null) return FileReadMode.extractValue(key);
  if (mentionsRole(ROLE_WORKSPACE_INSPECTION_ACTION, lower)) return FileReadMode.Audit;
  if (lower.includes('summarize') || lower.includes('summary')) return FileReadMode.Summary;
  const slice = lineSlice(prompt);
  return slice === null ? FileReadMode.Full : FileReadMode.lineSlice(slice);
}

/** Mirrors `fn extract_value_key`. */
function extractValueKey(prompt) {
  const marker = 'value of ';
  const at = toAsciiLowercase(prompt).indexOf(marker);
  if (at < 0) return null;
  const rest = prompt.slice(at + marker.length);
  const key = trimMatches(splitWhitespace(rest)[0] ?? '', (character) => !isFilePathChar(character));
  return key ? key : null;
}

/** Mirrors `fn leading_cat_path`. */
function leadingCatPath(prompt) {
  const trimmed = trim(trimMatches(trim(prompt), (character) => character === '`'));
  const parts = splitWhitespace(trimmed);
  if (!parts.length || !eqIgnoreAsciiCase(parts[0], 'cat') || parts.length < 2) return null;
  const path = cleanFileToken(parts[1]);
  return path ? path : null;
}

/** Mirrors `fn local_file_paths` (consecutive duplicates removed). */
function localFilePaths(prompt) {
  const out = [];
  for (const token of splitWhitespace(prompt).map(cleanFileToken)) {
    if (!looksLikeLocalFilePath(token)) continue;
    if (out[out.length - 1] !== token) out.push(token);
  }
  return out;
}

const SENTENCE_MARKS = new Set([',', ';', ':', '!', '?', ')', '(', '[', ']', '{', '}']);

/** Mirrors `fn clean_file_token`. */
export function cleanFileToken(token) {
  return peelSentencePunctuation(token, (current) => {
    let text = trimMatches(current, (character) => character === '`');
    text = trimMatches(text, (character) => character === '"');
    text = trimMatches(text, (character) => character === "'");
    return trimMatches(text, (character) => SENTENCE_MARKS.has(character));
  });
}

/** Mirrors `fn looks_like_local_file_path`. */
export function looksLikeLocalFilePath(token) {
  if (!token || token.includes('://') || token.startsWith('http:') || token.startsWith('https:')
    || isDottedNumber(token)) return false;
  const allPathChars = Array.from(token).every(isFilePathChar);
  if (token.includes('/')) return allPathChars;
  const split = rsplitOnce(token, '.');
  if (split === null) return false;
  const [stem, extension] = split;
  return stem !== '' && extension !== '' && new TextEncoder().encode(extension).length <= 12 && allPathChars;
}

/** Mirrors `const fn is_file_path_char`. */
export function isFilePathChar(character) {
  return /^[0-9A-Za-z_\-./\\@]$/.test(character);
}

/** Mirrors `fn directory_for_list_read`. */
function directoryForListRead(prompt) {
  const tokens = splitWhitespace(prompt).map(cleanFileToken);
  for (let index = 0; index + 3 <= tokens.length; index += 1) {
    const [first, second, third] = tokens.slice(index, index + 3);
    if (eqIgnoreAsciiCase(first, 'the') && second !== ''
      && (eqIgnoreAsciiCase(third, 'folder') || eqIgnoreAsciiCase(third, 'directory'))) return second;
    if ((eqIgnoreAsciiCase(first, 'in') || eqIgnoreAsciiCase(first, 'inside'))
      && eqIgnoreAsciiCase(second, 'the') && third !== '') return third;
  }
  for (let index = 0; index + 2 <= tokens.length; index += 1) {
    const [first, second] = tokens.slice(index, index + 2);
    const lower = toAsciiLowercase(first);
    if (first !== '' && (eqIgnoreAsciiCase(second, 'folder') || eqIgnoreAsciiCase(second, 'directory'))
      && !['the', 'a', 'this', 'current'].includes(lower)) return first;
  }
  return null;
}

/** Mirrors `fn read_arguments`. */
export function readArguments(path, mode) {
  const args = { filePath: path, path, file_path: path };
  if (isAudit(mode)) {
    args.offset = 0;
    args.limit = AUDIT_READ_LINE_LIMIT;
    args.columnOffset = 0;
    args.columnLimit = AUDIT_READ_COLUMN_LIMIT;
  }
  return jsonText(args);
}

/** Mirrors `fn grep_arguments`. */
export function grepArguments(path, pattern) {
  return jsonText({ path, pattern });
}

/** Mirrors `fn file_analysis_pattern`. */
export function fileAnalysisPattern() {
  const alternatives = ['\\[\\s*\\]'];
  for (const surface of wordsForRole(ROLE_FILE_ANALYSIS_GAP_MARKER)) {
    if (trim(surface) !== '') alternatives.push(regexEscape(surface));
  }
  alternatives.sort(byteOrder);
  const unique = alternatives.filter((value, index) => index === 0 || alternatives[index - 1] !== value);
  return `(?i)(?:${unique.join('|')})`;
}

/** Mirrors `fn regex_escape`. */
function regexEscape(surface) {
  return Array.from(surface, (character) => ('.+*?()|[]{}^$\\'.includes(character) ? `\\${character}` : character))
    .join('');
}

/** Mirrors `fn read_command_for`. */
export function readCommandFor(path, mode) {
  switch (mode.kind) {
    case 'first_line':
      return `head -n 1 ${shellPath(path)}`;
    case 'extract_value':
      return ['sed', '-n', shellString(`s/^${mode.key}=//p`), shellPath(path)].join(' ');
    case 'audit':
      return ['sed', '-n', `'1,${AUDIT_READ_LINE_LIMIT}p'`, shellPath(path), '|', 'cut', '-c',
        `1-${AUDIT_READ_COLUMN_LIMIT}`].join(' ');
    default:
      return `cat ${shellPath(path)}`;
  }
}

/** Mirrors `fn list_files_command`. */
function listFilesCommand(directory) {
  if (directory === '.') return "find . -maxdepth 1 -type f | sed 's#^./##' | sort";
  return `find ${shellPath(directory)} -maxdepth 1 -type f | sed 's#^.*/##' | sort`;
}

/** Mirrors `fn cat_many_command`. */
function catManyCommand(paths) {
  return paths.map((path) => `printf '==> %s <==\\n' ${shellPath(path)}; cat ${shellPath(path)}`).join('; ');
}

/** Mirrors `fn shell_path`. */
function shellPath(path) {
  if (/^[0-9A-Za-z_\-./]*$/.test(path)) return path;
  return `'${replaceAllLiteral(path, "'", "'\\''")}'`;
}

/** Mirrors `fn shell_string`. */
function shellString(value) {
  return `'${replaceAllLiteral(value, "'", "'\\''")}'`;
}

/** Mirrors `fn selected_paths_from_listing`. */
function selectedPathsFromListing(directory, listing, selection) {
  const entries = rustLines(listing)
    .map(trim)
    .filter((line) => line !== '')
    .map((line) => line.replace(/^(?:\.\/)+/u, ''));
  entries.sort(byteOrder);
  const unique = entries.filter((entry, index) => index === 0 || entries[index - 1] !== entry);
  let selected;
  if (selection === 'first') selected = unique.slice(0, 1);
  else if (selection === 'last') selected = unique.slice(-1);
  else selected = unique;
  return selected.map((entry) => (directory === '.' || entry.includes('/') ? entry : `${directory}/${entry}`));
}

/** Mirrors `fn tool_for` in rust/src/agentic_coding/file_read.rs (first advertised name of `capability`). */
function toolFor(toolNames, capability) {
  return toolNames.find((name) => classifyTool(name) === capability) ?? null;
}

/** Mirrors `fn plan_one`. */
function planOne(tool, args) {
  return toolCalls([plannedCall(tool, args)]);
}

