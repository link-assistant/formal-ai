// Grounded workspace rewrites and composite module changes
// (rust/src/agentic_coding/workspace_change.rs).

import { scopedDecline } from './edit_scope.mjs';
import { absentTextAnswer, replaceList, replacedInOrder } from './replace_list.mjs';
import { Capability } from './capability.mjs';
import { classifyTool, toolFor } from './capability_router.mjs';
import { sourceFromAgentReadResult, sourceFromReadResult } from './code_artifact.mjs';
import { renderRustTemplate, renderSeededChange, renderSeededOutcome, rustSourceForTask } from './code_task.mjs';
import { plainText } from './content.mjs';
import { contentsSource, withContents } from './contents_source.mjs';
import { insertedInSection, sectionScope } from './markdown_section.mjs';
import { lineRangeMove, planLineRangeMoveStep } from './line_range_move.mjs';
import { composeEditRequest } from './general_planner.mjs';
import { editArguments } from './intent_router.mjs';
import { FinalDisposition, finalAnswer, jsonText, planOne, resolvedFinalAnswer, writeArguments } from './plan.mjs';
import { evidenceWindowStart } from './planner/continuation.mjs';
import {
  anchorContext, composePositionalInsert, indentsLinesAt, introducedBlock, leadingIndentation, positionalInserts, rebasedBlock,
  joinedLiteralLines, unescapeProseNewlines, unquotedPathTokens,
} from './positional_edit.mjs';
import { bareSurfaces, cleanCueToken, cleanPathToken, looksLikeFilePath, safeRelativePath, tokens } from './write_request.mjs';
import { commandArgument, failureMessage } from './tool_result.mjs';
import { quotedSegmentSpans, quotedSegments, quotesWhole, unwrapTransportQuotes } from './crate/normal_markov.mjs';
import { sha256Hex } from './crate/source_fetch.mjs';
import { correctedSpelling } from './crate/spelling.mjs';
import {
  RewriteScope, executeScopedWorkspaceRewrite, isIdentifierWord, wordScopedMatches,
} from './crate/workspace_change_learning.mjs';
import { meaningEvidencedIn, mentionsRole, wordsForRole } from './write_lexicon.mjs';
import { assignedSetting, replacedLines, statedOldValue } from './workspace_setting.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { isAsciiAlphanumeric, lines, matchIndices, splitWhitespace, trim, trimEndMatches } from './write_str.mjs';
import { dropsMostOfFile, groundedLineOperation, namedTargetAndPayloads, withoutPathWords } from './workspace_line_operation.mjs';

const eqIgnoreAsciiCase = (left, right) => left.replace(/[A-Z]/g, (c) => c.toLowerCase()) === right.replace(/[A-Z]/g, (c) => c.toLowerCase());
const finalOrNull = (text, disposition = FinalDisposition.Unknown, origin = null) =>
  (text === null ? null : resolvedFinalAnswer(text, disposition, origin));

const statedIntent = (rewrite) => rewrite.intent ?? (rewrite.renaming ? 'coding_identifier_renamed' : 'coding_text_replaced');
const statedSlots = (rewrite) => rewrite.slots ?? [['{old}', rewrite.pattern], ['{new}', rewrite.replacement]];

/**
 * Mirrors `fn plan_workspace_change_step`.
 * @param {string} rawTask
 * @param {Array<object>} messages
 * @param {Array<string>} toolNames
 */
export function planWorkspaceChangeStep(rawTask, messages, toolNames) {
  const currentTurn = messages.slice(evidenceWindowStart(messages));
  // `Append the contents of a.txt to b.txt`: the source is read, and the
  // request is restated with its lines as the block placed (PR #1188 G50).
  const sourced = sourcedRequest(unwrapTransportQuotes(rawTask), currentTurn);
  if (sourced?.read) return planWithTool(toolNames, Capability.Read, readArguments(sourced.read));
  const task = sourced?.task ?? unwrapTransportQuotes(rawTask);
  const change = compositeModuleChange(task);
  if (change) return planCompositeStep(task, currentTurn, toolNames, change);
  // Numbered lines moved to another file: placed there, then removed (G85).
  const rangeMove = lineRangeMove(task);
  if (rangeMove) return planLineRangeMoveStep(task, currentTurn, toolNames, rangeMove);
  // A whole-line operation (numbered, adjacent, moved, swapped lines) is read
  // before a rewrite: `move 'x' after 'y'` is not an insertion (PR #1188).
  const lineOperation = groundedLineOperation(task);
  if (lineOperation) return planComputedChangeStep(task, currentTurn, toolNames, { ...lineOperation, edit: changedLinesEdit });
  const inserts = insertSequence(task);
  if (inserts) return planInsertSequenceStep(task, currentTurn, toolNames, inserts);
  // Several replacements asked in one sentence are made in order (G82).
  const list = groundedReplaceList(task);
  if (list) return planComputedChangeStep(task, currentTurn, toolNames, list);
  const rewrite = groundedRewrite(task);
  if (rewrite) return planRewriteStep(task, currentTurn, toolNames, rewrite);
  const computed = groundedEndInsertion(task) ?? groundedRemoval(task) ?? groundedDeclarationRemoval(task)
    ?? groundedLineReplacement(task) ?? groundedSetting(task) ?? groundedTypoFix(task);
  return computed ? planComputedChangeStep(task, currentTurn, toolNames, computed) : null;
}

/**
 * Mirrors `fn sourced_request`: for a request whose additive edit places
 * another file's contents, `{read: path}` until that file is read in this
 * turn, then `{task}` restated with its lines; null otherwise, or when the
 * restated request is no additive edit the composers place.
 */
function sourcedRequest(task, currentTurn) {
  const source = contentsSource(task);
  if (source === null) return null;
  const places = (request) => insertSequence(request) !== null || groundedEndInsertion(request) !== null;
  if (!places(withContents(task, source, 'x'))) return null;
  const read = resultForPath(currentTurn, Capability.Read, source.path, null);
  if (read === null) return { read: source.path };
  if (sourceFromAgentReadResult(read) === null && failureMessage(read, false, true) !== null) return null;
  return { task: withContents(task, source, sourceFromReadResult(read)) };
}

/** Mirrors `fn is_verification_failure_answer`. */
export function isVerificationFailureAnswer(rawTask, answer) {
  const task = unwrapTransportQuotes(rawTask);
  const edit = composeEditRequest(task);
  const targets = [...(edit ? [edit[0]] : []), ...rustPaths(task)];
  const named = namedTargetAndPayloads(task);
  if (named && !targets.includes(named.target)) targets.push(named.target);
  return targets.some((target) => ['coding_workspace_verification_failed', 'coding_workspace_fragment_refused']
    .some((intent) => renderSeededOutcome(intent, task, target) === answer));
}

function failed(task, target) {
  return finalOrNull(renderSeededOutcome('coding_workspace_verification_failed', task, target),
    FinalDisposition.Failure, 'coding_workspace_verification_failed');
}

function planWithTool(toolNames, capability, args) {
  const tool = toolFor(toolNames, capability);
  return tool ? planOne(tool, args) : null;
}

