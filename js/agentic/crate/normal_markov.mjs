// Bounded normal (Markov) string-rewrite algorithms and literal-slot readers
// (rust/src/normal_markov.rs). Spans are JavaScript string indices; the
// execution trace reports UTF-8 byte offsets, as Rust does.

import { isAsciiAlphanumeric, trim, utf8Len } from '../write_str.mjs';

/** Mirrors `RewriteRule::new` (plus `.terminal()` through `terminal`). */
export function rewriteRule(pattern, replacement, terminal = false) {
  return { pattern, replacement, terminal };
}

/** Mirrors `RewriteProgram::new`. */
export function rewriteProgram(rules, maxSteps) {
  return { rules, max_steps: maxSteps };
}

/**
 * Mirrors `RewriteProgram::execute`: `{output, trace: [{rule_index,
 * byte_offset}], halt: {kind: 'no_applicable_rule'|'terminal_rule'|'step_limit', index?}}`.
 */
export function executeRewrite(program, input) {
  let output = input;
  const trace = [];
  for (let step = 0; step < program.max_steps; step += 1) {
    let ruleIndex = -1;
    let at = -1;
    for (let index = 0; index < program.rules.length; index += 1) {
      const found = output.indexOf(program.rules[index].pattern);
      if (found >= 0) {
        ruleIndex = index;
        at = found;
        break;
      }
    }
    if (ruleIndex < 0) return { output, trace, halt: { kind: 'no_applicable_rule' } };
    const rule = program.rules[ruleIndex];
    const byteOffset = utf8Len(output.slice(0, at));
    output = output.slice(0, at) + rule.replacement + output.slice(at + rule.pattern.length);
    trace.push({ rule_index: ruleIndex, byte_offset: byteOffset });
    if (rule.terminal) return { output, trace, halt: { kind: 'terminal_rule', index: ruleIndex } };
  }
  return { output, trace, halt: { kind: 'step_limit' } };
}

/** Mirrors `fn quoted_segments`. @param {string} text @returns {Array<string>} */
export function quotedSegments(text) {
  return quotedSegmentSpans(text).map((segment) => segment.text);
}

/**
 * Mirrors `fn quoted_segment_spans`: `{text, start, end}` with `start` at the
 * opening delimiter and `end` just past the closing one.
 * @param {string} text
 */
export function quotedSegmentSpans(text) {
  const result = [];
  let cursor = 0;
  while (cursor < text.length) {
    const next = nextDelimiter(text, cursor);
    if (!next) break;
    const [openAt, open, close] = next;
    const contentStart = openAt + open.length;
    const contentEnd = closingDelimiter(text, contentStart, close);
    if (contentEnd === null) break;
    const segmentEnd = contentEnd + close.length;
    result.push({ text: text.slice(contentStart, contentEnd), start: openAt, end: segmentEnd });
    cursor = segmentEnd;
  }
  return result;
}

/** Mirrors `fn unwrap_transport_quotes`. @param {string} text */
export function unwrapTransportQuotes(text) {
  const trimmed = trim(text);
  for (const quote of ['"', "'"]) {
    // A lone quote has no inner text: `strip_prefix` then `strip_suffix` fails.
    if (trimmed.length >= 2 && trimmed.startsWith(quote) && trimmed.endsWith(quote)) {
      const inner = trimmed.slice(1, -1);
      if (!inner.includes(quote)) return inner;
    }
  }
  return trimmed;
}

const PAIRS = [
  ['```', '```'], ["'", "'"], ['"', '"'], ['`', '`'], ['«', '»'],
  ['“', '”'], ['‘', '’'], ['「', '」'], ['『', '』'], ['《', '》'],
];

function nextDelimiter(text, cursor) {
  let best = null;
  for (const [open, close] of PAIRS) {
    const found = nextCompletePair(text, cursor, open, close);
    if (!found) continue;
    if (best === null || found[0] < best[0] || (found[0] === best[0] && found[1].length > best[1].length)) best = found;
  }
  return best;
}

function previousChar(text, index) {
  const before = Array.from(text.slice(0, index));
  return before[before.length - 1];
}

function nextCompletePair(text, cursor, open, close) {
  let from = cursor;
  for (;;) {
    const openAt = text.indexOf(open, from);
    if (openAt < 0) return null;
    const previousIsAsciiWord = open === "'" && isAsciiAlphanumeric(previousChar(text, openAt));
    const contentStart = openAt + open.length;
    if (!previousIsAsciiWord && closingDelimiter(text, contentStart, close) !== null) return [openAt, open, close];
    from = contentStart;
  }
}

/**
 * Mirrors `fn closing_delimiter`: the first close that is not an apostrophe
 * inside a word, or, for a single quote, the one that closes a payload quoting
 * a single-quoted literal of its own (`'- G1 "Delete the line 'x'." -'`,
 * PR #1188 G61): a quote with a space before it and a word character after it
 * opens a nested literal there, and the payload closes after that literal
 * does. When such nesting never closes, the first close stands.
 */
function closingDelimiter(text, cursor, close) {
  return close === "'" ? nestedClosingDelimiter(text, cursor) ?? plainClosingDelimiter(text, cursor, close)
    : plainClosingDelimiter(text, cursor, close);
}

function plainClosingDelimiter(text, cursor, close) {
  let from = cursor;
  for (;;) {
    const closeAt = text.indexOf(close, from);
    if (closeAt < 0) return null;
    const afterClose = closeAt + close.length;
    const isAsciiApostrophe = close === "'"
      && isAsciiAlphanumeric(previousChar(text, closeAt))
      && isAsciiAlphanumeric(Array.from(text.slice(afterClose, afterClose + 2))[0]);
    if (!isAsciiApostrophe) return closeAt;
    from = afterClose;
  }
}

/** Mirrors `fn nested_closing_delimiter`. */
function nestedClosingDelimiter(text, cursor) {
  let depth = 0;
  let from = cursor;
  for (;;) {
    const closeAt = text.indexOf("'", from);
    if (closeAt < 0) return null;
    from = closeAt + 1;
    const before = previousChar(text, closeAt);
    const after = Array.from(text.slice(from, from + 2))[0];
    if (isAsciiAlphanumeric(before) && isAsciiAlphanumeric(after)) continue;
    if (before !== undefined && /\s/u.test(before) && isAsciiAlphanumeric(after)) depth += 1;
    else if (depth === 0) return closeAt;
    else depth -= 1;
  }
}
