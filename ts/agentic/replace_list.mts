// Several replacements in one file, asked in one sentence (PR #1188 G82): `In
// b.lino replace 'x' with 'y', replace 'p' with 'q', and replace 'm' with
// 'n'.` The sentence is cut before every repeated seeded edit action outside
// its quotes; a clause that names no file takes the file another one names,
// and each clause is the edit request one replacement alone would be. The
// replacements are applied in order to the file as read, and the change is
// stated pair by pair. The JavaScript twin of rust/src/agentic_coding/replace_list.rs.

import { quotedSegmentSpans, quotedSegments } from './crate/normal_markov.mjs';
import {
  bareSurfaces, cleanCueToken, cleanPathToken, composeEditClauses, composeEditRequest, cuedWriteTargets, looksLikeFilePath,
  safeRelativePath, spanOf, tokens,
} from './write_request.mjs';
import { sentences } from './shell_command_policy.mjs';
import { renderSeededChange } from './code_task.mjs';
import { instructionEnd, unescapeProseNewlines } from './positional_edit.mjs';

const CLAUSE_TAIL = /[\s,;，；]+$/u;

/**
 * Mirrors `fn replace_clauses`: the request cut before every repeated
 * unquoted edit action of its instruction, each clause without the commas
 * and seeded joiners that led into the next; null unless there are at least
 * two.
 * @param {string} request
 */
export function replaceClauses(request) {
  const segments = quotedSegmentSpans(request);
  const quoted = (token) => segments.some((segment) => token.start < segment.end && token.end > segment.start);
  const actions = bareSurfaces('file_edit_action_cue');
  const joiners = bareSurfaces('file_edit_joiner_cue');
  const end = instructionEnd(request);
  const cuts = tokens(request)
    .filter((token) => token.start < end && !quoted(token) && actions.includes(cleanCueToken(token.text)))
    .map((token) => token.start);
  if (cuts.length < 2) return null;
  const pieces = [];
  for (let index = 0; index < cuts.length; index += 1) {
    const from = index === 0 ? 0 : cuts[index];
    const to = index + 1 < cuts.length ? cuts[index + 1] : request.length;
    let piece = request.slice(from, to).replace(CLAUSE_TAIL, '');
    for (;;) {
      const words = piece.split(/\s+/u);
      if (words.length < 2 || !joiners.includes(cleanCueToken(words.at(-1)))) break;
      piece = piece.slice(0, piece.length - words.at(-1).length).replace(CLAUSE_TAIL, '');
    }
    pieces.push(piece);
  }
  return pieces;
}

/**
 * Mirrors `fn replace_list`: `{target, pairs}` -- the one file and the quoted
 * `[old, new]` pairs, in order -- for a request whose every clause is one
 * replacement of quoted text in that file, or null.
 * @param {string} request
 */
export function replaceList(request) {
  const clauses = replaceClauses(request);
  if (clauses === null) return null;
  // The file may be named in any clause: `In m.js, replace ...` names it
  // first and `Replace ... and replace ... in m.js` last (PR #1188 G89).
  const named = clauses.map((clause) => composeEditRequest(clause)).find((edit) => edit !== null);
  if (named === undefined) return null;
  const [target] = named;
  // A quoted text may spell a newline as an escape, which the composed edit
  // has already turned into the character (PR #1188 G88).
  const quoted = quotedSegments(request).flatMap((segment) => [segment, unescapeProseNewlines(segment)]);
  const pairs = [];
  for (const clause of clauses) {
    const edit = clause.includes(target) ? composeEditRequest(clause) : composeEditRequest(`${clause} in ${target}`);
    if (edit === null || edit[0] !== target || !quoted.includes(edit[1]) || !quoted.includes(edit[2])) return null;
    if (edit[1] === '' || edit[1] === edit[2]) return null;
    pairs.push([edit[1], edit[2]]);
  }
  return { target, pairs };
}

/**
 * Mirrors `fn unplanned_edit_clause`: the first clause of a request asking
 * for several edits that cannot be read as one replacement of quoted text in
 * the request's file, when the first can (PR #1188 G106). Such a request
 * would otherwise make the first edit and drop the rest without a word, so
 * it is declined, naming the clause. A clause is an edit of its own only
 * when a clause mark or a seeded joiner opens it (`, replace`, `and replace`);
 * null when there is no such clause or every one is a replacement.
 * @param {string} request
 */
export function unplannedEditClause(request) {
  const clauses = replaceClauses(request);
  if (clauses === null || replaceList(request) !== null) return null;
  const first = composeEditRequest(clauses[0]);
  if (first === null) return null;
  const [target] = first;
  const joiners = bareSurfaces('file_edit_joiner_cue');
  let from = 0;
  for (const clause of clauses.slice(1)) {
    const at = request.indexOf(clause, from);
    from = at + clause.length;
    const lead = request.slice(0, at).trimEnd();
    const opened = /[,;，；]$/u.test(lead) || joiners.includes(cleanCueToken(lead.split(/\s+/u).at(-1) ?? ''));
    if (!opened) continue;
    const edit = clause.includes(target) ? composeEditRequest(clause) : composeEditRequest(`${clause} in ${target}`);
    const segments = quotedSegments(clause);
    if (edit === null || edit[0] !== target || !segments.includes(edit[1]) || !segments.includes(edit[2])) {
      return clause.replace(/[.!?。！？]+$/u, '');
    }
  }
  return null;
}