function planRewriteStep(task, currentTurn, toolNames, grounded) {
  const read = resultForPath(currentTurn, Capability.Read, grounded.target, null);
  if (read === null) return planWithTool(toolNames, Capability.Read, readArguments(grounded.target));
  const source = sourceFromReadResult(read);
  const rewrite = asWritten(source, grounded);
  const updated = rewrittenSource(source, rewrite);
  if (updated === null) {
    // A replace asked again finds its old text gone: that is the answer (G87).
    const absent = absentTextAnswer(task, rewrite.target, source, rewrite.pattern, rewrite.replacement);
    return absent === null ? failed(task, rewrite.target) : finalAnswer(absent);
  }
  const occurrences = rewrite.scope === RewriteScope.Substring
    ? matchIndices(source, rewrite.pattern).length
    : wordScopedMatches(source, rewrite.pattern).length;
  const scoped = occurrences > 1 ? scopedDecline(task, rewrite.target, source) : null;
  if (scoped !== null) return finalOrNull(scoped);
  const verified = { target: rewrite.target, expected: updated, intent: statedIntent(rewrite), slots: statedSlots(rewrite) };
  if (occurrences === 1) {
    const tool = toolFor(toolNames, Capability.Edit);
    const edit = rewrite.unique
      ? anchoredLines(source, rewrite).slice(1)
      : matchIndices(source, rewrite.pattern).length === 1
        ? [rewrite.pattern, rewrite.replacement]
        : changedLinesEdit(source, updated);
    if (tool && edit) {
      if (resultForEdit(currentTurn, rewrite.target, edit[0], edit[1]) === null) {
        return planOne(tool, editArguments(rewrite.target, edit[0], edit[1]));
      }
      return planDigestVerification(task, currentTurn, toolNames, verified);
    }
  } else if (toolFor(toolNames, Capability.Edit)) {
    const command = repeatedIdentifierRewriteCommand(rewrite);
    const tool = command === null ? null : toolFor(toolNames, Capability.Run);
    if (tool) {
      if (resultForCommand(currentTurn, command) === null) return planOne(tool, jsonText({ command }));
      return planDigestVerification(task, currentTurn, toolNames, verified);
    }
  }
  if (resultForPath(currentTurn, Capability.Write, rewrite.target, updated) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(rewrite.target, updated));
  }
  const command = `cat ${rewrite.target}`;
  const observed = resultForCommand(currentTurn, command);
  if (observed === null) return planWithTool(toolNames, Capability.Run, jsonText({ command }));
  if (observed !== updated) return failed(task, rewrite.target);
  return finalOrNull(renderSeededChange(statedIntent(rewrite), task, rewrite.target, statedSlots(rewrite)),
    FinalDisposition.Finding, statedIntent(rewrite));
}

/**
 * Mirrors `fn grounded_replace_list`: several quoted replacements in one file,
 * asked in one sentence, as one computed change (PR #1188 G82).
 */
function groundedReplaceList(task) {
  const list = replaceList(task);
  if (list === null) return null;
  const { target, pairs } = list;
  return {
    target,
    compute: (source, missing) => (missing ? null : replacedInOrder(source, pairs)),
    edit: changedLinesEdit,
    intent: 'coding_texts_replaced',
    slots: [['{pairs}', pairs.map(([old, next]) => `\`${old}\` → \`${next}\``).join(', ')], ['{count}', String(pairs.length)]],
    reported: null,
    bounded: false,
  };
}

/**
 * Mirrors `fn as_written`: the request unescapes `\n` / `\t` in its literals;
 * when the file holds no such text but holds the needle as written (source
 * code quoting `'\n'`), the literals are the text as written (PR #1188 G65).
 */
function asWritten(source, rewrite) {
  const { verbatim } = rewrite;
  if (!verbatim || (rewrite.pattern !== verbatim.pattern && source.includes(rewrite.pattern))
    || !source.includes(verbatim.pattern)) return rewrite;
  return { ...rewrite, pattern: verbatim.pattern, replacement: verbatim.replacement, verbatim: null };
}

function planCompositeStep(task, currentTurn, toolNames, change) {
  if (resultForPath(currentTurn, Capability.Write, change.source_path, change.source) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(change.source_path, change.source));
  }
  const sourceCommand = `cat ${change.source_path}`;
  const observedSource = resultForCommand(currentTurn, sourceCommand);
  if (observedSource === null) return planWithTool(toolNames, Capability.Run, jsonText({ command: sourceCommand }));
  if (observedSource !== change.source) return failed(task, change.source_path);
  const read = resultForPath(currentTurn, Capability.Read, change.registration_path, null);
  if (read === null) return planWithTool(toolNames, Capability.Read, readArguments(change.registration_path));
  const current = sourceFromReadResult(read);
  const updated = insertRegistration(current, change.registration);
  const registered = `\`${trim(change.registration)}\``;
  const members = [['{members}', registered]];
  if (updated === current) {
    return finalOrNull(renderSeededChange('coding_member_already_present', task, change.registration_path, members));
  }
  const compact = compactRegistrationEdit(current, change.registration);
  const editTool = compact ? toolFor(toolNames, Capability.Edit) : null;
  if (compact && editTool) {
    const [old, next] = compact;
    if (resultForEdit(currentTurn, change.registration_path, old, next) === null) {
      return planOne(editTool, editArguments(change.registration_path, old, next));
    }
    return planDigestVerification(task, currentTurn, toolNames, {
      target: change.registration_path, expected: updated, intent: 'coding_member_inserted', slots: members,
    });
  }
  if (resultForPath(currentTurn, Capability.Write, change.registration_path, updated) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(change.registration_path, updated));
  }
  const registrationCommand = `cat ${change.registration_path}`;
  const observed = resultForCommand(currentTurn, registrationCommand);
  if (observed === null) return planWithTool(toolNames, Capability.Run, jsonText({ command: registrationCommand }));
  if (observed !== updated) return failed(task, change.registration_path);
  return finalOrNull(renderSeededChange('coding_member_inserted', task, change.registration_path, members));
}

/**
 * The file after `rewrite`, or null when it cannot apply. A positional
 * insertion replaces its one anchor occurrence; an anchor found zero or
 * several times does not say where the line goes.
 */
function rewrittenSource(source, rewrite) {
  if (rewrite.singlePass) return source.includes(rewrite.pattern) ? source.split(rewrite.pattern).join(rewrite.replacement) : null;
  if (rewrite.unique) {
    const lines = anchoredLines(source, rewrite);
    return lines && `${source.slice(0, lines[0])}${lines[2]}${source.slice(lines[0] + lines[1].length)}`;
  }
  const execution = executeScopedWorkspaceRewrite(source, rewrite.pattern, rewrite.replacement, rewrite.scope);
  return execution.ok ? execution.ok.output : null;
}

/**
 * A positional insertion's one anchor occurrence widened to the whole lines
 * it sits on, as `[start, oldLines, newLines]`, or null unless the anchor
 * occurs exactly once. The inserted text is a line, so an anchor that is only
 * part of a line (`after the line containing '| T20 |'`) must not split it.
 */
function anchoredLines(source, rewrite) {
  const at = matchIndices(source, rewrite.pattern);
  if (at.length !== 1) return null;
  const { pattern, replacement } = rewrite;
  const start = at[0] === 0 ? 0 : source.lastIndexOf('\n', at[0] - 1) + 1;
  const tail = at[0] + pattern.length;
  const lineEnd = pattern.endsWith('\n') ? tail - 1 : source.indexOf('\n', tail);
  const end = lineEnd < 0 ? source.length : lineEnd;
  const old = source.slice(start, end);
  const next = replacement.startsWith(pattern)
    ? `${old}${replacement.slice(pattern.length)}`
    : `${replacement.slice(0, replacement.length - pattern.length)}${old}`;
  return [start, old, next];
}

/**
 * The smallest run of whole lines holding every change from `source` to
 * `updated`, as an edit pair, when that run occurs once in `source` — the
 * edit tool refuses an `oldString` it finds twice, and a bare word can sit
 * inside a longer one (`smal` in `small`).
 */
