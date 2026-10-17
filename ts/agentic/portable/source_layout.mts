// Canonical pure layout normalization; observers supply immutable lexical protection.
/**
 * @typedef {{ $: 'refused', reason: string } | { $: 'normalized', source: string, removed: number }} SourceLayoutResult
 */
/** @param {string[]} characters @param {boolean[]} protectedCharacters @returns {SourceLayoutResult} */
export function normalizeTrailingLayout(characters, protectedCharacters) {
  if (characters.length !== protectedCharacters.length) {
    return { $: 'refused', reason: 'SourceMaintenanceObservationMismatch' };
  }
  let source = '';
  let removed = 0;
  let index = 0;
  while (index < characters.length) {
    const character = characters[index];
    if (character === ' ' || character === '\t') {
      let end = index;
      let protectedRun = false;
      while (end < characters.length && (characters[end] === ' ' || characters[end] === '\t')) {
        if (protectedCharacters[end]) protectedRun = true;
        end += 1;
      }
      let trailing = end === characters.length;
      if (!trailing) trailing = characters[end] === '\n';
      if (!trailing && characters[end] === '\r' && end + 1 < characters.length) {
        trailing = characters[end + 1] === '\n';
      }
      if (trailing) {
        if (protectedRun) return { $: 'refused', reason: 'SourceMaintenanceProtectedLexeme' };
        removed += end - index;
      } else {
        for (let cursor = index; cursor < end; cursor += 1) source += characters[cursor];
      }
      index = end;
    } else {
      source += character;
      index += 1;
    }
  }
  return { $: 'normalized', source, removed };
}
