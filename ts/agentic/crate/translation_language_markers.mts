// `detect_response_language` from rust/src/translation/language_markers.rs.

import { languageForConceptSlug } from './language.mjs';
import { meaningsWithRole, words } from './seed_meanings.mjs';

const ROLE_RESPONSE_LANGUAGE_MARKER = 'response_language_marker';
const ROLE_COMPREHENSION_FAILURE_MARKER = 'comprehension_failure_marker';

/** Mirrors `fn detect_marker_language`. */
function detectMarkerLanguage(role, normalized) {
  for (const meaning of meaningsWithRole(role)) {
    if (!words(meaning).some((word) => normalized.includes(word))) continue;
    for (const slug of meaning.defined_by) {
      const code = languageForConceptSlug(slug);
      if (code !== null) return code;
    }
  }
  return null;
}

/** Mirrors `fn detect_response_language`. */
export function detectResponseLanguage(normalized) {
  return detectMarkerLanguage(ROLE_RESPONSE_LANGUAGE_MARKER, normalized);
}

/** Mirrors `fn detect_comprehension_failure`. */
export function detectComprehensionFailure(normalized) {
  return meaningsWithRole(ROLE_COMPREHENSION_FAILURE_MARKER)
    .some((meaning) => words(meaning).some((word) => normalized.includes(word)));
}