export function changedLinesEdit(source, updated) {
  const MAX_EDIT_BYTES = 4096;
  let prefix = 0;
  while (prefix < source.length && prefix < updated.length && source[prefix] === updated[prefix]) prefix += 1;
  let suffix = 0;
  while (suffix < source.length - prefix && suffix < updated.length - prefix
    && source[source.length - 1 - suffix] === updated[updated.length - 1 - suffix]) suffix += 1;
  let start = source.lastIndexOf('\n', prefix - 1) + 1;
  const lineEnd = source.indexOf('\n', source.length - suffix);
  const end = lineEnd < 0 ? source.length : lineEnd;
  // Lines inserted between two lines change none: the line above carries them.
  if (start === end && start > 0) start = start < 2 ? 0 : source.lastIndexOf('\n', start - 2) + 1;
  const old = source.slice(start, end);
  const next = updated.slice(start, updated.length - (source.length - end));
  if (old === '' || new TextEncoder().encode(old).length > MAX_EDIT_BYTES) return null;
  return matchIndices(source, old).length === 1 ? [old, next] : null;
}

/**
 * `Insert 'x' after the line 'y' in f`: the anchor line, replaced once by the
 * anchor beside the new line, stated by the position meaning that placed it.
 */
function groundedPositionalInsert(task) {
  const insert = composePositionalInsert(task);
  if (!insert) return null;
  const [target, anchor, replacement] = insert;
  const after = replacement.startsWith(`${anchor}\n`);
  const inserted = after ? replacement.slice(anchor.length + 1) : replacement.slice(0, -(anchor.length + 1));
  return {
    target, pattern: anchor, replacement, scope: RewriteScope.Substring, unique: true,
    intent: insertIntent(after, inserted),
    slots: [['{new}', inserted], ['{anchor}', anchor]],
  };
}

/** Mirrors `fn insert_intent`: the sentence stating an insert; an empty line is stated in its own words. */
const insertIntent = (after, inserted) => {
  if (inserted === '') return 'file_edit_blank_line';
  return after ? 'file_edit_position_after' : 'file_edit_position_before';
};

/**
 * Mirrors `fn insert_sequence`: every insert the request asks for, each its
 * own anchored edit -- several clauses (`…, and insert …`), an anchor named
 * by the line it follows (T31, T34), lines rebased on the anchor (G16), or
 * one insert whose anchor is a whole line also found inside others.
 */
function insertSequence(task) {
  return positionalInserts(task);
}

/**
 * Mirrors `fn lone_line_occurrences`: where `text` occurs in `source` as a
 * whole line but for its indentation -- `the line 'b'` is the line `b`, not
 * the `b` inside `table a`.
 */
function loneLineOccurrences(source, text) {
  return matchIndices(source, text).filter((index) => {
    const lineStart = index === 0 ? 0 : source.lastIndexOf('\n', index - 1) + 1;
    const newline = source.indexOf('\n', index + text.length);
    const rest = source.slice(index + text.length, newline < 0 ? source.length : newline);
    return trim(source.slice(lineStart, index)) === '' && (text.endsWith('\n') || trim(rest) === '');
  });
}

/**
 * Mirrors `fn insert_edit`: `[start, old, new]` -- the whole lines from the
 * anchor context (or the anchor) through the anchor, and those lines with the
 * inserted text beside the anchor's line -- or null unless the context (with
 * none, the anchor) occurs once and the anchor occurs after it.
 */
/**
 * Mirrors `fn resolved_spans`: an insert whose anchor and context are unquoted
 * word spans, each resolved to the longest line of `source` it ends with
 * (`the line table u` ends with the line `table u`, PR #1188 G51), or null
 * when a span names no line.
 */
function resolvedSpans(source, insert) {
  if (!insert.spans) return writtenEscapes(source, insert);
  const lines = [...new Set(source.split('\n').map((line) => trim(line)).filter((line) => line !== ''))];
  const resolve = (span) => lines.filter((line) => span === line || span.endsWith(` ${line}`))
    .reduce((best, line) => (best === null || line.length > best.length ? line : best), null);
  const anchor = resolve(insert.anchor);
  const context = insert.context === null ? null : resolve(insert.context);
  if (anchor === null || (insert.context !== null && context === null)) return null;
  return { ...insert, anchor, context, spans: false };
}

/**
 * Mirrors `fn written_escapes`: the insert with an anchor or context the file
 * holds only as written. A quoted `\n` or `\t` reads as a line break or a tab
 * where the file holds it so, and as its two characters where the file holds
 * those inside a line of code (`split_inclusive('\n')`, PR #1188 G96).
 */
function writtenEscapes(source, insert) {
  const written = (text) => {
    if (text === null || source.includes(text)) return null;
    const raw = text.replaceAll('\n', '\\n').replaceAll('\t', '\\t');
    return raw !== text && source.includes(raw) ? raw : null;
  };
  const anchor = written(insert.anchor);
  const context = written(insert.context);
  if (anchor === null && context === null) return insert;
  return { ...insert, anchor: anchor ?? insert.anchor, context: context ?? insert.context };
}

/**
 * Mirrors `fn repeated_anchor_count`: how many places a repeated anchor could
 * mean, or 0 when it names one place (PR #1188 G72).
 */
function repeatedAnchorCount(source, insert) {
  const lead = insert.context ?? insert.anchor;
  if (lead === '') return 0;
  const lone = loneLineOccurrences(source, lead).length;
  const count = lone === 0 ? matchIndices(source, lead).length : lone;
  return count > 1 ? count : 0;
}

function insertEdit(source, given) {
  const insert = resolvedSpans(source, given);
  if (insert === null) return null;
  const lead = insert.context ?? insert.anchor;
  let leads = matchIndices(source, lead);
  // A lead found inside other lines too names the one line it is by itself.
  if (leads.length > 1) leads = loneLineOccurrences(source, lead);
  if (leads.length !== 1 || insert.anchor === '') return null;
  // After a context, a line that is exactly the anchor outranks one that only
  // contains it (`text set` is not `text setting`).
  let at = leads[0];
  if (insert.context !== null) {
    const whole = (index) => (index === 0 || source[index - 1] === '\n')
      && (insert.anchor.endsWith('\n') || index + insert.anchor.length === source.length
        || source[index + insert.anchor.length] === '\n');
    const found = [];
    for (let index = source.indexOf(insert.anchor, leads[0] + lead.length); index >= 0;
      index = source.indexOf(insert.anchor, index + insert.anchor.length)) found.push(index);
    at = found.find(whole) ?? found[0] ?? -1;
  }
  if (at < 0) return null;
  const lineStart = (index) => (index === 0 ? 0 : source.lastIndexOf('\n', index - 1) + 1);
  const start = lineStart(leads[0]);
  const tail = at + insert.anchor.length;
  const newline = source.indexOf('\n', tail);
  const end = insert.anchor.endsWith('\n') ? tail - 1 : newline < 0 ? source.length : newline;
  const old = source.slice(start, end);
  const anchorLine = lineStart(at);
  const inserted = insertedText(source, anchorLine, insert);
  const next = insert.after
    ? `${old}\n${inserted}`
    : `${source.slice(start, anchorLine)}${inserted}\n${source.slice(anchorLine, end)}`;
  // The edit tool needs its old text once in the file: a line that also ends
  // another line (`];`) is widened back over the lines before it (PR #1188 G73).
  const from = uniqueFrom(source, start, end);
  if (from === null) return null;
  return [from, source.slice(from, end), `${source.slice(from, start)}${next}`, inserted];
}

/** Mirrors `fn blank_beside`: the lines without the one empty line an insert adds beside them. */
function blankBeside(inserted) {
  if (inserted.length > 1 && inserted.startsWith('\n')) return inserted.slice(1);
  if (inserted.length > 1 && inserted.endsWith('\n')) return inserted.slice(0, -1);
  return inserted;
}

