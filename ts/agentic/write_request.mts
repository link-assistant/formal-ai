// Recovering the parts of a write request from its prose
// (rust/src/agentic_coding/write_request.rs).
//
// Byte offsets of the Rust module are JavaScript string indices here; every
// offset is produced and consumed inside the same string, so slicing agrees.
// Sentences come from shell_command_policy.mjs as `{text, span: {start, end}}`.

import { firstRawPrefixLeadEnd } from './write_request/lowercase_spans.mjs';
export { firstRawContentLeadEnd, firstRawPrefixLeadEnd, rawContentLeadClose } from './write_request/lowercase_spans.mjs';
import { isDottedNumber, peelSentencePunctuation } from './file_path_shape.mjs';
import { proseSentences, sentences } from './shell_command_policy.mjs';
import { composePositionalInsert, introducedBlock, literalText, unquotedPathTokens } from './positional_edit.mjs';
import { resolveCensusTarget } from './general_planner.mjs';
import { containsCjk } from './crate/coding_catalog.mjs';
import { quotedSegmentSpans, wrappedInQuotePair } from './crate/normal_markov.mjs';
import { wholePayloadEnd } from './quote_nesting.mjs';
import { meaningEvidencedIn, mentionsRole, roleWordForms, wordsForRole } from './write_lexicon.mjs';
import { normalizePrompt } from './crate/engine.mjs';
import {
  charIn, isAlphanumeric, isAscii, isAsciiPunctuation, isWhitespace, minByKey, trim, trimEnd,
  trimEndMatches, trimMatches, trimStart, trimStartMatches,
} from './write_str.mjs';

/** A sentence's span as `{start, end}`, whichever shape the splitter returns. */
export function spanOf(sentence) {
  const span = sentence.span;
  return Array.isArray(span) ? { start: span[0], end: span[1] } : span;
}

const spanContains = (span, index) => index >= span.start && index < span.end;

/**
 * Mirrors `fn tokens`: whitespace and ideographic punctuation separate tokens;
 * each token is `{text, start, end}`.
 * @param {string} request
 */
export function tokens(request) {
  const out = [];
  let start = null;
  let index = 0;
  for (const character of request) {
    if (isWhitespace(character) || isIdeographicPunctuation(character)) {
      if (start !== null) {
        out.push({ text: request.slice(start, index), start, end: index, request });
        start = null;
      }
    } else if (start === null) {
      start = index;
    }
    index += character.length;
  }
  if (start !== null) out.push({ text: request.slice(start), start, end: request.length, request });
  return out;
}

/** Mirrors `const fn is_ideographic_punctuation`. */
function isIdeographicPunctuation(character) {
  const cp = character.codePointAt(0);
  return (cp >= 0x3001 && cp <= 0x3003) || (cp >= 0x3008 && cp <= 0x3011) || (cp >= 0x3014 && cp <= 0x301f)
    || cp === 0xff01 || cp === 0xff08 || cp === 0xff09 || cp === 0xff0c || cp === 0xff1a || cp === 0xff1b
    || cp === 0xff1f;
}

/** Mirrors `fn bare_surfaces`. @param {string} role @returns {Array<string>} */
export function bareSurfaces(role) {
  return roleWordForms(role).filter((form) => form.slot === 'bare').map((form) => form.text.toLowerCase());
}

/** Mirrors `fn clean_path_token`. @param {string} word */
export function cleanPathToken(word) {
  return peelSentencePunctuation(word, (token) =>
    trimEndMatches(trimMatches(token, charIn('`"\',:;')), charIn('!?')));
}

/** Mirrors `fn looks_like_file_path`. @param {string} path */
export function looksLikeFilePath(path) {
  const fileName = path.split('/').pop();
  // A component of dots alone (`.`, `..`) names a directory, never a file.
  return !path.includes('://') && !isDottedNumber(path) && fileName.includes('.') && /[^.]/u.test(fileName);
}

/** Mirrors `fn clean_cue_token`. @param {string} word */
export function cleanCueToken(word) {
  return trimMatches(word, charIn('`"\',:;.!?।॥')).toLowerCase();
}

