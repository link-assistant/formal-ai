import { readFileSync } from 'node:fs';
import { jsonText } from '../agentic/plan.mjs';
let configured;
/** Process configuration is immutable, matching native ServeArgs; replay data is never a grant. */
export function configurePrerequisiteGrant(text) {
  const value = text === null ? null : JSON.parse(text);
  if (value !== null && (value.schema !== 'formal-ai-workspace-prerequisites-v1' || value.scope !== 'workspace' || !Array.isArray(value.programs) || value.programs.some(program => typeof program !== 'string'))) throw new Error('invalid_prerequisite_grant');
  const identity = jsonText(value);
  if (configured !== undefined && configured.identity !== identity) throw new Error('prerequisite_grant_already_configured');
  const freeze = value => {
    if (value && typeof value === 'object') {
      Object.values(value).forEach(freeze);
      Object.freeze(value);
    }
    return value;
  };
  if (configured === undefined) configured = {
    identity,
    value: freeze(value)
  };
  return configured.value;
}

/** Trusted startup values only; fetched text and replay contents never call this resolver. */
export function resolvePrerequisiteGrantText(explicit, legacy) {
  if (explicit !== null && explicit !== undefined) return explicit;
  if (legacy === null || legacy === undefined || legacy === '') return null;
  if (legacy !== 'workspace') throw new Error('invalid_legacy_prerequisite_grant');
  return readFileSync(new URL('../../data/meta/workspace-prerequisite-grant.json', import.meta.url), 'utf8');
}