/**
 * Mirrors `fn unique_from`: the start of the shortest run of whole lines
 * ending at `end` whose text occurs once in `source`, or null.
 */
function uniqueFrom(source, start, end) {
  let from = start;
  while (end > from && matchIndices(source, source.slice(from, end)).length > 1) {
    if (from === 0) return null;
    from = source.lastIndexOf('\n', from - 2) + 1;
  }
  return from;
}

/**
 * Mirrors `fn inserted_text`: the lines an insert puts beside the anchor's
 * line. An unfenced block keeps the indentation it was given when the file
 * already indents lines so (G93, as `appendedBlock` does); otherwise it is
 * rebased on the anchor line's indentation, so it lands as its siblings
 * (PR #1188 G16).
 */
function insertedText(source, anchorLine, insert) {
  if (insert.rebase === null || insert.rebase === undefined) return insert.inserted;
  if (indentsLinesAt(source, insert.rebase)) return rebasedBlock(insert.inserted, insert.rebase);
  const newline = source.indexOf('\n', anchorLine);
  return rebasedBlock(insert.inserted, leadingIndentation(source.slice(anchorLine, newline < 0 ? source.length : newline)));
}

/**
 * Mirrors `fn plan_insert_sequence_step`: read each file once, make each
 * insert as its own anchored edit over the file as the earlier inserts left
 * it, then check every file's digest and state every insert.
 */
function planInsertSequenceStep(task, currentTurn, toolNames, inserts) {
  const editTool = toolFor(toolNames, Capability.Edit);
  if (!editTool) return null;
  const expected = new Map();
  const stated = [];
  for (const insert of inserts) {
    if (!expected.has(insert.target)) {
      const read = resultForPath(currentTurn, Capability.Read, insert.target, null);
      if (read === null) return planWithTool(toolNames, Capability.Read, readArguments(insert.target));
      expected.set(insert.target, sourceFromReadResult(read));
    }
    const source = expected.get(insert.target);
    const edit = insertEdit(source, insert);
    // An anchor line found more than once names no one place: the answer says
    // so instead of reporting a failed verification (PR #1188 G72).
    const occurrences = edit === null ? repeatedAnchorCount(source, insert) : 0;
    if (occurrences > 0) {
      return finalOrNull(renderSeededChange('file_edit_anchor_repeated', task, insert.target, [
        ['{anchor}', trimEndMatches(insert.context ?? insert.anchor, (character) => character === '\n')],
        ['{count}', String(occurrences)],
      ]));
    }
    if (edit === null) return failed(task, insert.target);
    const [start, old, next, inserted] = edit;
    if (resultForEdit(currentTurn, insert.target, old, next) === null) {
      return planOne(editTool, editArguments(insert.target, old, next));
    }
    expected.set(insert.target, `${source.slice(0, start)}${next}${source.slice(start + old.length)}`);
    // An empty line beside the lines is stated in its own words (PR #1188 G74).
    const shown = blankBeside(inserted);
    stated.push(renderSeededChange(insertIntent(insert.after, shown), task,
      insert.target, [['{new}', shown], ['{anchor}', insert.anchor]]));
    if (shown !== inserted) stated.push(renderSeededChange('file_edit_blank_line', task, insert.target, []));
  }
  for (const [target, content] of expected) {
    const command = `sha256sum -- ${target}`;
    const observed = resultForCommand(currentTurn, command);
    if (observed === null) return planWithTool(toolNames, Capability.Run, jsonText({ command }));
    if (splitWhitespace(observed)[0] !== sha256Hex(content)) return failed(task, target);
  }
  return stated.includes(null) ? null : resolvedFinalAnswer(stated.join('\n'), FinalDisposition.Finding, 'workspace_insertions_verified');
}

function groundedRewrite(task) {
  const positional = groundedPositionalInsert(task);
  if (positional) return positional;
  const edit = composeEditRequest(task);
  if (!edit) return null;
  const [target, oldClause, newClause] = edit;
  const renaming = mentionsRole('coding_identifier_rename_action', task.toLowerCase());
  // A request that names a line replaces whole lines (`groundedLineReplacement`).
  if (!renaming && namesLine(task)) return null;
  // The edit request unescapes `\n` / `\t` in its literals, so the quoted
  // segments are compared unescaped too: a multi-line replacement written with
  // `\n` was otherwise not "quoted", and the general fallback wrote its new
  // text over the whole file (PR #1188 sub-agent gap 3).
  const quoted = quotedSegments(task).map(unescapeProseNewlines);
  let old;
  let next;
  let verbatim = null;
  // A literal quoted whole counts even when backtick spans inside it pair
  // among themselves (PR #1188 G63).
  const isLiteral = (clause) => quoted.includes(clause) || quotesWhole(task, clause);
  if (isLiteral(oldClause) && isLiteral(newClause)) {
    [old, next] = [oldClause, newClause];
    const asQuoted = (text) => quotedSegments(task).find((raw) => raw !== text && unescapeProseNewlines(raw) === text) ?? text;
    if (asQuoted(old) !== old || asQuoted(next) !== next) verbatim = { pattern: asQuoted(old), replacement: asQuoted(next) };
  } else if (renaming) {
    old = identifierTokens(oldClause).pop();
    next = identifierTokens(newClause)[0];
    if (old === undefined || next === undefined) return null;
  } else return null;
  // A word replaced by a longer word that contains it (`smal` -> `small`) is
  // only safe word by word: a substring rewrite would never terminate.
  const words = isIdentifierWord(old) && isIdentifierWord(next);
  const scope = words && (renaming || next.includes(old)) ? RewriteScope.Word : RewriteScope.Substring;
  if (old === '' || old === next) return null;
  // A new text holding the old one (`key 3` -> `key 33`) is replaced in one
  // pass over the file, never rewritten again inside its own result (PR #1188
  // G78).
  const singlePass = scope === RewriteScope.Substring && next.includes(old);
  return { target, pattern: old, replacement: next, scope, renaming: renaming && scope === RewriteScope.Word, verbatim, singlePass };
}

/**
 * The one workspace path a request names and its one quoted segment that is
 * not that path: `{target, text}` or null. An unquoted path outranks a quoted
 * one, which is then the payload (PR #1188 T35).
 */
function quotedPayloadAndPath(task) {
  const named = namedTargetAndPayloads(task);
  return named && named.payloads.length === 1 ? { target: named.target, text: named.payloads[0] } : null;
}

/**
 * Mirrors the section arm of `fn grounded_end_insertion`: the text placed at
 * one end of a Markdown section, stated as inserted after the line it now
 * follows.
 */
function groundedSectionInsertion(section, atEnd) {
  const text = unescapeProseNewlines(section.text);
  const placed = (source) => insertedInSection(source, section.heading, text, atEnd);
  return {
    target: section.target,
    compute: (source, missing) => (missing ? null : placed(source)?.updated ?? null),
    edit: changedLinesEdit,
    intent: 'file_edit_position_after',
    slots: [['{new}', text], ['{anchor}', section.heading]],
    reported: (source) => {
      const result = placed(source);
      return result === null ? null : { intent: 'file_edit_position_after', slots: [['{new}', text], ['{anchor}', result.anchor]] };
    },
  };
}

/**
 * Mirrors `fn quoted_lines_and_path`: several quoted literals, joined only by
 * seeded joiners and commas, are the lines added, each its own line, as
 * written (PR #1188 G53, G54).
 */
function quotedLinesAndPath(task) {
  const named = namedTargetAndPayloads(task);
  if (!named || named.payloads.length < 2) return null;
  const text = joinedLiteralLines(task, named.target);
  return text === null ? null : { target: named.target, text, lines: true };
}

