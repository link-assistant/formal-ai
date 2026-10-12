// Rust standard-library semantics the JavaScript twins of the `scripts/*.rs`
// checks need to print byte-identical output (PR #1188, SCRIPTS-B).
//
// A twin is only worth having if CI can diff it against the Rust original, so
// the small places where JavaScript and Rust disagree are written down here
// once: `str::lines`, Unicode `trim`, `split_whitespace`, byte-wise `String`
// ordering, `{:?}` string escaping, `io::Error` and `ParseIntError` messages,
// `Path::extension`, and the order `walkdir` visits a tree in.
//
// Node built-ins only.

import { lstatSync, opendirSync, readFileSync, statSync } from 'node:fs';
import { join } from 'node:path';

// `char::is_whitespace`: the Unicode White_Space property. JavaScript's `\s`
// adds U+FEFF and drops U+0085, so it is spelled out.
const WHITESPACE = '\t\n\v\f\r \u0085         '
  + '        　';

/** `char::is_whitespace`. */
export function isWhitespace(ch) {
  return ch.length > 0 && WHITESPACE.includes(ch);
}

/** `str::trim_start`. */
export function trimStart(text) {
  let start = 0;
  while (start < text.length && WHITESPACE.includes(text[start])) start += 1;
  return text.slice(start);
}

/** `str::trim_end`. */
export function trimEnd(text) {
  let end = text.length;
  while (end > 0 && WHITESPACE.includes(text[end - 1])) end -= 1;
  return text.slice(0, end);
}

/** `str::trim`. */
export function trim(text) {
  return trimEnd(trimStart(text));
}

/** `str::split_whitespace`. */
export function splitWhitespace(text) {
  const words = [];
  let word = '';
  for (const ch of text) {
    if (WHITESPACE.includes(ch)) {
      if (word) words.push(word);
      word = '';
    } else {
      word += ch;
    }
  }
  if (word) words.push(word);
  return words;
}

/** `str::lines`: split on `\n`, drop one trailing `\r`, no final empty line. */
export function lines(text) {
  if (text === '') return [];
  const parts = text.split('\n');
  if (parts.at(-1) === '') parts.pop();
  return parts.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
}

/** `str::lines().count()` without building the array. */
export function lineCount(text) {
  if (text === '') return 0;
  let count = 0;
  let at = text.indexOf('\n');
  while (at !== -1) {
    count += 1;
    at = text.indexOf('\n', at + 1);
  }
  return text.endsWith('\n') ? count : count + 1;
}

/** `str::matches(needle).count()`: non-overlapping occurrences. */
export function countMatches(text, needle) {
  let count = 0;
  let at = text.indexOf(needle);
  while (at !== -1) {
    count += 1;
    at = text.indexOf(needle, at + needle.length);
  }
  return count;
}

/** `str::split_once`. */
export function splitOnce(text, separator) {
  const at = text.indexOf(separator);
  return at === -1 ? null : [text.slice(0, at), text.slice(at + separator.length)];
}

/**
 * `Ord for str`: byte-wise over UTF-8, which is code-point order. JavaScript's
 * `<` compares UTF-16 code units and disagrees above U+FFFF.
 */
export function compareStrings(left, right) {
  if (left === right) return 0;
  const a = Array.from(left);
  const b = Array.from(right);
  const length = Math.min(a.length, b.length);
  for (let index = 0; index < length; index += 1) {
    if (a[index] !== b[index]) return a[index].codePointAt(0) - b[index].codePointAt(0);
  }
  return a.length - b.length;
}

/** `Ord for (String, String)`, as a derived `Ord` on a two-field struct sorts. */
export function compareTuples(left, right) {
  for (let index = 0; index < left.length; index += 1) {
    const order = compareStrings(left[index], right[index]);
    if (order) return order;
  }
  return 0;
}

/** A map's keys in `BTreeMap` order. */
export function sortedKeys(map) {
  return [...map.keys()].sort(compareStrings);
}

/** The file name of a `/`-separated path (`Path::file_name`). */
export function fileName(path) {
  const trimmed = path.replace(/\/+$/, '');
  return trimmed.slice(trimmed.lastIndexOf('/') + 1);
}

/**
 * `Path::extension`: the text after the last `.` of the file name, `null` when
 * there is none or the only `.` leads the name (`.gitignore`).
 */
export function extension(path) {
  const name = fileName(path);
  if (name === '..') return null;
  const at = name.lastIndexOf('.');
  return at <= 0 ? null : name.slice(at + 1);
}

// `char::escape_debug` escapes what Rust's `is_printable` rejects (control,
// format, surrogate, private-use and unassigned code points, line and paragraph
// separators, every space separator but ' ') and grapheme extenders.
const NON_PRINTABLE = /[\p{Cc}\p{Cf}\p{Cs}\p{Co}\p{Cn}\p{Zl}\p{Zp}\p{Grapheme_Extend}]|(?! )\p{Zs}/u;

/** `format!("{:?}", text)` for a `&str`. */
export function debugString(text) {
  let out = '"';
  for (const ch of text) {
    switch (ch) {
      case '"': out += '\\"'; break;
      case '\\': out += '\\\\'; break;
      case '\n': out += '\\n'; break;
      case '\r': out += '\\r'; break;
      case '\t': out += '\\t'; break;
      case '\0': out += '\\0'; break;
      default:
        out += NON_PRINTABLE.test(ch) ? `\\u{${ch.codePointAt(0).toString(16)}}` : ch;
    }
  }
  return `${out}"`;
}

