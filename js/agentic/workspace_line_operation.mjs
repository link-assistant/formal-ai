// Whole-line operations a request states by position, adjacency or exchange
// (rust/src/agentic_coding/workspace_line_operation.rs).
//
// PR #1188 dogfooding: `Delete lines 266-267 from f.lino.` only read the file
// (T30), `Insert the line '…' after line 12 in f.lino.` planned nothing (T33),
// `delete the line 'A' and the 'B' line directly above it` was routed to a
// whole-file write of a fragment (T29), `Move the line 'x' to the top of f`
// duplicated the line and `Swap the lines 'a' and 'b' in f` planned nothing.
// Each is now a computed change over the file's lines: read, compute, the
// smallest unique edit, the digest check, and the seeded sentence that states
// it. Every word that carries the operation is seeded (numbered_line_noun,
// numbered_line_lead, line_range_connector, line_adjacent_above,
// line_adjacent_below, line_move_action, line_swap_action) in every registered
// language.

import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { textOutsideQuotedSegments } from './crate/coding_program_contract.mjs';
import { unescapeProseNewlines } from './positional_edit.mjs';
import { sentenceWords } from './workspace_change.mjs';
import { cleanPathToken, looksLikeFilePath, safeRelativePath, tokens } from './write_request.mjs';
import { meaningEvidencedIn, mentionsRole, wordsForRole } from './write_lexicon.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { trim } from './write_str.mjs';

/** A digit run longer than this is not a line number. */
const MAX_NUMBER_DIGITS = 9;
const DASHES = '-–—';
const PUNCTUATION = '!"#$%&\'()*+,./:;<=>?@[\\]^_`{|}~«»।॥‘’“”、。！，：；？';

const isAsciiDigit = (character) => character >= '0' && character <= '9';
const isAsciiPrintable = (text) => /^[\x21-\x7e]+$/u.test(text);

/** Mirrors `fn is_han`: one Han ideograph, a word of its own. */
function isHan(character) {
  const point = character.codePointAt(0);
  return (point >= 0x3400 && point <= 0x4dbf) || (point >= 0x4e00 && point <= 0x9fff) || (point >= 0xf900 && point <= 0xfaff);
}

/**
 * Mirrors `fn named_target_and_payloads`: the one workspace path the request
 * names and its quoted payloads. A path the request leaves unquoted is the
 * file; only when it names none does a quoted literal that is exactly one path
 * name it, so `Delete the lines containing 'src/x.rs' from list.txt` edits
 * `list.txt` (PR #1188 T35).
 */
export function namedTargetAndPayloads(task) {
  const segments = quotedSegmentSpans(task);
  const isPath = (path) => looksLikeFilePath(path) && safeRelativePath(path);
  const quotedOver = (token) => segments.some((segment) => token.start < segment.end && token.end > segment.start);
  const unquoted = [...new Set(tokens(task).filter((token) => !quotedOver(token))
    .map((token) => cleanPathToken(token.text)).filter(isPath))];
  const quoted = [...new Set(segments.map((segment) => segment.text).filter(isPath))];
  let target = null;
  if (unquoted.length === 1) [target] = unquoted;
  else if (unquoted.length === 0 && quoted.length === 1) [target] = quoted;
  if (target === null) return null;
  return { target, payloads: segments.map((segment) => segment.text).filter((text) => text !== target && trim(text) !== '') };
}

/**
 * Mirrors `fn line_pieces`: the request outside its quotes as words, numbers
 * and dashes. A path is one opaque piece; a Han ideograph is a word of its own.
 */
function linePieces(task) {
  const pieces = [];
  for (const token of tokens(textOutsideQuotedSegments(task))) {
    const path = cleanPathToken(token.text);
    if (isAsciiPrintable(path) && looksLikeFilePath(path)) {
      pieces.push({ kind: 'path', text: path });
      continue;
    }
    let run = '';
    let kind = null;
    const flush = () => {
      if (run !== '') {
        const number = kind === 'number' && run.length <= MAX_NUMBER_DIGITS;
        pieces.push(number ? { kind, value: Number(run) } : { kind: kind === 'dash' ? kind : 'word', text: run.toLowerCase() });
      }
      run = '';
      kind = null;
    };
    for (const character of token.text) {
      let next = 'word';
      if (isAsciiDigit(character)) next = 'number';
      else if (DASHES.includes(character)) next = 'dash';
      else if (isHan(character)) next = 'han';
      else if (PUNCTUATION.includes(character)) next = null;
      if (next !== kind || next === 'dash' || next === 'han') flush();
      if (next === null) continue;
      kind = next;
      run += character;
    }
    flush();
  }
  return pieces;
}

const roleWords = (role) => wordsForRole(role).map((word) => word.toLowerCase());

/**
 * Mirrors `fn numbered_lines`: every line number or range the request names —
 * a seeded `numbered_line_noun` before the number (`line 12`, `lines 3-5`,
 * `lines from 3 to 5`, `строки с 3 по 5`), or, after a seeded lead, after it
 * (`第3到5行`) — as `{first, last}`, one-based and inclusive.
 */
