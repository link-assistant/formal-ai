// Issue #1487 — the proposition structure a coding requirement holds that no
// discovered coding part reads. Program synthesis composes a candidate from
// the coding-structure meanings a requirement evidences; a structural word no
// such meaning claims — a negation (`not vowels`), an explicit count (`twice
// the number`), a disjunction (`a or b`), an ordered comparison or operator
// (`x < 10`) — would otherwise be dropped in silence, and `Return twice the
// number of vowels` would receive the program for `Return the number of
// vowels`. The residual structure is the obligation the candidate leaves
// unresolved; the caller withholds the candidate rather than claiming it.
//
// The surfaces a discovered meaning claims (`below zero`, `less than or
// equal`, `if not`) are blanked before the requirement's surface structure is
// read, so structure that is read by a part is not reported. Quoted literal
// text (`prints "a or b"`) is data the program reproduces, not structure, so
// it is blanked too, and so is a code identifier (`below_zero`), whose
// underscore joins words into a name rather than a phrase. A hyphen between
// two words of two or more letters (`reverse-sorted`, `kebab-case`) joins a
// compound word; between single letters (`a-b`) it stays an operator. The Rust twin is rust/src/coding/unread_structure.rs.

import { surfaceStructure } from './proposition_structure.mjs';
import { containsCjk, meaning, wordForms } from './seed_meanings.mjs';
import { byteOrder, isAlphabetic, isAlphanumeric, lower, trim } from './rust_str.mjs';

/** Mirrors `fn claimed_surfaces` in rust/src/coding/unread_structure.rs: lower-cased surfaces of the meanings `ids`, longest first. */
export function claimedSurfaces(ids) {
  const out = [];
  for (const id of ids) {
    const found = meaning(id);
    if (!found) continue;
    for (const form of wordForms(found)) {
      const surface = trim(lower(String(form.text)));
      if (surface && !out.includes(surface)) out.push(surface);
    }
  }
  return out.sort((left, right) => (Array.from(right).length - Array.from(left).length) || byteOrder(left, right));
}

/** Mirrors `fn bounded_at` in rust/src/coding/unread_structure.rs: a surface occurrence that starts and ends on a word boundary. */
function boundedAt(text, start, surface) {
  if (containsCjk(surface)) return true;
  const before = Array.from(text.slice(0, start)).pop();
  const after = Array.from(text.slice(start + surface.length))[0];
  return (before === undefined || !isAlphanumeric(before)) && (after === undefined || !isAlphanumeric(after));
}

/** Mirrors `QUOTE_PAIRS` in rust/src/coding/unread_structure.rs: literal delimiters that need no word boundary. */
export const QUOTE_PAIRS = Object.freeze([['"', '"'], ['`', '`'], ['\u201c', '\u201d'], ['\u00ab', '\u00bb'], ['\u300c', '\u300d'], ['\u300e', '\u300f']]);

/** Mirrors `fn blank_quoted` in rust/src/coding/unread_structure.rs: every closed quoted span replaced by one space. */
export function blankQuoted(text) {
  let out = '';
  let rest = String(text);
  for (;;) {
    const opener = Array.from(rest).find((character) => QUOTE_PAIRS.some(([open]) => open === character));
    if (opener === undefined) return out + rest;
    const at = rest.indexOf(opener);
    const close = QUOTE_PAIRS.find(([open]) => open === opener)[1];
    const body = rest.slice(at + opener.length);
    const end = body.indexOf(close);
    if (end < 0) return out + rest;
    out += `${rest.slice(0, at)} `;
    rest = body.slice(end + close.length);
  }
}

/** Mirrors `fn blank_identifiers` in rust/src/coding/unread_structure.rs: every word run joined by an underscore replaced by one space. */
export function blankIdentifiers(text) {
  let out = '';
  let run = '';
  const flush = () => {
    out += run.includes('_') ? ' ' : run;
    run = '';
  };
  for (const character of String(text)) {
    if (character === '_' || isAlphanumeric(character)) {
      run += character;
    } else {
      flush();
      out += character;
    }
  }
  flush();
  return out;
}

/** Mirrors `fn split_compounds` in rust/src/coding/unread_structure.rs: a hyphen joining two words of two or more letters becomes a space. */
export function splitCompounds(text) {
  const characters = Array.from(String(text));
  const letters = (from, step) => {
    let count = 0;
    for (let index = from; index >= 0 && index < characters.length && isAlphabetic(characters[index]); index += step) count += 1;
    return count;
  };
  return characters.map((character, index) => (character === '-' && letters(index - 1, -1) >= 2 && letters(index + 1, 1) >= 2 ? ' ' : character)).join('');
}

/** Mirrors `fn mask_claimed` in rust/src/coding/unread_structure.rs: `text` lower-cased, quoted spans and identifiers blanked, compounds split, and every claimed surface blanked. */
export function maskClaimed(text, surfaces) {
  let masked = lower(splitCompounds(blankIdentifiers(blankQuoted(text))));
  for (const surface of surfaces) {
    let from = 0;
    for (;;) {
      const start = masked.indexOf(surface, from);
      if (start < 0) break;
      if (boundedAt(masked, start, surface)) {
        masked = `${masked.slice(0, start)} ${masked.slice(start + surface.length)}`;
        from = start + 1;
      } else {
        from = start + surface.length;
      }
    }
  }
  return masked;
}

/**
 * Mirrors `fn unread_structure` in rust/src/coding/unread_structure.rs: the
 * structure of `text` left once the surfaces of the discovered coding
 * meanings `ids` are blanked — `''` when every structural word is read.
 */
export function unreadStructure(text, ids) {
  const { structure, scope } = surfaceStructure(maskClaimed(text, claimedSurfaces(ids)));
  const parts = [];
  if (structure) parts.push(structure);
  if (scope) parts.push(`scope=${scope}`);
  return parts.join(';');
}
