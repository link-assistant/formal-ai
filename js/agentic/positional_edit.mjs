// Additive edits and the words that place them (issues #1115, #1116, #1133):
// the JavaScript twin of rust/src/agentic_coding/positional_edit.rs.
//
// Literal and cue offsets are compared as gaps, so they are kept in UTF-8
// bytes exactly as the Rust original measures them.

import { cleanContent, cleanPathToken, looksLikeFilePath, safeRelativePath, tokens } from './write_request.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { usesPostpositions } from './crate/language.mjs';
import { meaningsWithRole, mentionsRole } from './crate/seed_meanings.mjs';
import { replaceAllLiteral, trim, trimStartMatches, utf16ToByte, utf8Len } from './crate/rust_str.mjs';

/**
 * Mirrors `fn literal_text` in rust/src/agentic_coding/positional_edit.rs.
 * @param {string} span
 * @returns {string|null}
 */
export function literalText(span) {
  const text = quotedVerbatim(span) ?? cleanContent(span) ?? null;
  return text === null ? null : unescapeProseNewlines(text);
}

/**
 * Mirrors `fn compose_positional_insert` in rust/src/agentic_coding/positional_edit.rs:
 * `[target, anchor, new]` or null.
 * @param {string} request
 */
export function composePositionalInsert(request) {
  const normalized = normalizePrompt(request);
  const after = mentionsRole('file_edit_position_after', normalized);
  const before = mentionsRole('file_edit_position_before', normalized);
  if (after === before || !mentionsRole('coding_member_add_action', normalized)) return null;
  const literals = quotedLiterals(request);
  if (literals.length !== 2) return null;
  const [first, second] = literals;
  const target = tokens(request)
    .map((token) => cleanPathToken(token.text))
    .find((candidate) => looksLikeFilePath(candidate) && safeRelativePath(candidate));
  if (target === undefined) return null;
  const role = after ? 'file_edit_position_after' : 'file_edit_position_before';
  const [insertedLiteral, anchorLiteral] = cueGovernedLiteral(request, role, first, second) === 1
    ? [first, second]
    : [second, first];
  const inserted = unescapeProseNewlines(insertedLiteral.text);
  const anchor = unescapeProseNewlines(anchorLiteral.text);
  const replacement = after ? `${anchor}\n${inserted}` : `${inserted}\n${anchor}`;
  return [target, anchor, replacement];
}

/**
 * Mirrors `fn quoted_literals`: `{start, end, text}` with byte offsets — every
 * delimited literal slot `quotedSegmentSpans` reads (double, single, backtick,
 * guillemet and CJK quotes), verbatim, so a quoted line keeps its indentation.
 */
function quotedLiterals(request) {
  return quotedSegmentSpans(request).map((segment) => ({
    start: utf16ToByte(request, segment.start),
    end: utf16ToByte(request, segment.end),
    text: segment.text,
  }));
}

/** Mirrors `fn cue_governed_literal`: 0 or 1. */
function cueGovernedLiteral(request, role, first, second) {
  const lowered = request.toLowerCase();
  const occurrences = [];
  for (const meaning of meaningsWithRole(role)) {
    for (const lexeme of meaning.lexemes) {
      const postpositional = usesPostpositions(lexeme.language);
      for (const word of lexeme.words) {
        const surface = word.text.toLowerCase();
        if (surface === '') continue;
        let from = 0;
        for (;;) {
          const at = lowered.indexOf(surface, from);
          if (at < 0) break;
          const start = utf16ToByte(lowered, at);
          occurrences.push([start, start + utf8Len(surface), postpositional]);
          from = at + surface.length;
        }
      }
    }
  }
  const gap = (literal) => {
    let best = null;
    for (const [cueStart, cueEnd, postpositional] of occurrences) {
      const distance = postpositional ? cueStart - literal.end : literal.start - cueEnd;
      if (distance >= 0 && (best === null || distance < best)) best = distance;
    }
    return best;
  };
  const before = gap(first);
  const later = gap(second);
  if (before !== null && later !== null) return later <= before ? 1 : 0;
  if (before !== null) return 0;
  return 1;
}

const LEADING_MARKS = new Set([':', '-', '—', '–']);

function quotedVerbatim(span) {
  const trimmed = trim(trimStartMatches(trim(span), (character) => LEADING_MARKS.has(character)));
  const characters = Array.from(trimmed);
  if (characters.length < 2) return null;
  const first = characters[0];
  const last = characters[characters.length - 1];
  if (first !== last || (first !== '"' && first !== '`')) return null;
  const inner = trimmed.slice(1, trimmed.length - 1);
  return inner.includes(first) ? null : inner;
}

/**
 * Mirrors `fn names_local_edit` in rust/src/agentic_coding/positional_edit.rs.
 * @param {string} task
 */
export function namesLocalEdit(task) {
  const normalized = normalizePrompt(task);
  const edits = mentionsRole('file_edit_action_cue', normalized) || mentionsRole('coding_member_add_action', normalized);
  return edits && tokens(task).some((token) => {
    const candidate = cleanPathToken(token.text);
    return looksLikeFilePath(candidate) && safeRelativePath(candidate);
  });
}

const unescapeProseNewlines = (text) => replaceAllLiteral(replaceAllLiteral(text, '\\n', '\n'), '\\t', '\t');
