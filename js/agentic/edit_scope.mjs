// The part of a file a Replace is scoped to (PR #1188 G79): `In languages.rs,
// in the row whose slug is rust, replace 'x' with 'y'`. The edit composer
// reads the path, the old text and the new text; a further clause that a
// seeded target cue leads (`in the row …`) narrows where the old text is
// meant. Where the old text occurs once the clause changes nothing; where it
// occurs more than once, no part of the file is named that the planner can
// find, so nothing is changed and the answer names the clause. The JavaScript
// twin of rust/src/agentic_coding/edit_scope.rs.

import { bareSurfaces, cleanCueToken, composeEditClauses, tokens } from './write_request.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { lineBlockStarts } from './workspace_setting.mjs';
import { renderSeededChange } from './code_task.mjs';

const CLAUSE_MARKS = /[,;:]$/u;
const SENTENCE_END = /[.!?。！？]\s+/gu;
const EDGE_MARKS = /^[^\p{Alphabetic}\p{N}]+|[^\p{Alphabetic}\p{N}]+$/gu;

/**
 * Mirrors `fn edit_scope`: the words of a clause that narrows an edit request
 * to a part of its file (`the row whose slug is rust`), or null. The clause
 * stands outside the edit's own clauses -- between them, or before them on
 * the request's line -- and a seeded target cue leads it.
 * @param {string} request
 */
export function editScope(request) {
  const clauses = composeEditClauses(request);
  if (clauses === null || clauses.spans === null) return null;
  const [first, second] = [...clauses.spans].sort((a, b) => a[0] - b[0]);
  // Each gap counts from its line's start and past its last sentence end: a
  // clause of another sentence (`Open f.txt in the editor.`) is no scope.
  const lineStart = first[0] === 0 ? 0 : request.lastIndexOf('\n', first[0] - 1) + 1;
  const sentenceStart = (from, to) => {
    const ends = [...request.slice(from, to).matchAll(SENTENCE_END)];
    return ends.length === 0 ? from : from + ends.at(-1).index + ends.at(-1)[0].length;
  };
  const quoted = quotedSegmentSpans(request);
  const cues = bareSurfaces('file_edit_target_cue');
  const toks = tokens(request).filter((token) => !quoted.some((segment) => token.start < segment.end && token.end > segment.start));
  for (const [start, to] of [[lineStart, first[0]], [first[1], second[0]]]) {
    const from = sentenceStart(start, to);
    const words = toks.filter((token) => token.start >= from && token.end <= to);
    const lead = words.findIndex((token) => cues.includes(cleanCueToken(token.text)));
    if (lead < 0) continue;
    const clause = [];
    for (const token of words.slice(lead + 1)) {
      clause.push(token);
      if (CLAUSE_MARKS.test(token.text)) break;
    }
    if (!clause.some((token) => /\p{Alphabetic}/u.test(token.text) && !cues.includes(cleanCueToken(token.text)))) continue;
    return request.slice(clause[0].start, clause.at(-1).end).replace(EDGE_MARKS, '');
  }
  return null;
}

/**
 * Mirrors `fn scoped_occurrences`: `{scope, old, count}` when the request
 * narrows its edit to a part of the file and its old text occurs more than
 * once in `source` (as text, or as one run of whole lines), or null.
 * @param {string} request
 * @param {string} source
 */
export function scopedOccurrences(request, source) {
  const scope = editScope(request);
  if (scope === null) return null;
  const [, old] = composeEditClauses(request).edit;
  if (old === '') return null;
  const count = Math.max(source.split(old).length - 1, old.includes('\n') ? lineBlockStarts(source, old).length : 0);
  return count > 1 ? { scope, old, count } : null;
}

/**
 * Mirrors `fn scoped_decline`: the answer declining a request that narrows
 * its edit to a part of the file where its old text occurs more than once,
 * naming the part, or null.
 * @param {string} request
 * @param {string} target
 * @param {string} source
 */
export function scopedDecline(request, target, source) {
  const scoped = scopedOccurrences(request, source);
  if (scoped === null) return null;
  return renderSeededChange('file_edit_scope_unapplied', request, target, [
    ['{old}', scoped.old.split('\n').join('`, `')], ['{scope}', scoped.scope], ['{count}', String(scoped.count)],
  ]);
}