/**
 * Mirrors `fn several_edit_targets`: the distinct files an edit request names
 * outside its quotes when it names more than one, or null. One edit request
 * changes one file, so such a request is declined with the files named
 * rather than applied to the first alone (PR #1188 G91).
 * @param {string} request
 */
export function severalEditTargets(request) {
  const composed = composeEditClauses(request);
  if (composed === null) return null;
  const segments = quotedSegmentSpans(request);
  const toks = tokens(request);
  const [from, end] = editRegion(request, composed.spans);
  const counted = (index) => toks[index].start >= from && toks[index].start < end
    && !segments.some((segment) => toks[index].start < segment.end && toks[index].end > segment.start);
  const targets = cuedWriteTargets(toks).filter(([index]) => counted(index));
  // A file listed after a cued one shares its cue (PR #1188 G104):
  // `In a.mjs and b.mjs, replace ...` names b.mjs as much as a.mjs.
  for (const [index, path] of coordinatedPaths(toks, targets.map(([cued]) => cued))) {
    if (counted(index)) targets.push([index, path]);
  }
  const paths = [];
  for (const [, path] of targets.sort(([a], [b]) => a - b)) {
    if (!paths.includes(path)) paths.push(path);
  }
  return paths.length > 1 ? paths : null;
}

/**
 * Mirrors `fn edit_region`: `[from, end]` -- the sentences that hold the
 * composed edit's clauses (`spans`), or the whole instruction when the edit
 * has none. A path in a later sentence (`Leave supporting evidence in
 * proof.md.`) is no target of the edit (PR #1188 ladder regression of G91).
 * @param {string} request
 * @param {Array<[number, number]>|null} spans
 */
function editRegion(request, spans) {
  const end = instructionEnd(request);
  if (spans === null) return [0, end];
  const first = Math.min(...spans.map(([start]) => start));
  const last = Math.max(...spans.map(([, stop]) => stop));
  const parts = sentences(request).map((sentence) => spanOf(sentence));
  const opening = parts.find((span) => first >= span.start && first < span.end);
  const closing = parts.find((span) => last > span.start && last <= span.end);
  return [opening ? opening.start : 0, Math.min(end, closing ? closing.end : end)];
}

/**
 * Mirrors `fn coordinated_paths`: the paths listed right after one of the
 * `cued` path tokens -- after a comma that closes the token before, or after
 * a seeded joiner word (`and`) -- each with its token index.
 * @param {Array<{text: string}>} toks
 * @param {number[]} cued
 */
export function coordinatedPaths(toks, cued) {
  const joiners = bareSurfaces('file_edit_joiner_cue');
  const pathOf = (token) => {
    const cleaned = token === undefined ? '' : cleanPathToken(token.text);
    return looksLikeFilePath(cleaned) && safeRelativePath(cleaned) ? cleaned : null;
  };
  const out = [];
  for (const start of cued) {
    let index = start;
    for (;;) {
      const listed = toks[index].text.endsWith(',');
      let next = index + 1;
      if (toks[next] !== undefined && joiners.includes(cleanCueToken(toks[next].text))) next += 1;
      else if (!listed) break;
      const path = pathOf(toks[next]);
      if (path === null) break;
      out.push([next, path]);
      index = next;
    }
  }
  return out;
}

/**
 * Mirrors `fn replaced_in_order`: `source` with each pair's old text replaced
 * everywhere by its new text, in order, or null when an old text is missing
 * by the time its turn comes.
 * @param {string} source
 * @param {Array<[string, string]>} pairs
 */
export function replacedInOrder(source, pairs) {
  let text = source;
  for (const [old, next] of pairs) {
    if (!text.includes(old)) return null;
    text = text.split(old).join(next);
  }
  return text;
}

/** How a slot lists several names: `a`, `b`. */
const LISTED = '`, `';

/**
 * Mirrors `fn absent_text_answer`: when the file no longer holds `old`, the
 * honest answer -- the replacement is already made when it holds `next`, else
 * the text does not occur -- never a failed verification of an effect nothing
 * planned (PR #1188 G87). Null while `old` is still there.
 * @param {string} task
 * @param {string} target
 * @param {string} source
 * @param {string} old
 * @param {string|null} next
 */
export function absentTextAnswer(task, target, source, old, next) {
  // A slot listing several names (`a`, `b`) is absent only when each is.
  if (old === '' || old.split(LISTED).some((name) => source.includes(name))) return null;
  const done = next !== null && next !== '' && source.includes(next);
  const slots = done ? [['{old}', old], ['{new}', next]] : [['{old}', old]];
  return renderSeededChange(done ? 'coding_text_already_replaced' : 'coding_text_not_found', task, target, slots);
}
