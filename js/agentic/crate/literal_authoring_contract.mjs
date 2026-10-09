// Shared semantic-versus-literal decision; callers supply their parsed prose view.
import { roleWordForms, mentionsRole } from '../write_lexicon.mjs';
import { normalizePrompt } from './engine.mjs';
import { quotedSegments } from './normal_markov.mjs';
import { containsCjk } from './coding_catalog.mjs';
import { literalInstructionView } from './literal_instruction_view.mjs';
import { ownsLiteralBody } from './literal_body_ownership.mjs';
import { operationOwner } from './operation_owner.mjs';
export { operationOwner };
import { isAlphanumeric, trimStart } from '../write_str.mjs';

/** Mirrors fn semantic_authoring_lead in general_planner/content_shape.rs. */
export function semanticAuthoringLead(request) {
  const lowered = trimStart(request).toLowerCase();
  const action = roleWordForms('software_authoring_action').filter((form) => form.slot === 'bare')
    .map((form) => form.text.toLowerCase()).sort((left, right) => right.length - left.length)
    .find((surface) => {
      const next = Array.from(lowered.slice(surface.length))[0];
      return lowered.startsWith(surface) && (next === undefined || !isAlphanumeric(next) && next !== '_' && next !== '-' || containsCjk(surface));
    });
  return action !== undefined && !mentionsRole('file_whole_write_action', normalizePrompt(action));
}

/** Mirrors fn describes_code_to_author; parsed prose is lexical context, never a goal certificate. */
export function describesCodeToAuthor(request, content, prose, contentLeadPresent) {
  if (content === '') return false;
  if (semanticAuthoringLead(request)) return !ownsLiteralBody(request, content);
  if (quotedSegments(request).some((segment) => segment.includes(content))) return false;
  const normalized = normalizePrompt(prose);
  return (!contentLeadPresent && mentionsRole('coding_request_object', normalizePrompt(content)))
    || mentionsRole('coding_request_object', normalized) && mentionsRole('coding_request_verb', normalized);
}

/** Mirrors fn owned_semantic_authoring_lead: preserve unknown authoring while disambiguating observed operation objects. */
export function ownedSemanticAuthoringLead(request, contract = null) {
  const view = literalInstructionView(request, contract?.payload ?? null);
  if (view === null || !semanticAuthoringLead(view)) return false;
  const owner = operationOwner(request, contract);
  return owner === null || owner.role !== 'capability_web_scope';
}