/** Mirrors `fn first_content_lead_end`. @returns {[number, number]|null} */
export function firstContentLeadEnd(lowered) {
  return firstPrefixLeadEnd(lowered, 'file_write_content_lead');
}

/** Mirrors `fn content_lead_close`. @returns {number|null} */
export function contentLeadClose(lowered, from) {
  let best = null;
  for (const form of roleWordForms('file_write_content_lead')) {
    if (form.slot !== 'circumfix') continue;
    const opener = trim(form.before).toLowerCase();
    const closer = trim(form.after).toLowerCase();
    if (!opener || !closer || !trimEnd(lowered.slice(0, from)).endsWith(opener)) continue;
    const relative = lowered.slice(from).indexOf(closer);
    if (relative < 0) continue;
    if (best === null || from + relative < best) best = from + relative;
  }
  return best;
}

/** Mirrors `fn first_prefix_lead_end`. @returns {[number, number]|null} */
export function firstPrefixLeadEnd(lowered, role) {
  const markers = roleWordForms(role)
    .filter((form) => form.slot === 'prefix' || form.slot === 'circumfix')
    .map((form) => [trim(form.before).toLowerCase(), trim(form.after).toLowerCase()])
    .filter(([marker]) => marker !== '');
  let best = null;
  for (const [marker, closer] of markers) {
    let from = 0;
    for (;;) {
      const start = lowered.indexOf(marker, from);
      if (start < 0) break;
      const end = start + marker.length;
      const cjk = !marker.includes(' ') && !isAscii(marker);
      const before = Array.from(lowered.slice(0, start)).pop();
      const after = Array.from(lowered.slice(end, end + 2))[0];
      const beforeOk = cjk || start === 0 || isWhitespace(before);
      const afterOk = cjk || end === lowered.length || isWhitespace(after) || isAsciiPunctuation(after);
      const closed = closer === '' || lowered.slice(end).includes(closer);
      if (beforeOk && afterOk && closed) {
        if (best === null || start < best[0] || (start === best[0] && end > best[1])) best = [start, end];
        break;
      }
      from = end;
    }
  }
  return best;
}

/** `CueFamily` variants. */
export const CueFamily = Object.freeze({ Target: 'target', Destination: 'destination', Action: 'action' });

/** Mirrors `fn cued_write_target`: `[index, path]` or null. */
export function cuedWriteTarget(toks) {
  const binding = preferredBinding(toks);
  return binding ? [binding.index, binding.path] : null;
}

/** Mirrors `fn ranked_bindings` (a stable sort by rank). */
export function rankedBindings(toks) {
  return writeBindings(toks)
    .map((binding, order) => ({ binding, order }))
    .sort((left, right) => bindingRank(left.binding) - bindingRank(right.binding) || left.order - right.order)
    .map(({ binding }) => binding);
}

/** Mirrors `fn preferred_binding`. */
export function preferredBinding(toks) {
  return rankedBindings(toks)[0] ?? null;
}

function bindingRank(binding) {
  if (binding.cue_precedes) return binding.family === CueFamily.Action ? 1 : 0;
  return 2;
}

/** Mirrors `fn cued_write_targets`: every `[index, path]` whose cue precedes it. */
export function cuedWriteTargets(toks) {
  return writeBindings(toks).filter((binding) => binding.cue_precedes).map((binding) => [binding.index, binding.path]);
}

