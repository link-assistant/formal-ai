// Match seeded lowercase cues while retaining original string boundaries.
import { contentLeadClose, firstPrefixLeadEnd } from '../crate/literal_content.mjs';

import { rawLowercaseBoundary, rawLowercaseSpan } from './raw_lowercase_span.mjs';
export { rawLowercaseSpan };

/** Mirrors fn first_raw_prefix_lead_end: lowercase is a matching view only. */
export function firstRawPrefixLeadEnd(request, role) {
  return rawLowercaseSpan(request, firstPrefixLeadEnd(request.toLowerCase(), role));
}

/** Mirrors fn first_raw_content_lead_end. */
export function firstRawContentLeadEnd(request) {
  return firstRawPrefixLeadEnd(request, 'file_write_content_lead');
}

/** Mirrors fn raw_content_lead_close: accept only a raw code point boundary. */
export function rawContentLeadClose(request, from) {
  if (rawLowercaseBoundary(request, request.slice(0, from).toLowerCase().length) !== from) return null;
  const close = contentLeadClose(request.toLowerCase(), request.slice(0, from).toLowerCase().length);
  return close === null ? null : rawLowercaseBoundary(request, close);
}
