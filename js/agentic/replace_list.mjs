// Several replacements in one file, asked in one sentence (PR #1188 G82): `In
// b.lino replace 'x' with 'y', replace 'p' with 'q', and replace 'm' with
// 'n'.` The sentence is cut before every repeated seeded edit action outside
// its quotes; a clause that names no file takes the file the first one names,
// and each clause is the edit request one replacement alone would be. The
// replacements are applied in order to the file as read, and the change is
// stated pair by pair. The JavaScript twin of rust/src/agentic_coding/replace_list.rs.

import { quotedSegmentSpans, quotedSegments } from './crate/normal_markov.mjs';
import { bareSurfaces, cleanCueToken, composeEditRequest, tokens } from './write_request.mjs';

const CLAUSE_TAIL = /[\s,;，；]+$/u;

/**
 * Mirrors `fn replace_clauses`: the request cut before every repeated
 * unquoted edit action, each clause without the commas and seeded joiners
 * that led into the next; null unless there are at least two.
 * @param {string} request
 */
export function replaceClauses(request) {
  const segments = quotedSegmentSpans(request);
  const quoted = (token) => segments.some((segment) => token.start < segment.end && token.end > segment.start);
  const actions = bareSurfaces('file_edit_action_cue');
  const joiners = bareSurfaces('file_edit_joiner_cue');
  const cuts = tokens(request).filter((token) => !quoted(token) && actions.includes(cleanCueToken(token.text)))
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
  const first = composeEditRequest(clauses[0]);
  if (first === null) return null;
  const [target] = first;
  const quoted = quotedSegments(request);
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
