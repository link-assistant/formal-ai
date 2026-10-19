// Canonical semantic choices over lexical observations; no source prose is executable authority.
/** @typedef {{ $: 'unresolved' } | { $: 'grounded', subject: string, object: string }} ConceptRelationEndpoints */
/** @param {string[]} identities @param {number[]} beforePositions @param {number[]} afterPositions @returns {ConceptRelationEndpoints} */
export function conceptRelationEndpoints(identities, beforePositions, afterPositions) {
  if (identities.length !== beforePositions.length || identities.length !== afterPositions.length) return {$: 'unresolved'};
  let subject = '';
  let object = '';
  let before = -1;
  let after = -1;
  for (let index = 0; index < identities.length; index += 1) {
    if (identities[index] === '') return {$: 'unresolved'};
    const left = beforePositions[index];
    const right = afterPositions[index];
    if (left >= 0 && left >= before) {
      before = left;
      subject = identities[index];
    }
    if (right >= 0 && (after < 0 || right < after)) {
      after = right;
      object = identities[index];
    }
  }
  if (subject === '' || object === '') return {$: 'unresolved'};
  return {$: 'grounded', subject, object};
}
