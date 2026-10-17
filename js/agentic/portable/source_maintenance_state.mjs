import { researchQueryWhitespace, researchQueryLower } from './coding_research_query.mjs';
/** @param {string[]} characters @returns {string} */
export function sourceMaintenanceNormalizeCueKernel(characters) {
  let start = 0;
  let end = characters.length;
  while (start < end && researchQueryWhitespace(characters[start])) start += 1;
  while (end > start && researchQueryWhitespace(characters[end - 1])) end -= 1;
  while (end > start && (characters[end - 1] === '.' || characters[end - 1] === '!' || characters[end - 1] === '?')) end -= 1;
  let normalized = '';
  let whitespace = false;
  for (let index = start; index < end; index += 1) {
    if (researchQueryWhitespace(characters[index])) {
      whitespace = true;
    } else {
      if (whitespace && normalized !== '') normalized += ' ';
      whitespace = false;
      normalized += researchQueryLower(characters[index]);
    }
  }
  return normalized;
}
/** @param {string[]} characters @returns {string} */
export function sourceMaintenanceLanguageKernel(characters) {
  let target = '';
  for (let index = 0; index < characters.length; index += 1) target += researchQueryLower(characters[index]);
  if (target.endsWith('.mts') || target.endsWith('.cts') || target.endsWith('.ts')) return 'typescript';
  if (target.endsWith('.mjs') || target.endsWith('.cjs') || target.endsWith('.js')) return 'javascript';
  return '';
}
/**
 * @param {boolean} candidate @param {number} payloads @param {string} language
 * @param {string} prefix @param {string} suffix @param {boolean} grammarValid
 * @param {string[]} actions @param {string[]} properties @param {string[]} prepositions @param {string[]} scopes
 * @param {boolean[]} constraints @returns {boolean}
 */
export function sourceMaintenanceOwnershipKernel(candidate, payloads, language, prefix, suffix, grammarValid,
  actions, properties, prepositions, scopes, constraints) {
  if (!candidate || payloads !== 0 || !grammarValid || (language !== 'javascript' && language !== 'typescript')) return false;
  let scoped = suffix === '';
  for (let index = 0; index < scopes.length; index += 1) if (suffix === scopes[index]) scoped = true;
  if (!scoped) return false;
  let operation = false;
  for (let action = 0; action < actions.length; action += 1) {
    for (let property = 0; property < properties.length; property += 1) {
      for (let preposition = 0; preposition < prepositions.length; preposition += 1) {
        if (prefix === actions[action] + ' ' + properties[property] + ' ' + prepositions[preposition]) operation = true;
      }
    }
  }
  if (!operation) return false;
  for (let index = 0; index < constraints.length; index += 1) if (!constraints[index]) return false;
  return true;
}
/**
 * @typedef {{ $: 'read' } | { $: 'derive' } | { $: 'finding' } | { $: 'edit' } | { $: 'refused', reason: string, failure: boolean }} SourceMaintenanceDecision
 */
/**
 * @param {boolean} readTool @param {boolean} observed @param {boolean} complete
 * @param {boolean} editFailed @param {boolean} editSeen @param {boolean} readbackSeen @param {boolean} readbackMatches
 * @returns {SourceMaintenanceDecision}
 */
export function sourceMaintenanceStageKernel(readTool, observed, complete, editFailed, editSeen, readbackSeen, readbackMatches) {
  if (!readTool) return { $: 'refused', reason: 'SourceMaintenanceReadPermissionMissing', failure: false };
  if (!observed) return { $: 'read' };
  if (!complete) return { $: 'refused', reason: 'SourceMaintenanceCompleteReadRequired', failure: false };
  if (editFailed) return { $: 'refused', reason: 'SourceMaintenanceEditFailed', failure: true };
  if (editSeen && !readbackSeen) return { $: 'read' };
  if (editSeen && !readbackMatches) return { $: 'refused', reason: 'SourceMaintenanceReadbackMismatch', failure: true };
  return { $: 'derive' };
}
/** @param {string} derivedGap @param {boolean} same @param {boolean} editSeen @param {boolean} editTool @returns {SourceMaintenanceDecision} */
export function sourceMaintenanceEffectKernel(derivedGap, same, editSeen, editTool) {
  if (derivedGap !== '') return { $: 'refused', reason: derivedGap, failure: false };
  if (same) return { $: 'finding' };
  if (editSeen) return { $: 'refused', reason: 'SourceMaintenancePropertyNotSatisfied', failure: true };
  if (!editTool) return { $: 'refused', reason: 'SourceMaintenanceEditPermissionMissing', failure: false };
  return { $: 'edit' };
}