/** Mirrors `fn write_bindings`. */
function writeBindings(toks) {
  const families = [
    [CueFamily.Destination, bareSurfaces('file_write_destination_cue'), true],
    [CueFamily.Target, bareSurfaces('file_write_target_cue'), true],
    [CueFamily.Action, bareSurfaces('file_write_action_cue'), false],
  ];
  const out = [];
  const quoted = quotedSegmentSpans(toks[0]?.request ?? '');
  toks.forEach((token, index) => {
    const cleaned = cleanPathToken(token.text);
    if (!looksLikeFilePath(cleaned) || !safeRelativePath(cleaned)) return;
    if (index > 0) {
      for (const [family, cues, fused] of families) {
        const span = trailingCue(toks[index - 1], cues, fused);
        if (span) {
          out.push({ index, path: cleaned, cue_start: span[0], cue_end: span[1], family, cue_precedes: true });
          return;
        }
      }
    }
    const next = toks[index + 1];
    if (!next) return;
    for (const [family, cues, fused] of families.slice(0, 2)) {
      const span = leadingCue(next, cues, fused);
      if (span) {
        out.push({ index, path: cleaned, cue_start: span[0], cue_end: span[1], family, cue_precedes: false });
        return;
      }
    }
  });
  return out.filter((binding) => !quoted.some((segment) =>
    binding.cue_start < segment.end && binding.cue_end > segment.start));
}

function trailingCue(token, cues, fused) {
  const cleaned = cleanCueToken(token.text);
  for (const cue of cues) {
    if (cleaned === cue) return [token.start, token.end];
    if (fused && containsCjk(cue) && cleaned.endsWith(cue)) return [Math.max(0, token.end - cue.length), token.end];
  }
  return null;
}

function leadingCue(token, cues, fused) {
  const cleaned = cleanCueToken(token.text);
  for (const cue of cues) {
    if (cleaned === cue) return [token.start, token.end];
    if (fused && containsCjk(cue) && cleaned.startsWith(cue)) return [token.start, token.start + cue.length];
  }
  return null;
}

/** Mirrors `fn quoted_as_value`. */
function quotedAsValue(word) {
  const bare = peelSentencePunctuation(word, (token) => trimEndMatches(token, charIn(',:;')));
  return bare.length >= 2 && bare.startsWith('"') && bare.endsWith('"');
}

/** Mirrors `fn stated_write_target`. @param {string} request @returns {string|null} */
export function statedWriteTarget(request) {
  return cuedWriteTarget(tokens(request))?.[1] ?? null;
}

/** Mirrors `fn typed_write_target`. @param {string} request @param {string} extension */
export function typedWriteTarget(request, extension) {
  return cuedWriteTargets(tokens(request)).map(([, path]) => path).find((path) => pathExtension(path) === extension) ?? null;
}

/** `Path::extension` of a relative path, or null: Rust built-in `Path::extension`. */
export function pathExtension(path) {
  const name = path.split('/').filter(Boolean).pop() ?? '';
  if (name === '..') return null;
  const dot = name.lastIndexOf('.');
  if (dot <= 0) return null;
  return name.slice(dot + 1);
}

/** Mirrors `fn states_write_action`. */
export function statesWriteAction(request) {
  return firstActionCueEnd(tokens(request)) !== null;
}

/** Mirrors `fn delivered_write_target`. @param {string} sentence */
export function deliveredWriteTarget(sentence) {
  const toks = tokens(sentence);
  const action = firstActionCueStart(toks);
  if (action === null) return null;
  const found = cuedWriteTargets(toks).find(([index]) => toks[index].start > action && !quotedAsValue(toks[index].text));
  return found ? found[1] : null;
}

/** Mirrors `fn is_stated_write_target`. */
export function isStatedWriteTarget(request, path) {
  return sentences(request).some((sentence) =>
    statesWriteAction(sentence.text) && statedWriteTarget(sentence.text) === path);
}

/** Mirrors `fn pinned_first_line`. @param {string} sentence @returns {string|null} */
export function pinnedFirstLine(sentence) {
  const lead = firstRawPrefixLeadEnd(sentence, 'file_leading_line_constraint_lead');
  if (!lead) return null;
  const raw = trim(trimStartMatches(trim(sentence.slice(lead[1])), charIn(':-—–')));
  const line = trimMatches(delimitedFirstLine(raw) ?? unquotedMachineFirstLine(raw) ?? raw, charIn('`"\''));
  return line ? line : null;
}

function delimitedFirstLine(raw) {
  const delimiter = Array.from(raw)[0];
  if (delimiter !== '`' && delimiter !== '"' && delimiter !== "'") return null;
  const afterOpen = raw.slice(1);
  const close = afterOpen.indexOf(delimiter);
  return close < 0 ? null : afterOpen.slice(0, close);
}

