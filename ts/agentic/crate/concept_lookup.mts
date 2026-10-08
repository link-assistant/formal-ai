// Unknown-surface detection of the concept lookup (rust/src/concept_lookup.rs).
//
// Ported: `unknown_surfaces`, `is_unknown_surface`, `unknown_surface_spans`,
// `outside_quotes`, `has_quoted_span`, `seeded_surfaces`. The registry walk
// (`RegistrySourceLookup`, `lookup_surface`) reads captured sources through
// `CachedSourceClient`; offline (as the agentic recipe runs it) it answers from
// `<cache>/source-cache` only. `offlineRegistryLookup` is that lookup for a
// host without such a cache: every concept need is `NotFound`.

import { cached } from '../host.mjs';
import { lexicon, words } from './seed_meanings.mjs';
import { isWordBoundary } from './source_walk.mjs';
import { charCount, isNumeric, splitWhitespace, trim, utf8Len } from './rust_str.mjs';
import { byteSlice, charIndices } from './formalization_segment.mjs';

const PAIRS = [['"', '"'], ['«', '»'], ['“', '”'], ['‘', '’'], ['「', '」']];

/** Mirrors `fn seeded_surfaces`: every surface the seed lexicon declares, folded. */
function seededSurfaces() {
  return cached('concept-lookup-seeded-surfaces', () => new Set(
    lexicon().flatMap((meaning) => words(meaning)).flatMap((surface) => splitWhitespace(surface).map((word) => word.toLowerCase())),
  ));
}

/** Mirrors `fn outside_quotes`. @param {string} value */
function outsideQuotes(value) {
  let out = '';
  let closing = null;
  for (const character of value) {
    if (closing !== null) {
      if (character === closing) closing = null;
      continue;
    }
    const pair = PAIRS.find(([open]) => open === character);
    if (pair) closing = pair[1];
    else out += character;
  }
  return out;
}

/** Mirrors `fn has_quoted_span`. @param {string} value */
export function hasQuotedSpan(value) {
  return outsideQuotes(value) !== value;
}

/** Mirrors `fn is_unknown_surface`. @param {string} token */
export function isUnknownSurface(token) {
  const folded = token.toLowerCase();
  return charCount(token) >= 3
    && !Array.from(token).every(isNumeric)
    && !seededSurfaces().has(folded);
}

/**
 * Mirrors `fn unknown_surfaces`: the surfaces no seeded meaning accounts for.
 * @param {string} normalized
 * @param {string} language
 * @returns {Array<string>}
 */
export function unknownSurfaces(normalized, language) {
  void language;
  const out = [];
  const tokens = [];
  let current = '';
  for (const character of outsideQuotes(normalized)) {
    if (isWordBoundary(character)) {
      tokens.push(current);
      current = '';
    } else {
      current += character;
    }
  }
  tokens.push(current);
  for (const raw of tokens) {
    const token = trim(raw);
    if (!token) continue;
    const folded = token.toLowerCase();
    if (!isUnknownSurface(token) || out.includes(folded)) continue;
    out.push(folded);
  }
  return out;
}

/**
 * Mirrors `fn unknown_surface_spans`: `[surface, start, end]` with UTF-8 byte
 * spans relative to `text`.
 * @param {string} text
 * @returns {Array<[string, number, number]>}
 */
export function unknownSurfaceSpans(text) {
  const out = [];
  let closing = null;
  let start = null;
  const push = (begin, end) => {
    const token = byteSlice(text, begin, end);
    if (isUnknownSurface(token)) out.push([token, begin, end]);
  };
  for (const [offset, character] of charIndices(text)) {
    if (closing !== null) {
      if (character === closing) {
        closing = null;
        start = null;
      }
      continue;
    }
    const pair = PAIRS.find(([open]) => open === character);
    if (pair) {
      if (start !== null) push(start, offset);
      start = null;
      closing = pair[1];
    } else if (isWordBoundary(character)) {
      if (start !== null) push(start, offset);
      start = null;
    } else if (start === null) {
      start = offset;
    }
  }
  if (start !== null) push(start, utf8Len(text));
  return out;
}

/**
 * The offline `RegistrySourceLookup` over a host with no `source-cache`
 * captures: `SourceLookup::lookup` answers `NotFound` for every need, which is
 * what the Rust lookup answers when `CachedSourceClient` (offline) misses.
 * native-only: rust/src/concept_lookup.rs `RegistrySourceLookup` over
 * `CachedSourceClient`; reading captured glosses needs the source walk,
 * the extractors and the service-accessibility cache.
 */
export const offlineRegistryLookup = Object.freeze({
  lookup() {
    return { kind: 'not_found', consulted: [] };
  },
});
