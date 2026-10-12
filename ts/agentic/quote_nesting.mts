// A quoted text that holds its own quote mark (PR #1188 G90, G107). With one
// mark for both ends (', ", `), `Replace 'a('x')' with 'b' in f.rs.` pairs
// the outer opening quote with the first inner one. A closing mark is
// followed by a space, the end, or punctuation; one followed straight away by
// a letter, a digit or another quote mark opened an inner quote instead.
// When the inner spans pair among themselves and the payload closes at the
// end of the instruction, the payload runs there, whatever sentences it
// holds (`In f.md, replace `x` with `a. `b` c`.`). When the edit read from
// the request still does not hold such a quote whole, the request is
// declined, naming the quote, rather than guessed: the run had answered a
// fragment, failed with `oldString not found`, or taken the request for a
// whole-file write. The JavaScript twin of rust/src/agentic_coding/quote_nesting.rs.

import { quotedSegmentSpans, wrappedInQuotePair } from './crate/normal_markov.mjs';
import { pathsIn } from './module_function.mjs';
import { instructionEnd } from './positional_edit.mjs';
import { composeEditRequest } from './write_request.mjs';
import { isAsciiAlphanumeric, isWhitespace, trimEnd } from './write_str.mjs';

/** The quote marks that open and close a quoted text alike. */
const SAME_MARKS = ["'", '"', '`'];
/** The mark of a fenced block, whose body may hold any quote. */
const FENCE = '```';
/** How many characters of the request the decline quotes. */
const FAULT_FRAGMENT_CHARS = 32;
/** What may stand right after a closing mark. */
const CLOSE_FOLLOWERS = '.,;:!?)]}\'"`»';
/** What may stand right before an opening mark. */
const OPEN_LEADERS = '([{';

/** Whether the character at `at` opens an inner quote right after a close. */
function opensInner(text, at) {
  const after = text[at];
  return after !== undefined && (isAsciiAlphanumeric(after) || SAME_MARKS.includes(after));
}

/**
 * Mirrors `fn whole_payload_end`: where a new text that starts at `from`
 * ends, given the sentence end `cut`. When the payload cut there is no whole
 * literal, or its closing mark opens an inner quote, its marks are read by
 * their shape: a mark after a space or an opening bracket and before a word
 * opens an inner quote, a mark after a word and before a space or
 * punctuation closes one, and the payload ends where its own opening mark is
 * closed. `cut` when a mark has neither shape or the payload never closes.
 * @param {string} request
 * @param {number} from
 * @param {number} cut
 */
export function wholePayloadEnd(request, from, cut) {
  const payload = request.slice(from, cut).trim();
  if (wrappedInQuotePair(payload) && !opensInner(request, cut)) return cut;
  const open = from + (request.slice(from).length - request.slice(from).trimStart().length);
  const mark = request[open];
  if (!SAME_MARKS.includes(mark) || request.startsWith(FENCE, open)) return cut;
  const end = instructionEnd(request);
  let depth = 1;
  for (let at = open + 1; at < end; at += 1) {
    if (request[at] !== mark) continue;
    const before = request[at - 1];
    const after = request[at + 1];
    const closes = !isWhitespace(before) && (after === undefined || isWhitespace(after) || CLOSE_FOLLOWERS.includes(after));
    const opens = (isWhitespace(before) || OPEN_LEADERS.includes(before)) && after !== undefined && !isWhitespace(after);
    if (opens && !closes) depth += 1;
    else if (closes) depth -= 1;
    else return cut;
    if (depth === 0) return at + 1;
  }
  return cut;
}

/**
 * Mirrors `fn nested_quote_fault`: `{kind, intent, at, fragment}` for the
 * first quoted text of a request naming a file whose closing mark opens an
 * inner quote (a letter, a digit or a quote mark straight after it) and that
 * the old and new texts of the request's edit, if one can be read, do not
 * hold whole; else null.
 * @param {string} text
 */
export function nestedQuoteFault(text) {
  const edit = composeEditRequest(text);
  // A request naming a file whose edit cannot be read at all is declined too.
  if (edit === null && pathsIn(text).length === 0) return null;
  for (const segment of quotedSegmentSpans(text)) {
    if (!SAME_MARKS.includes(text[segment.start]) || text.startsWith(FENCE, segment.start)) continue;
    if (!opensInner(text, segment.end)) continue;
    const across = text.slice(segment.start + 1, segment.end + 1);
    if (edit !== null && (edit[1].includes(across) || edit[2].includes(across))) continue;
    const fragment = trimEnd(Array.from(text.slice(segment.start)).slice(0, FAULT_FRAGMENT_CHARS).join(''));
    return { kind: 'nested', intent: 'request-quote-nested', at: segment.start, fragment };
  }
  return null;
}
