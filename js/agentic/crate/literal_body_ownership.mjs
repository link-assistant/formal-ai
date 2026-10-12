// Literal body ownership is a parser decision, independent of semantic authoring.
import { quotedSegmentSpans } from './normal_markov.mjs';
import { firstRawPrefixLeadEnd } from '../write_request/lowercase_spans.mjs';
import { cleanContent } from './literal_content.mjs';
/** Mirrors fn owns_literal_body. */
export function ownsLiteralBody(request, content) {
  const quotes = quotedSegmentSpans(request);
  const outside = (lead) => lead !== null && !quotes.some((span) => lead[0] >= span.start && lead[0] < span.end);
  const lead = firstRawPrefixLeadEnd(request, 'file_write_content_lead');
  if (outside(lead) && quotes.some((span) => span.start >= lead[1]
    && /^[\s:]*$/u.test(request.slice(lead[1], span.start))
    && cleanContent(request.slice(span.start, span.end)) === content)) return true;
  const authoritative = firstRawPrefixLeadEnd(request, 'file_write_authoritative_content_lead');
  return outside(authoritative) && request.slice(authoritative[1]).replace(/^[\s:]*/u, '').startsWith(content);
}