function numberedLines(task) {
  const pieces = linePieces(task);
  const nouns = roleWords('numbered_line_noun');
  const leads = roleWords('numbered_line_lead');
  const connectors = roleWords('line_range_connector');
  const is = (index, words) => index >= 0 && index < pieces.length && pieces[index].kind === 'word' && words.includes(pieces[index].text);
  const isNumber = (index) => index >= 0 && index < pieces.length && pieces[index].kind === 'number';
  const found = [];
  for (let index = 0; index < pieces.length; index += 1) {
    if (!isNumber(index)) continue;
    let end = index;
    const joint = index + 1;
    if ((joint < pieces.length && pieces[joint].kind === 'dash') || is(joint, connectors)) {
      const second = is(joint + 1, leads) ? joint + 2 : joint + 1;
      if (isNumber(second)) end = second;
    }
    let before = index - 1;
    while (before >= index - 2 && is(before, leads)) before -= 1;
    const led = before < index - 1;
    if (is(before, nouns) || (led && is(end + 1, nouns))) {
      found.push({ first: pieces[index].value, last: pieces[end].value });
      index = end;
    }
  }
  return found;
}

/** Mirrors `fn file_lines`: the file's lines without their `\n`, and whether it ended in one. */
function fileLines(source) {
  const trailing = source.endsWith('\n');
  const body = trailing ? source.slice(0, -1) : source;
  return { lines: source === '' ? [] : body.split('\n'), trailing };
}

function joinedLines(lines, trailing) {
  return lines.length === 0 ? '' : `${lines.join('\n')}${trailing ? '\n' : ''}`;
}

const bare = (line) => (line.endsWith('\r') ? line.slice(0, -1) : line);

/**
 * Mirrors `fn unique_line`: the one line that is exactly `text`, else the one
 * line containing it; null when there are none or several.
 */
function uniqueLine(lines, text) {
  const exact = lines.flatMap((line, index) => (bare(line) === text ? [index] : []));
  if (exact.length > 0) return exact.length === 1 ? exact[0] : null;
  const containing = lines.flatMap((line, index) => (bare(line).includes(text) ? [index] : []));
  return containing.length === 1 ? containing[0] : null;
}

/** Mirrors `fn removed_range`: lines `first..=last` (one-based) gone. */
function removedRange(source, first, last) {
  const { lines, trailing } = fileLines(source);
  if (first < 1 || last < first || last > lines.length) return null;
  return joinedLines([...lines.slice(0, first - 1), ...lines.slice(last)], trailing);
}

/** Mirrors `fn inserted_at_line`: `text` as a line after (or before) line `line`. */
function insertedAtLine(source, line, text, after) {
  const { lines, trailing } = fileLines(source);
  if (line < 1 || line > lines.length) return null;
  const at = after ? line : line - 1;
  return joinedLines([...lines.slice(0, at), ...text.split('\n'), ...lines.slice(at)], trailing);
}

/**
 * Mirrors `fn adjacent_pair`: the indices of the line `anchor` names and the
 * line directly above (or below) it, when exactly one such pair exists and the
 * neighbour, if the request names it, is that line.
 */
function adjacentPair(lines, anchor, neighbour, above) {
  const fits = (index) => neighbour === null || bare(lines[index]) === neighbour
    || trim(bare(lines[index])) === trim(neighbour) || bare(lines[index]).includes(neighbour);
  const pairs = (matches) => lines.flatMap((line, index) => {
    const other = above ? index - 1 : index + 1;
    return matches(bare(line)) && other >= 0 && other < lines.length && fits(other) ? [[index, other]] : [];
  });
  let found = pairs((line) => line === anchor);
  if (found.length === 0) found = pairs((line) => line.includes(anchor));
  return found.length === 1 ? found[0] : null;
}

function removedAdjacent(source, anchor, neighbour, above) {
  const { lines, trailing } = fileLines(source);
  const pair = adjacentPair(lines, anchor, neighbour, above);
  return pair === null ? null : joinedLines(lines.filter((_, index) => !pair.includes(index)), trailing);
}

/**
 * Mirrors `fn moved_line`: the line `text` names taken out and put back at the
 * start, the end, or beside the line `anchor` names among the rest.
 */
function movedLine(source, text, destination, anchor) {
  const { lines, trailing } = fileLines(source);
  const from = uniqueLine(lines, text);
  if (from === null) return null;
  const rest = lines.filter((_, index) => index !== from);
  let at = destination === 'start' ? 0 : rest.length;
  if (destination === 'after' || destination === 'before') {
    const target = uniqueLine(rest, anchor);
    if (target === null) return null;
    at = destination === 'after' ? target + 1 : target;
  }
  return joinedLines([...rest.slice(0, at), lines[from], ...rest.slice(at)], trailing);
}

