/** Mirrors fn raw_lowercase_boundary: expanded characters have no interior raw boundary. */
export function rawLowercaseBoundary(request, offset) {
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
