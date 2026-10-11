// The engine helpers the planner calls (rust/src/engine.rs).

import { cached, readText } from '../host.mjs';

const ALPHANUMERIC = /[\p{Alphabetic}\p{N}]/u;

/** Mirrors `const fn is_script_combining_mark` in rust/src/engine.rs. @param {string} character */
function isScriptCombiningMark(character) {
  const code = character.codePointAt(0);
  return (code >= 0x0300 && code <= 0x036f)
    || (code >= 0x0900 && code <= 0x094f)
    || (code >= 0x0951 && code <= 0x0957)
    || (code >= 0x0962 && code <= 0x0963)
    || (code >= 0x0980 && code <= 0x09ff)
    || (code >= 0x0a00 && code <= 0x0a7f)
    || (code >= 0x0a80 && code <= 0x0aff)
    || (code >= 0x0b00 && code <= 0x0b7f);
}

/** Mirrors `fn normalize_prompt` in rust/src/engine.rs. @param {string} prompt */
export function normalizePrompt(prompt) {
  const canonical = Array.from(String(prompt), (character) => character.toLowerCase())
    .join('')
    .replaceAll('c++', ' cpp ')
    .replaceAll('c#', ' csharp ');
  let normalized = '';
  for (const character of canonical) {
    normalized += ALPHANUMERIC.test(character) || isScriptCombiningMark(character) ? character : ' ';
  }
  return expandContractions(normalized.split(/\s+/u).filter(Boolean).join(' '));
}

const LANGUAGES_FILE = 'data/seed/languages.lino';

/**
 * Mirrors `fn contractions` in rust/src/engine.rs: the seeded contracted
 * token pairs (`contraction` / `expansion` records of data/seed/languages.lino),
 * each with the words it stands for (PR #1188 G70).
 */
function contractions() {
  return cached('seeded-contractions', () => {
    const pairs = [];
    let contracted = null;
    for (const line of readText(LANGUAGES_FILE).split('\n')) {
      const trimmed = line.trim();
      const space = trimmed.indexOf(' ');
      if (space < 0) continue;
      const key = trimmed.slice(0, space);
      const value = trimmed.slice(space + 1).trim().replace(/^"+|"+$/gu, '');
      if (key === 'contraction') contracted = value.split(/\s+/u).filter(Boolean);
      else if (key === 'expansion') {
        if (contracted !== null && contracted.length > 0) pairs.push([contracted, value]);
        contracted = null;
      }
    }
    return pairs;
  });
}

/** Mirrors `fn expand_contractions`: every seeded contracted pair rewritten into its expansion. */
function expandContractions(normalized) {
  const pairs = contractions();
  const tokens = normalized.split(' ');
  const out = [];
  let index = 0;
  while (index < tokens.length) {
    const pair = pairs.find(([contracted]) => index + contracted.length <= tokens.length
      && contracted.every((token, offset) => tokens[index + offset] === token));
    out.push(pair ? pair[1] : tokens[index]);
    index += pair ? pair[0].length : 1;
  }
  return out.join(' ');
}