/** Mirrors `fn swapped_lines`: the two named lines exchanged. */
function swappedLines(source, first, second) {
  const { lines, trailing } = fileLines(source);
  const one = uniqueLine(lines, first);
  const two = uniqueLine(lines, second);
  if (one === null || two === null || one === two) return null;
  const out = [...lines];
  [out[one], out[two]] = [lines[two], lines[one]];
  return joinedLines(out, trailing);
}

/** Exactly one of the two seeded meanings, outside the request's quotes: true for the first. */
function eitherOf(outside, first, second) {
  const one = mentionsRole(first, outside);
  return one === mentionsRole(second, outside) ? null : one;
}

/**
 * Mirrors `fn grounded_line_operation`: the whole-line operation `task` asks
 * for, as a computed change (`{target, compute, intent, slots, reported,
 * bounded}`), or null. `bounded` marks an operation whose request states its
 * whole extent (a numbered range), which may remove most of a file.
 */
export function groundedLineOperation(task) {
  const named = namedTargetAndPayloads(task);
  if (!named) return null;
  const { target, payloads } = named;
  const outside = sentenceWords(textOutsideQuotedSegments(task));
  const change = (compute, intent, slots, extra = {}) => ({
    target, compute: (source, missing) => (missing ? null : compute(source)), intent, slots, reported: null, bounded: false, ...extra,
  });
  if (mentionsRole('line_swap_action', outside)) {
    if (payloads.length !== 2) return null;
    const [first, second] = payloads;
    return change((source) => swappedLines(source, first, second), 'lines_swapped', [['{old}', first], ['{new}', second]]);
  }
  if (mentionsRole('line_move_action', outside)) {
    const [text, anchor] = payloads;
    const atStart = eitherOf(outside, 'file_edit_position_start', 'file_edit_position_end');
    const after = eitherOf(outside, 'file_edit_position_after', 'file_edit_position_before');
    if (payloads.length === 1 && atStart !== null) {
      const destination = atStart ? 'start' : 'end';
      return change((source) => movedLine(source, text, destination, null), `line_moved_${destination}`, [['{old}', text]]);
    }
    if (payloads.length === 2 && after !== null) {
      const destination = after ? 'after' : 'before';
      return change((source) => movedLine(source, text, destination, anchor), `line_moved_${destination}`,
        [['{old}', text], ['{anchor}', anchor]]);
    }
    return null;
  }
  const removing = mentionsRole('coding_text_remove_action', outside) && !mentionsRole('coding_member_add_action', outside);
  const above = eitherOf(outside, 'line_adjacent_above', 'line_adjacent_below');
  const numbered = numberedLines(task);
  if (numbered.length > 1) return null;
  if (numbered.length === 1) {
    const { first, last } = numbered[0];
    if (removing && payloads.length === 0) {
      const from = above === true ? first - 1 : first;
      const to = above === false ? last + 1 : last;
      const lines = from === to ? String(from) : `${from}-${to}`;
      return change((source) => removedRange(source, from, to), 'numbered_lines_removed', [['{lines}', lines]], { bounded: true });
    }
    const after = eitherOf(outside, 'file_edit_position_after', 'file_edit_position_before');
    if (!removing && first === last && payloads.length === 1 && after !== null) {
      const text = unescapeProseNewlines(payloads[0]);
      return change((source) => insertedAtLine(source, first, text, after),
        after ? 'numbered_line_insert_after' : 'numbered_line_insert_before', [['{new}', text], ['{line}', String(first)]]);
    }
    return null;
  }
  const namesLine = meaningEvidencedIn('line', normalizePrompt(textOutsideQuotedSegments(task)).toLowerCase());
  if (removing && namesLine && above !== null && (payloads.length === 1 || payloads.length === 2)) {
    const [anchor, neighbour = null] = payloads;
    return change((source) => removedAdjacent(source, anchor, neighbour, above), 'coding_text_remove', [['{old}', anchor]], {
      reported: (source) => {
        const { lines } = fileLines(source);
        const pair = adjacentPair(lines, anchor, neighbour, above);
        if (pair === null) return null;
        const removed = [...pair].sort((left, right) => left - right).map((index) => bare(lines[index]));
        return { intent: 'coding_text_remove', slots: [['{old}', removed.join('`, `')]] };
      },
    });
  }
  return null;
}

/** A file below this many lines is too short for "most of it" to mean anything. */
const MIN_GUARDED_LINES = 8;

/**
 * Mirrors `fn drops_most_of_file`: whether `updated` keeps fewer than half of
 * the lines of a file that had at least `MIN_GUARDED_LINES`. A computed change
 * that does so without the request stating that extent is refused (PR #1188
 * T29: a 380-line file was once replaced by one line).
 */
export function dropsMostOfFile(source, updated) {
  const before = fileLines(source).lines.length;
  return before >= MIN_GUARDED_LINES && fileLines(updated).lines.length * 2 < before;
}
