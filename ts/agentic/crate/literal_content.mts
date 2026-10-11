// Shared raw literal content and cue boundaries; no planner dependency.
import { roleWordForms } from '../write_lexicon.mjs';
import { quotedSegmentSpans } from './normal_markov.mjs';
import { proseSentences } from '../shell_command_policy.mjs';
import { charIn, isAlphanumeric, isAscii, isAsciiPunctuation, isWhitespace, minByKey, trim, trimEnd, trimStart, trimStartMatches } from '../write_str.mjs';
const bareSurfaces = (role) => roleWordForms(role).filter((form) => form.slot === 'bare').map((form) => form.text.toLowerCase());

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

/** Mirrors `fn payload_continues_past_its_first_line`. */
export function payloadContinuesPastItsFirstLine(request, from, sentenceEnd) {
  if (from > request.length) return false;
  const tail = request.slice(from);
  const breakAt = tail.indexOf('\n');
  if (breakAt < 0) return false;
  return from + breakAt < sentenceEnd && Array.from(tail.slice(breakAt)).some(isAlphanumeric);
}

/** Mirrors fn end_of_statement in general_planner/literal_request.rs. */
export function endOfStatement(request, from, limit) {
  let scoped = request;
  for (const segment of quotedSegmentSpans(request)) {
    scoped = scoped.slice(0, segment.start) + ' '.repeat(segment.end - segment.start) + scoped.slice(segment.end);
  }
  const sentence = proseSentences(scoped).find((candidate) => from >= candidate.span.start && from < candidate.span.end);
  if (!sentence) return limit;
  const end = sentence.span.end;
  const tail = from <= end && end <= request.length ? request.slice(from, end) : null;
  if (tail !== null && Array.from(tail).some(isAlphanumeric) && !payloadContinuesPastItsFirstLine(request, from, end)) {
    let terminal = end;
    while (terminal < request.length && /[.!?。！？।]/u.test(request[terminal])) terminal += 1;
    return Math.min(terminal, limit);
  }
  return limit;
}