/** `format!("{:?}", slice)` for a `&[&str]`. */
export function debugStringList(items) {
  return `[${items.map(debugString).join(', ')}]`;
}

// `strerror` text Rust's `io::Error` prints, for the errors a check can meet.
const STRERROR = {
  EPERM: 'Operation not permitted',
  ENOENT: 'No such file or directory',
  EIO: 'Input/output error',
  EACCES: 'Permission denied',
  EEXIST: 'File exists',
  ENOTDIR: 'Not a directory',
  EISDIR: 'Is a directory',
  EMFILE: 'Too many open files',
  ENOSPC: 'No space left on device',
  EROFS: 'Read-only file system',
  ELOOP: 'Too many levels of symbolic links',
  ENAMETOOLONG: 'File name too long',
};

/** An error whose `message` is already the text Rust's `Display` would print. */
export class RustError extends Error {}

/** `impl Display for io::Error` for an OS error from a Node call. */
export function ioErrorMessage(error) {
  if (error instanceof RustError) return error.message;
  const description = STRERROR[error.code] ?? error.code ?? String(error.message);
  return typeof error.errno === 'number'
    ? `${description} (os error ${Math.abs(error.errno)})`
    : description;
}

const UTF8 = new TextDecoder('utf-8', { fatal: true });

/** `fs::read_to_string`: throws a `RustError` with Rust's message on failure. */
export function readToString(path) {
  let bytes;
  try {
    bytes = readFileSync(path);
  } catch (error) {
    throw new RustError(ioErrorMessage(error));
  }
  try {
    const text = UTF8.decode(bytes);
    // `TextDecoder` drops a leading byte-order mark; `read_to_string` keeps it.
    return bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf ? `﻿${text}` : text;
  } catch {
    throw new RustError('stream did not contain valid UTF-8');
  }
}

/** Rust's `Path::is_file` (follows links, false on any error). */
export function isFile(path) {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}

/** Rust's `Path::is_dir` (follows links, false on any error). */
export function isDir(path) {
  try {
    return statSync(path).isDirectory();
  } catch {
    return false;
  }
}

/**
 * `str::parse::<u64>()` (and `usize` on a 64-bit target): `{ value }` or
 * `{ error }` with `ParseIntError`'s message. Rust accepts one leading `+`.
 */
export function parseUnsigned(text) {
  if (text === '') return { error: 'cannot parse integer from empty string' };
  const digits = text.startsWith('+') && text.length > 1 ? text.slice(1) : text;
  if (!/^[0-9]+$/.test(digits)) return { error: 'invalid digit found in string' };
  const value = BigInt(digits);
  if (value > 18446744073709551615n) return { error: 'number too large to fit in target type' };
  return { value: Number(value) };
}

/**
 * The entries of one directory in the order the OS returns them (`readdir`),
 * which is the order `fs::read_dir` and `walkdir` see. `fs.readdirSync`
 * sorts, so it would not match the Rust original's output order.
 */
export function readDirEntries(directory) {
  const handle = opendirSync(directory);
  const entries = [];
  try {
    for (let entry = handle.readSync(); entry !== null; entry = handle.readSync()) {
      entries.push({ name: entry.name, path: join(directory, entry.name), type: direntType(entry, join(directory, entry.name)) });
    }
  } finally {
    handle.closeSync();
  }
  return entries;
}

function direntType(entry, path) {
  if (entry.isFile()) return 'file';
  if (entry.isDirectory()) return 'dir';
  if (entry.isSymbolicLink()) return 'symlink';
  if (entry.isFIFO() || entry.isSocket() || entry.isCharacterDevice() || entry.isBlockDevice()) return 'other';
  // DT_UNKNOWN: walkdir falls back to `symlink_metadata`.
  try {
    const stat = lstatSync(path);
    if (stat.isFile()) return 'file';
    if (stat.isDirectory()) return 'dir';
    if (stat.isSymbolicLink()) return 'symlink';
  } catch {
    // An entry that vanished reads as nothing walkable.
  }
  return 'other';
}

/**
 * `WalkDir::new(root)` with default options: the root first (its links
 * followed), then a pre-order depth-first walk in `readdir` order that does
 * not follow links. Yields `{ path, type }`, or `{ path, error }` when a
 * directory cannot be read (walkdir's `Err` item). `skip(path)` prunes a
 * directory the caller would ignore every file of anyway.
 */
export function* walkDir(root, skip = () => false) {
  let type;
  try {
    const stat = statSync(root);
    type = stat.isFile() ? 'file' : stat.isDirectory() ? 'dir' : 'other';
  } catch (error) {
    yield { path: root, error: walkError(root, error) };
    return;
  }
  yield { path: root, type };
  if (type === 'dir') yield* walkChildren(root, skip);
}

function* walkChildren(directory, skip) {
  let handle;
  try {
    handle = opendirSync(directory);
  } catch (error) {
    yield { path: directory, error: walkError(directory, error) };
    return;
  }
  try {
    for (let entry = handle.readSync(); entry !== null; entry = handle.readSync()) {
      const path = join(directory, entry.name);
      const type = direntType(entry, path);
      yield { path, type };
      if (type === 'dir' && !skip(path)) yield* walkChildren(path, skip);
    }
  } finally {
    handle.closeSync();
  }
}

/** `impl Display for walkdir::Error` for an I/O error on `path`. */
function walkError(path, error) {
  return `IO error for operation on ${path}: ${ioErrorMessage(error)}`;
}
