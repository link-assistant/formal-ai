// Rust `str` / `char` semantics the planner ports lean on (Unicode
// `White_Space` trimming, `split_whitespace`, `char::is_alphanumeric`, UTF-8
// lengths, ASCII-only case folding). Shared by the agentic ports so every
// module reads a string the way the Rust module it mirrors does.

const WHITE_SPACE = /\p{White_Space}/u;
const LEADING = /^\p{White_Space}+/u;
const TRAILING = /\p{White_Space}+$/u;

/** `char::is_whitespace`. */
export const isWhitespace = (character) => WHITE_SPACE.test(character);
/** `char::is_alphabetic`. */
export const isAlphabetic = (character) => /^\p{Alphabetic}$/u.test(character);
/** `char::is_alphanumeric`. */
export const isAlphanumeric = (character) => /^[\p{Alphabetic}\p{N}]$/u.test(character);
/** `char::is_numeric`. */
export const isNumeric = (character) => /^\p{N}$/u.test(character);
/** `char::is_ascii_digit`. */
export const isAsciiDigit = (character) => /^[0-9]$/.test(character);
/** `char::is_ascii_alphanumeric`. */
export const isAsciiAlphanumeric = (character) => /^[0-9A-Za-z]$/.test(character);
/** `char::is_uppercase`. */
export const isUppercase = (character) => /^\p{Uppercase}$/u.test(character);
/** `char::is_lowercase`. */
export const isLowercase = (character) => /^\p{Lowercase}$/u.test(character);
/** `char::is_ascii_punctuation`. */
export const isAsciiPunctuation = (character) => /^[!-/:-@[-`{-~]$/.test(character);

/** `str::trim`. */
export const trim = (text) => text.replace(LEADING, '').replace(TRAILING, '');
/** `str::trim_start`. */
export const trimStart = (text) => text.replace(LEADING, '');
/** `str::trim_end`. */
export const trimEnd = (text) => text.replace(TRAILING, '');

/** `str::split_whitespace`. */
export const splitWhitespace = (text) => text.split(/\p{White_Space}+/u).filter(Boolean);

/** `str::len` (UTF-8 bytes). */
export const utf8Len = (text) => new TextEncoder().encode(text).length;

/** `str::chars().count()`. */
export const charCount = (text) => Array.from(text).length;

/** `str::to_ascii_lowercase`. */
export const toAsciiLowercase = (text) => text.replace(/[A-Z]/g, (letter) => letter.toLowerCase());

/** `str::eq_ignore_ascii_case`. */
export const eqIgnoreAsciiCase = (left, right) => toAsciiLowercase(left) === toAsciiLowercase(right);

/** `str::trim_matches(predicate)` over characters. */
export function trimMatches(text, predicate) {
  const chars = Array.from(text);
  let start = 0;
  let end = chars.length;
  while (start < end && predicate(chars[start])) start += 1;
  while (end > start && predicate(chars[end - 1])) end -= 1;
  return chars.slice(start, end).join('');
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

/** `str::split_once(separator)` -> `[before, after]` or null. */
export function splitOnce(text, separator) {
  const at = text.indexOf(separator);
  return at < 0 ? null : [text.slice(0, at), text.slice(at + separator.length)];
}

/** `str::rsplit_once(separator)` -> `[before, after]` or null. */
export function rsplitOnce(text, separator) {
  const at = text.lastIndexOf(separator);
  return at < 0 ? null : [text.slice(0, at), text.slice(at + separator.length)];
}

/** `str::strip_prefix` -> rest or null. */
export const stripPrefix = (text, prefix) => (text.startsWith(prefix) ? text.slice(prefix.length) : null);

/** `str::strip_suffix` -> rest or null. */
export const stripSuffix = (text, suffix) => (text.endsWith(suffix) ? text.slice(0, text.length - suffix.length) : null);

/** `str::replace(from, to)` without `$` pattern expansion. */
export const replaceAllLiteral = (text, from, to) => text.split(from).join(to);

/** `str::split_inclusive('\n')`. */
export function splitInclusiveNewline(text) {
  const out = [];
  let start = 0;
  for (let at = text.indexOf('\n'); at >= 0; at = text.indexOf('\n', start)) {
    out.push(text.slice(start, at + 1));
    start = at + 1;
  }
  if (start < text.length) out.push(text.slice(start));
  return out;
}

/** `serde_json::to_string_pretty` of a parsed value (keys sorted like a `BTreeMap`). */
export function prettyJson(value) {
  return JSON.stringify(sortKeys(value), null, 2);
}

/** `serde_json::to_string` (compact, keys sorted). */
export function compactJson(value) {
  return JSON.stringify(sortKeys(value));
}

function sortKeys(value) {
  if (Array.isArray(value)) return value.map(sortKeys);
  if (value && typeof value === 'object') {
    const out = {};
    for (const key of Object.keys(value).sort(byteOrder)) out[key] = sortKeys(value[key]);
    return out;
  }
  return value;
}

/** Byte-order (code point) comparison, the `Ord` of Rust `str`. */
export function byteOrder(left, right) {
  const a = Array.from(left, (character) => character.codePointAt(0));
  const b = Array.from(right, (character) => character.codePointAt(0));
  for (let index = 0; index < Math.min(a.length, b.length); index += 1) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return a.length - b.length;
}

/** `serde_json::from_str::<Value>` -> parsed value or undefined. */
export function parseJson(text) {
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

/** Whether `value` is a JSON object (serde `Value::Object`). */
export const isObject = (value) => Boolean(value) && typeof value === 'object' && !Array.isArray(value);

/** serde `Value::get(key)` (only objects answer). */
export const jsonGet = (value, key) => (isObject(value) && Object.prototype.hasOwnProperty.call(value, key) ? value[key] : undefined);

/** serde `Value::as_str`. */
export const asStr = (value) => (typeof value === 'string' ? value : null);

/** serde `Value::as_i64` (integers only). */
export const asI64 = (value) => (typeof value === 'number' && Number.isInteger(value) ? value : null);

/** Rust `str::parse::<i64>()` -> number or null. */
export function parseI64(text) {
  if (!/^[+-]?[0-9]+$/.test(text)) return null;
  const parsed = BigInt(text);
  if (parsed > 9223372036854775807n || parsed < -9223372036854775808n) return null;
  return Number(parsed);
}

/** Rust `str::parse::<usize>()` -> number or null. */
export function parseUsize(text) {
  if (!/^\+?[0-9]+$/.test(text)) return null;
  return Number(text);
}

/**
 * The UTF-16 index of the UTF-8 byte offset `byteIndex` in `text`, or null
 * when the offset is past the end or not on a character boundary (Rust
 * `str::get(byte..)` returning `None`).
 */
export function byteToUtf16(text, byteIndex) {
  if (byteIndex < 0) return null;
  let bytes = 0;
  let units = 0;
  for (const character of text) {
    if (bytes === byteIndex) return units;
    if (bytes > byteIndex) return null;
    bytes += utf8Len(character);
    units += character.length;
  }
  return bytes === byteIndex ? units : null;
}

/** The UTF-8 byte offset of the UTF-16 index `index` in `text`. */
export const utf16ToByte = (text, index) => utf8Len(text.slice(0, index));

/** Rust `str::get(byte..)`: the tail from a UTF-8 byte offset, or null. */
export function sliceFromByte(text, byteIndex) {
  const at = byteToUtf16(text, byteIndex);
  return at === null ? null : text.slice(at);
}

/** Rust `str::find` as a UTF-8 byte offset, or -1. */
export function byteFind(text, needle) {
  const at = text.indexOf(needle);
  return at < 0 ? -1 : utf16ToByte(text, at);
}

/** `Iterator::max_by_key`: the LAST element with the greatest key, or undefined. */
export function maxByKey(items, key, compare = (left, right) => left - right) {
  let best;
  let bestKey;
  for (const item of items) {
    const value = key(item);
    if (best === undefined || compare(value, bestKey) >= 0) {
      best = item;
      bestKey = value;
    }
  }
  return best;
}

/** `Iterator::min_by_key`: the FIRST element with the least key, or undefined. */
export function minByKey(items, key, compare = (left, right) => left - right) {
  let best;
  let bestKey;
  for (const item of items) {
    const value = key(item);
    if (best === undefined || compare(value, bestKey) < 0) {
      best = item;
      bestKey = value;
    }
  }
  return best;
}

/**
 * Lexicographic comparison of numeric tuples (Rust tuple `Ord`).
 * @param {number[]} left
 * @param {number[]} right
 * @returns {number}
 */
export function compareTuples(left, right) {
  for (let index = 0; index < Math.min(left.length, right.length); index += 1) {
    if (left[index] !== right[index]) return left[index] < right[index] ? -1 : 1;
  }
  return left.length - right.length;
}
