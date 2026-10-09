// Match seeded lowercase cues while retaining original string boundaries.
import { contentLeadClose, firstPrefixLeadEnd } from '../crate/literal_content.mjs';

/** Mirrors fn raw_lowercase_boundary: expanded characters have no interior raw boundary. */
function rawLowercaseBoundary(request, offset) {
  let lowered = 0;
  let original = 0;
  for (const character of request) {
    if (offset === lowered) return original;
    lowered += character.toLowerCase().length;
    original += character.length;
    if (offset < lowered) return null;
  }
  return offset === lowered ? original : null;
}

/** Mirrors fn raw_lowercase_span: UTF16 indices here, original UTF8 byte indices in Rust. */
export function rawLowercaseSpan(request, span) {
  if (span === null || span[0] > span[1]) return null;
  const start = rawLowercaseBoundary(request, span[0]);
  const end = rawLowercaseBoundary(request, span[1]);
  return start === null || end === null ? null : [start, end];
}

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
