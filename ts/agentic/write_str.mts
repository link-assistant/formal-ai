// Rust `str` / `char` semantics the write-side planner modules share
// (agent "write" helper; no Rust module of its own). Offsets are JavaScript
// string indices (UTF-16 code units); `utf8Offset` converts one to the UTF-8
// byte offset Rust would report where a byte offset reaches the output.

const WHITESPACE = /^\p{White_Space}$/u;
const ALPHANUMERIC = /^[\p{Alphabetic}\p{N}]$/u;
const encoder = new TextEncoder();

/** `char::is_whitespace`. @param {string|undefined} character */
export function isWhitespace(character) {
  return character !== undefined && WHITESPACE.test(character);
}

/** `char::is_alphanumeric`. @param {string|undefined} character */
export function isAlphanumeric(character) {
  return character !== undefined && ALPHANUMERIC.test(character);
}

/** `char::is_ascii_alphanumeric`. @param {string|undefined} character */
export function isAsciiAlphanumeric(character) {
  return character !== undefined && /^[A-Za-z0-9]$/.test(character);
}

/** `char::is_ascii_digit`. @param {string|undefined} character */
export function isAsciiDigit(character) {
  return character !== undefined && /^[0-9]$/.test(character);
}

/** `char::is_ascii_punctuation`. @param {string|undefined} character */
export function isAsciiPunctuation(character) {
  return character !== undefined && /^[!-/:-@[-`{-~]$/.test(character);
}

/** `str::is_ascii`. @param {string} text */
export function isAscii(text) {
  return /^[\x00-\x7f]*$/.test(text);
}

/** `str::len`: UTF-8 byte length. @param {string} text */
export function utf8Len(text) {
  return encoder.encode(text).length;
}

/** The UTF-8 byte offset of JS index `index` in `text`. */
export function utf8Offset(text, index) {
  return utf8Len(text.slice(0, index));
}

/** `str::trim_start_matches(predicate)`. */
export function trimStartMatches(text, predicate) {
  const chars = Array.from(text);
  let start = 0;
  while (start < chars.length && predicate(chars[start])) start += 1;
  return chars.slice(start).join('');
}

/** `str::trim_end_matches(predicate)`. */
export function trimEndMatches(text, predicate) {
  const chars = Array.from(text);
  let end = chars.length;
  while (end > 0 && predicate(chars[end - 1])) end -= 1;
  return chars.slice(0, end).join('');
}

/** `str::trim_matches(predicate)`. */
export function trimMatches(text, predicate) {
  return trimEndMatches(trimStartMatches(text, predicate), predicate);
}

/** A predicate over a set of characters. @param {string|Array<string>} set */
export function charIn(set) {
  const members = new Set(Array.isArray(set) ? set : Array.from(set));
  return (character) => members.has(character);
}

/** `str::trim`. */
export function trim(text) {
  return trimMatches(text, isWhitespace);
}

/** `str::trim_start`. */
export function trimStart(text) {
  return trimStartMatches(text, isWhitespace);
}

/** `str::trim_end`. */
export function trimEnd(text) {
  return trimEndMatches(text, isWhitespace);
}

/** `str::split_whitespace`. @param {string} text */
export function splitWhitespace(text) {
  return text.split(/\p{White_Space}+/u).filter(Boolean);
}

/** The last character of `text`, or undefined. */
export function lastChar(text) {
  const chars = Array.from(text);
  return chars[chars.length - 1];
}

/** The first character of `text`, or undefined. */
export function firstChar(text) {
  return Array.from(text)[0];
}

/** `str::to_lowercase`. */
export function lower(text) {
  return text.toLowerCase();
}

/** `str::strip_prefix`: the rest, or null. */
export function stripPrefix(text, prefix) {
  return text.startsWith(prefix) ? text.slice(prefix.length) : null;
}

/** `str::strip_suffix`: the rest, or null. */
export function stripSuffix(text, suffix) {
  return text.endsWith(suffix) ? text.slice(0, text.length - suffix.length) : null;
}

/** `Iterator::min_by_key`: the first minimum, or null. */
export function minByKey(items, key) {
  let best = null;
  let bestKey;
  for (const item of items) {
    const value = key(item);
    if (best === null || compareKeys(value, bestKey) < 0) {
      best = item;
      bestKey = value;
    }
  }
  return best;
}

/** `Iterator::max_by_key`: the last maximum, or null. */
export function maxByKey(items, key) {
  let best = null;
  let bestKey;
  for (const item of items) {
    const value = key(item);
    if (best === null || compareKeys(value, bestKey) >= 0) {
      best = item;
      bestKey = value;
    }
  }
  return best;
}

function compareKeys(left, right) {
  if (Array.isArray(left)) {
    for (let index = 0; index < left.length; index += 1) {
      const order = compareKeys(left[index], right[index]);
      if (order !== 0) return order;
    }
    return 0;
  }
  if (typeof left === 'string') return left < right ? -1 : left > right ? 1 : 0;
  return left - right;
}

/** `str::cmp` (byte order of UTF-8, which is code point order). */
export function compareStr(left, right) {
  const a = Array.from(left, (character) => character.codePointAt(0));
  const b = Array.from(right, (character) => character.codePointAt(0));
  for (let index = 0; index < Math.min(a.length, b.length); index += 1) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return a.length - b.length;
}

/** `str::lines` (shared twin of content.mjs `rustLines`). */
export { rustLines as lines } from './content.mjs';

/** `str::match_indices` for a non-empty literal: non-overlapping start indices. */
export function matchIndices(text, pattern) {
  const out = [];
  if (pattern === '') return out;
  for (let at = text.indexOf(pattern); at >= 0; at = text.indexOf(pattern, at + pattern.length)) out.push(at);
  return out;
}
