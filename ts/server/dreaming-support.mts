// Small text/byte utilities shared by the dreaming planner, and the
// data-grounded lexicon and requirement cues it reads:
// rust/src/dreaming/support.rs, rust/src/dreaming/lexicon.rs and
// rust/src/dreaming/cues.rs, plus the Rust `str` semantics those modules lean
// on (UTF-8 byte lengths, byte-order string comparison, the `{:?}` Debug
// rendering a reconstruction record's id hashes).

import fs from 'node:fs';
import path from 'node:path';

import { REPO_ROOT } from './lino.mjs';

const encoder = new TextEncoder();

/** `str::trim` (Unicode `White_Space`). */
export function trim(text) {
  return String(text).replace(/^\p{White_Space}+/u, '').replace(/\p{White_Space}+$/u, '');
}

/** `str::split_whitespace`. */
export function splitWhitespace(text) {
  return String(text).split(/\p{White_Space}+/u).filter(Boolean);
}

/** `char::is_alphanumeric`. */
export function isAlphanumeric(character) {
  return /^[\p{Alphabetic}\p{N}]$/u.test(character);
}

/** `str::trim_matches(predicate)` over characters. */
export function trimMatches(text, predicate) {
  const chars = Array.from(text);
  let start = 0;
  let end = chars.length;
  while (start < end && predicate(chars[start])) start += 1;
  while (end > start && predicate(chars[end - 1])) end -= 1;
  return chars.slice(start, end).join('');
}

/**
 * rust/src/links_format.rs `push_lino_node`: `out` plus one `name value` line,
 * the value sanitized onto one line and quoted the way Links Notation quotes
 * (`format_lino_value`).
 */
export function pushLinoNode(out, indent, name, value) {
  if (value === null || value === undefined) return `${out}${' '.repeat(indent)}${name}\n`;
  const flat = String(value).replaceAll('\\', '\\\\').replaceAll('\r', '\\r').replaceAll('\n', '\\n').replaceAll('\t', '\\t');
  const hasSingle = flat.includes("'");
  const hasDouble = flat.includes('"');
  let quoted = `"${flat}"`;
  if (hasDouble && !hasSingle) quoted = `'${flat}'`;
  else if (hasSingle && hasDouble) quoted = `'${flat.replaceAll("'", "''")}'`;
  return `${out}${' '.repeat(indent)}${name} ${quoted}\n`;
}

/** `str::len`: the UTF-8 byte length. @param {string} value @returns {number} */
export function stringBytes(value) {
  return encoder.encode(String(value)).length;
}

/** Mirrors rust/src/dreaming/support.rs `option_bytes`. */
export function optionBytes(value) {
  return value === null || value === undefined ? 0 : stringBytes(value);
}

/** Mirrors rust/src/dreaming/support.rs `estimate_event_bytes`. */
export function estimateEventBytes(event) {
  let total = 64 + stringBytes(event.id);
  for (const field of [
    'kind', 'role', 'intent', 'tool', 'inputs', 'outputs', 'content',
    'sent_at', 'demo_label', 'conversation_id', 'conversation_title',
  ]) {
    total += optionBytes(event[field]);
  }
  for (const entry of event.evidence) total += stringBytes(entry);
  for (const [key, value] of event.unknown_fields) total += stringBytes(key) + stringBytes(value);
  return total;
}

/** Mirrors rust/src/dreaming/support.rs `selected_bytes`. */
export function selectedBytes(actions) {
  return actions.reduce((sum, action) => sum + action.estimated_bytes, 0);
}

/** Mirrors rust/src/dreaming/support.rs `required_reclaim_bytes` (saturating). */
export function requiredReclaimBytes(targetFreeBytes, freeBytes, incomingBytes) {
  if (targetFreeBytes === null || targetFreeBytes === undefined) return 0;
  const freeAfterIncoming = Math.max((freeBytes ?? 0) - incomingBytes, 0);
  return Math.max(targetFreeBytes - freeAfterIncoming, 0);
}

/** Mirrors rust/src/dreaming/support.rs `percent_ceil` (exact for any u64 capacity). */
export function percentCeil(total, percent) {
  if (percent === 0 || total === 0) return 0;
  return Number((BigInt(total) * BigInt(percent) + 99n) / 100n);
}

/** Mirrors rust/src/dreaming/support.rs `contains_any`. */
export function containsAny(haystack, needles) {
  return needles.some((needle) => haystack.includes(needle));
}

/** `str::to_ascii_lowercase`. */
export function asciiLower(value) {
  return String(value).replace(/[A-Z]/g, (letter) => letter.toLowerCase());
}

/** Mirrors rust/src/dreaming/support.rs `lower_opt`. */
export function lowerOpt(value) {
  return value === null || value === undefined ? '' : asciiLower(value);
}

/** Mirrors rust/src/dreaming/support.rs `normalized`. */
export function normalized(value) {
  return value === null || value === undefined ? '' : asciiLower(trim(String(value)));
}

/**
 * The `Ord` of Rust `str`: UTF-8 byte order, which is code point order.
 * JavaScript's `<` compares UTF-16 code units, which disagrees once a
 * supplementary-plane character meets one in U+E000..U+FFFF.
 * @returns {number}
 */
export function cmpStr(left, right) {
  if (left === right) return 0;
  const length = Math.min(left.length, right.length);
  for (let index = 0; index < length; index += 1) {
    let a = left.charCodeAt(index);
    let b = right.charCodeAt(index);
    if (a === b) continue;
    if (a >= 0xd800) a += a >= 0xe000 ? -0x800 : 0x2000;
    if (b >= 0xd800) b += b >= 0xe000 ? -0x800 : 0x2000;
    return a < b ? -1 : 1;
  }
  return left.length - right.length;
}

