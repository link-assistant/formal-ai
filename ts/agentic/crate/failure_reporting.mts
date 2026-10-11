// `append_invitation` from rust/src/failure_reporting.rs.

import { localizedResponse } from './seed.mjs';
import { trim, trimEnd } from './rust_str.mjs';

/** Mirrors `fn append_invitation` in rust/src/failure_reporting.rs. */
export function appendInvitation(body, language) {
  const trimmedBody = trimEnd(body);
  const invitation = localizedResponse('detected_failure_report_invitation', language);
  if (invitation === null) return trimmedBody;
  const text = trim(invitation);
  if (!text) return trimmedBody;
  return trimmedBody ? `${trimmedBody}\n\n${text}` : text;
}