/**
 * An additive edit at one end of a named file (`append 'x' to notes.txt`,
 * `prepend "x" to a.md`), positioned by the seeded `file_edit_position_end` /
 * `file_edit_position_start` meanings. A missing file is created, as `>>`
 * would.
 */
function groundedEndInsertion(task) {
  // `Append these lines to f:` followed by lines: the first line is the
  // request and the lines under it are the text added, verbatim (PR #1188).
  const block = introducedBlock(task);
  // Position cues are read outside the quoted payload: a line saying "at the
  // start of x" is text being appended (PR #1188 G64).
  const lowered = block === null ? outsideQuotes(task) : sentenceWords(block.head);
  const atEnd = mentionsRole('file_edit_position_end', lowered);
  if (atEnd === mentionsRole('file_edit_position_start', lowered)) return null;
  // `… at the end of the section '## Usage' in README.md` (PR #1188 G35).
  const section = block === null ? sectionScope(task, outsideQuotes(task)) : null;
  if (section) return groundedSectionInsertion(section, atEnd);
  const named = block === null
    ? quotedPayloadAndPath(task) ?? quotedLinesAndPath(task) ?? blankLineAndPath(task) ?? linePayloadAndPath(task)
    : blockPayloadAndPath(block);
  if (!named) return null;
  const text = block === null && !named.lines ? unescapeProseNewlines(named.text) : named.text;
  return {
    target: named.target,
    compute: (source) => insertedAtEnd(source, block === null ? text : appendedBlock(source, block), atEnd),
    edit: (source, updated) => (source === '' ? null : compactEndEdit(source, updated, atEnd)),
    // An empty line has no text to quote back; it is stated in its own words.
    intent: named.text === '' ? 'file_edit_blank_line' : atEnd ? 'file_edit_position_end' : 'file_edit_position_start',
    slots: [['{new}', named.text]],
  };
}

/**
 * `Delete the line 'x' from notes.txt`, `remove "TODO" from a.md`: the seeded
 * `coding_text_remove_action` with one quoted payload and one path. Every
 * line that is exactly the payload goes; otherwise its one occurrence inside
 * a line does. A missing file, or a payload found nowhere or more than once
 * inside lines, is not a removal anyone can verify.
 *
 * A request that names a line outside its quotes (the seeded `line` meaning:
 * `the line containing '…'`, `lines`, `строку`, `पंक्ति`, `的行`) removes whole
 * lines, never only the payload inside one: the lines equal to the payload,
 * else its one containing line, or with the seeded `line_containment_cue`
 * every line containing it (PR #1188 G60).
 */
function groundedRemoval(task) {
  const lowered = sentenceWords(task);
  if (!mentionsRole('coding_text_remove_action', lowered) || mentionsRole('coding_member_add_action', lowered)) return null;
  const named = quotedPayloadAndPath(task);
  if (!named) return null;
  const byLine = namesLine(task);
  const containing = byLine && mentionsRole('line_containment_cue', outsideQuotes(task));
  return {
    target: named.target,
    compute: (source, missing) => {
      if (missing) return null;
      return byLine ? removedLines(source, named.text, containing) : removedLiteral(source, named.text);
    },
    edit: changedLinesEdit,
    intent: 'coding_text_remove',
    slots: [['{old}', named.text]],
    reported: byLine ? (source) => removedLinesReport(source, named.text, containing) : null,
  };
}

/**
 * `Delete the functions a and b from util.js`: the seeded
 * `coding_text_remove_action` and `coding_declaration_noun` with no add
 * action, no quoted text and one named path. The names are the request's
 * identifiers that the file itself declares as functions (a seeded
 * `function_declaration_keyword` right before the name); each goes whole,
 * with the comments and attributes directly above it. A name the file does
 * not declare is not removed; a request none of whose names is declared is
 * not a removal anyone can verify (PR #1188 dogfooding).
 */
function groundedDeclarationRemoval(task) {
  const lowered = sentenceWords(task);
  if (!mentionsRole('coding_text_remove_action', lowered) || mentionsRole('coding_member_add_action', lowered)) return null;
  if (!mentionsRole('coding_declaration_noun', lowered) || quotedSegmentSpans(task).length > 0) return null;
  const paths = [...new Set(unquotedPathTokens(task).map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path) && safeRelativePath(path)))];
  if (paths.length !== 1) return null;
  const names = [...new Set(tokens(task).map((token) => token.text)
    .filter((text) => cleanPathToken(text) !== paths[0]).flatMap(identifierTokens))];
  if (names.length === 0) return null;
  return {
    target: paths[0],
    compute: (source, missing) => (missing ? null : removedDeclarations(source, names)?.source ?? null),
    edit: changedLinesEdit,
    intent: 'coding_text_remove',
    slots: [['{old}', names.join('`, `')]],
    // The request names each declaration it removes: its extent is stated.
    bounded: true,
    reported: (source) => {
      const removed = removedDeclarations(source, names);
      return removed ? { intent: 'coding_text_remove', slots: [['{old}', removed.names]] } : null;
    },
  };
}

/**
 * Mirrors `fn removed_declarations`: `source` without the function each of
 * `names` declares, and the names that were declared; null when none is.
 */
function removedDeclarations(source, names) {
  const sourceLines = source.split(/(?<=\n)/u);
  const keywords = wordsForRole('function_declaration_keyword');
  const removed = new Set();
  const declared = [];
  for (const name of names) {
    const headers = sourceLines.map((line, index) => [index, line]).filter(([, line]) => declaresFunction(line, name, keywords));
    if (headers.length !== 1) continue;
    const span = declarationSpan(sourceLines, headers[0][0]);
    if (span === null) continue;
    for (let index = span[0]; index <= span[1]; index += 1) removed.add(index);
    declared.push(name);
  }
  if (declared.length === 0) return null;
  const blank = (index) => index < 0 || (index < sourceLines.length && trim(sourceLines[index]) === '');
  for (const index of [...removed].sort((left, right) => left - right)) {
    // A block left between two blank lines takes one of them with it; a
    // block that ended the file takes the blank line before it.
    if (removed.has(index + 1)) continue;
    let start = index;
    while (removed.has(start - 1)) start -= 1;
    if (index + 1 === sourceLines.length) {
      if (start > 0 && blank(start - 1)) removed.add(start - 1);
    } else if (blank(index + 1) && blank(start - 1)) {
      removed.add(index + 1);
    }
  }
  return { source: sourceLines.filter((_, index) => !removed.has(index)).join(''), names: declared };
}

/** Mirrors `fn declares_function`: a seeded keyword, then `name`, then `(` or `<`. */
function declaresFunction(line, name, keywords) {
  const words = line.split(/(?=[^A-Za-z0-9_$])|(?<=[^A-Za-z0-9_$])/u);
  for (let index = 0; index < words.length; index += 1) {
    if (!keywords.includes(words[index])) continue;
    let next = index + 1;
    while (next < words.length && trim(words[next]) === '') next += 1;
    if (words[next] !== name) continue;
    next += 1;
    while (next < words.length && trim(words[next]) === '') next += 1;
    if (words[next] === '(' || words[next] === '<') return true;
  }
  return false;
}

/**
 * Mirrors `fn declaration_span`: the first and last line of the declaration
 * whose header is line `header`, with the comment and attribute lines directly
 * above it. A braced body ends at the first later line that closes at the
 * header's indentation; an indented body (a header ending in `:`) at the last
 * line indented deeper.
 */
