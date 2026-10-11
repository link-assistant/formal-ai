// Numbered lines moved from one file to another (PR #1188 G85): `Move lines
// 671-690 of A to the end of B` was planned as `mv A B`, a rename of the whole
// file. A line-range move is the range placed in the destination (its end,
// or its start when the request says so) and then removed from the source,
// each the computed change it would be alone: read, compute, the smallest
// unique edit, the digest check. The destination is the path after the last
// seeded destination or position cue that a path follows (before the last
// cue, in a postpositional language); the source is the other path. The
// JavaScript twin of rust/src/agentic_coding/line_range_move.rs.

import { Capability } from './capability.mjs';
import { toolFor } from './capability_router.mjs';
import { renderSeededChange } from './code_task.mjs';
import { sourceFromAgentReadResult, sourceFromReadResult } from './code_artifact.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { textOutsideQuotedSegments } from './crate/coding_program_contract.mjs';
import { finalAnswer, planOne } from './plan.mjs';
import {
  changedLinesEdit, insertedAtEnd, planComputedChangeStep, readArguments, resultForPath, sentenceWords,
} from './workspace_change.mjs';
import { numberedLines, removedRange } from './workspace_line_operation.mjs';
import { failureMessage } from './tool_result.mjs';
import { bareSurfaces, cleanCueToken, cleanPathToken, looksLikeFilePath, safeRelativePath, tokens } from './write_request.mjs';
import { mentionsRole } from './write_lexicon.mjs';

const CUE_ROLES = ['file_write_destination_cue', 'file_edit_position_end', 'file_edit_position_start'];

/**
 * Mirrors `fn line_range_move`: `{source, target, first, last, atEnd}` when
 * `task` moves one numbered line range from one file to another, else null.
 * @param {string} task
 */
export function lineRangeMove(task) {
  const outside = sentenceWords(textOutsideQuotedSegments(task));
  if (!mentionsRole('line_move_action', outside)) return null;
  const segments = quotedSegmentSpans(task);
  const isPath = (path) => looksLikeFilePath(path) && safeRelativePath(path);
  if (segments.some((segment) => !isPath(segment.text.trim()))) return null;
  const ranges = numberedLines(task);
  if (ranges.length !== 1) return null;
  const words = tokens(task).filter((token) => !segments.some((segment) => token.start < segment.end && token.end > segment.start));
  const cues = new Set(CUE_ROLES.flatMap(bareSurfaces));
  const paths = words.flatMap((token, index) => (isPath(cleanPathToken(token.text)) ? [[index, cleanPathToken(token.text)]] : []));
  if (new Set(paths.map(([, path]) => path)).size !== 2) return null;
  const cueAt = words.flatMap((token, index) => (cues.has(cleanCueToken(token.text)) ? [index] : []));
  const followed = cueAt.filter((at) => paths.some(([index]) => index > at)).pop();
  let target = null;
  if (followed !== undefined) target = paths.find(([index]) => index > followed)[1];
  else if (cueAt.length > 0) target = paths.filter(([index]) => index < cueAt[cueAt.length - 1]).pop()?.[1] ?? null;
  if (target === null) return null;
  const source = paths.find(([, path]) => path !== target)[1];
  const atStart = mentionsRole('file_edit_position_start', outside) && !mentionsRole('file_edit_position_end', outside);
  return { source, target, first: ranges[0].first, last: ranges[0].last, atEnd: !atStart };
}

/** Mirrors `fn range_text`: lines `first..=last` (one-based) of `source`, joined, or null. */
export function rangeText(source, first, last) {
  const body = source.endsWith('\n') ? source.slice(0, -1) : source;
  const lines = source === '' ? [] : body.split('\n');
  return first < 1 || last < first || last > lines.length ? null : lines.slice(first - 1, last).join('\n');
}

/**
 * Mirrors `fn plan_line_range_move_step`: read the source, place the range in
 * the destination, remove it from the source, and state the move once both
 * are observed.
 */
export function planLineRangeMoveStep(task, currentTurn, toolNames, move) {
  const read = resultForPath(currentTurn, Capability.Read, move.source, null);
  if (read === null) {
    const tool = toolFor(toolNames, Capability.Read);
    return tool ? planOne(tool, readArguments(move.source)) : null;
  }
  const lines = move.first === move.last ? String(move.first) : `${move.first}-${move.last}`;
  // A read that came back as the client's file block is the file, whatever
  // its text says; any other failed read is a missing file.
  const missing = sourceFromAgentReadResult(read) === null && failureMessage(read, false, true) !== null;
  const text = missing ? null : rangeText(sourceFromReadResult(read), move.first, move.last);
  if (text === null) {
    const count = missing ? 0 : sourceFromReadResult(read).replace(/\n$/u, '').split('\n').length;
    const answer = renderSeededChange('line_range_outside_file', task, move.source, [['{lines}', lines], ['{count}', String(count)]]);
    return answer === null ? null : finalAnswer(answer);
  }
  const single = move.first === move.last;
  const intent = move.atEnd ? (single ? 'line_number_moved_end' : 'line_range_moved_end')
    : (single ? 'line_number_moved_start' : 'line_range_moved_start');
  const slots = [['{lines}', lines], ['{source}', move.source]];
  const placed = planComputedChangeStep(task, currentTurn, toolNames, {
    target: move.target, compute: (source) => insertedAtEnd(source, text, move.atEnd), edit: changedLinesEdit,
    intent, slots, bounded: false, reported: null,
  });
  const stated = renderSeededChange(intent, task, move.target, slots);
  if (placed?.kind !== 'final' || placed.answer !== stated) return placed;
  const removedSlots = [['{lines}', lines]];
  const removed = planComputedChangeStep(task, currentTurn, toolNames, {
    target: move.source, compute: (source, gone) => (gone ? null : removedRange(source, move.first, move.last)),
    edit: changedLinesEdit, intent: 'numbered_lines_removed', slots: removedSlots, bounded: true, reported: null,
  });
  if (removed?.kind !== 'final' || removed.answer !== renderSeededChange('numbered_lines_removed', task, move.source, removedSlots)) {
    return removed;
  }
  return placed;
}
