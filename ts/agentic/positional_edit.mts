// Additive edits and the words that place them (issues #1115, #1116, #1133):
// the JavaScript twin of rust/src/agentic_coding/positional_edit.rs.
//
// Literal and cue offsets are compared as gaps, so they are kept in UTF-8
// bytes exactly as the Rust original measures them.

import {
  bareSurfaces, cleanContent, cleanCueToken, cleanPathToken, looksLikeFilePath, safeRelativePath, tokens,
} from './write_request.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import { quotedSegmentSpans } from './crate/normal_markov.mjs';
import { usesPostpositions } from './crate/language.mjs';
import { meaningsWithRole, mentionsRole } from './crate/seed_meanings.mjs';
import {
  byteToUtf16, replaceAllLiteral, trim, trimStartMatches, utf16ToByte, utf8Len,
} from './crate/rust_str.mjs';
import { splitWhitespace } from './write_str.mjs';
import { meaningEvidencedIn } from './write_lexicon.mjs';

/**
 * Mirrors `fn literal_text` in rust/src/agentic_coding/positional_edit.rs.
 * @param {string} span
 * @returns {string|null}
 */
export function literalText(span) {
  const text = quotedVerbatim(span) ?? listedLines(span) ?? describedLiteral(span) ?? cleanContent(span) ?? null;
  return text === null ? null : unescapeProseNewlines(text);
}

/**
 * Mirrors `fn listed_lines`: `the three lines 'a', 'b' and 'c'` -- several
 * quoted literals led by words that name lines (the seeded `line` meaning)
 * and joined only by commas and seeded joiners -- stands for those lines as
 * one consecutive block (PR #1188 G80).
 */
