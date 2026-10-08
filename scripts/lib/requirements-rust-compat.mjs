// Rust standard-library semantics the requirement-pipeline twins rely on.
//
// The JavaScript twins of `scripts/assemble-requirements.rs`,
// `scripts/generate-requirement-status.rs`, `scripts/render-status.rs` and
// `scripts/check-requirement-status.rs` must print the same lines and write the
// same bytes as the Rust originals, so the few places where JavaScript and Rust
// disagree are spelled out here once: `str::lines`, `str::trim` (Unicode
// `White_Space`, which differs from JavaScript's `\s`), byte-wise `String`
// ordering, component-wise `PathBuf` ordering, `Path::extension`, the `{:?}`
// form of a string list, and the `Display` text of an `io::Error`.
import { readdirSync, readFileSync, statSync } from 'node:fs';

// Rust's `char::is_whitespace` is the Unicode `White_Space` property:
// JavaScript's `\s` adds U+FEFF and leaves out U+0085.
const WHITESPACE = '\\t\\n\\v\\f\\r \\u0085\\u00a0\\u1680\\u2000-\\u200a\\u2028\\u2029\\u202f\\u205f\\u3000';
const LEADING = new RegExp(`^[${WHITESPACE}]+`, 'u');
const TRAILING = new RegExp(`[${WHITESPACE}]+$`, 'u');
const ONE = new RegExp(`^[${WHITESPACE}]$`, 'u');

/** `char::is_whitespace`. */
export const isWhitespace = (character) => ONE.test(character);
/** `str::trim_start`. */
export const trimStart = (text) => text.replace(LEADING, '');
/** `str::trim_end`. */
export const trimEnd = (text) => text.replace(TRAILING, '');
/** `str::trim`. */
export const trim = (text) => trimEnd(trimStart(text));

/** `str::trim_matches` for a set of single characters. */
export function trimMatches(text, characters) {
  let start = 0;
  let end = text.length;
  while (start < end && characters.includes(text[start])) start += 1;
  while (end > start && characters.includes(text[end - 1])) end -= 1;
  return text.slice(start, end);
}

/**
 * `str::lines`: split at `\n`, drop one `\r` before it, and do not yield an
 * empty line after a final newline.
 */
export function lines(text) {
  if (text === '') return [];
  const parts = text.split('\n');
  const last = parts.pop();
  const result = parts.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
  if (last !== '') result.push(last);
  return result;
}

/** `str::to_ascii_lowercase`. */
export const asciiLowercase = (text) => text.replace(/[A-Z]+/g, (upper) => upper.toLowerCase());
/** `char::is_ascii_digit`. */
export const isAsciiDigit = (character) => character >= '0' && character <= '9';
/** `char::is_ascii_alphanumeric`. */
export const isAsciiAlphanumeric = (character) => /^[0-9A-Za-z]$/.test(character);
/** `char::is_alphanumeric`: Unicode `Alphabetic` or `Numeric`. */
export const isAlphanumeric = (character) => /^[\p{Alphabetic}\p{N}]$/u.test(character);

/** The leading run of ASCII digits, as `take_while(char::is_ascii_digit)`. */
export const leadingDigits = (text) => /^[0-9]*/.exec(text)[0];

/** `str::parse::<u32>()` (or `u64`) of a digit run, `null` where Rust errs. */
export function parseUnsigned(text, bits = 32) {
  if (!/^\+?[0-9]+$/.test(text)) return null;
  const value = BigInt(text);
  return value < (1n << BigInt(bits)) ? value : null;
}

/** `Ord for str`: UTF-8 byte order, which is code-point order. */
export function compareStrings(left, right) {
  const a = [...left];
  const b = [...right];
  const length = Math.min(a.length, b.length);
  for (let index = 0; index < length; index += 1) {
    const difference = a[index].codePointAt(0) - b[index].codePointAt(0);
    if (difference !== 0) return difference < 0 ? -1 : 1;
  }
  return a.length === b.length ? 0 : a.length < b.length ? -1 : 1;
}

/** A sorted copy, as a `BTreeSet<String>` or `slice::sort` orders it. */
export const sortedStrings = (values) => [...values].sort(compareStrings);

const components = (path) => path.split('/').filter((part, index) => part !== '.' && (part !== '' || index === 0));

