// The bounded registry walk's word-boundary rule (rust/src/source_walk.rs).
// Only `is_word_boundary` is reachable from the agentic planner without a
// transport; the walk itself is native-only (see concept_lookup.mjs).

import { isWhitespace } from './rust_str.mjs';

/**
 * Mirrors `fn is_word_boundary` in rust/src/source_walk.rs.
 * @param {string} character
 */
export function isWordBoundary(character) {
  const code = character.codePointAt(0);
  return isWhitespace(character)
    || (code < 0x80 && !/^[0-9A-Za-z]$/.test(character) && character !== '_');
}