function unquotedMachineFirstLine(raw) {
  const lowered = raw.toLowerCase();
  const candidates = bareSurfaces('skill_procedure_clause_separator').flatMap((separator) => {
    const boundary = lowered.indexOf(` ${separator} `);
    if (boundary < 0) return [];
    const candidate = trim(raw.slice(0, boundary));
    return !/\p{White_Space}/u.test(candidate) && /[=:]/.test(candidate) ? [candidate] : [];
  });
  return minByKey(candidates, (candidate) => candidate.length);
}

/** Mirrors `fn pinned_first_line_of_request`. */
export function pinnedFirstLineOfRequest(request) {
  for (const sentence of sentences(request)) {
    const line = pinnedFirstLine(sentence.text);
    if (line !== null) return line;
  }
  return null;
}

/** Mirrors `fn honouring_pinned_first_line`. @returns {string|null} */
export function honouringPinnedFirstLine(request, content) {
  const line = pinnedFirstLineOfRequest(request);
  if (line === null || content.startsWith(line)) return null;
  const body = withoutPinningSentences(content);
  return body === '' ? `${line}\n` : `${line}\n\n${body}\n`;
}

function withoutPinningSentences(content) {
  return sentences(content)
    .filter((sentence) => pinnedFirstLine(sentence.text) === null)
    .map((sentence) => {
      const span = spanOf(sentence);
      return trim(content.slice(span.start, span.end));
    })
    .filter(Boolean)
    .join(' ');
}

function firstActionCue(toks) {
  const actions = bareSurfaces('file_write_action_cue');
  for (const token of toks) {
    const span = leadingCue(token, actions, false);
    if (span) return span;
  }
  return null;
}

/** Mirrors `fn first_action_cue_end`. @returns {number|null} */
export function firstActionCueEnd(toks) {
  return firstActionCue(toks)?.[1] ?? null;
}

/** Mirrors `fn action_cue_start_after`. @returns {number|null} */
export function actionCueStartAfter(toks, from) {
  const actions = bareSurfaces('file_write_action_cue');
  for (const token of toks) {
    if (token.start < from) continue;
    const span = leadingCue(token, actions, false);
    if (span) return span[0];
  }
  return null;
}

/** Mirrors `fn first_action_cue_start`. @returns {number|null} */
export function firstActionCueStart(toks) {
  return firstActionCue(toks)?.[0] ?? null;
}