function declarationSpan(sourceLines, header) {
  const indent = (line) => line.length - line.replace(/^[ \t]+/u, '').length;
  const headerLine = sourceLines[header].replace(/\r?\n$/u, '');
  const depth = indent(headerLine);
  let end = null;
  if (/[};]\s*$/u.test(headerLine)) end = header;
  else if (/:\s*$/u.test(headerLine)) {
    end = header;
    for (let index = header + 1; index < sourceLines.length; index += 1) {
      if (trim(sourceLines[index]) === '') continue;
      if (indent(sourceLines[index]) <= depth) break;
      end = index;
    }
  } else {
    for (let index = header + 1; index < sourceLines.length && end === null; index += 1) {
      if (indent(sourceLines[index]) === depth && trim(sourceLines[index]).startsWith('}')) end = index;
    }
  }
  if (end === null) return null;
  let start = header;
  while (start > 0) {
    const above = trim(sourceLines[start - 1]);
    if (above.endsWith('*/')) {
      let open = start - 1;
      while (open > 0 && !trim(sourceLines[open]).startsWith('/*')) open -= 1;
      if (!trim(sourceLines[open]).startsWith('/*')) break;
      start = open;
    } else if (['//', '#[', '@'].some((lead) => above.startsWith(lead))) {
      start -= 1;
    } else break;
  }
  return [start, end];
}

/**
 * Mirrors `fn sentence_words`: the lowered request with the marks that end a
 * sentence or clause set off as spaces, so a verb that closes its sentence
 * (`… पंक्ति हटाओ।`, `… удали.`) is still a whole word. A dot inside a token
 * (`t.md`) is not followed by a space and stays.
 */
export function sentenceWords(task) {
  return task.toLowerCase().replace(/[.!?;:,\u0964\u0965\u3002\uff01\uff1f]+(?=\s|$)/gu, ' ');
}

/** The lowered request outside its quotes (each quote set off as a space). */
function outsideQuotes(task) {
  let outside = '';
  let cursor = 0;
  for (const segment of quotedSegmentSpans(task)) {
    if (segment.start < cursor) continue;
    outside += `${task.slice(cursor, segment.start)} `;
    cursor = segment.end;
  }
  return normalizePrompt(outside + task.slice(cursor)).toLowerCase();
}

/** Mirrors `fn names_line`: the seeded `line` meaning, outside the request's quotes. */
function namesLine(task) {
  // A path's own words are no line: `x-line.lino` names a file (PR #1188 G83).
  return meaningEvidencedIn('line', outsideQuotes(withoutPathWords(task)));
}

/**
 * Mirrors `fn removed_line_indices`: the lines a line removal takes. With the
 * seeded `line_containment_cue` (`the lines containing 'x'`) every line that
 * contains the payload; otherwise the lines that are the payload but for
 * their indentation, else the one line containing it (PR #1188 G60:
 * `Delete the line 'x'` removed every line holding the letter x).
 */
function removedLineIndices(source, text, containing) {
  const bare = source.split(/(?<=\n)/u).map((line) => line.replace(/\r?\n$/u, ''));
  const holding = bare.flatMap((line, index) => (line.includes(text) ? [index] : []));
  if (containing) return holding;
  const equal = holding.filter((index) => trim(bare[index]) === trim(text));
  if (equal.length > 0) return equal;
  return holding.length === 1 ? holding : [];
}

/** Mirrors `fn removed_lines`: `source` without the lines [`removedLineIndices`] takes. */
function removedLines(source, text, containing) {
  const taken = new Set(removedLineIndices(source, text, containing));
  if (taken.size === 0) return null;
  return source.split(/(?<=\n)/u).filter((_, index) => !taken.has(index)).join('');
}

/**
 * Mirrors `fn removed_lines_report`: lines that were exactly the payload are
 * reported as the payload's removal; lines that only contained it are
 * reported as lines, with their count.
 */
function removedLinesReport(source, text, containing) {
  const bare = source.split(/(?<=\n)/u).map((line) => line.replace(/\r?\n$/u, ''));
  const removed = removedLineIndices(source, text, containing).map((index) => bare[index]);
  if (removed.every((line) => line === text)) return null;
  return { intent: 'line', slots: [['{old}', text], ['{count}', String(removed.length)]] };
}

/**
 * `Change the value of "debug" to true in config.json`: the seeded
 * `config_value_lead` names a setting, the edit request's old clause names
 * its key (quoted, or its last identifier) and the new clause its value. The
 * one line assigning that key (`"debug": …`, `debug: …`, `debug = …`,
 * `const DEBUG: bool = …;`) gets the new value; a key assigned nowhere or
 * more than once is not a change anyone can verify.
 *
 * `Change MAXIMUM_RATIO from 8 to 16 in p.rs` states the value the key holds
 * now (the seeded `file_edit_old_lead_cue`): the line assigning the key that
 * value is the one changed, and a key assigned nowhere leaves the stated
 * value itself, when it occurs once as a word (PR #1188 G2).
 */
function groundedSetting(task) {
  const edit = composeEditRequest(task);
  if (!edit) return null;
  const [target, oldClause, value] = edit;
  const stated = statedOldValue(oldClause);
  if (stated === null && !mentionsRole('config_value_lead', sentenceWords(task))) return null;
  // `set the contents of note.txt to hello` writes the file; its contents are
  // not a key (PR #1188 G68, issue #745).
  if (stated === null && mentionsRole('file_contents_source_cue', sentenceWords(task))) return null;
  const keyClause = stated === null ? oldClause : stated.key;
  const quoted = quotedSegments(keyClause);
  const key = quoted.length === 1 ? quoted[0] : identifierTokens(keyClause).pop();
  if (key === undefined || trim(value) === '') return null;
  const held = stated === null ? null : stated.old;
  return {
    target,
    compute: (source, missing) => (missing ? null : assignedSetting(source, key, trim(value), target, held)),
    edit: changedLinesEdit,
    intent: 'setting',
    slots: [['{old}', key], ['{new}', trim(value)]],
  };
}

/**
 * `Replace the line 'x' with 'y' in f`: the seeded `line` meaning outside the
 * quotes, both clauses quoted. A line replacement never touches text inside a
 * longer line (T32: `text pays` became `text "pay"s`). With a seeded anchor
 * context (`the line 'x' that follows the line 'z'`), the first such line
 * after `z` is the one replaced (T62).
 */
function groundedLineReplacement(task) {
  if (!namesLine(task) || mentionsRole('coding_identifier_rename_action', task.toLowerCase())) return null;
  const context = anchorContext(task);
  const request = context === null ? task : `${task.slice(0, context.start)} ${task.slice(context.end)}`;
  const edit = composeEditRequest(request);
  if (!edit || composePositionalInsert(task) !== null) return null;
  const [target, old, next] = edit;
  const quoted = quotedSegments(request).map(unescapeProseNewlines);
  // Lines under the request (G22) replace the line as its siblings, unless
  // they were fenced or quoted.
  const block = introducedBlock(task);
  const lines = block !== null && block.text === next;
  // Listed lines (`the three lines 'a', 'b' and 'c'`) are quoted one by one (G80).
  const isQuoted = (text) => quoted.includes(text) || text.split('\n').every((line) => quoted.includes(line));
  if (!isQuoted(old) || (!lines && !quoted.includes(next)) || old === '' || old === next) return null;
  const after = context === null ? null : context.text;
  const replacement = (source) => (lines && !block.verbatim ? rebasedBlock(next, lineIndentation(source, old)) : next);
  return {
    target,
    compute: (source, missing) => (missing ? null : replacedLines(source, old, replacement(source), after)),
    edit: (source, updated) => contextLinesEdit(source, updated, after),
    intent: 'coding_text_replaced',
    // Listed lines are named one by one, not as one span holding line breaks.
    slots: [['{old}', quoted.includes(old) ? old : old.split('\n')], ['{new}', next]],
  };
}