/** Rust `Ord::cmp` for integers. */
export function cmpNum(left, right) {
  return left < right ? -1 : left > right ? 1 : 0;
}

const NON_PRINTABLE = /^[\p{Cc}\p{Cf}\p{Cs}\p{Co}\p{Cn}\p{Zl}\p{Zp}\p{Zs}]$/u;
const GRAPHEME_EXTEND = /^\p{Grapheme_Extend}$/u;
const SIMPLE_ESCAPES = { '\0': '\\0', '\t': '\\t', '\r': '\\r', '\n': '\\n', '\\': '\\\\', '"': '\\"' };

/**
 * Rust `<str as Debug>::fmt`: double-quoted, `char::escape_debug_ext` with
 * grapheme-extend escaping on and single quotes left alone. Printability
 * follows `core::unicode::printable` (categories Zs/Zl/Zp/Cc/Cf/Cs/Co/Cn
 * other than the ASCII space are escaped as `\u{…}`).
 * @param {string} text
 */
export function debugStr(text) {
  let out = '"';
  for (const character of String(text)) {
    if (Object.prototype.hasOwnProperty.call(SIMPLE_ESCAPES, character)) {
      out += SIMPLE_ESCAPES[character];
    } else if (character !== ' ' && (GRAPHEME_EXTEND.test(character) || NON_PRINTABLE.test(character))) {
      out += `\\u{${character.codePointAt(0).toString(16)}}`;
    } else {
      out += character;
    }
  }
  return `${out}"`;
}

/** Rust `{:?}` of an `Option<String>`. */
export function debugOption(value) {
  return value === null || value === undefined ? 'None' : `Some(${debugStr(value)})`;
}

/** Mirrors rust/src/dreaming/lexicon.rs `data_document_path`. */
export function dataDocumentPath(fileName, env = process.env) {
  const dir = env.FORMAL_AI_DATA_DIR;
  if (dir !== undefined) {
    const trimmed = trim(dir);
    if (trimmed) {
      const candidate = path.join(trimmed, fileName);
      if (isFile(candidate)) return candidate;
    }
  }
  const repoRelative = path.join('data', 'meta', fileName);
  return isFile(repoRelative) ? repoRelative : null;
}

function isFile(candidate) {
  try {
    return fs.statSync(candidate).isFile();
  } catch {
    return false;
  }
}

/**
 * Mirrors rust/src/dreaming/lexicon.rs `load_data_document`: the on-disk
 * document, else the copy the Rust binary compiles in
 * (`rust/embedded/data/meta/<file>`).
 */
export function loadDataDocument(fileName, env = process.env) {
  const found = dataDocumentPath(fileName, env);
  if (found) {
    try {
      return fs.readFileSync(found, 'utf8');
    } catch {
      // fall through to the embedded copy, like `.ok()` in Rust
    }
  }
  return fs.readFileSync(path.join(REPO_ROOT, 'rust', 'embedded', 'data', 'meta', fileName), 'utf8');
}

/** Rust `str::lines`. */
export function rustStrLines(text) {
  const lines = String(text).split('\n');
  if (lines.length > 0 && lines[lines.length - 1] === '') lines.pop();
  return lines.map((line) => (line.endsWith('\r') ? line.slice(0, -1) : line));
}

const LEXICON_KEYS = {
  task_kind_cue: 'task_kind_cues',
  task_intent_cue: 'task_intent_cues',
  topic_stopword: 'topic_stopwords',
  learning_kind_cue: 'learning_kind_cues',
  learning_content_cue: 'learning_content_cues',
  cache_kind_cue: 'cache_kind_cues',
  cache_tool_cue: 'cache_tool_cues',
  intermediate_kind_cue: 'intermediate_kind_cues',
};

/** Mirrors rust/src/dreaming/lexicon.rs `parse_lexicon`. */
export function parseLexicon(text) {
  const lexicon = Object.fromEntries(Object.values(LEXICON_KEYS).map((field) => [field, []]));
  for (const raw of rustStrLines(text)) {
    const line = trim(raw);
    const space = line.indexOf(' ');
    if (space < 0) continue;
    const key = line.slice(0, space);
    const rest = trim(line.slice(space + 1));
    if (!rest.startsWith('"')) continue;
    const tail = rest.slice(1);
    if (!tail.endsWith('"')) continue;
    if (Object.prototype.hasOwnProperty.call(LEXICON_KEYS, key)) {
      lexicon[LEXICON_KEYS[key]].push(tail.slice(0, -1).toLowerCase());
    }
  }
  return lexicon;
}

let lexiconCache = null;

/** Mirrors rust/src/dreaming/lexicon.rs `lexicon` (parsed once per process). */
export function lexicon() {
  if (!lexiconCache) lexiconCache = parseLexicon(loadDataDocument('dreaming-lexicon.lino'));
  return lexiconCache;
}

/** Mirrors rust/src/dreaming/cues.rs `parse_requirement_cues`. */
export function parseRequirementCues(text) {
  const cues = [];
  for (const raw of rustStrLines(text)) {
    const line = trim(raw);
    if (!line.startsWith('cue "')) continue;
    const tail = line.slice('cue "'.length);
    if (!tail.endsWith('"')) continue;
    cues.push(tail.slice(0, -1).toLowerCase());
  }
  return cues;
}

let cuesCache = null;

/** Mirrors rust/src/dreaming/cues.rs `requirement_cues` (parsed once per process). */
export function requirementCues() {
  if (!cuesCache) cuesCache = parseRequirementCues(loadDataDocument('dreaming-cues.lino'));
  return cuesCache;
}
