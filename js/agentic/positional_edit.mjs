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
  const text = quotedVerbatim(span) ?? describedLiteral(span) ?? cleanContent(span) ?? null;
  return text === null ? null : unescapeProseNewlines(text);
}

/**
 * Mirrors `fn described_literal`: `the heading '# Title'` -- one quoted
 * literal led only by the words that say what it is -- stands for the literal.
 */
function describedLiteral(span) {
  const segments = quotedSegmentSpans(span);
  if (segments.length !== 1) return null;
  const [segment] = segments;
  const lead = span.slice(0, segment.start);
  if (span.slice(segment.end).trim() !== '' || !/^[\p{Alphabetic}\p{White_Space}]*$/u.test(lead)) return null;
  return segment.text;
}

/**
 * Mirrors `fn compose_positional_insert` in rust/src/agentic_coding/positional_edit.rs:
 * `[target, anchor, new]` or null.
 * @param {string} request
 */
export function composePositionalInsert(request) {
  // `… after the line 'x':` followed by lines: the first line is the request
  // and the lines under it are the text inserted, whatever they quote.
  const block = introducedBlock(request);
  const sentence = block === null ? request : block.head;
  const normalized = normalizePrompt(sentence);
  // The position is the request's, not the payload's: a cue inside a quoted
  // literal ("Insert the line «… before …» after …") is text being inserted.
  const outside = normalizePrompt(quotedSegmentSpans(sentence)
    .reduceRight((text, segment) => `${text.slice(0, segment.start)} ${text.slice(segment.end)}`, sentence));
  const after = mentionsRole('file_edit_position_after', outside);
  const before = mentionsRole('file_edit_position_before', outside);
  if (after === before || !mentionsRole('coding_member_add_action', normalized)) return null;
  const literals = quotedLiterals(sentence);
  if (literals.length !== (block === null ? 2 : 1)) return null;
  const target = unquotedPathTokens(sentence)
    .map((token) => cleanPathToken(token.text))
    .find((candidate) => looksLikeFilePath(candidate) && safeRelativePath(candidate));
  if (target === undefined) return null;
  let inserted;
  let anchor;
  if (block === null) {
    const [first, second] = literals;
    const role = after ? 'file_edit_position_after' : 'file_edit_position_before';
    const governed = cueGovernedLiteral(request, role, first, second);
    if (governed === null) return null;
    const [insertedLiteral, anchorLiteral] = governed === 1 ? [first, second] : [second, first];
    inserted = unescapeProseNewlines(insertedLiteral.text);
    anchor = unescapeProseNewlines(anchorLiteral.text);
  } else {
    inserted = block.text;
    anchor = unescapeProseNewlines(literals[0].text);
  }
  const replacement = after ? `${anchor}\n${inserted}` : `${inserted}\n${anchor}`;
  return [target, anchor, replacement];
}

/**
 * Mirrors `fn introduced_block`: a request whose first line ends in a colon
 * and is followed by lines -- `{head, text}`, the lines with their shared
 * indentation removed (one quoted literal stands for itself), or null.
 */
function introducedBlock(request) {
  const breakAt = request.indexOf('\n');
  if (breakAt < 0) return null;
  const head = request.slice(0, breakAt).trimEnd();
  if (!head.endsWith(':')) return null;
  const lines = request.slice(breakAt + 1).split('\n').map((line) => line.trimEnd());
  while (lines.length > 0 && lines[0] === '') lines.shift();
  while (lines.length > 0 && lines[lines.length - 1] === '') lines.pop();
  if (lines.length === 0) return null;
  const indentOf = (line) => line.length - line.trimStart().length;
  const shared = Math.min(...lines.filter((line) => line !== '').map(indentOf));
  const text = lines.map((line) => line.slice(Math.min(shared, indentOf(line)))).join('\n');
  return { head, text: quotedVerbatim(text) ?? text };
}

/**
 * Mirrors `fn unquoted_path_tokens`: the request's tokens that are not part of
 * a quoted literal. A path inside the text being inserted or replaced
 * (`'import x from './a.mjs';'`) is payload, not the file the request edits; a
 * literal that is exactly one path (`'notes.txt'`) still names the file.
 * @param {string} request
 */
export function unquotedPathTokens(request) {
  const segments = quotedSegmentSpans(request);
  return tokens(request).filter((token) => !segments.some((segment) => token.start >= segment.start
    && token.end <= segment.end && cleanPathToken(token.text) !== segment.text));
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

/**
 * Mirrors `fn cue_governed_literal`: 0 or 1, or null when the cue occurs but
 * governs neither literal (a time, not a position: issue #1069).
 */
function cueGovernedLiteral(request, role, first, second) {
  const lowered = request.toLowerCase();
  // A cue word inside a quoted literal is payload ("Insert the line «… before
  // …» after …"), never the position the request asks for.
  const literals = quotedLiterals(request);
  const insideLiteral = (start) => literals.some((literal) => start >= literal.start && start < literal.end);
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
          if (!insideLiteral(start)) occurrences.push([start, start + utf8Len(surface), postpositional]);
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
  if (later === null && occurrences.length) return null;
  return 1;
}

const LEADING_MARKS = new Set([':', '-', '—', '–']);

function quotedVerbatim(span) {
  const trimmed = trim(trimStartMatches(trim(span), (character) => LEADING_MARKS.has(character)));
  // A span that is exactly one quoted literal of any delimiter the shared
  // reader knows (single quotes included, apostrophes told apart) is verbatim.
  const segments = quotedSegmentSpans(trimmed);
  if (segments.length === 1 && segments[0].start === 0 && segments[0].end === trimmed.length) {
    return segments[0].text;
  }
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

export const unescapeProseNewlines = (text) => replaceAllLiteral(replaceAllLiteral(text, '\\n', '\n'), '\\t', '\t');