/** `Ord for Path`: component by component, each compared as bytes. */
export function comparePaths(left, right) {
  const a = components(left);
  const b = components(right);
  const length = Math.min(a.length, b.length);
  for (let index = 0; index < length; index += 1) {
    const order = compareStrings(a[index], b[index]);
    if (order !== 0) return order;
  }
  return a.length === b.length ? 0 : a.length < b.length ? -1 : 1;
}

/** `Path::join` for `/`-separated paths. */
export function joinPath(base, tail) {
  if (tail.startsWith('/')) return tail;
  if (base === '' || base.endsWith('/')) return `${base}${tail}`;
  return `${base}/${tail}`;
}

/** `Path::file_name`, as a string. */
export function fileName(path) {
  const parts = path.split('/').filter((part) => part !== '' && part !== '.');
  const last = parts.at(-1) ?? '';
  return last === '..' ? '' : last;
}

/** `Path::extension`: `null` for a name with no dot or only a leading one. */
export function extension(path) {
  const name = fileName(path);
  if (name === '' || name === '..') return null;
  const dot = name.lastIndexOf('.');
  return dot <= 0 ? null : name.slice(dot + 1);
}

/** `Path::parent` of a non-root path. */
export function parent(path) {
  const slash = path.lastIndexOf('/');
  if (slash < 0) return '';
  return slash === 0 ? '/' : path.slice(0, slash);
}

/** `Path::strip_prefix(root)`, or the path itself when it is not under root. */
export function stripRoot(path, root) {
  if (path === root) return '';
  const prefix = root.endsWith('/') ? root : `${root}/`;
  return path.startsWith(prefix) ? path.slice(prefix.length) : path;
}

/** `Path::is_file`: follows symbolic links, false on any error. */
export function isFile(path) {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}

/** `Path::exists`. */
export function exists(path) {
  try {
    statSync(path);
    return true;
  } catch {
    return false;
  }
}

const OS_ERRORS = {
  ENOENT: 'No such file or directory',
  EACCES: 'Permission denied',
  EISDIR: 'Is a directory',
  ENOTDIR: 'Not a directory',
  EPERM: 'Operation not permitted',
  ELOOP: 'Too many levels of symbolic links',
  ENAMETOOLONG: 'File name too long',
};

/** The `Display` text of the `io::Error` Rust reports for a failed call. */
export function ioErrorDisplay(error) {
  if (error?.rustDisplay) return error.rustDisplay;
  const description = OS_ERRORS[error?.code];
  if (description && typeof error.errno === 'number') {
    return `${description} (os error ${Math.abs(error.errno)})`;
  }
  return String(error?.message ?? error);
}

const STRICT_UTF8 = new TextDecoder('utf-8', { fatal: true, ignoreBOM: true });

/** `fs::read_to_string`: throws an error whose `ioErrorDisplay` matches Rust. */
export function readToString(path) {
  const bytes = readFileSync(path);
  try {
    return STRICT_UTF8.decode(bytes);
  } catch {
    const error = new Error('stream did not contain valid UTF-8');
    error.rustDisplay = 'stream did not contain valid UTF-8';
    throw error;
  }
}

/** `fs::read_to_string(path).unwrap_or_default()`. */
export function readOrEmpty(path) {
  try {
    return readToString(path);
  } catch {
    return '';
  }
}

/** `fs::read_to_string(path).ok()`. */
export function readOrNull(path) {
  try {
    return readToString(path);
  } catch {
    return null;
  }
}

/** `fs::read_dir(directory)` entry paths (unsorted in Rust; sort before use). */
export const readDirPaths = (directory) => readdirSync(directory).map((name) => joinPath(directory, name));

/** `{:?}` of a `&str`: quoted, with Rust's escapes. */
export function debugString(text) {
  let out = '"';
  for (const character of text) {
    const code = character.codePointAt(0);
    if (character === '"') out += '\\"';
    else if (character === '\\') out += '\\\\';
    else if (character === '\n') out += '\\n';
    else if (character === '\r') out += '\\r';
    else if (character === '\t') out += '\\t';
    else if (character === '\0') out += '\\0';
    else if (code < 0x20 || (code >= 0x7f && code <= 0x9f)) out += `\\u{${code.toString(16)}}`;
    else out += character;
  }
  return `${out}"`;
}

/** `{:?}` of a `Vec<String>`. */
export const debugList = (values) => `[${values.map(debugString).join(', ')}]`;

const RUNS = new RegExp(`[${WHITESPACE}]+`, 'u');

/** `str::split_whitespace`, collected. */
export const splitWhitespace = (text) => trim(text).split(RUNS).filter((word) => word !== '');