function listedLines(span) {
  const literals = quotedLiterals(span);
  if (literals.length < 2 || !joinedOnly(span, literals) || span.slice(literals.at(-1).to).trim() !== '') return null;
  const lead = normalizePrompt(span.slice(0, literals[0].from)).toLowerCase();
  return meaningEvidencedIn('line', lead) ? literals.map((literal) => literal.text).join('\n') : null;
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
 * `[target, anchor, new]` or null -- one insert at an anchor that stands on
 * its own (no anchor context, one clause).
 * @param {string} request
 */
export function composePositionalInsert(request) {
  const inserts = positionalInserts(request);
  if (inserts === null || inserts.length !== 1 || inserts[0].context !== null) return null;
  const { target, anchor, inserted, after } = inserts[0];
  return [target, anchor, after ? `${anchor}\n${inserted}` : `${inserted}\n${anchor}`];
}

/**
 * Mirrors `fn positional_inserts`: every insert the request asks for, in its
 * order, as `{target, anchor, inserted, after, context}`, or null unless every
 * clause is one. Clauses are the request cut before a repeated add action
 * that a seeded joiner or a clause mark leads (`…, and insert …`); a clause
 * that names no file takes the one file the request names.
 * @param {string} request
 */
export function positionalInserts(request) {
  // `… after the line 'x':` followed by lines: the first line is the request
  // and the lines under it are the text inserted, whatever they quote.
  const block = introducedBlock(request);
  const clauses = block === null ? insertClauses(request) : [block.head];
  const inserts = clauses.map((clause) => clauseInsert(clause, block === null ? null : block.text));
  if (inserts.some((insert) => insert === null)) return null;
  // Lines under the request that were neither fenced nor quoted keep the
  // indentation they were given when the file uses it, else are rebased on the
  // anchor's indentation once the file is read (PR #1188 G16, G93). `rebase`
  // is that given indentation, or null for lines kept as written.
  for (const insert of inserts) insert.rebase = block !== null && !block.verbatim ? block.indentation : null;
  const named = [...new Set(namedPaths(request))];
  for (const insert of inserts) {
    if (insert.target !== null) continue;
    if (named.length !== 1) return null;
    insert.target = named[0];
  }
  return inserts;
}

/**
 * Mirrors `fn clause_insert`: one clause's insert, its target null when the
 * clause names no file. The anchor is the literal the position cue governs;
 * a seeded anchor context (`the line 'y' that follows 'z'`) names a literal
 * the anchor comes after; every other literal is a line inserted, in order,
 * and those lines may be separated only by seeded joiners and commas.
 */
function clauseInsert(sentence, blockText) {
  const normalized = normalizePrompt(sentence);
  // The position is the request's, not the payload's: a cue inside a quoted
  // literal ("Insert the line «… before …» after …") is text being inserted.
  const outside = normalizePrompt(quotedSegmentSpans(sentence)
    .reduceRight((text, segment) => `${text.slice(0, segment.start)} ${text.slice(segment.end)}`, sentence));
  const after = mentionsRole('file_edit_position_after', outside);
  const before = mentionsRole('file_edit_position_before', outside);
  if (after === before || !mentionsRole('coding_member_add_action', normalized)) return null;
  const target = namedPaths(sentence)[0] ?? null;
  const literals = quotedLiterals(sentence).filter((literal) => literal.text !== target);
  const contexts = cueOccurrences(sentence, 'file_edit_anchor_context_cue');
  const role = after ? 'file_edit_position_after' : 'file_edit_position_before';
  // A position word inside the context cue ("के बाद आने वाली") is the
  // context's, not the anchor's.
  const positions = cueOccurrences(sentence, role)
    .filter(([start]) => !contexts.some(([from, to]) => start >= from && start < to));
  if (blockText !== null && literals.length === 0) {
    return unquotedAnchorInsert(sentence, positions, contexts, target, blockText, after);
  }
  if (blockText !== null) {
    // One literal is the anchor: a context cue then names no second line, so
    // `the following lines` speaks of the block (PR #1188 G16). With a seeded
    // anchor context, a second literal is the line the anchor follows (G51).
    if (literals.length === 1) {
      return { target, anchor: unescapeProseNewlines(literals[0].text), inserted: blockText, after, context: null };
    }
    if (literals.length !== 2 || contexts.length === 0) return null;
    const anchorAt = governedLiteral(positions, literals, [0, 1]);
    const contextAt = anchorAt === null ? null : governedLiteral(contexts, literals, [1 - anchorAt]);
    if (contextAt === null) return null;
    return {
      target,
      anchor: unescapeProseNewlines(literals[anchorAt].text),
      inserted: blockText,
      after,
      context: unescapeProseNewlines(literals[contextAt].text),
    };
  }
  // `Insert an empty line before the line '## Probe'`: the seeded blank line
  // is the line inserted beside the one literal (PR #1188 G31).
  if (literals.length === 1 && contexts.length === 0 && mentionsRole('file_edit_blank_line', outside)) {
    return { target, anchor: unescapeProseNewlines(literals[0].text), inserted: '', after, context: null };
  }
  if (literals.length < 2) return null;
  const indices = literals.map((_, index) => index);
  const anchorAt = governedLiteral(positions, literals, indices);
  if (anchorAt === null) return null;
  let contextAt = null;
  if (contexts.length > 0) {
    contextAt = governedLiteral(contexts, literals, indices.filter((index) => index !== anchorAt));
    if (contextAt === null) return null;
  }
  const inserted = literals.filter((_, index) => index !== anchorAt && index !== contextAt);
  if (inserted.length === 0 || !joinedOnly(sentence, inserted)) return null;
  // Lines given as separate literals are already lines: an escape inside one is content (PR #1188 G53).
  const lines = inserted.map((literal) => (inserted.length > 1 ? literal.text : unescapeProseNewlines(literal.text))).join('\n');
  // `'x' followed by an empty line`: the seeded blank line goes on the side of
  // the lines its words stand on, never dropped (PR #1188 G74).
  const blanks = cueOccurrences(sentence, 'file_edit_blank_line').map(([start]) => start);
  const blankFirst = blanks.length > 0 && Math.min(...blanks) < inserted[0].start;
  return {
    target,
    anchor: unescapeProseNewlines(literals[anchorAt].text),
    inserted: blanks.length === 0 ? lines : blankFirst ? `\n${lines}` : `${lines}\n`,
    after,
    context: contextAt === null ? null : unescapeProseNewlines(literals[contextAt].text),
  };
}

/**
 * Mirrors `fn unquoted_anchor_insert`: a block placed by unquoted line words
 * (`after the line row a that follows the line table u in f.lino:`, PR #1188
 * G51). The words after the position cue, up to a seeded context cue or the
 * target clause, are the anchor span, and the words after the context cue the
 * context span; each is resolved against the file's lines once it is read
 * (`spans`). A postpositional cue is left to quoted anchors.
 */
function unquotedAnchorInsert(sentence, positions, contexts, target, blockText, after) {
  const earliest = (cues) => cues.reduce((best, cue) => (best === null || cue[0] < best[0]
    || (cue[0] === best[0] && cue[1] > best[1]) ? cue : best), null);
  const cue = earliest(positions);
  if (cue === null || cue[2]) return null;
  const context = earliest(contexts.filter(([start, , postpositional]) => start >= cue[1] && !postpositional));
  const at = target === null ? -1 : sentence.lastIndexOf(target);
  const end = at < 0 || utf16ToByte(sentence, at) < cue[1] ? utf8Len(sentence) : utf16ToByte(sentence, at);
  const span = (from, to) => withoutTargetCue(sentence.slice(byteToUtf16(sentence, from), byteToUtf16(sentence, to)));
  const anchor = span(cue[1], context === null ? end : context[0]);
  const contextText = context === null ? null : span(context[1], end);
  if (anchor === '' || contextText === '') return null;
  return { target, anchor, inserted: blockText, after, context: contextText, spans: true };
}

/** Mirrors `fn without_target_cue`: a span without its closing mark and a trailing seeded target cue. */
function withoutTargetCue(span) {
  const words = splitWhitespace(trim(span).replace(/[:：]$/u, ''));
  if (words.length > 1 && bareSurfaces('file_edit_target_cue').includes(words[words.length - 1].toLowerCase())) words.pop();
  return words.join(' ');
}

/**
 * Mirrors `fn anchor_context`: `{text, start, end}` -- the literal a seeded
 * anchor context names (`that follows 'z'`) and the span from the cue
 * through it (UTF-16 offsets), or null.
 * @param {string} sentence
 */
export function anchorContext(sentence) {
  const contexts = cueOccurrences(sentence, 'file_edit_anchor_context_cue');
  if (contexts.length === 0) return null;
  const literals = quotedLiterals(sentence);
  const at = governedLiteral(contexts, literals, literals.map((_, index) => index));
  if (at === null) return null;
  const literal = literals[at];
  // The cue that governs the literal: the nearest on its language's side.
  let cue = null;
  for (const occurrence of contexts) {
    const [cueStart, cueEnd, postpositional] = occurrence;
    const distance = postpositional ? cueStart - literal.end : literal.start - cueEnd;
    if (distance >= 0 && (cue === null || distance < cue[0])) cue = [distance, occurrence];
  }
  const [cueStart, cueEnd, postpositional] = cue[1];
  const start = postpositional ? literal.from : byteToUtf16(sentence, cueStart);
  const end = postpositional ? byteToUtf16(sentence, cueEnd) : literal.to;
  return { text: unescapeProseNewlines(literal.text), start, end };
}

/**
 * Mirrors `fn insert_clauses`: the request cut before every repeated add
 * action that a seeded joiner or a clause mark leads -- after it, in a
 * postpositional language, whose verb closes its clause.
 */
function insertClauses(request) {
  const segments = quotedSegmentSpans(request);
  const toks = tokens(request);
  const quoted = (token) => segments.some((segment) => token.start < segment.end && token.end > segment.start);
  const joiners = bareSurfaces('file_edit_joiner_cue');
  const joins = (token) => token !== undefined && !quoted(token) && joiners.includes(cleanCueToken(token.text));
  const closes = (token) => /[,;\u0964]$/u.test(token.text);
  const actions = new Map();
  for (const meaning of meaningsWithRole('coding_member_add_action')) {
    for (const lexeme of meaning.lexemes) {
      for (const word of lexeme.words) actions.set(word.text.toLowerCase(), usesPostpositions(lexeme.language));
    }
  }
  // Only a repeated action opens a clause: the first one (the last, where the
  // verb closes its clause) belongs to the clause before it.
  const verbs = toks.map((token, index) => [index, actions.get(cleanCueToken(token.text))])
    .filter(([index, postpositional]) => postpositional !== undefined && !quoted(toks[index]));
  const cuts = [];
  verbs.forEach(([index, postpositional], at) => {
    const token = toks[index];
    if (!postpositional && at > 0) {
      if (joins(toks[index - 1])) cuts.push([toks[index - 1].start, toks[index - 1].end]);
      else if (closes(toks[index - 1])) cuts.push([token.start, token.start]);
    } else if (postpositional && at + 1 < verbs.length) {
      if (joins(toks[index + 1])) cuts.push([toks[index + 1].start, toks[index + 1].end]);
      else if (closes(token)) cuts.push([token.end, token.end]);
    }
  });
  const clauses = [];
  let from = 0;
  for (const [start, end] of cuts) {
    // Two verbs may claim one joiner; the second claim is already made.
    if (start < from) continue;
    clauses.push(request.slice(from, start));
    from = end;
  }
  clauses.push(request.slice(from));
  return clauses.map((clause) => clause.trim()).filter((clause) => clause !== '');
}

/**
 * Mirrors `fn joined_only`: consecutive inserted literals are separated by
 * nothing but whitespace, commas and seeded joiners (`'a' and 'b'`).
 */
/**
 * Mirrors `fn joined_literal_lines`: the request's quoted literals other than
 * `target`, one line each, when there are several and only seeded joiners and
 * commas separate them (`Append the lines 'a', 'b' to f`, PR #1188 G54); an
 * escape inside one is content (G53).
 */
export function joinedLiteralLines(request, target) {
  const literals = quotedLiterals(request).filter((literal) => literal.text !== target);
  if (literals.length < 2 || !joinedOnly(request, literals)) return null;
  return literals.map((literal) => literal.text).join('\n');
}

function joinedOnly(sentence, literals) {
  const joiners = bareSurfaces('file_edit_joiner_cue');
  return literals.slice(1).every((literal, index) => splitWhitespace(sentence.slice(literals[index].to, literal.from))
    .map(cleanCueToken).every((word) => word === '' || joiners.includes(word)));
}

/**
 * Mirrors `fn cue_occurrences`: `[start, end, postpositional]` in UTF-8 bytes
 * for every surface of `role` in `sentence` outside its quoted literals, each
 * with the side its own language's adposition governs.
 */
function cueOccurrences(sentence, role) {
  const lowered = sentence.toLowerCase();
  const literals = quotedLiterals(sentence);
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
  return occurrences;
}

/**
 * Mirrors `fn named_paths`: the workspace paths `sentence` names, in order --
 * those it leaves unquoted, or, when it leaves none, the quoted literals that
 * are exactly one path. In `Insert the line 'foo' after the line
 * 'js/ocr.bundle.js' in paths.txt.` the quoted path is the anchor and
 * `paths.txt` the file (PR #1188 G21).
 * @param {string} sentence
 */
function namedPaths(sentence) {
  const segments = quotedSegmentSpans(sentence);
  const quoted = (token) => segments.some((segment) => token.start < segment.end && token.end > segment.start);
  const paths = unquotedPathTokens(sentence)
    .map((token) => ({ quoted: quoted(token), path: cleanPathToken(token.text) }))
    .filter(({ path }) => looksLikeFilePath(path) && safeRelativePath(path));
  const bare = paths.filter((candidate) => !candidate.quoted);
  return (bare.length > 0 ? bare : paths).map(({ path }) => path);
}

/**
 * Mirrors `fn rebased_block`: `text` with `indentation` before each of its
 * non-empty lines -- lines given with their shared indentation removed, set
 * as siblings of a line indented by `indentation` (PR #1188 G16).
 * @param {string} text
 * @param {string} indentation
 */
export function rebasedBlock(text, indentation) {
  return text.split('\n').map((line) => (line === '' ? line : `${indentation}${line}`)).join('\n');
}

/**
 * Mirrors `fn indents_lines_at`: whether `source` has a non-blank line
 * indented by exactly `indentation`; never for the empty indentation.
 * @param {string} source
 * @param {string} indentation
 */
export function indentsLinesAt(source, indentation) {
  if (indentation === '') return false;
  return source.split('\n').some((line) => line.trim() !== '' && leadingIndentation(line) === indentation);
}

/** Mirrors `fn leading_indentation`: the whitespace a line starts with. */
export function leadingIndentation(line) {
  return line.slice(0, line.length - line.trimStart().length);
}

/**
 * Mirrors `fn instruction_end`: where the instruction of `request` ends -- at
 * the end of its first line when that line ends in a colon and lines follow
 * it, since those lines are the payload, whatever they say (PR #1188 G94, G95);
 * else at the end of the request.
 * @param {string} request
 */
export function instructionEnd(request) {
  const block = introducedBlock(request);
  return block === null ? request.length : block.head.length;
}

/**
 * Mirrors `fn introduced_block`: a request whose first line ends in a colon
 * and is followed by lines -- `{head, text, verbatim, indentation}`, the
 * lines with their shared `indentation` removed (one quoted literal stands
 * for itself; a fenced block keeps its own, and both are `verbatim`), or null.
 */
export function introducedBlock(request) {
  const breakAt = request.indexOf('\n');
  if (breakAt < 0) return null;
  const head = request.slice(0, breakAt).trimEnd();
  if (!head.endsWith(':')) return null;
  const lines = request.slice(breakAt + 1).split('\n').map((line) => line.trimEnd());
  while (lines.length > 0 && lines[0] === '') lines.shift();
  while (lines.length > 0 && lines[lines.length - 1] === '') lines.pop();
  if (lines.length === 0) return null;
  const indentOf = (line) => line.length - line.trimStart().length;
  // A fenced block (```` ``` ```` lines around it) is kept as written, less
  // the fence's own indentation: its indentation is the text's.
  const fenced = lines.length >= 2 && lines[0].trimStart().startsWith(FENCE) && lines[lines.length - 1].trim() === FENCE;
  const body = fenced ? lines.slice(1, -1) : lines;
  const shared = fenced ? indentOf(lines[0]) : Math.min(...lines.filter((line) => line !== '').map(indentOf));
  const text = body.map((line) => line.slice(Math.min(shared, indentOf(line)))).join('\n');
  const quoted = fenced ? null : quotedVerbatim(text);
  const indentation = fenced ? '' : leadingIndentation(lines.find((line) => line !== ''));
  return { head, text: quoted ?? text, verbatim: fenced || quoted !== null, indentation: indentation.slice(0, shared) };
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
    from: segment.start,
    to: segment.end,
    text: segment.text,
  }));
}