/** Mirrors `fn clean_content`. @param {string} raw @returns {string|null} */
export function cleanContent(raw) {
  const led = stripClauseLead(raw);
  const fence = /^`{3,}/u.exec(led)?.[0];
  if (fence && led.length >= fence.length * 2 && led.endsWith(fence)) {
    return fencedBody(led.slice(fence.length, led.length - fence.length)) || null;
  }
  // One quoted literal, in any pair of quotes (`'a'`, «a», “a”; PR #1188 G100),
  // is the content; the sentence's closing mark after it is the sentence's:
  // `containing 'hello'.` writes `hello`, not `'hello'.`.
  const closed = trim(led.replace(/[.!?\u0964\u3002\uff01\uff1f]$/u, ''));
  const [only, ...others] = quotedSegmentSpans(closed);
  if (only && others.length === 0 && only.start === 0 && only.end === closed.length) {
    return only.text.length > 0 ? only.text : null;
  }
  let result = led;
  const bytes = new TextEncoder().encode(led);
  if (bytes.length >= 2) {
    const first = bytes[0];
    const last = bytes[bytes.length - 1];
    if (first === last && (first === 0x60 || first === 0x22 || first === 0x27)) result = trim(led.slice(1, -1));
  }
  return result ? result : null;
}

/**
 * Mirrors `fn fenced_body`: the bytes of a fenced block. The opening line's
 * info string names the language and is not content, and the line break
 * before the closing fence ends the last line (PR #1188 G101). A fence on one
 * line is its trimmed text.
 * @param {string} inner the text between the fences
 */
function fencedBody(inner) {
  const newline = inner.indexOf('\n');
  if (newline < 0 || /\s/u.test(trim(inner.slice(0, newline)))) return trim(inner);
  return inner.slice(newline + 1);
}

function stripClauseLead(raw) {
  const modifiers = bareSurfaces('file_write_content_qualifier');
  const qualifiers = [...modifiers];
  for (const role of ['file_write_content_lead', 'file_write_authoritative_content_lead']) {
    for (const form of roleWordForms(role).filter((item) => item.slot === 'prefix')) {
      const lead = trim(form.before).toLowerCase();
      for (const modifier of modifiers) {
        const start = lead.indexOf(modifier);
        if (start >= 0 && (start === 0 || isWhitespace(lead[start - 1]))) qualifiers.push(lead.slice(start));
      }
      qualifiers.push(lead);
    }
  }
  let led = trim(raw);
  for (;;) {
    const separated = trim(trimStartMatches(led, charIn(':-—–')));
    const shortened = stripLeadingQualifier(separated, qualifiers);
    if (shortened.length === led.length) return led;
    led = shortened;
  }
}

function stripLeadingQualifier(text, qualifiers) {
  const lowered = text.toLowerCase();
  const rests = qualifiers
    .filter((qualifier) => lowered.startsWith(qualifier))
    .map((qualifier) => trimStart(text.slice(qualifier.length)))
    .filter((rest) => /^[:\-—–]/u.test(rest));
  return minByKey(rests, (rest) => new TextEncoder().encode(rest).length) ?? text;
}

/** Mirrors `fn safe_relative_path`. @param {string} path */
export function safeRelativePath(path) {
  return !path.startsWith('/')
    && !path.startsWith('-')
    && !path.split('/').some((part) => part === '..' || part === '')
    && Array.from(path).every((character) => isAlphanumeric(character) || '/._-'.includes(character));
}

/**
 * Mirrors `fn compose_edit_request`: `[target, old, new]` or null.
 * @param {string} request
 */
export function composeEditRequest(request) {
  return composeEditClauses(request)?.edit ?? null;
}

/** The seeded articles and other function words (`the`, `el`). */
const FUNCTION_WORD_ROLE = 'request_function_word';
const SCRIPT_WITHOUT_SPACES = /[\u3040-\u9fff]/u;
const wordCharacter = (character) => character !== undefined && /[\p{L}\p{N}]/u.test(character)
  && !SCRIPT_WITHOUT_SPACES.test(character);

/**
 * Mirrors `fn without_all_occurrence_cues`: `request` with every seeded
 * all-occurrences cue outside its quotes (`everywhere`, `all occurrences`,
 * `везде`, `हर जगह`) blanked, offsets kept: a replace replaces every
 * occurrence already, and the words belong to neither text (PR #1188 G84).
 * @param {string} request
 */
export function withoutAllOccurrenceCues(request) {
  const lowered = request.toLowerCase();
  if (lowered.length !== request.length) return request;
  const quoted = quotedSegmentSpans(request);
  let out = request;
  for (const surface of wordsForRole('file_edit_all_occurrences_cue')) {
    const needle = surface.toLowerCase();
    if (needle === '') continue;
    for (let at = lowered.indexOf(needle); at >= 0; at = lowered.indexOf(needle, at + 1)) {
      const end = at + needle.length;
      if (wordCharacter(request[at - 1]) || wordCharacter(request[end])) continue;
      if (quoted.some((segment) => at < segment.end && end > segment.start)) continue;
      out = `${out.slice(0, at)}${' '.repeat(end - at)}${out.slice(end)}`;
    }
  }
  return out;
}

/**
 * Mirrors `fn compose_edit_clauses`: `{edit, spans}` -- the `[target, old,
 * new]` of `composeEditRequest` and, when the request words them as clauses,
 * the spans its file clause and its action through the new text take (null
 * for a positional insert or lines under the request) -- or null.
 * @param {string} request
 */
export function composeEditClauses(raw) {
  // `everywhere`, `all occurrences`: no part of the old or new text (G84).
  const request = withoutAllOccurrenceCues(raw);
  const positional = composePositionalInsert(request);
  if (positional) return { edit: positional, spans: null };
  const block = introducedBlock(request);
  if (block !== null) {
    // `Replace the line 'x' with these three lines in f:` followed by lines:
    // a new clause that only describes lines (the seeded `line` meaning,
    // unquoted) stands for the lines under the request (PR #1188 G22).
    const head = composeEditRequest(block.head);
    if (head !== null && describesLines(block.head, head[2])) return { edit: [head[0], head[1], block.text], spans: null };
  }
  const toks = tokens(request);
  const actionCues = bareSurfaces('file_edit_action_cue');
  const newLeads = bareSurfaces('file_edit_new_lead_cue');
  const targetCues = bareSurfaces('file_edit_target_cue');
  const isTargetCue = (index) => targetCues.includes(cleanCueToken(toks[index].text));
  const isActionCue = (index) => actionCues.includes(cleanCueToken(toks[index].text));
  let fileIndex = -1;
  let target = null;
  const unquoted = new Set(unquotedPathTokens(request).map((token) => token.start));
  // A path the request leaves unquoted names the file before a literal that
  // is exactly a path does: in "Replace 'a' with 'data/x.lino' in f.mjs" the
  // quoted path is the new text, and the cue after it ("in") does not make it
  // the target.
  const segments = quotedSegmentSpans(request);
  const isQuoted = (token) => segments.some((segment) => token.start >= segment.start && token.end <= segment.end);
  const isPathCandidate = (token) => {
    const cleaned = cleanPathToken(token.text);
    return unquoted.has(token.start)
      && (resolveCensusTarget(cleaned) !== null || (looksLikeFilePath(cleaned) && safeRelativePath(cleaned)));
  };
  for (const quotedPass of [false, true]) {
    for (let index = 0; index < toks.length; index += 1) {
      if (!unquoted.has(toks[index].start) || isQuoted(toks[index]) !== quotedPass) continue;
      const cleaned = cleanPathToken(toks[index].text);
      const resolved = resolveCensusTarget(cleaned);
      if (resolved === null && (!looksLikeFilePath(cleaned) || !safeRelativePath(cleaned))) continue;
      const prevIsCue = index > 0 && (isTargetCue(index - 1) || isActionCue(index - 1));
      const nextIsCue = index + 1 < toks.length && (isTargetCue(index + 1) || isActionCue(index + 1));
      // A cue between two paths introduces the later one: in "…`p.lino`` in
      // req.md" the payload's last span is no target (PR #1188 G63).
      const cueIntroducesNext = !prevIsCue && nextIsCue && index + 2 < toks.length && isPathCandidate(toks[index + 2]);
      if ((prevIsCue || nextIsCue) && !cueIntroducesNext) {
        fileIndex = index;
        target = resolved === null ? cleaned : resolved.module_path;
        break;
      }
    }
    if (fileIndex >= 0) break;
  }
  if (fileIndex < 0) return null;
  // The file clause runs back over target cues and the seeded function words
  // between them (`in the file f.txt`, `en el archivo f.txt`) to its first
  // cue; an article there is no part of the new text (PR #1188 G98). A cue
  // word inside a quoted literal is payload ("… the named file.'").
  const functionWords = bareSurfaces(FUNCTION_WORD_ROLE);
  const joinsClause = (index) => !isQuoted(toks[index])
    && (isTargetCue(index) || functionWords.includes(cleanCueToken(toks[index].text)));
  let clauseStartIndex = fileIndex;
  for (let index = fileIndex - 1; index >= 0 && joinsClause(index); index -= 1) {
    if (isTargetCue(index)) clauseStartIndex = index;
  }
  const fileClauseStart = toks[clauseStartIndex].start;
  // Cue words inside a quoted literal are payload (`replace 'covered by x'`).
  // A cue word quoted whole is payload too: `replace 'with' with ','`.
  const isCue = (token, cues) => unquoted.has(token.start) && !isQuoted(token) && cues.includes(cleanCueToken(token.text));
  const actionTokens = toks.filter((token) => isCue(token, actionCues));
  const action = actionTokens.find((token) => token.start > toks[fileIndex].end) ?? actionTokens[0];
  if (!action) return null;
  const actionEnd = action.end;
  const newLead = toks.find((token) => token.start >= actionEnd && isCue(token, newLeads));
  if (!newLead) return null;
  // `Bump the version in package.json to 1.1.0`: a file clause right before
  // the new lead ends the old text; anything else between them is no edit.
  const fileBetween = fileClauseStart >= actionEnd && fileClauseStart < newLead.start;
  if (fileBetween && request.slice(toks[fileIndex].end, newLead.start).trim() !== '') return null;
  const oldSpan = request.slice(actionEnd, fileBetween ? fileClauseStart : newLead.start);
  const containing = proseSentences(request).find((sentence) => spanContains(spanOf(sentence), newLead.end));
  let sentenceEnd = request.length;
  if (containing) {
    const span = spanOf(containing);
    const raw = request.slice(span.start, span.end);
    sentenceEnd = span.start + (raw.length - trimStart(raw).length) + containing.text.length;
  }
  // A sentence boundary inside a quoted literal (`'… mod.rs`.'`) is payload:
  // the new text runs to the end of the literal it falls in.
  const literalAround = quotedSegmentSpans(request)
    .find((segment) => segment.start < sentenceEnd && sentenceEnd < segment.end);
  // When the literal holding that boundary ends early, because the backtick
  // spans inside the payload paired among themselves, a payload quoted whole
  // up to the file clause runs to it (PR #1188 G63).
  if (literalAround) {
    sentenceEnd = fileClauseStart > literalAround.end && wrappedInQuotePair(request.slice(newLead.end, fileClauseStart))
      ? fileClauseStart : literalAround.end;
  }
  // A payload whose inner spans pair among themselves runs to the end of the
  // instruction when it closes there (PR #1188 G107).
  const newEnd = fileClauseStart > newLead.end
    ? Math.min(fileClauseStart, sentenceEnd)
    : wholePayloadEnd(request, newLead.end, sentenceEnd);
  if (newEnd < newLead.end) return null;
  const newSpan = request.slice(newLead.end, newEnd);
  const oldText = literalText(oldSpan);
  if (oldText === null) return null;
  const newText = literalText(newSpan);
  if (newText === null) return null;
  // `Rename the file m.py to math_utils.py`: an unquoted new name that is a
  // workspace path renames the file itself, which is no edit of its bytes
  // (PR #1188 G24); the shell intents carry it.
  // Quoted, the new path renames the file too when the old clause quotes
  // nothing (`Rename the file 'a.txt' to 'b.txt'`, G37).
  const renamedFile = (quotedSegmentSpans(newSpan).length === 0 || quotedSegmentSpans(oldSpan).length === 0)
    && looksLikeFilePath(cleanPathToken(trim(newSpan)))
    && mentionsRole('coding_identifier_rename_action', request.toLowerCase());
  if (renamedFile) return null;
  return { edit: [target, oldText, newText], spans: [[fileClauseStart, toks[fileIndex].end], [action.start, newEnd]] };
}

/**
 * Mirrors `fn describes_lines`: whether a clause `text` of `head` is no quoted
 * literal but words naming lines (`these three lines`, `the following
 * lines`, `эти строки`).
 */
function describesLines(head, text) {
  return !quotedSegmentSpans(head).some((segment) => segment.text === text)
    && meaningEvidencedIn('line', normalizePrompt(text).toLowerCase());
}

/** Mirrors `fn payload_continues_past_its_first_line`. */
export function payloadContinuesPastItsFirstLine(request, from, sentenceEnd) {
  if (from > request.length) return false;
  const tail = request.slice(from);
  const breakAt = tail.indexOf('\n');
  if (breakAt < 0) return false;
  return from + breakAt < sentenceEnd && Array.from(tail.slice(breakAt)).some(isAlphanumeric);
}
