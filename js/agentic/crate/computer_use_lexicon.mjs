// Language-independent recognition of computer-use requests (issue #707): a
// port of rust/src/computer_use/lexicon.rs.
//
// Every surface lives in data/seed/meanings-computer-use.lino; this module
// asks the seed lexicon which meanings carrying the operation, resource and
// capability-gap roles are evidenced in a prompt. A `Cue` is `{slug, position,
// length}`, where `position` and `length` are UTF-8 byte counts like Rust's.

import { rustLines } from '../content.mjs';
import { containsCjk, meaningsWithRole, words } from './seed_meanings.mjs';
import { byteOrder, isAlphanumeric, splitWhitespace, utf16ToByte, utf8Len } from './rust_str.mjs';

/** `crate::seed::ROLE_COMPUTER_USE_OPERATION_CUE`. */
export const ROLE_COMPUTER_USE_OPERATION_CUE = 'computer_use_operation_cue';
/** `crate::seed::ROLE_COMPUTER_USE_RESOURCE_CUE`. */
export const ROLE_COMPUTER_USE_RESOURCE_CUE = 'computer_use_resource_cue';
/** `crate::seed::ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE`. */
export const ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE = 'computer_use_capability_gap_cue';

const GAP_PREFIX = 'computer_use_gap_';

/** The inclusive code point ranges `fn is_combining_mark` accepts. */
const COMBINING_RANGES = [
  [0x0300, 0x036f], [0x0483, 0x0489], [0x0591, 0x05bd],
  [0x0610, 0x061a], [0x064b, 0x065f], [0x0670, 0x0670], [0x06d6, 0x06dc],
  [0x0900, 0x0903], [0x093a, 0x094f], [0x0951, 0x0957], [0x0962, 0x0963],
  [0x0981, 0x0983], [0x09bc, 0x09cd], [0x0a01, 0x0a03], [0x0a3c, 0x0a4d],
  [0x0b01, 0x0b4d], [0x0c00, 0x0c4d], [0x0d00, 0x0d4d],
  [0x0e31, 0x0e3a], [0x0e47, 0x0e4e],
  [0x1ab0, 0x1aff], [0x1dc0, 0x1dff], [0x20d0, 0x20f0], [0xfe20, 0xfe2f],
];

/** Mirrors `const fn is_combining_mark`. */
function isCombiningMark(character) {
  const cp = character.codePointAt(0);
  return COMBINING_RANGES.some(([low, high]) => cp >= low && cp <= high);
}

/**
 * Mirrors `fn normalize` in rust/src/computer_use/lexicon.rs: lower-case word
 * characters (alphanumeric, `/`, `.`, combining marks), every other character
 * a space, whitespace collapsed.
 * @param {string} prompt
 */
export function normalize(prompt) {
  let normalized = '';
  for (const character of prompt) {
    if (isAlphanumeric(character) || character === '/' || character === '.' || isCombiningMark(character)) {
      normalized += character.toLowerCase();
    } else {
      normalized += ' ';
    }
  }
  return splitWhitespace(normalized).join(' ');
}

/**
 * Mirrors `fn instruction_surface`: the prompt without indented payload lines
 * and without double-quoted spans.
 * @param {string} prompt
 */
export function instructionSurface(prompt) {
  let instruction = '';
  for (const line of rustLines(prompt)) {
    if (line.startsWith(' ') || line.startsWith('\t')) continue;
    let quoted = false;
    for (const character of line) {
      if (character === '"') {
        quoted = !quoted;
        instruction += ' ';
      } else if (!quoted) {
        instruction += character;
      }
    }
    instruction += '\n';
  }
  return instruction;
}

/** Mirrors `fn operation_cues`: operations in the order the speaker named them. */
export function operationCues(normalized) {
  return roleCues(ROLE_COMPUTER_USE_OPERATION_CUE, normalized).sort((left, right) =>
    left.position - right.position
    || right.length - left.length
    || byteOrder(left.slug, right.slug));
}

/** Mirrors `fn resource_cue`: the longest evidenced resource (`max_by` keeps the last tie). */
export function resourceCue(normalized) {
  let best = null;
  for (const cue of roleCues(ROLE_COMPUTER_USE_RESOURCE_CUE, normalized)) {
    if (best === null) {
      best = cue;
      continue;
    }
    const order = cue.length - best.length || best.position - cue.position;
    if (order >= 0) best = cue;
  }
  return best;
}

/** Mirrors `fn capability_gap_cue`: the earliest gap cue's capability name, or null. */
export function capabilityGapCue(normalized) {
  let first = null;
  for (const cue of roleCues(ROLE_COMPUTER_USE_CAPABILITY_GAP_CUE, normalized)) {
    if (first === null || cue.position < first.position) first = cue;
  }
  if (first === null || !first.slug.startsWith(GAP_PREFIX)) return null;
  return first.slug.slice(GAP_PREFIX.length);
}

/** Mirrors `fn role_cues`. */
function roleCues(role, normalized) {
  return meaningsWithRole(role)
    .map((meaning) => bestEvidence(meaning, normalized))
    .filter((cue) => cue !== null);
}

/** Mirrors `fn best_evidence`: the earliest (then longest) surface match (`min_by` keeps the first). */
function bestEvidence(meaning, normalized) {
  let best = null;
  for (const surface of words(meaning)) {
    const position = surfacePosition(normalized, surface);
    if (position === null) continue;
    const length = utf8Len(surface);
    if (best === null || position < best[0] || (position === best[0] && length > best[1])) {
      best = [position, length];
    }
  }
  return best === null ? null : { slug: meaning.slug, position: best[0], length: best[1] };
}

/**
 * Mirrors `fn surface_position`: the UTF-8 byte offset of `surface` in
 * `normalized` (CJK as a substring, other scripts as whole tokens), or null.
 */
function surfacePosition(normalized, surface) {
  if (!surface) return null;
  if (containsCjk(surface)) {
    const at = normalized.indexOf(surface);
    return at < 0 ? null : utf16ToByte(normalized, at);
  }
  let search = 0;
  for (let offset = normalized.indexOf(surface, search); offset >= 0; offset = normalized.indexOf(surface, search)) {
    const end = offset + surface.length;
    const startsToken = offset === 0 || normalized[offset - 1] === ' ';
    const endsToken = end === normalized.length || normalized[end] === ' ';
    if (startsToken && endsToken) return utf16ToByte(normalized, offset);
    search = offset + Math.max(surface.length, 1);
    if (search >= normalized.length) break;
  }
  return null;
}