/**
 * Mirrors `fn governed_literal`: which of the `candidates` (literal indices) a
 * cue governs -- the nearest on the side its language's adposition faces, a
 * later one on a tie -- or null when the cue occurs but governs none (a time,
 * not a position: issue #1069). With no cue at all, the last candidate.
 */
function governedLiteral(occurrences, literals, candidates) {
  const gap = (literal) => {
    let best = null;
    for (const [cueStart, cueEnd, postpositional] of occurrences) {
      const distance = postpositional ? cueStart - literal.end : literal.start - cueEnd;
      if (distance >= 0 && (best === null || distance < best)) best = distance;
    }
    return best;
  };
  let governed = null;
  let nearest = null;
  for (const index of candidates) {
    const distance = gap(literals[index]);
    if (distance !== null && (nearest === null || distance <= nearest)) {
      governed = index;
      nearest = distance;
    }
  }
  if (governed !== null) return governed;
  return occurrences.length > 0 || candidates.length === 0 ? null : candidates[candidates.length - 1];
}

const FENCE = '```';
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
 * Mirrors `fn own_text`: the words a request says itself -- the head of an edit
 * request whose lines follow it (the lines are its payload), else the request.
 * @param {string} request
 */
export function ownText(request) {
  const block = introducedBlock(request);
  return block !== null && namesLocalEdit(block.head) ? block.head : request;
}

