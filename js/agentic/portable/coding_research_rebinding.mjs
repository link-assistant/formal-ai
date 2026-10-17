// Canonical source rebinding decisions; rendered code remains an unverified proposal.
/**
 * @typedef {{ $: 'candidate', language: string, syntaxValid: boolean, moduleEffectsNone: boolean, moduleSyntaxSupported: boolean, importsCount: number, declarationsCount: number, contractSupported: boolean, callEffectsNone: boolean, name: string, parameters: string[], exported: boolean, pieces: string[], pieceBindings: string[] }} ResearchRebindingCandidate
 * @typedef {{ $: 'rejected' } | { $: 'rendered', source: string, bindingsFrom: string[], bindingsTo: string[] }} ResearchRebindingResult
 */

/** @param {string[]} values @param {string} expected @returns {number} */
export function researchRebindingIndex(values, expected) {
  for (let index = 0; index < values.length; index += 1) {
    if (values[index] === expected) return index;
  }
  return -1;
}

/** @param {string[]} values @returns {boolean} */
export function researchRebindingUnique(values) {
  for (let index = 0; index < values.length; index += 1) {
    if (values[index].trim() === '') return false;
    for (let earlier = 0; earlier < index; earlier += 1) {
      if (values[earlier] === values[index]) return false;
    }
  }
  return true;
}

/** @param {string[]} pieces @param {number} at @param {number} direction @returns {string} */
export function researchRebindingNeighbour(pieces, at, direction) {
  let index = at + direction;
  while (index >= 0 && index < pieces.length) {
    const value = pieces[index].trim();
    if (value !== '') return value;
    index += direction;
  }
  return '';
}

/**
 * @param {ResearchRebindingCandidate} candidate
 * @param {string} desiredLanguage
 * @param {string} desiredName
 * @param {string[]} desiredParameters
 * @returns {ResearchRebindingResult}
 */
export function codingResearchRebindingKernel(candidate, desiredLanguage, desiredName, desiredParameters) {
  switch (candidate.$) {
    case 'candidate': {
      const language = candidate.language;
      const syntaxValid = candidate.syntaxValid;
      const effectsNone = candidate.moduleEffectsNone;
      const syntaxSupported = candidate.moduleSyntaxSupported;
      const importsCount = candidate.importsCount;
      const declarationsCount = candidate.declarationsCount;
      const contractSupported = candidate.contractSupported;
      const callEffectsNone = candidate.callEffectsNone;
      const name = candidate.name;
      const parameters = candidate.parameters;
      const exported = candidate.exported;
      const pieces = candidate.pieces;
      const pieceBindings = candidate.pieceBindings;
      if (language !== desiredLanguage || !syntaxValid || !effectsNone || !syntaxSupported
        || importsCount !== 0 || declarationsCount !== 1 || !contractSupported || !callEffectsNone
        || parameters.length !== desiredParameters.length || pieces.length !== pieceBindings.length) {
        return {$: 'rejected'};
      }
      const from = [name, ...parameters];
      const to = [desiredName, ...desiredParameters];
      if (!researchRebindingUnique(from) || !researchRebindingUnique(to)) return {$: 'rejected'};
      let source = '';
      let functions = 0;
      for (let index = 0; index < pieces.length; index += 1) {
        const piece = pieces[index];
        const binding = pieceBindings[index];
        if (binding !== '' && binding !== piece) return {$: 'rejected'};
        const token = piece.trim();
        if (token.startsWith('/*') || token.startsWith('//')) return {$: 'rejected'};
        if (binding === 'function') functions += 1;
        if (binding === 'let' || binding === 'const' || binding === 'var' || binding === 'class'
          || binding === 'catch' || token === '=>') return {$: 'rejected'};
        const replacement = researchRebindingIndex(from, binding);
        if (binding !== '' && replacement >= 0) {
          const previous = researchRebindingNeighbour(pieces, index, -1);
          const next = researchRebindingNeighbour(pieces, index, 1);
          if (previous === '.' || previous === '?.' || next === ':'
            || ((previous === '{' || previous === ',') && (next === '}' || next === ','))) {
            return {$: 'rejected'};
          }
          source += to[replacement];
        } else source += piece;
      }
      if (functions !== 1) return {$: 'rejected'};
      for (let index = 0; index < from.length; index += 1) {
        if (researchRebindingIndex(pieceBindings, from[index]) < 0) return {$: 'rejected'};
      }
      if (!exported) source = 'export ' + source;
      if (!source.endsWith('\n')) source += '\n';
      return {$: 'rendered', source, bindingsFrom: from, bindingsTo: to};
    }
  }
}

/** No selection is -1; multiple independently valid proposals are ambiguous (-2). @param {boolean[]} valid @returns {number} */
export function canonicalUniqueSourceIndex(valid) {
  let selected = -1;
  for (let index = 0; index < valid.length; index += 1) {
    if (valid[index]) {
      if (selected !== -1) return -2;
      selected = index;
    }
  }
  return selected;
}