/**
 * Mirrors `fn line_indentation`: the indentation of the first line that is
 * `text` but for its indentation, or none.
 */
function lineIndentation(source, text) {
  const line = source.split('\n').find((candidate) => trim(candidate) === trim(text));
  return line === undefined ? '' : leadingIndentation(line);
}

/**
 * Mirrors `fn context_lines_edit`: the smallest unique changed run of lines,
 * or, when that run repeats, the lines from the context's line through the
 * change -- the context occurs once, so they do not.
 */
function contextLinesEdit(source, updated, context) {
  const compact = changedLinesEdit(source, updated);
  if (compact !== null || context === null) return compact;
  const at = matchIndices(source, context);
  if (at.length !== 1) return null;
  const start = at[0] === 0 ? 0 : source.lastIndexOf('\n', at[0] - 1) + 1;
  let suffix = 0;
  while (suffix < source.length - start && suffix < updated.length - start
    && source[source.length - 1 - suffix] === updated[updated.length - 1 - suffix]) suffix += 1;
  const lineEnd = source.indexOf('\n', source.length - suffix);
  const end = lineEnd < 0 ? source.length : lineEnd;
  return [source.slice(start, end), updated.slice(start, updated.length - (source.length - end))];
}

/**
 * `Fix the typo 'smal' in README.md`: the seeded `typo_fix_lead` with one
 * quoted word and one path, and no stated correction. The correction is
 * discovered (`correctedSpelling`), and the change is the word-scoped
 * replacement a stated correction would have made.
 */
function groundedTypoFix(task) {
  if (!mentionsRole('typo_fix_lead', sentenceWords(task))) return null;
  const named = quotedPayloadAndPath(task);
  if (!named || !isIdentifierWord(named.text)) return null;
  const correction = correctedSpelling(named.text);
  if (correction === null) return null;
  return {
    target: named.target,
    compute: (source, missing) => {
      if (missing) return null;
      const execution = executeScopedWorkspaceRewrite(source, named.text, correction, RewriteScope.Word);
      return execution.ok ? execution.ok.output : null;
    },
    edit: changedLinesEdit,
    intent: 'coding_text_replaced',
    slots: [['{old}', named.text], ['{new}', correction]],
  };
}

function removedLiteral(source, text) {
  const kept = source.split(/(?<=\n)/u).filter((line) => line.replace(/\r?\n$/u, '') !== text);
  const withoutLines = kept.join('');
  if (withoutLines !== source) return withoutLines;
  return matchIndices(source, text).length === 1 ? source.replace(text, '') : null;
}

/** The file after the insertion; a file without a final newline keeps none. */
export function insertedAtEnd(source, text, atEnd) {
  if (source === '') return `${text}\n`;
  if (!atEnd) return `${text}\n${source}`;
  return source.endsWith('\n') ? `${source}${text}\n` : `${source}\n${text}`;
}

/**
 * The smallest unique run of whole lines at the insertion end, and what it
 * becomes, so the edit tool carries the change rather than the whole file.
 */
function compactEndEdit(source, updated, atEnd) {
  const MAX_ANCHOR_BYTES = 4096;
  const breaks = matchIndices(source, '\n');
  const cuts = atEnd
    ? [0, ...breaks.map((index) => index + 1).filter((start) => start < source.length)].reverse()
    : [...breaks.map((index) => index + 1), source.length];
  for (const cut of cuts) {
    const anchor = atEnd ? source.slice(cut) : source.slice(0, cut);
    if (trim(anchor) === '') continue;
    if (new TextEncoder().encode(anchor).length > MAX_ANCHOR_BYTES) break;
    if (matchIndices(source, anchor).length !== 1) continue;
    return [anchor, atEnd ? updated.slice(cut) : updated.slice(0, updated.length - (source.length - cut))];
  }
  return null;
}

/**
 * A change whose new bytes are computed from the file: read it, compute, send
 * the smallest unique edit (or the whole file), check the digest, and state
 * the change from the seeded response for its intent.
 */
export function planComputedChangeStep(task, currentTurn, toolNames, change) {
  const { target } = change;
  const read = resultForPath(currentTurn, Capability.Read, target, null);
  if (read === null) return planWithTool(toolNames, Capability.Read, readArguments(target));
  // A read that came back as the client's file block is the file, whatever
  // its text says: a ledger that quotes "Error:" lines is not a missing file.
  const missing = sourceFromAgentReadResult(read) === null && failureMessage(read, false, true) !== null;
  const source = missing ? '' : sourceFromReadResult(read);
  const scoped = scopedDecline(task, target, source);
  if (scoped !== null) return finalOrNull(scoped);
  const updated = change.compute(source, missing);
  if (updated === null || updated === source) {
    // Text the change names that the file no longer holds is the answer (G87).
    const slot = (name) => (change.slots ?? []).find(([key]) => key === name)?.[1] ?? null;
    const old = slot('{old}');
    const textual = change.intent === 'coding_text_replaced' || change.intent === 'coding_text_remove';
    const absent = missing || old === null || !textual ? null : absentTextAnswer(task, target, source, old, slot('{new}'));
    return absent === null ? failed(task, target) : finalAnswer(absent);
  }
  // A change that would drop most of the file is refused unless its request
  // states that extent (PR #1188 T29).
  if (!change.bounded && dropsMostOfFile(source, updated)) {
    return finalOrNull(renderSeededOutcome('coding_workspace_fragment_refused', task, target));
  }
  const { intent, slots } = change.reported?.(source) ?? change;
  const compact = change.edit(source, updated);
  const editTool = compact ? toolFor(toolNames, Capability.Edit) : null;
  if (compact && editTool) {
    const [old, next] = compact;
    if (resultForEdit(currentTurn, target, old, next) === null) return planOne(editTool, editArguments(target, old, next));
    return planDigestVerification(task, currentTurn, toolNames, { target, expected: updated, intent, slots });
  }
  if (resultForPath(currentTurn, Capability.Write, target, updated) === null) {
    return planWithTool(toolNames, Capability.Write, writeArguments(target, updated));
  }
  return planDigestVerification(task, currentTurn, toolNames, { target, expected: updated, intent, slots });
}

function compositeModuleChange(task) {
  if (!mentionsRole('coding_module_registration_action', task.toLowerCase())) return null;
  const generated = rustSourceForTask(task);
  if (!generated) return null;
  const registrationPath = rustPaths(task).find((path) => path !== generated.path);
  if (registrationPath === undefined) return null;
  const file = generated.path.split('/').pop();
  if (!file.endsWith('.rs')) return null;
  const module = file.slice(0, -3);
  if (!validIdentifier(module)) return null;
  const registration = renderRustTemplate('coding_source_module_registration', [['{module}', module]]);
  if (registration === null) return null;
  return { source_path: generated.path, source: generated.content, registration_path: registrationPath, registration };
}

function insertRegistration(source, registration) {
  if (lines(source).some((line) => trim(line) === trim(registration))) return source;
  let updated = source;
  if (updated !== '' && !updated.endsWith('\n')) updated += '\n';
  return updated + registration;
}

function compactRegistrationEdit(source, registration) {
  const MAX_SUFFIX_BYTES = 4096;
  const starts = [0, ...matchIndices(source, '\n').map((index) => index + 1).filter((start) => start < source.length)];
  for (const start of starts.reverse()) {
    const suffix = source.slice(start);
    if (new TextEncoder().encode(suffix).length > MAX_SUFFIX_BYTES) break;
    if (matchIndices(source, suffix).length !== 1) continue;
    const replacement = `${suffix.endsWith('\n') ? suffix : `${suffix}\n`}${registration}`;
    return [suffix, replacement];
  }
  return null;
}