/**
 * Mirrors `fn names_local_edit` in rust/src/agentic_coding/positional_edit.rs.
 * @param {string} task
 */
export function namesLocalEdit(task) {
  const normalized = normalizePrompt(task);
  const edits = mentionsRole('file_edit_action_cue', normalized) || mentionsRole('coding_member_add_action', normalized);
  return edits && localEditPath(task) !== null;
}

/** Mirrors `fn local_edit_path`: the first workspace path the request names. */
function localEditPath(task) {
  for (const token of tokens(task)) {
    const candidate = cleanPathToken(token.text);
    if (looksLikeFilePath(candidate) && safeRelativePath(candidate)) return candidate;
  }
  return null;
}

/**
 * Mirrors `fn unquoted_addition_path`: the file a request leading with the
 * add verb names when it quotes no text (`add hello to config.txt`, PR #1188
 * G69). Its words may be the text or describe it, so the request earns a
 * question, never a write.
 * @param {string} task
 * @returns {string|null}
 */
export function unquotedAdditionPath(task) {
  if (quotedSegmentSpans(task).length > 0) return null;
  // The request leads with the add verb: in "Create a test for add in t.mjs"
  // `add` is a name, not the action.
  const [lead = ''] = normalizePrompt(task).split(' ');
  return mentionsRole('coding_member_add_action', lead) ? localEditPath(task) : null;
}

export const unescapeProseNewlines = (text) => replaceAllLiteral(replaceAllLiteral(text, '\\n', '\n'), '\\t', '\t');
