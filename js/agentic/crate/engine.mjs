// The engine helpers the planner calls (rust/src/engine.rs).

const ALPHANUMERIC = /[\p{Alphabetic}\p{N}]/u;

/** Mirrors `const fn is_script_combining_mark` in rust/src/engine.rs. */
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
  return normalized.split(/\s+/u).filter(Boolean).join(' ');
}
