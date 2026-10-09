// Shared semantic-versus-literal decision; callers supply their parsed prose view.
import { roleWordForms, mentionsRole } from '../write_lexicon.mjs';
import { normalizePrompt } from './engine.mjs';
import { quotedSegmentSpans, quotedSegments } from './normal_markov.mjs';
import { containsCjk } from './coding_catalog.mjs';
import { firstRawPrefixLeadEnd } from '../write_request/lowercase_spans.mjs';
import { cleanContent } from './literal_content.mjs';
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

function ownsLiteralBody(request, content) {
  const quotes = quotedSegmentSpans(request);
  const outside = (lead) => lead !== null && !quotes.some((span) => lead[0] >= span.start && lead[0] < span.end);
  const lead = firstRawPrefixLeadEnd(request, 'file_write_content_lead');
  if (outside(lead) && quotes.some((span) => span.start >= lead[1]
    && /^[\s:]*$/u.test(request.slice(lead[1], span.start))
    && cleanContent(request.slice(span.start, span.end)) === content)) return true;
  const authoritative = firstRawPrefixLeadEnd(request, 'file_write_authoritative_content_lead');
  return outside(authoritative) && request.slice(authoritative[1]).replace(/^[\s:]*/u, '').startsWith(content);
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