function repeatedIdentifierRewriteCommand(rewrite) {
  if (!shellSafeIdentifier(rewrite.pattern) || !shellSafeIdentifier(rewrite.replacement)) return null;
  const pattern = rewrite.scope === RewriteScope.Substring ? rewrite.pattern : `\\b${rewrite.pattern}\\b`;
  return ['perl', '-pi', '-e', `'s/${pattern}/${rewrite.replacement}/g'`, '--', rewrite.target].join(' ');
}

function shellSafeIdentifier(identifier) {
  return /^[A-Za-z0-9_]+$/.test(identifier);
}

function identifierTokens(text) {
  return text.split(/[^A-Za-z0-9_]/u).filter(validIdentifier);
}

function validIdentifier(identifier) {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(identifier) && !wordsForRole('identifier_reserved_word').includes(identifier);
}

const isPathCharacter = (character) => isAsciiAlphanumeric(character) || '_-./'.includes(character);

function rustPaths(task) {
  const paths = [];
  for (const suffix of matchIndices(task, '.rs')) {
    const end = suffix + 3;
    let start = end;
    while (start > 0 && isPathCharacter(task[start - 1])) start -= 1;
    const path = task.slice(start, end);
    if (path !== '' && !path.startsWith('/') && !path.split('/').some((component) => component === '..') && !paths.includes(path)) {
      paths.push(path);
    }
  }
  return paths;
}

function parseObject(text) {
  try {
    const value = JSON.parse(text);
    return value !== null && typeof value === 'object' && !Array.isArray(value) ? value : null;
  } catch {
    return null;
  }
}

const stringField = (value, key) => (typeof value[key] === 'string' ? value[key] : null);

export function resultForPath(messages, capability, path, expectedContent) {
  return matchingResult(messages, (name, args) => {
    if (classifyTool(name) !== capability) return false;
    const value = parseObject(args);
    if (!value) return false;
    return argumentMatchesPath(value, path) && (expectedContent === null || stringField(value, 'content') === expectedContent);
  });
}

export function resultForEdit(messages, path, old, next) {
  return matchingResult(messages, (name, args) => {
    if (classifyTool(name) !== Capability.Edit) return false;
    const value = parseObject(args);
    return value !== null && argumentMatchesPath(value, path)
      && ['oldString', 'old_string', 'old_str', 'old'].some((key) => stringField(value, key) === old)
      && ['newString', 'new_string', 'new_str', 'new'].some((key) => stringField(value, key) === next);
  });
}

function argumentMatchesPath(value, expected) {
  return ['path', 'filePath', 'file_path'].some((key) => {
    const observed = stringField(value, key);
    return observed !== null && workspacePathMatches(expected, observed);
  });
}

function components(path) {
  return path.split('/').filter((part, index) => part !== '' && (part !== '.' || index === 0));
}

function workspacePathMatches(expected, observed) {
  if (observed === expected) return true;
  if (expected.startsWith('/') || !observed.startsWith('/')) return false;
  const want = components(expected);
  const have = components(observed);
  return want.length <= have.length && want.every((part, index) => have[have.length - want.length + index] === part);
}

export function resultForCommand(messages, command) {
  return matchingResult(messages, (name, args) => classifyTool(name) === Capability.Run && commandArgument(args) === command);
}

function planDigestVerification(task, currentTurn, toolNames, change) {
  const command = `sha256sum -- ${change.target}`;
  const observed = resultForCommand(currentTurn, command);
  if (observed === null) return planWithTool(toolNames, Capability.Run, jsonText({ command }));
  if (splitWhitespace(observed)[0] !== sha256Hex(change.expected)) return failed(task, change.target);
  return finalOrNull(renderSeededChange(change.intent, task, change.target, change.slots),
    FinalDisposition.Finding, change.intent);
}

function matchingResult(messages, matches) {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    const message = messages[index];
    if (!eqIgnoreAsciiCase(message.role, 'tool')) continue;
    const id = message.tool_call_id;
    if (id === null || id === undefined) continue;
    let call = null;
    for (let prior = index - 1; prior >= 0 && call === null; prior -= 1) {
      const calls = messages[prior].tool_calls || [];
      for (let at = calls.length - 1; at >= 0; at -= 1) {
        if (calls[at].id === id) {
          call = calls[at];
          break;
        }
      }
    }
    if (call === null) continue;
    if (matches(call.function.name, call.function.arguments)) return plainText(message.content);
  }
  return null;
}

export function readArguments(path) {
  return jsonText({ path, filePath: path, file_path: path });
}

/**
 * `Append the line third to notes.txt`: no quoted text, one path, and the
 * seeded `file_edit_line_lead`; the line is the words after the lead up to
 * the last destination cue before the path (`go to bed to notes.txt` keeps
 * `go to bed`). A request without a lead is declined, never guessed.
 */

/**
 * `Append an empty line to notes.txt`: the seeded `file_edit_blank_line` and
 * one named path, with no quoted text, adds one empty line at the named end.
 */
function linePayloadAndPath(task) {
  if (quotedSegmentSpans(task).length > 0) return null;
  const toks = tokens(task);
  const pathIndices = toks.map((token, index) => [index, cleanPathToken(token.text)])
    .filter(([, path]) => looksLikeFilePath(path) && safeRelativePath(path));
  if (new Set(pathIndices.map(([, path]) => path)).size !== 1) return null;
  const [pathIndex, target] = pathIndices[0];
  const leads = wordsForRole('file_edit_line_lead').map((lead) => splitWhitespace(lead.toLowerCase()))
    .filter((words) => words.length > 0).sort((left, right) => right.length - left.length);
  let leadEnd = -1;
  for (let index = 0; index < pathIndex && leadEnd < 0; index += 1) {
    const lead = leads.find((words) => words.every((word, offset) => index + offset < pathIndex
      && cleanCueToken(toks[index + offset].text) === word));
    if (lead) leadEnd = index + lead.length;
  }
  if (leadEnd < 0) return null;
  const destinations = bareSurfaces('file_write_destination_cue');
  let end = -1;
  for (let index = leadEnd; index < pathIndex; index += 1) {
    if (destinations.includes(cleanCueToken(toks[index].text))) end = index;
  }
  if (end <= leadEnd) return null;
  return { target, text: task.slice(toks[leadEnd].start, toks[end - 1].end) };
}

/**
 * Mirrors `fn appended_block`: lines under an append request, at the
 * indentation they were given when the file already has lines indented
 * exactly so (a `.lino` table appended among its siblings), else at the
 * file's top level -- the indentation of lines written under a request is
 * otherwise only the request's layout (PR #1188 G16).
 */
function appendedBlock(source, block) {
  if (block.verbatim || !indentsLinesAt(source, block.indentation)) return block.text;
  return rebasedBlock(block.text, block.indentation);
}

/**
 * Mirrors `fn block_payload_and_path`: the one path a colon-ended request
 * line names, with nothing else quoted, and the lines under it.
 */
function blockPayloadAndPath(block) {
  const paths = [...new Set(unquotedPathTokens(block.head).map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path) && safeRelativePath(path)))];
  if (paths.length !== 1 || quotedSegmentSpans(block.head).some((segment) => segment.text !== paths[0])) return null;
  return { target: paths[0], text: block.text };
}

function blankLineAndPath(task) {
  if (quotedSegmentSpans(task).length > 0 || !mentionsRole('file_edit_blank_line', task.toLowerCase())) return null;
  const paths = [...new Set(unquotedPathTokens(task).map((token) => cleanPathToken(token.text))
    .filter((path) => looksLikeFilePath(path) && safeRelativePath(path)))];
  return paths.length === 1 ? { target: paths[0], text: '' } : null;
}
